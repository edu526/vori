use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::config::push_recent;
use crate::commands::launcher::open_project;
use crate::models::recents::{RecentItem, RecentType};
use crate::services::cli::{self, CliAction, FrontendRequest};
use crate::services::window;
use crate::state::AppState;

fn queue(app: &AppHandle, state: &AppState, request: FrontendRequest) {
    state.pending.lock().unwrap().push(request);
    // The page may not be loaded yet; it also drains the queue once it is.
    let _ = app.emit("cli-request", ());
}

fn notice(app: &AppHandle, state: &AppState, message: String) {
    queue(app, state, FrontendRequest::Notice { message });
    window::show(app);
}

/// Carry out something asked for from the command line or a `vori://` link.
/// `second_launch` is true when another instance forwarded it to this running one.
pub fn dispatch(app: &AppHandle, action: CliAction, second_launch: bool) {
    let Some(state) = app.try_state::<AppState>() else { return };
    let state: &AppState = &state;

    match action {
        CliAction::Show => {
            if second_launch {
                window::toggle(app);
            }
        }
        CliAction::AddOrReveal { path } => {
            let existing = cli::project_at_path(&state.projects.lock().unwrap(), &path);
            queue(app, state, FrontendRequest::AddOrReveal { path, existing });
            window::show(app);
        }
        CliAction::Open { name } => {
            let found = cli::find_project(&state.projects.lock().unwrap(), &name);
            match found {
                Err(message) => notice(app, state, message),
                Ok((key, path)) => {
                    let editor = state.preferences.lock().unwrap().default_editor.clone();
                    match open_project(&path, &editor, state) {
                        Err(e) => notice(app, state, e),
                        Ok(()) => {
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs_f64())
                                .unwrap_or(0.0);
                            let _ = push_recent(
                                state,
                                RecentItem {
                                    path: path.clone(),
                                    name: key.clone(),
                                    item_type: RecentType::Project,
                                    timestamp,
                                },
                            );
                            queue(app, state, FrontendRequest::Opened { path, name: key });
                        }
                    }
                }
            }
        }
    }
}

/// Requests from the CLI / deep links that the UI has not handled yet.
#[tauri::command]
pub fn take_pending_requests(state: State<AppState>) -> Vec<FrontendRequest> {
    std::mem::take(&mut *state.pending.lock().unwrap())
}
