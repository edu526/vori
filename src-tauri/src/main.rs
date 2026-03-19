// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Force XWayland so tauri-plugin-global-shortcut (which uses XGrabKey)
    // can register system-wide hotkeys. Pure Wayland blocks XGrabKey for security;
    // XWayland provides full compatibility with no user-visible difference for this app.
    #[cfg(target_os = "linux")]
    if std::env::var("GDK_BACKEND").is_err() {
        // SAFETY: called before any thread or display init
        unsafe { std::env::set_var("GDK_BACKEND", "x11") };
    }

    vori_lib::run()
}
