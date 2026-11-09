# Contributing to YTDownload

Thanks for being here. This is a small project on purpose, and that makes it a
good one to contribute to: there's no framework to learn and no build step for
the frontend.

Issues and pull requests in **English or Spanish** are equally welcome.

## Getting set up

```bash
git clone https://github.com/JSNavas/YTDownload.git
cd YTDownload
npm install
npm run dev
```

You need [Rust](https://rustup.rs) and [Node](https://nodejs.org) 18+, plus
`yt-dlp` and `ffmpeg` on your PATH.

Note that `npm run dev` watches `src-tauri/` and restarts the app on Rust
changes, but **not** `src/`. After editing the frontend, press Cmd/Ctrl+R in
the app window.

## Where things live

| Path | What it is |
|---|---|
| `src/index.html` | The whole UI. No templating. |
| `src/main.js` | Calls backend commands, renders `download-progress` events. |
| `src/styles.css` | Theme tokens on `:root`, light and dark. |
| `src-tauri/src/lib.rs` | Every backend command. Start reading at `run()`. |

## Before you open a PR

```bash
cd src-tauri && cargo build          # must finish with zero warnings
cd .. && node --check src/main.js    # frontend has no bundler, so check syntax
```

Then actually run the app and download something — a single video *and* a
playlist. Most bugs here only show up against real YouTube responses.

## Things worth knowing

- **Never add a flag the user didn't ask for.** The whole point of this project
  is that the output contains exactly what was requested. If you add an option,
  it must be off by default and it must have a matching `--no-*` flag in the
  args list.
- **`--ignore-config` stays.** It's what stops a user's global yt-dlp config
  from changing behaviour behind their back.
- **Don't add timeouts.** A slow download is not a failed download. This was a
  real bug in the app that inspired this one.
- **Parse progress from `--progress-template`,** never from yt-dlp's
  human-readable output, which changes between versions.
- **No telemetry. Ever.** No analytics, no crash reporting that phones home, no
  unique identifiers. A PR that adds any of these will be closed. If you add a
  network request for any reason, it goes in the README table.
- Comments explain *why*, not *what*. Assume the reader can read code.

## Good first issues

- **A third language.** English and Spanish are done; every string lives in the
  `I18N` object at the top of `src/main.js`, so another locale is one entry.
- **Windows and Linux testing.** The code paths exist but get far less use,
  especially `reveal_file` and the tool detection in `find_tool`.
- **Thumbnails in the queue rows** — a "YouTube" badge and the duration
  overlaid on the image, rather than as text beside it.
- **Pause and resume** a download, rather than only cancel and retry.
- **Bandwidth limit** (`--limit-rate`), exposed in Settings.

## Commit messages

Plain and descriptive. `fix: playlist subfolder was created even when unchecked`
is perfect. Conventional Commits are welcome but not required.

## Code of conduct

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
