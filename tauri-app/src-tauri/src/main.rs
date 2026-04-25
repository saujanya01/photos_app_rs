#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_directory,
            commands::get_timeline,
            commands::get_media_detail,
            commands::get_full_path,
            commands::get_stats,
            commands::search_media,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
