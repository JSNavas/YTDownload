use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

// ---------------------------------------------------------------- herramientas

/// Busca un binario en las rutas habituales de macOS/Linux y en el PATH.
fn find_tool(name: &str) -> Option<PathBuf> {
    let candidates = [
        format!("/opt/homebrew/bin/{name}"),
        format!("/usr/local/bin/{name}"),
        format!("/usr/bin/{name}"),
        format!("/opt/local/bin/{name}"),
    ];
    for c in candidates.iter() {
        let p = PathBuf::from(c);
        if p.is_file() {
            return Some(p);
        }
    }
    // Fallback: resolver via `which` heredando el PATH del shell de login.
    let out = std::process::Command::new("/bin/sh")
        .args(["-lc", &format!("command -v {name}")])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !s.is_empty() && PathBuf::from(&s).is_file() {
        Some(PathBuf::from(s))
    } else {
        None
    }
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

#[derive(Serialize)]
pub struct Probe {
    kind: String, // "video" | "playlist"
    title: String,
    uploader: Option<String>,
    thumbnail: Option<String>,
    duration: Option<f64>,
    count: usize,
    /// Nombre de carpeta ya saneado donde iría una playlist.
    folder: Option<String>,
    entries: Vec<Entry>,
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

        Ok(Probe {
            kind: "playlist".into(),
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

#[derive(Deserialize)]
// El frontend manda camelCase; Tauri solo convierte los argumentos del
// comando, no los campos de un struct anidado. De ahí el rename_all.
#[serde(rename_all = "camelCase")]
pub struct DownloadOpts {
    url: String,
    /// "mp3" | "video"
    format: String,
    /// mp3: "320" | "256" | "192" | "128" | "best". video: "2160" | "1440" | "1080" | "720" | "480" | "best"
    quality: String,
    /// Carpeta base elegida por el usuario.
    dest: String,
    /// Para playlists: subcarpeta donde va todo (ya saneada por probe_url, editable).
    folder: Option<String>,
    is_playlist: bool,
    /// Descargas en paralelo dentro de una playlist (1 = secuencial).
    concurrency: u32,
}

#[derive(Serialize, Clone)]
struct Progress {
    phase: String, // "start" | "item" | "progress" | "post" | "done" | "error" | "skipped"
    index: u32,
    total: u32,
    title: String,
    percent: f64,
    speed: String,
    eta: String,
    message: String,
}

#[derive(Default)]
pub struct Running(Mutex<Option<Child>>);

/// Carpeta final donde acabará la descarga. Se calcula igual en Rust y se muestra en la UI.
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

#[tauri::command]
fn default_dest(app: tauri::AppHandle, format: String) -> String {
    let _ = &app;
    let base = if format == "mp3" {
        dirs::audio_dir().or_else(dirs::home_dir)
    } else {
        dirs::video_dir().or_else(dirs::home_dir)
    };
    let mut p = base.unwrap_or_else(|| PathBuf::from("."));
    p.push("YTDownload");
    p.display().to_string()
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    std::process::Command::new("/usr/bin/open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn cancel_download(state: State<'_, Running>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(child) = guard.as_mut() {
        let _ = child.start_kill();
    }
    *guard = None;
    Ok(())
}

#[tauri::command]
async fn start_download(app: tauri::AppHandle, opts: DownloadOpts) -> Result<String, String> {
    let bin =
        find_tool("yt-dlp").ok_or("yt-dlp no está instalado. Ejecuta: brew install yt-dlp")?;
    let ffmpeg = find_tool("ffmpeg");

    let target = PathBuf::from(resolve_target(
        opts.dest.clone(),
        opts.folder.clone(),
        opts.is_playlist,
    ));
    std::fs::create_dir_all(&target)
        .map_err(|e| format!("no se pudo crear {}: {e}", target.display()))?;

    // Plantilla de nombre: en playlist numeramos para que el orden se mantenga.
    let out_tpl = if opts.is_playlist {
        target.join("%(playlist_index)03d - %(title)s.%(ext)s")
    } else {
        target.join("%(title)s.%(ext)s")
    };

    let mut args: Vec<String> = vec![
        // --ignore-config: nunca leemos ~/.config/yt-dlp/config, para que una
        // config vieja no pueda reintroducir nada que el usuario no pidió.
        "--ignore-config".into(),
        "--newline".into(),
        "--no-mtime".into(),
        // Negativos explícitos: aquí no hay portadas, ni metadata, ni .json,
        // ni subtítulos, ni ficheros sueltos. Solo el archivo que pediste.
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
        // Progreso en un formato que parseamos sin ambigüedad.
        "--progress-template".into(),
        "MF|%(info.playlist_index)s|%(info.title)s|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s".into(),
        "--print".into(),
        "before_dl:MFITEM|%(playlist_index)s|%(title)s".into(),
        "--print".into(),
        "after_move:MFFILE|%(filepath)s".into(),
    ];

    if let Some(f) = &ffmpeg {
        args.push("--ffmpeg-location".into());
        args.push(f.display().to_string());
    }

    if opts.is_playlist {
        args.push("--yes-playlist".into());
        // Sigue con el resto si un video falla, en vez de abortar toda la playlist.
        args.push("--ignore-errors".into());
        // Archivo de histórico: si relanzas, no baja de nuevo lo que ya está.
        args.push("--download-archive".into());
        args.push(target.join(".ytdownload-archive.txt").display().to_string());
        let n = opts.concurrency.clamp(1, 5);
        if n > 1 {
            args.push("-N".into());
            args.push(n.to_string());
        }
    } else {
        args.push("--no-playlist".into());
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
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        *guard = Some(child);
    }

    let emit = |app: &tauri::AppHandle, p: Progress| {
        let _ = app.emit("download-progress", p);
    };

    // stderr en paralelo: avisos y errores por item.
    let app_err = app.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            let lower = line.to_lowercase();
            if lower.starts_with("error") || lower.contains("error:") {
                let _ = app_err.emit(
                    "download-progress",
                    Progress {
                        phase: "error".into(),
                        index: 0,
                        total: 0,
                        title: String::new(),
                        percent: 0.0,
                        speed: String::new(),
                        eta: String::new(),
                        message: friendly_error(&line),
                    },
                );
            }
        }
    });

    let mut lines = BufReader::new(stdout).lines();
    let mut cur_index: u32 = 0;
    let mut cur_title = String::new();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(rest) = line.strip_prefix("MFITEM|") {
            let mut it = rest.splitn(2, '|');
            let idx = it.next().unwrap_or("").trim();
            cur_index = idx.parse().unwrap_or(cur_index + 1);
            cur_title = it.next().unwrap_or("").trim().to_string();
            emit(
                &app,
                Progress {
                    phase: "item".into(),
                    index: cur_index,
                    total: 0,
                    title: cur_title.clone(),
                    percent: 0.0,
                    speed: String::new(),
                    eta: String::new(),
                    message: String::new(),
                },
            );
        } else if let Some(rest) = line.strip_prefix("MF|") {
            let f: Vec<&str> = rest.split('|').collect();
            if f.len() >= 5 {
                if let Ok(i) = f[0].trim().parse::<u32>() {
                    cur_index = i;
                }
                if !f[1].trim().is_empty() && f[1].trim() != "NA" {
                    cur_title = f[1].trim().to_string();
                }
                let percent = f[2]
                    .trim()
                    .trim_end_matches('%')
                    .parse::<f64>()
                    .unwrap_or(0.0);
                emit(
                    &app,
                    Progress {
                        phase: "progress".into(),
                        index: cur_index,
                        total: 0,
                        title: cur_title.clone(),
                        percent,
                        speed: f[3].trim().to_string(),
                        eta: f[4].trim().to_string(),
                        message: String::new(),
                    },
                );
            }
        } else if let Some(path) = line.strip_prefix("MFFILE|") {
            emit(
                &app,
                Progress {
                    phase: "done".into(),
                    index: cur_index,
                    total: 0,
                    title: cur_title.clone(),
                    percent: 100.0,
                    speed: String::new(),
                    eta: String::new(),
                    message: path.trim().to_string(),
                },
            );
        } else if line.contains("has already been downloaded") {
            emit(
                &app,
                Progress {
                    phase: "skipped".into(),
                    index: cur_index,
                    total: 0,
                    title: cur_title.clone(),
                    percent: 100.0,
                    speed: String::new(),
                    eta: String::new(),
                    message: "ya estaba descargado".into(),
                },
            );
        } else if line.starts_with("[ExtractAudio]") || line.starts_with("[Merger]") {
            emit(
                &app,
                Progress {
                    phase: "post".into(),
                    index: cur_index,
                    total: 0,
                    title: cur_title.clone(),
                    percent: 100.0,
                    speed: String::new(),
                    eta: String::new(),
                    message: "convirtiendo…".into(),
                },
            );
        }
    }

    // Esperamos el final sin ningún timeout artificial: una playlist larga
    // puede tardar horas y eso no es un fallo.
    let status = {
        let state = app.state::<Running>();
        let mut taken = {
            let mut guard = state.0.lock().map_err(|e| e.to_string())?;
            guard.take()
        };
        match taken.as_mut() {
            Some(c) => c.wait().await.map_err(|e| e.to_string())?.code(),
            // cancel_download se llevó el proceso.
            None => return Ok("cancelado".into()),
        }
    };

    match status {
        Some(0) => Ok(target.display().to_string()),
        // 1 con --ignore-errors = algunos items fallaron, el resto bajó bien.
        Some(1) if opts.is_playlist => Ok(target.display().to_string()),
        Some(c) => Err(format!("yt-dlp terminó con código {c}")),
        None => Ok("cancelado".into()),
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
            update_ytdlp,
            probe_url,
            start_download,
            cancel_download,
            resolve_target,
            default_dest,
            open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar la app");
}
