use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use std::sync::Mutex;

static CURRENT_TRAY_ID: Mutex<Option<String>> = Mutex::new(None);

pub fn toggle(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let mut current_id = CURRENT_TRAY_ID.lock().unwrap();
    if current_id.is_some() {
        return Ok(());
    }

    let show_item = MenuItem::new(app, "Mostrar Vori", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit_item = MenuItem::new(app, "Salir", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let show_id = show_item.id().clone();
    let quit_id = quit_item.id().clone();

    let menu = Menu::with_items(app, &[&show_item, &quit_item])
        .map_err(|e| e.to_string())?;

    let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../../icons/tray_icon.png"))
        .unwrap_or_else(|_| app.default_window_icon().unwrap().clone());

    let tray_uuid = format!("vori_tray_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());

    TrayIconBuilder::with_id(&tray_uuid)
        .icon(tray_icon)
        .menu(&menu)
        .tooltip("Vori")
        .on_menu_event(move |app, event| {
            if event.id == show_id {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            } else if event.id == quit_id {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::services::window::toggle(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    *current_id = Some(tray_uuid);

    Ok(())
}

pub fn remove_tray(app: &AppHandle) {
    let mut current_id = CURRENT_TRAY_ID.lock().unwrap();
    if let Some(id) = current_id.take() {
        let _ = app.remove_tray_by_id(&id);
    }
}
