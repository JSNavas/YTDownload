use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

// ---------------------------------------------------------------- herramientas

/// Localiza un binario sin depender de que la app herede el PATH del shell:
/// una app lanzada desde el Finder, el Explorador o un lanzador de escritorio
/// no lo hereda. Mira primero las rutas habituales de cada sistema, luego el
/// PATH del proceso y, en Unix, pregunta al shell de login como último recurso.
fn find_tool(name: &str) -> Option<PathBuf> {
    // En Windows los ejecutables llevan extensión.
    #[cfg(windows)]
    let names: Vec<String> = vec![format!("{name}.exe"), name.to_string()];
    #[cfg(not(windows))]
    let names: Vec<String> = vec![name.to_string()];

    let mut dirs: Vec<PathBuf> = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // Homebrew en Apple Silicon, Homebrew en Intel, sistema, MacPorts.
        for d in [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/opt/local/bin",
        ] {
            dirs.push(PathBuf::from(d));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for d in [
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/snap/bin",
            "/var/lib/flatpak/exports/bin",
        ] {
            dirs.push(PathBuf::from(d));
        }
        // pipx y pip --user, que es como se instala yt-dlp en muchas distros.
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/bin"));
        }
    }

    #[cfg(windows)]
    {
        // winget deja enlaces aquí; chocolatey y scoop tienen su propia carpeta.
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            dirs.push(PathBuf::from(&local).join(r"Microsoft\WinGet\Links"));
            dirs.push(PathBuf::from(&local).join(r"Programs\Python\Scripts"));
        }
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(r"scoop\shims"));
        }
        dirs.push(PathBuf::from(r"C:\ProgramData\chocolatey\bin"));
    }

    // El PATH del proceso, sea cual sea el sistema.
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }

    for d in &dirs {
        for n in &names {
            let p = d.join(n);
            if p.is_file() {
                return Some(p);
            }
        }
    }

    // Último recurso en Unix: preguntar al shell de login, que sí carga el
    // perfil del usuario con sus rutas propias. En Windows no hay equivalente
    // y tampoco hace falta: allí el PATH del proceso ya es el bueno.
    #[cfg(unix)]
    {
        let out = std::process::Command::new("/bin/sh")
            .args(["-lc", &format!("command -v {name}")])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() && PathBuf::from(&s).is_file() {
            return Some(PathBuf::from(s));
        }
    }

    None
}

#[derive(Serialize)]
pub struct ToolStatus {
    ytdlp: Option<String>,
    ffmpeg: Option<String>,
    ytdlp_version: Option<String>,
}

#[tauri::command]
fn check_tools() -> ToolStatus {
    let ytdlp = find_tool("yt-dlp");
    let ffmpeg = find_tool("ffmpeg");
    let ytdlp_version = ytdlp.as_ref().and_then(|p| {
        let out = std::process::Command::new(p)
            .arg("--version")
            .output()
            .ok()?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    });
    ToolStatus {
        ytdlp: ytdlp.map(|p| p.display().to_string()),
        ffmpeg: ffmpeg.map(|p| p.display().to_string()),
        ytdlp_version,
    }
}

