# Changelog

All notable changes to Yakata are documented here.

## [0.3.1] - 2026-11-20

### Added
- Portable mode via `portable.txt`, storing library, settings, covers, and logs in a local `data/` folder.
- Direct VNDB lookup by ID, such as `v17`.
- Browse button for selecting local cover images in the Add VN form.
- `SHA256SUMS.txt` checksums in release artifacts.

### Changed
- Moved library saves, settings writes, and Discord Rich Presence off the UI thread.
- Shared a single HTTP client with a 15-second timeout for VNDB requests and cover downloads.
- Deepened the dark burgundy palette, removed excess glow shadows, and unified rating badge styling.
- Improved folder title parsing for dots, version suffixes, and Japanese brackets.

### Fixed
- Deleting a visual novel no longer removes custom cover files located outside Yakata's cache.
- Prevented play sessions from being overwritten when editing the library while a game closes.
- Switched Windows URL and Steam launching to `ShellExecuteW` without console window flashes, with URL scheme validation and proper query encoding.
- Detected Steam's install folder from the Windows registry and ignored sleep gaps and sessions under 10 seconds in playtime tracking.
- Ignored outdated VNDB search responses, blocked duplicate route names, and fixed the cancel button in description and notes editors.
- Fixed topbar layout on smaller windows, checkbox click flashes, and primary button hover states.

## [0.3.0] - 2026-11-15

### Added
- Single-step visual novel creation with automatic VNDB metadata fetching, tags, and cover art download.
- Manual play session logging and deletion in the History tab.
- In-app GitHub update checker and notification banner.
- Sort direction toggle in topbar with ascending/descending modes.
- Toggle button for Locale Emulator in launch settings.
- VNDB search sorting controls and rating badges on cover previews.
- Interactive story route tracking with active route selection and progress counter.
- Two-line Discord Rich Presence with natural route naming.
- VNDB spoiler and engine feature tag filtering.

### Changed
- Unified custom checkbox controls across the entire application.
- Replaced static background image with procedural two-point lighting falloff and vector film grain.
- Upgraded Dioxus framework dependencies and optimized VNDB search response time.
- Refined tag chips with fixed layout dimensions and smooth hover highlighting.

### Fixed
- Reset filters action in empty library view updating UI signals immediately.
- Prevented layout reflow, button overlap, and flickering when interacting with tags.
- Hidden spinner arrows in session duration number inputs.

## [0.2.1] - 2026-11-10

### Fixed
- Detail panel: aligned notes edit button inline next to placeholder text to match tags style and prevent overlap with delete button.

## [0.2.0] - 2026-11-03

### Added
- Rebranded application to Yakata with Bordeaux Velvet crystal identity and custom typography.
- Novel tracking statuses: Reading, Completed, Plan to Read, On Hold, Dropped.
- Topbar status filter dropdown aligned horizontally with existing controls.
- Personal score system (1-10) with cover badges and rating sorting.
- Windows process watchdog using Win32 ToolHelp to track game runtime across wrapper executables.
- Locale Emulator (LEProc.exe) support for Japanese visual novels.
- Direct "Open on VNDB" button on the details panel.
- JSON library backup export and import tools in settings.
- Custom background wallpaper support preserving ambient starfield overlays.
- Automatic library migration from Kakera on first launch.

### Changed
- Bumped application version to 0.2.0.
- Normalized logo color filters in CSS to preserve original burgundy crystal assets.
