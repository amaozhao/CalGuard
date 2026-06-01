pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let database_path = app_dir.join("calguard.sqlite3");
            app.manage(calguard_tauri::AppState::open(database_path)?);
            if let Some(window) = app.get_webview_window("main") {
                let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png"))?;
                window.set_icon(icon)?;
            }
            Ok(())
        })
        .invoke_handler(calguard_tauri::command_handlers())
        .run(tauri::generate_context!())
        .expect("failed to run CalGuard desktop application");
}