fn version_of(bin: &PathBuf) -> String {
    std::process::Command::new(bin)
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// ¿Hay una versión de yt-dlp más nueva que la instalada?
/// Devuelve Some(version) solo si realmente hay algo que actualizar.
/// Cualquier fallo (sin red, API caída) devuelve None: no molestamos con
/// un aviso que no podemos confirmar.
#[tauri::command]
async fn check_update() -> Option<String> {
    let bin = find_tool("yt-dlp")?;
    let local = version_of(&bin);
    if local.is_empty() {
        return None;
    }
    let out = Command::new("/usr/bin/curl")
        .args([
            "-fsSL",
            "--max-time",
            "8",
            "-H",
            "Accept: application/vnd.github+json",
            "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest",
        ])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let latest = v
        .get("tag_name")?
        .as_str()?
        .trim_start_matches('v')
        .to_string();
    // Las versiones de yt-dlp son fechas YYYY.MM.DD, así que el orden
    // lexicográfico coincide con el cronológico.
    if latest > local {
        Some(latest)
    } else {
        None
    }
}

/// La versión con la que se compiló esta app. La interfaz la tenía escrita
/// a mano y por eso siempre mostraba la misma.
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// ¿Hay una versión más nueva de YTDownload publicada?
/// Compara la versión compilada con la última release de GitHub. Devuelve
/// None ante cualquier fallo: no avisamos de algo que no podemos confirmar.
#[tauri::command]
async fn check_app_update() -> Option<serde_json::Value> {
    let current = env!("CARGO_PKG_VERSION");
    let out = Command::new("/usr/bin/curl")
        .args([
            "-fsSL",
            "--max-time",
            "8",
            "-H",
            "Accept: application/vnd.github+json",
            "https://api.github.com/repos/JSNavas/YTDownload/releases/latest",
        ])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let latest = v
        .get("tag_name")?
        .as_str()?
        .trim_start_matches('v')
        .to_string();
    let url = v.get("html_url")?.as_str()?.to_string();

    if version_is_newer(&latest, current) {
        Some(serde_json::json!({ "version": latest, "url": url }))
    } else {
        None
    }
}

/// Compara versiones tipo 0.1.10 por número, no como texto:
/// "0.1.10" es mayor que "0.1.9" aunque alfabéticamente sea al revés.
fn version_is_newer(candidate: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split(['.', '-'])
            .map(|p| p.parse::<u32>().unwrap_or(0))
            .collect()
    };
    let (a, b) = (parse(candidate), parse(current));
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
}

/// Abre una URL en el navegador del usuario.
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    // Solo http/https: no queremos abrir rutas ni esquemas arbitrarios.
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("URL no permitida".into());
    }
    opener_open(&url, false)
}

/// Actualiza yt-dlp (la causa #1 de "no me descarga" cuando YouTube cambia algo).
/// Devuelve siempre un mensaje legible: actualizado, ya al día, o qué falló.
#[tauri::command]
async fn update_ytdlp() -> Result<String, String> {
    let bin =
        find_tool("yt-dlp").ok_or("yt-dlp no está instalado. Ejecuta: brew install yt-dlp")?;
    let before = version_of(&bin);

    // Si yt-dlp vino por Homebrew, su propio `-U` no puede actualizarlo.
    let via_brew = bin.starts_with("/opt/homebrew") || bin.starts_with("/usr/local");

    let output = if via_brew {
        // No dependemos del PATH: una app lanzada desde Finder no hereda el
        // PATH del shell, así que brew no aparecería por su nombre.
        let brew = find_tool("brew")
            .ok_or("yt-dlp está instalado por Homebrew pero no encuentro el comando brew.")?;
        Command::new(&brew)
            .args(["upgrade", "yt-dlp"])
            .env(
                "PATH",
                "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
            )
            .env("HOMEBREW_NO_AUTO_UPDATE", "1")
            .env("HOMEBREW_NO_ENV_HINTS", "1")
            .output()
            .await
            .map_err(|e| format!("no se pudo ejecutar brew: {e}"))?
    } else {
        Command::new(&bin)
            .arg("-U")
            .output()
            .await
            .map_err(|e| format!("no se pudo ejecutar yt-dlp -U: {e}"))?
    };

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let low = combined.to_lowercase();
    let after = version_of(&bin);

    if !before.is_empty() && !after.is_empty() && before != after {
        return Ok(format!("Actualizado: {before} → {after}"));
    }

    // Mismo número de versión: o ya estaba al día, o algo impidió actualizar.
    if low.contains("already installed")
        || low.contains("up-to-date")
        || low.contains("up to date")
        || low.contains("no available formula")
        || output.status.success()
    {
        return Ok(format!("Ya tienes la última versión ({after})"));
    }

    let detail = combined
        .lines()
        .rfind(|l| !l.trim().is_empty())
        .unwrap_or("error desconocido")
        .trim()
        .to_string();
    Err(format!("No se pudo actualizar: {detail}"))
}

// ------------------------------------------------------------------- inspección

#[derive(Serialize)]
pub struct Entry {
    index: u32,
    title: String,
    url: String,
    duration: Option<f64>,
}

/// El video concreto al que apunta una URL que además lleva `list=`.
#[derive(Serialize)]
pub struct VideoRef {
    title: String,
    url: String,
    duration: Option<f64>,
    thumbnail: Option<String>,
}

