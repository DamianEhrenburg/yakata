<p align="center">
  <img src="./assets/yakatalogo.png" alt="Yakata Logo" width="440">
</p>

<h1 align="center">Yakata</h1>

<p align="center">
  <a href="https://github.com/DamianEhrenburg/yakata/releases/latest">
    <img src="https://img.shields.io/badge/release-v0.3.2-9b1b30?style=flat-square" alt="Latest release">
  </a>
  <a href="https://github.com/DamianEhrenburg/yakata/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-3f3f46?style=flat-square" alt="License">
  </a>
  <img src="https://img.shields.io/badge/platform-Windows-1e1b4b?style=flat-square" alt="Platform Windows">
</p>

<p align="center">
  Visual novel library, launcher, and playtime tracker for Windows, built with Rust and Dioxus.
</p>

<p align="center">
  <img src="./assets/screenshot.png" alt="Yakata Library Preview" width="100%">
</p>

<p align="center">
  <img src="./assets/discord_activity.png" alt="Discord Rich Presence Preview" width="85%">
</p>

## Overview

Yakata is a desktop visual novel launcher and tracker for Windows. It catalogs your collection, pulls metadata and covers from VNDB, monitors playtime across multi-process engine wrappers, and displays Discord Rich Presence.

## Features

- Add visual novels manually or fetch metadata and cover art from VNDB
- Read synopses with expandable spoiler blocks
- Track total playtime and session logs across child processes and engine wrappers
- Launch Japanese titles with Locale Emulator directly from the app
- Filter and sort by reading status, 10-point personal score, favourites, and tags
- Track story routes, playtime sessions, and personal notes
- Discord Rich Presence with game status, route, and elapsed playtime
- Blur adult covers with reveal on hover
- Export and import library backups as JSON

## Download

Get the installer or portable archive from [Releases](https://github.com/DamianEhrenburg/yakata/releases/latest):

- `Yakata-Setup.exe` - Windows Installer (NSIS)
- `Yakata-Portable.zip` - Standalone portable build (stores data in a local `data/` folder via `portable.txt`)
- `SHA256SUMS.txt` - SHA-256 checksums for release files

## Building from Source

Requires [Rust](https://rustup.rs/) and [Dioxus CLI](https://dioxuslabs.com):

```bash
cargo install --locked dioxus-cli
dx build --release
```

## Acknowledgments

Based on [kakera](https://github.com/enotan/kakera) by [enotan](https://github.com/enotan).

## License

This project is licensed under the [MIT License](LICENSE).
