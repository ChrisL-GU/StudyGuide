mod api;
mod commands;
mod parser;
mod settings;
mod voice;

use commands::DocumentStore;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(DocumentStore {
            documents: vec![],
        }))
        .manage(Mutex::new(voice::WhisperState { context: None }))
        .invoke_handler(tauri::generate_handler![
            commands::load_document,
            commands::list_documents,
            commands::remove_document,
            commands::get_settings,
            commands::save_settings,
            commands::chat,
            commands::get_voice_model_status,
            commands::download_voice_models,
            commands::transcribe,
            commands::speak,
        ])
        .setup(|app| {
            // Enable microphone permissions for WebKitGTK on Linux
            #[cfg(target_os = "linux")]
            {
                use tauri::Manager;
                use webkit2gtk::{PermissionRequestExt, WebViewExt};
                let webview = app.get_webview_window("main").unwrap();
                webview.with_webview(move |wv| {
                    wv.inner().connect_permission_request(|_, request: &webkit2gtk::PermissionRequest| {
                        request.allow();
                        true
                    });
                }).ok();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
