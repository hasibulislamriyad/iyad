use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

static AUTOHIDE: AtomicBool = AtomicBool::new(true);

#[derive(Serialize, Default)]
struct MediaInfo { available: bool, title: String, artist: String, playing: bool }

#[cfg(windows)]
mod win {
    use super::MediaInfo;
    use windows::Media::Control::*;
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};
    use windows::Win32::System::DataExchange::{IsClipboardFormatAvailable, RegisterClipboardFormatW};
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::w;

    fn session() -> Option<GlobalSystemMediaTransportControlsSession> {
        unsafe { let _ = RoInitialize(RO_INIT_MULTITHREADED); }
        let m = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().ok()?.get().ok()?;
        m.GetCurrentSession().ok()
    }

    pub fn info() -> MediaInfo {
        let Some(s) = session() else { return MediaInfo::default() };
        let mut out = MediaInfo { available: true, ..Default::default() };
        if let Ok(op) = s.TryGetMediaPropertiesAsync() {
            if let Ok(p) = op.get() {
                out.title = p.Title().map(|t| t.to_string()).unwrap_or_default();
                out.artist = p.Artist().map(|t| t.to_string()).unwrap_or_default();
            }
        }
        if let Ok(pi) = s.GetPlaybackInfo() {
            if let Ok(st) = pi.PlaybackStatus() {
                out.playing = st == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
            }
        }
        out
    }

    pub fn cmd(c: &str) {
        if let Some(s) = session() {
            let _ = match c {
                "toggle" => s.TryTogglePlayPauseAsync().and_then(|o| o.get()),
                "next" => s.TrySkipNextAsync().and_then(|o| o.get()),
                "prev" => s.TrySkipPreviousAsync().and_then(|o| o.get()),
                _ => Ok(false),
            };
        }
    }

    // Password managers flag secrets they copy; never record those.
    pub fn clip_secret() -> bool {
        unsafe {
            let f = RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"));
            f != 0 && IsClipboardFormatAvailable(f).is_ok()
        }
    }

    pub fn fullscreen() -> bool {
        unsafe {
            let h = GetForegroundWindow();
            if h.is_invalid() { return false; }
            let mut cls = [0u16; 64];
            let n = GetClassNameW(h, &mut cls) as usize;
            let c = String::from_utf16_lossy(&cls[..n]);
            if c == "Progman" || c == "WorkerW" || c == "Shell_TrayWnd" { return false; }
            let mut r = RECT::default();
            if GetWindowRect(h, &mut r).is_err() { return false; }
            let mon = MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
            if !GetMonitorInfoW(mon, &mut mi).as_bool() { return false; }
            r.left <= mi.rcMonitor.left && r.top <= mi.rcMonitor.top
                && r.right >= mi.rcMonitor.right && r.bottom >= mi.rcMonitor.bottom
        }
    }
}

#[cfg(windows)] fn media_now() -> MediaInfo { win::info() }
#[cfg(not(windows))] fn media_now() -> MediaInfo { MediaInfo::default() }
#[cfg(windows)] fn media_do(c: &str) { win::cmd(c) }
#[cfg(not(windows))] fn media_do(_c: &str) {}
#[cfg(windows)] fn clip_secret() -> bool { win::clip_secret() }
#[cfg(not(windows))] fn clip_secret() -> bool { false }
#[cfg(windows)] fn is_fullscreen() -> bool { win::fullscreen() }
#[cfg(not(windows))] fn is_fullscreen() -> bool { false }

#[tauri::command]
fn set_expanded(app: AppHandle, expanded: bool) {
    if let Some(w) = app.get_webview_window("main") {
        let (cw, ch) = if expanded { (680.0, 300.0) } else { (296.0, 36.0) };
        let _ = w.set_size(LogicalSize::new(cw, ch));
        if expanded {
            // keep the window fully on screen after it grows
            if let (Ok(Some(m)), Ok(p)) = (w.current_monitor(), w.outer_position()) {
                let nw = cw * m.scale_factor();
                let (mx, mw) = (m.position().x as f64, m.size().width as f64);
                let x = (p.x as f64).min(mx + mw - nw).max(mx);
                let _ = w.set_position(tauri::PhysicalPosition::new(x as i32, p.y));
            }
            let _ = w.set_focus();
        }
    }
}

