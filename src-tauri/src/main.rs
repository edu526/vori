// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Elevated write helper mode — invoked via UAC/sudo/osascript re-exec.
    // Runs BEFORE any Tauri/Linux init so the elevated child does only the write.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("--vori-write") {
        vori_write_main(&args);
    }

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

fn vori_write_main(args: &[String]) -> ! {
    if args.len() != 4 {
        eprintln!("vori-elevate: usage --vori-write <target> <temp>");
        std::process::exit(2);
    }
    let target = std::path::PathBuf::from(&args[2]);
    let temp = std::path::PathBuf::from(&args[3]);
    let result = std::fs::read_to_string(&temp)
        .and_then(|c| std::fs::write(&target, c))
        .and_then(|_| std::fs::remove_file(&temp));
    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("vori-elevate: {e}");
            std::process::exit(1);
        }
    }
}