<div align="center">

# YTDownload

**Un descargador de YouTube que hace justo lo que le pediste. Nada más.**

Descarga un video suelto o una playlist entera, en MP3 o MP4.
Sin anuncios. Sin límites. Sin cuentas. Sin telemetría.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB.svg)](https://tauri.app)

[English](README.md) · [Cómo contribuir](CONTRIBUTING.md) · [Reportar un fallo](../../issues/new?template=bug_report.yml)

</div>

---

## Por qué existe

Casi todas las apps de escritorio para "descargar de YouTube gratis" son una
capa fina sobre [yt-dlp](https://github.com/yt-dlp/yt-dlp), más anuncios, más
una versión de pago, más un SDK de telemetría que nadie te pidió aceptar. Este
proyecto nació al abrir una de ellas y encontrar dentro una librería de
*desktop back-connect* y un envío de datos con `device_id`, `total_ram_mb`,
`available_storage` y bastante más.

Así que: el mismo motor, sin nada de eso. La app entera son cuatro archivos.
Puedes leértela en una tarde y arreglarla tú el día que YouTube cambie algo.

## Qué hace

- **Videos y playlists.** Pegas la URL y él distingue cuál es.
- **MP3 o MP4**, eligiendo calidad (320–128 kbps, o de 4K a 480p).
- **Tú decides dónde se guarda.** La subcarpeta con el nombre de la playlist es
  opcional y viene desactivada, porque crear carpetas que nadie pidió molesta.
  La ruta final se ve siempre antes de darle a Descargar.
- **Solo el archivo que pediste.** Ni portadas, ni metadata, ni `.json` sueltos,
  ni subtítulos. Se fuerza con flags negativos explícitos y con `--ignore-config`,
  para que ninguna configuración global pueda colártelos.
- **Sin timeouts inventados.** Que una playlist tarde horas no es un fallo.
- **Un video roto no tumba la cola.** Los privados o borrados se saltan y se
  cuentan; el resto sigue.
- **Reanudable.** Guarda un histórico en la carpeta de destino, así que
  relanzar no vuelve a bajar lo que ya tienes.
- **Aviso de actualización honesto.** El botón solo aparece cuando de verdad
  hay una versión más nueva de yt-dlp.

## Instalación

### Requisitos

Necesitas `yt-dlp` y `ffmpeg` en el sistema. YTDownload no los empaqueta: así
la app pesa poco y actualizas por tu cuenta la pieza que de verdad importa.

```bash
# macOS
brew install yt-dlp ffmpeg

# Debian / Ubuntu
sudo apt install ffmpeg && sudo pipx install yt-dlp

# Windows
winget install yt-dlp.yt-dlp Gyan.FFmpeg
```

Si falta alguno, la app te dice cuál y el comando exacto para instalarlo.

### Descargar una versión

En [Releases](../../releases).

> **macOS:** la app no está firmada (una cuenta de Apple Developer cuesta 99 $/año).
> La primera vez macOS dirá que está dañada o que viene de un desarrollador
> no identificado. Para permitirla:
> ```bash
> xattr -dr com.apple.quarantine /Applications/YTDownload.app
> ```
> O compílala tú, que son dos minutos.

### Compilar desde el código

```bash
git clone https://github.com/JSNavas/YTDownload.git
cd YTDownload
npm install
npm run dev     # desarrollo
npm run build   # genera el instalador en src-tauri/target/release/bundle/
```

Necesitas [Rust](https://rustup.rs) y [Node](https://nodejs.org) 18+.

## Cómo está hecho

```
src/                      Frontend: HTML/CSS/JS a pelo, sin framework ni build
  index.html
  main.js                 Llama a los comandos de Rust y escucha download-progress
  styles.css              Tema claro/oscuro automático
src-tauri/src/lib.rs      Backend: localiza las herramientas, analiza URLs, descarga
```

El progreso se parsea con un `--progress-template` propio, mucho más fiable que
leer la salida humana de yt-dlp, y llega a la interfaz como eventos.

### Qué sale a internet, todo

La app hace exactamente dos peticiones, las dos opcionales y ninguna lleva nada tuyo:

| Petición | Para qué |
|---|---|
| `api.github.com/repos/yt-dlp/yt-dlp/releases/latest` | Ver si hay un yt-dlp más nuevo |
| `i.ytimg.com` | Mostrar la miniatura en la ventana |

Todo lo demás es `yt-dlp` hablando con YouTube, como es de esperar.

## Contribuir

Las contribuciones son bienvenidas de verdad: es poco código y se lee bien, así
que es un buen sitio para empezar. Mira [CONTRIBUTING.md](CONTRIBUTING.md).

La interfaz está solo en español, así que **traducirla (i18n) es ahora mismo la
contribución más útil**.

## Aviso legal

YTDownload es una interfaz para yt-dlp. Está pensado para descargar contenido
propio, de dominio público, con una licencia que lo permita, o tus propios
archivos para verlos sin conexión. Descargar material con copyright sin permiso
puede incumplir los Términos de Servicio de YouTube y la ley de tu país. Lo que
descargues es responsabilidad tuya.

## Licencia

[MIT](LICENSE). yt-dlp y ffmpeg son proyectos aparte con sus propias licencias.
