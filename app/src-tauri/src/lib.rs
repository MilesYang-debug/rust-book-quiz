use std::fs;
use std::path::PathBuf;

/// Directory holding the runtime question bank: `bank/` next to the exe.
/// On mobile there is no such directory — read_dir fails and the frontend
/// falls back to the embedded bank snapshot.
fn bank_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("bank")))
        .unwrap_or_else(|| PathBuf::from("bank"))
}

/// Return the raw contents of every bank/*.json file, sorted by file name.
#[tauri::command]
fn load_bank_files() -> Vec<String> {
    let mut entries: Vec<PathBuf> = match fs::read_dir(bank_dir()) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect(),
        Err(_) => Vec::new(),
    };
    entries.sort();
    entries
        .into_iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .collect()
}

/// Open a link in the system default browser (all platforms, incl. Android).
#[tauri::command]
fn open_url(app: tauri::AppHandle, url: String) {
    use tauri_plugin_opener::OpenerExt;
    if url.starts_with("https://") || url.starts_with("http://") {
        let _ = app.opener().open_url(url, None::<&str>);
    }
}

/* Window controls for the custom (frameless) desktop titlebar.
   No-ops on mobile — the buttons are hidden there anyway. */

#[tauri::command]
#[allow(unused_variables)]
fn win_minimize(window: tauri::WebviewWindow) {
    #[cfg(desktop)]
    let _ = window.minimize();
}

#[tauri::command]
#[allow(unused_variables)]
fn win_toggle_maximize(window: tauri::WebviewWindow) {
    #[cfg(desktop)]
    {
        if window.is_maximized().unwrap_or(false) {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
#[allow(unused_variables)]
fn win_close(window: tauri::WebviewWindow) {
    #[cfg(desktop)]
    let _ = window.close();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_bank_files,
            open_url,
            win_minimize,
            win_toggle_maximize,
            win_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running rust-book-quiz");
}