#[tauri::command] fn set_autohide(on: bool) { AUTOHIDE.store(on, Ordering::Relaxed); }
// Opens an app, shortcut, file or folder the user dropped on the notch.
#[tauri::command]
fn launch(path: String) {
    #[cfg(windows)]
    { let _ = std::process::Command::new("explorer").arg(&path).spawn(); }
    #[cfg(not(windows))]
    { let _ = &path; }
}
#[tauri::command] fn quit_app(app: AppHandle) { app.exit(0); }
#[tauri::command]
fn set_clipboard(text: String) {
    if let Ok(mut c) = arboard::Clipboard::new() { let _ = c.set_text(text); }
}
#[tauri::command] async fn media_info() -> MediaInfo { media_now() }
#[tauri::command] async fn media_cmd(cmd: String) { media_do(&cmd) }

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") { let _ = w.show(); }
        }))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _sc, ev| {
                    if ev.state() == ShortcutState::Pressed { let _ = app.emit("hotkey", ()); }
                })
                .build(),
        )
        .setup(|app| {
            app.global_shortcut().register("CommandOrControl+Alt+Space")?;

            // Tray icon: the notch has no taskbar button, so this is the way to open or quit it.
            {
                use tauri::menu::{Menu, MenuItem};
                use tauri::tray::TrayIconBuilder;
                let toggle = MenuItem::with_id(app, "toggle", "Open / close notch", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit", "Quit Notch", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&toggle, &quit])?;
                let mut tb = TrayIconBuilder::new().menu(&menu).tooltip("Notch")
                    .on_menu_event(|app, ev| match ev.id().0.as_str() {
                        "toggle" => { let _ = app.emit("hotkey", ()); }
                        "quit" => app.exit(0),
                        _ => {}
                    });
                if let Some(ic) = app.default_window_icon() { tb = tb.icon(ic.clone()); }
                tb.build(app)?;
            }
            set_expanded(app.handle().clone(), false);
            if let Some(w) = app.get_webview_window("main") {
                if let Ok(Some(m)) = w.current_monitor() {
                    let sf = m.scale_factor();
                    let mw = m.size().width as f64 / sf;
                    let (mx, my) = (m.position().x as f64 / sf, m.position().y as f64 / sf);
                    let _ = w.set_position(LogicalPosition::new(mx + (mw - 236.0) / 2.0, my));
                }
            }

            // Auto-hide while a fullscreen app (game, video) is in front.
            let h = app.handle().clone();
            std::thread::spawn(move || {
                let mut hidden = false;
                loop {
                    std::thread::sleep(Duration::from_millis(1000));
                    let want = AUTOHIDE.load(Ordering::Relaxed) && is_fullscreen();
                    if want != hidden {
                        hidden = want;
                        if let Some(w) = h.get_webview_window("main") {
                            let _ = if want { w.hide() } else { w.show() };
                        }
                    }
                }
            });

            // Clipboard history: poll for text changes, send to the UI.
            let h2 = app.handle().clone();
            std::thread::spawn(move || {
                let Ok(mut cb) = arboard::Clipboard::new() else { return };
                let mut last = cb.get_text().unwrap_or_default();
                loop {
                    std::thread::sleep(Duration::from_millis(700));
                    if let Ok(t) = cb.get_text() {
                        if t != last && !t.trim().is_empty() {
                            last = t.clone();
                            if clip_secret() { continue; }
                            let _ = h2.emit("clip", t);
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_expanded, set_autohide, launch, set_clipboard, media_info, media_cmd, quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Notch");
}