#[derive(Serialize)]
pub struct Probe {
    /// "video" | "playlist" | "both" (un video que además pertenece a una lista)
    kind: String,
    title: String,
    uploader: Option<String>,
    thumbnail: Option<String>,
    duration: Option<f64>,
    count: usize,
    /// Nombre de carpeta ya saneado donde iría una playlist.
    folder: Option<String>,
    entries: Vec<Entry>,
    /// Solo con kind = "both": el video al que apuntaba la URL.
    video: Option<VideoRef>,
}

/// Quita los caracteres que no valen en un nombre de carpeta, sin destrozar acentos.
fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if (c as u32) < 0x20 => ' ',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim();
    let mut out = trimmed.to_string();
    if out.is_empty() {
        out = "Sin titulo".into();
    }
    // Limite sano para HFS+/APFS.
    if out.chars().count() > 120 {
        out = out.chars().take(120).collect::<String>().trim().to_string();
    }
    out
}

/// Saca el id de 11 caracteres de una URL de YouTube, si lo lleva.
fn video_id_of(url: &str) -> Option<String> {
    let take11 = |rest: &str| {
        let id: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if id.len() == 11 {
            Some(id)
        } else {
            None
        }
    };
    if let Some(i) = url.find("v=") {
        if let Some(id) = take11(&url[i + 2..]) {
            return Some(id);
        }
    }
    if let Some(i) = url.find("youtu.be/") {
        if let Some(id) = take11(&url[i + 9..]) {
            return Some(id);
        }
    }
    if let Some(i) = url.find("/shorts/") {
        if let Some(id) = take11(&url[i + 8..]) {
            return Some(id);
        }
    }
    None
}

#[tauri::command]
async fn probe_url(url: String) -> Result<Probe, String> {
    let bin =
        find_tool("yt-dlp").ok_or("yt-dlp no está instalado. Ejecuta: brew install yt-dlp")?;
    let out = Command::new(&bin)
        .args([
            "--dump-single-json",
            "--flat-playlist",
            "--no-warnings",
            "--ignore-config",
            "--socket-timeout",
            "20",
            &url,
        ])
        .output()
        .await
        .map_err(|e| format!("no se pudo ejecutar yt-dlp: {e}"))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(friendly_error(&err));
    }

    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("respuesta ilegible: {e}"))?;

    let is_playlist = v
        .get("_type")
        .and_then(|t| t.as_str())
        .map(|t| t == "playlist" || t == "multi_video")
        .unwrap_or(false);

    let title = v
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Sin titulo")
        .to_string();

    if is_playlist {
        let raw = v
            .get("entries")
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();
        let entries: Vec<Entry> = raw
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                let id = e.get("id").and_then(|x| x.as_str())?;
                Some(Entry {
                    index: (i + 1) as u32,
                    title: e
                        .get("title")
                        .and_then(|x| x.as_str())
                        .unwrap_or("(no disponible)")
                        .to_string(),
                    url: e
                        .get("url")
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={id}")),
                    duration: e.get("duration").and_then(|x| x.as_f64()),
                })
            })
            .collect();

        // Una URL tipo .../watch?v=ID&list=... es ambigua: puede querer decir
        // ese video o la lista entera. Devolvemos las dos cosas y que elija
        // el usuario, en vez de decidir por él.
        let wanted = video_id_of(&url);
        let picked = wanted.as_ref().and_then(|id| {
            entries
                .iter()
                .find(|e| e.url.contains(id.as_str()))
                .map(|e| VideoRef {
                    title: e.title.clone(),
                    url: e.url.clone(),
                    duration: e.duration,
                    thumbnail: Some(format!("https://i.ytimg.com/vi/{id}/mqdefault.jpg")),
                })
        });

        Ok(Probe {
            kind: if picked.is_some() {
                "both".into()
            } else {
                "playlist".into()
            },
            video: picked,
            folder: Some(sanitize(&title)),
            uploader: v
                .get("uploader")
                .or_else(|| v.get("channel"))
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            thumbnail: entries.first().map(|e| {
                let id = e.url.rsplit("v=").next().unwrap_or("").to_string();
                format!("https://i.ytimg.com/vi/{id}/mqdefault.jpg")
            }),
            duration: None,
            count: entries.len(),
            title,
            entries,
        })
    } else {
        Ok(Probe {
            kind: "video".into(),
            video: None,
            folder: None,
            uploader: v
                .get("uploader")
                .or_else(|| v.get("channel"))
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            thumbnail: v
                .get("thumbnail")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            duration: v.get("duration").and_then(|x| x.as_f64()),
            count: 1,
            title,
            entries: vec![],
        })
    }
}

