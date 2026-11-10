<div align="center">

<img src="assets/app-icon.svg" width="104" alt="" />

# YTDownload

**A YouTube downloader that does exactly what you asked for. Nothing else.**

Download a single video or a whole playlist, as MP3 or MP4.
No ads. No limits. No accounts. No telemetry.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB.svg)](https://tauri.app)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[Español](README.es.md) · [Contributing](CONTRIBUTING.md) · [Report a bug](../../issues/new?template=bug_report.yml)

</div>

---

## Why this exists

Most "free YouTube downloader" desktop apps are a thin wrapper around
[yt-dlp](https://github.com/yt-dlp/yt-dlp) — plus ads, plus an upsell, plus a
telemetry SDK you never agreed to. This project started after taking one of
them apart and finding a bundled "desktop back-connect" library and a telemetry
payload carrying `device_id`, `total_ram_mb`, `available_storage` and more.

So: same engine, none of that. The whole app is four source files. You can read
all of it in an afternoon, and fix it yourself when YouTube changes something.

## Features

**Downloading**
- Single videos and whole playlists, as **MP3** or **MP4**, with a quality
  picker (320–128 kbps, or 4K down to 480p).
- Paste the URL of a video that also belongs to a list and it asks whether you
  want **just that video or the whole playlist**. Picking the video uses the
  cleaned URL, so yt-dlp cannot pull the list in anyway.
- Up to **10 simultaneous downloads**, configurable.
- **Only the file you asked for.** No cover art, no metadata, no `.json`
  sidecars, no subtitles — enforced with explicit negative flags *and*
  `--ignore-config`, so no stray global config can slip them back in.

**The queue**
- Tabs for **all / queued / downloading / pending / completed / failed**, each
  with a count.
- **Download later**: instead of queueing, it parks what you analysed in
  **Pending**, where nothing downloads until you say so — one row, the
  selected ones, or all of them.
- The action bar is **contextual**: Failed offers "clear failed", Pending
  offers only "download now", and buttons with nothing to act on stay hidden.
- Active downloads are highlighted and sorted to the top; when one finishes it
  stays put instead of jumping to the bottom.
- **Retry** one failure or all of them. Select several and remove them.
- **Remove only drops the row — it never deletes the downloaded file.**
- **Show in folder** opens your file manager with the file already selected.
- A private or deleted video **does not kill the queue**: it is skipped and counted.
- Rows render 10 at a time as you scroll, so a 300-item playlist does not
  rebuild the whole list on every progress tick.

**It remembers**
- The queue is **saved to disk** and restored next time you open the app.
  Anything left mid-download returns as queued rather than resuming on its own.
- Separate default folders for **music and video**, so they never mix.
- **English and Spanish**, and your choice is remembered.

**Details**
- **No invented timeouts.** A long playlist is not a failure.
- A notification per song, on success and on failure.
- The yt-dlp update button **only appears when a newer version actually exists**.
- The app **tells you when a newer version of itself is out** and links to it.
  It does not install anything behind your back — it says one exists, you decide.
  **Settings** also has a **check for updates** button and shows the version you
  are running.
- History keeps up to **10,000 downloads**.

## How to use it

1. **Paste the URL** of a video or a playlist and press **Analyse**.
2. If the URL carries both a video and a list, pick **what to download**.
3. Choose **MP3 or Video** and the quality. For a playlist you can create a
   **subfolder named after it** (off by default). The final path is always
   shown before you download.
4. **Add to queue** — it starts on its own, nothing else to press. Or
   **Download later**, and it waits in Pending until you want it.
5. Follow progress in the tabs. When a download finishes, the folder icon on
   its row takes you straight to the file.

**Settings** (the gear icon) holds the language, how many downloads run at
once, the default folders, and a button to clear the history — which never
touches any downloaded file.

## Install

### Requirements

`yt-dlp` and `ffmpeg` must be on your system. YTDownload does not bundle them —
that keeps the app tiny and lets you update the part that actually matters.

```bash
# macOS
brew install yt-dlp ffmpeg

# Debian / Ubuntu
sudo apt install ffmpeg && sudo pipx install yt-dlp

# Windows
winget install yt-dlp.yt-dlp Gyan.FFmpeg
```

If either is missing, the app tells you which one and the exact command to fix it.

### Download a release

Grab the latest build from [Releases](../../releases).

#### First launch on macOS

YTDownload is not signed or notarised by Apple (that needs a developer account
at $99/yr), so macOS blocks it the first time. This is a one-off:

Download the `.dmg` that matches your Mac: **aarch64** for Apple Silicon
(M1, M2, M3, M4) and **x64** for Intel. Run `uname -m` if you are unsure —
`arm64` or `x86_64`.

Drag **YTDownload** into Applications, then open Terminal **once** and run:

```bash
xattr -dr com.apple.quarantine /Applications/YTDownload.app
```

After that it opens on a double-click like anything else.

> **Yes, the Terminal is required.** Without an Apple certificate, macOS does
> not say "unidentified developer" (which Settings can override with *Open
> Anyway*) — it says **"is damaged and can't be opened"**. For that specific
> message macOS offers **no way to allow it**, so clearing the quarantine flag
> is the only route. Nothing is damaged: the command just removes the label
> the browser attaches to downloaded files.

Or build it yourself — two minutes, and no warning at all.

### Build from source

```bash
git clone https://github.com/JSNavas/YTDownload.git
cd YTDownload
npm install
npm run dev     # development
npm run build   # produces an installer in src-tauri/target/release/bundle/
```

You need [Rust](https://rustup.rs) and [Node](https://nodejs.org) 18+.

## How it works

```
src/                      Frontend — plain HTML/CSS/JS, no framework, no build step
  index.html
  main.js                 Calls the Rust commands, listens for download-progress
  styles.css              Automatic light/dark theme
src-tauri/src/lib.rs      Backend — finds the tools, inspects URLs, runs downloads
```

The queue lives in the frontend and the backend downloads **one video per
call**. That is what makes per-item state, retries and per-item cancellation
possible — yt-dlp never sees the whole playlist.

Progress is parsed from a custom `--progress-template`, which is far more
reliable than scraping yt-dlp's human-readable output, and streamed to the UI
as events. One sharp edge worth knowing: `--print` puts yt-dlp into quiet mode,
and that **suppresses progress lines entirely**, so `--progress --no-quiet` are
required to get them back.

The backend exposes 14 commands: `check_tools`, `check_update`, `update_ytdlp`,
`probe_url`, `download_item`, `cancel_item`, `cancel_all`, `resolve_target`,
`default_dest`, `open_folder`, `reveal_file`, `load_history`, `save_history`
and `clear_history`.

History is stored in the user data directory and written atomically (temp file
plus rename), so a crash mid-write cannot leave it truncated.

### Network activity, in full

The app itself makes exactly two requests, both optional and neither carrying
anything about you:

| Request | Why |
|---|---|
| `api.github.com/repos/yt-dlp/yt-dlp/releases/latest` | Check whether a newer yt-dlp exists |
| `api.github.com/repos/JSNavas/YTDownload/releases/latest` | Check whether a newer version of YTDownload exists |
| `i.ytimg.com` | Show the thumbnail in the window |

Everything else is `yt-dlp` talking to YouTube, as you'd expect.

## Contributing

Contributions are genuinely welcome — this is a small, readable codebase, which
makes it a good one to start with. See [CONTRIBUTING.md](CONTRIBUTING.md).

Good places to start are marked
[`good first issue`](../../issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22).
The interface ships in English and Spanish. **Adding a third language is a
good first contribution**: every string lives in one `I18N` object in
`src/main.js`, so a new locale is one more entry in it.

## Legal

YTDownload is a front-end for yt-dlp. It is intended for downloading content you
own, content in the public domain, content under a license that permits it, or
your own media for offline use. Downloading copyrighted material without
permission may violate YouTube's Terms of Service and the law where you live.
You are responsible for what you download.

## License

[MIT](LICENSE). yt-dlp and ffmpeg are separate projects under their own licenses.
