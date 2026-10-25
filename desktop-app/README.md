# Notch: desktop companion for Windows

A notch bar at the top-center of your screen with a mascot. Built with Tauri 2 (Rust + WebView2).

## Run it

1. Install Rust: https://rustup.rs (choose the MSVC toolchain)
2. Install "Desktop development with C++" from Visual Studio Build Tools
3. Install Node.js 18+ (WebView2 already ships with Windows 11)
4. In this folder:

```
npm install
npm run dev        # run in development
npm run build      # makes an installer in src-tauri/target/release/bundle
```

First build downloads crates and takes several minutes.

## What works (real, not simulated)

- Always-on-top transparent notch, centered at the top of the current monitor, expands and collapses
- Media: title, artist, play/pause/next/previous for Spotify, browsers, anything using Windows media controls
- Clipboard history: text only, search, pin, click to copy again (kept in the app's local storage)
- Notes: saved locally
- Focus timer
- Auto-hide when a fullscreen app is in front (game or video)
- Ctrl+Alt+Space opens and closes the notch
- Mascot moods: happy on copy, focused during a session, sleepy when idle

## Not built yet

Weather, calendar, Windows notifications, mic/camera indicator, file shelf, screen-time tracking, start with Windows, plugin system, settings window, auto-update.

## Known limits (honest list)

- This project was written without being compiled. Expect to fix small compile errors on first build.
- Fullscreen detection compares the front window to the monitor size. A maximized window on an auto-hiding taskbar can be a false positive.
- The notch overlays other windows. It does not reserve screen space yet.
- Clipboard history stores copied text on disk. Passwords copied from a password manager would be saved too. Before release, add an "ignore these apps" list and a "pause history" toggle.
- Collapsed, the window is 320x56 and sits at the very top, so it covers that strip of any window underneath.

## Files

- `src/index.html` all UI (HTML, CSS, JS in one file, no external requests)
- `src-tauri/src/lib.rs` window placement, media controls, clipboard watcher, fullscreen detection, hotkey
- `src-tauri/tauri.conf.json` transparent, frameless, always-on-top window

## v0.3 updates

- Drag apps, shortcuts or files onto the notch (Apps / Files tabs). Click to open, right-click to remove.
- Drag-in no longer collapses the notch; uses Tauri's native drop events.
- Clipboard privacy: ignores copies flagged by password managers, "Pause clipboard history" switch, Clear button.
- Settings: day start, bedtime, Start with Windows. Focus length remembered, sessions per day counted.
- Welcome tip on first run, all text in English, single-instance app.
