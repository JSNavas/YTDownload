## What does this change?

<!-- A sentence or two. Link the issue if there is one: Fixes #123 -->

## How did you test it?

<!-- Which URLs, which format, which OS. "Downloaded a 35-item playlist as MP3
     on macOS and a single 1080p video on Windows" is exactly right. -->

- [ ] `cargo build` finishes with no warnings
- [ ] `node --check src/main.js` passes
- [ ] I ran the app and downloaded both a single video and a playlist

## Checklist

- [ ] No new flag is passed to yt-dlp that the user didn't ask for
- [ ] No telemetry, analytics, or unique identifiers added
- [ ] Any new network request is documented in the README table
- [ ] No new timeouts on downloads