fn friendly_error(stderr: &str) -> String {
    let e = stderr.to_lowercase();
    if e.contains("sign in to confirm") || e.contains("bot") {
        "YouTube pide verificación. Prueba otra vez o usa cookies del navegador.".into()
    } else if e.contains("private video") {
        "El video es privado.".into()
    } else if e.contains("video unavailable") || e.contains("unavailable") {
        "El video no está disponible (borrado o bloqueado por región).".into()
    } else if e.contains("is not a valid url") || e.contains("unsupported url") {
        "Esa URL no parece de YouTube.".into()
    } else if e.contains("requested format is not available") {
        "Esa calidad no existe para este video. Prueba otra.".into()
    } else {
        let last = stderr
            .lines()
            .rfind(|l| !l.trim().is_empty())
            .unwrap_or("error desconocido");
        format!("yt-dlp: {}", last.trim())
    }
}

// ------------------------------------------------------------------- descarga
//
// La cola vive en el frontend: aquí descargamos UN video por llamada. Eso es lo
// que permite estados por item, reintentos individuales y cancelar uno solo sin
// tocar los demás. yt-dlp nunca ve la playlist entera.

/// Procesos vivos, indexados por el id del item que los lanzó.
#[derive(Default)]
pub struct Running(Mutex<HashMap<String, Child>>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemOpts {
    id: String,
    url: String,
    /// "mp3" | "video"
    format: String,
    quality: String,
    /// Carpeta final, ya resuelta por el frontend.
    dest: String,
    /// Posición en la playlist, para numerar el archivo. None = video suelto.
    index: Option<u32>,
}

#[derive(Serialize, Clone)]
struct ItemEvent {
    id: String,
    /// "progress" | "post" | "skipped"
    phase: String,
    percent: f64,
    speed: String,
    eta: String,
    message: String,
}

#[tauri::command]
fn resolve_target(dest: String, folder: Option<String>, is_playlist: bool) -> String {
    let mut p = PathBuf::from(&dest);
    if is_playlist {
        if let Some(f) = folder.as_ref().map(|f| sanitize(f)) {
            p.push(f);
        }
    }
    p.display().to_string()
}

/// Carpeta sugerida la primera vez: Descargas/YTDownload, con una subcarpeta
/// por tipo para que la música y los vídeos no acaben mezclados. Se resuelve
/// con la carpeta real del sistema, así que vale en cualquier idioma y
/// cualquier sistema operativo. Los nombres en disco van en inglés a
/// propósito: no cambian si el usuario cambia el idioma de la app.
#[tauri::command]
fn default_dest(format: String) -> String {
    let mut p = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    p.push("YTDownload");
    p.push(if format == "mp3" { "Music" } else { "Video" });
    p.display().to_string()
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    opener_open(&path, false)
}

/// Abre el Finder con el archivo seleccionado, en vez de solo la carpeta.
#[tauri::command]
fn reveal_file(path: String) -> Result<(), String> {
    opener_open(&path, true)
}

fn opener_open(path: &str, reveal: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let mut c = std::process::Command::new("/usr/bin/open");
        if reveal {
            c.arg("-R");
        }
        c.arg(path).spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        let mut c = std::process::Command::new("explorer");
        if reveal {
            c.arg(format!("/select,{path}"));
        } else {
            c.arg(path);
        }
        c.spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        // xdg-open no sabe seleccionar: abrimos la carpeta contenedora.
        let target = if reveal {
            PathBuf::from(path)
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| path.to_string())
        } else {
            path.to_string()
        };
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────── historial
//
// La cola se guarda en disco para que al reabrir la app siga estando lo de
// antes. Se guarda JSON tal cual lo manda el frontend: así añadir un campo
// nuevo a un item no obliga a tocar el backend.

fn history_path() -> Result<PathBuf, String> {
    let mut p = dirs::data_dir().ok_or("no encuentro la carpeta de datos del usuario")?;
    p.push("YTDownload");
    std::fs::create_dir_all(&p).map_err(|e| format!("no se pudo crear {}: {e}", p.display()))?;
    p.push("history.json");
    Ok(p)
}

#[tauri::command]
fn load_history() -> Vec<serde_json::Value> {
    // Un historial ilegible o inexistente no es un error: se empieza vacío.
    history_path()
        .ok()
        .and_then(|p| std::fs::read(p).ok())
        .and_then(|b| serde_json::from_slice::<Vec<serde_json::Value>>(&b).ok())
        .unwrap_or_default()
}

#[tauri::command]
fn save_history(items: Vec<serde_json::Value>) -> Result<(), String> {
    let p = history_path()?;
    // Tope alto a propósito: con 500 se perdían descargas reales en cuanto
    // alguien bajaba un par de playlists grandes, y se perdían en silencio.
    // 10.000 entradas son unos 6 MB de JSON, que no es problema para nadie.
    const MAX: usize = 10_000;
    let trimmed: Vec<_> = if items.len() > MAX {
        items[items.len() - MAX..].to_vec()
    } else {
        items
    };
    let data = serde_json::to_vec_pretty(&trimmed).map_err(|e| e.to_string())?;
    // Escritura atómica: si la app muere a media escritura, el historial
    // anterior sigue intacto en vez de quedar truncado.
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, &data).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &p).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn clear_history() -> Result<(), String> {
    let p = history_path()?;
    if p.exists() {
        std::fs::remove_file(&p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn cancel_item(id: String, state: State<'_, Running>) -> Result<(), String> {
    let mut map = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(child) = map.get_mut(&id) {
        let _ = child.start_kill();
    }
    map.remove(&id);
    Ok(())
}

#[tauri::command]
fn cancel_all(state: State<'_, Running>) -> Result<(), String> {
    let mut map = state.0.lock().map_err(|e| e.to_string())?;
    for child in map.values_mut() {
        let _ = child.start_kill();
    }
    map.clear();
    Ok(())
}

/// Descarga un único video. Devuelve la ruta del archivo resultante.
#[tauri::command]
async fn download_item(app: tauri::AppHandle, opts: ItemOpts) -> Result<String, String> {
    let bin = find_tool("yt-dlp").ok_or("yt-dlp no está instalado")?;
    let ffmpeg = find_tool("ffmpeg");

    let target = PathBuf::from(&opts.dest);
    std::fs::create_dir_all(&target)
        .map_err(|e| format!("no se pudo crear {}: {e}", target.display()))?;

    let out_tpl = match opts.index {
        Some(i) => target.join(format!("{i:03} - %(title)s.%(ext)s")),
        None => target.join("%(title)s.%(ext)s"),
    };

    let mut args: Vec<String> = vec![
        // Nunca leemos la config global del usuario: nada puede colarse.
        "--ignore-config".into(),
        "--newline".into(),
        "--no-mtime".into(),
        "--no-playlist".into(),
        // Imprescindibles: `--print` activa el modo silencioso de yt-dlp y
        // eso se lleva por delante las líneas de progreso, dejando la barra
        // clavada en 0%. Estos dos las devuelven.
        "--progress".into(),
        "--no-quiet".into(),
        // Solo el archivo pedido. Ni portadas, ni metadata, ni sidecars.
        "--no-embed-thumbnail".into(),
        "--no-embed-metadata".into(),
        "--no-write-thumbnail".into(),
        "--no-write-info-json".into(),
        "--no-write-description".into(),
        "--no-write-subs".into(),
        "--no-write-auto-subs".into(),
        "--no-embed-subs".into(),
        "--no-embed-chapters".into(),
        "--no-keep-video".into(),
        "--retries".into(),
        "10".into(),
        "--fragment-retries".into(),
        "10".into(),
        "--concurrent-fragments".into(),
        "4".into(),
        "--socket-timeout".into(),
        "30".into(),
        "-o".into(),
        out_tpl.display().to_string(),
        "--progress-template".into(),
        "YTD|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s".into(),
        "--print".into(),
        "after_move:YTDFILE|%(filepath)s".into(),
    ];

    if let Some(f) = &ffmpeg {
        args.push("--ffmpeg-location".into());
        args.push(f.display().to_string());
    }

    if opts.format == "mp3" {
        args.push("-f".into());
        args.push("bestaudio/best".into());
        args.push("--extract-audio".into());
        args.push("--audio-format".into());
        args.push("mp3".into());
        args.push("--audio-quality".into());
        args.push(if opts.quality == "best" {
            "0".into()
        } else {
            format!("{}K", opts.quality)
        });
    } else {
        let sel = if opts.quality == "best" {
            "bestvideo[ext=mp4]+bestaudio[ext=m4a]/bestvideo+bestaudio/best".to_string()
        } else {
            let h = &opts.quality;
            format!(
                "bestvideo[height<={h}][ext=mp4]+bestaudio[ext=m4a]/\
                 bestvideo[height<={h}]+bestaudio/\
                 best[height<={h}]/best"
            )
        };
        args.push("-f".into());
        args.push(sel);
        args.push("--merge-output-format".into());
        args.push("mp4".into());
    }

    args.push(opts.url.clone());

    let mut child = Command::new(&bin)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("no se pudo lanzar yt-dlp: {e}"))?;

    let stdout = child.stdout.take().ok_or("sin stdout")?;
    let stderr = child.stderr.take().ok_or("sin stderr")?;

    {
        let state = app.state::<Running>();
        let mut map = state.0.lock().map_err(|e| e.to_string())?;
        map.insert(opts.id.clone(), child);
    }

    // Guardamos stderr completo: si falla, el último renglón explica por qué.
    let errbuf = std::sync::Arc::new(Mutex::new(String::new()));
    let errbuf2 = errbuf.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(mut b) = errbuf2.lock() {
                b.push_str(&line);
                b.push('\n');
            }
        }
    });

    let id = opts.id.clone();
    let mut filepath = String::new();
    let mut skipped = false;
    let mut lines = BufReader::new(stdout).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(rest) = line.strip_prefix("YTD|") {
            let f: Vec<&str> = rest.split('|').collect();
            if f.len() >= 3 {
                let percent = f[0]
                    .trim()
                    .trim_end_matches('%')
                    .parse::<f64>()
                    .unwrap_or(0.0);
                let _ = app.emit(
                    "item",
                    ItemEvent {
                        id: id.clone(),
                        phase: "progress".into(),
                        percent,
                        speed: f[1].trim().to_string(),
                        eta: f[2].trim().to_string(),
                        message: String::new(),
                    },
                );
            }
        } else if let Some(p) = line.strip_prefix("YTDFILE|") {
            filepath = p.trim().to_string();
        } else if line.contains("has already been downloaded") {
            skipped = true;
        } else if line.starts_with("[ExtractAudio]") || line.starts_with("[Merger]") {
            let _ = app.emit(
                "item",
                ItemEvent {
                    id: id.clone(),
                    phase: "post".into(),
                    percent: 100.0,
                    speed: String::new(),
                    eta: String::new(),
                    message: String::new(),
                },
            );
        }
    }

    // Sin timeout: una descarga lenta no es una descarga fallida.
    let status = {
        let state = app.state::<Running>();
        let mut taken = {
            let mut map = state.0.lock().map_err(|e| e.to_string())?;
            map.remove(&id)
        };
        match taken.as_mut() {
            Some(c) => c.wait().await.map_err(|e| e.to_string())?.code(),
            // cancel_item se llevó el proceso.
            None => return Err("CANCELLED".into()),
        }
    };

    match status {
        Some(0) => {
            if filepath.is_empty() && skipped {
                Ok(String::new())
            } else {
                Ok(filepath)
            }
        }
        None => Err("CANCELLED".into()),
        Some(_) => {
            let msg = errbuf
                .lock()
                .map(|b| friendly_error(&b))
                .unwrap_or_else(|_| "error desconocido".into());
            Err(msg)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Running::default())
        .invoke_handler(tauri::generate_handler![
            check_tools,
            check_update,
            check_app_update,
            app_version,
            open_url,
            update_ytdlp,
            probe_url,
            download_item,
            cancel_item,
            cancel_all,
            resolve_target,
            default_dest,
            open_folder,
            reveal_file,
            load_history,
            save_history,
            clear_history
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar la app");
}
