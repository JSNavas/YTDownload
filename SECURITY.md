# Security Policy

## Supported versions

The latest release is the supported one. This is a small volunteer project;
please update before reporting.

## Reporting a vulnerability

Please **do not open a public issue** for a security problem.

Use GitHub's [private vulnerability reporting](../../security/advisories/new)
on this repository. If that isn't available to you, open a normal issue saying
only that you've found a security problem and asking for a contact — no details.

Please include, where you can: what the problem is, how to reproduce it, and
what an attacker could do with it. You'll get a first response within a week.

## Scope

YTDownload runs external programs (`yt-dlp`, `ffmpeg`, `brew`, `curl`, `open`)
and writes files to a user-chosen directory. Things we consider real issues:

- Command or argument injection through a URL, a playlist title or a folder name
- Path traversal that writes outside the chosen destination folder
- Executing a binary from an attacker-controlled location rather than a trusted path
- Any unexpected network request, or any data about the user leaving the machine

Out of scope: vulnerabilities in yt-dlp or ffmpeg themselves — please report
those to those projects directly.
