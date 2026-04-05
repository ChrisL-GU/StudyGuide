use crate::{api, parser, settings, voice};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

pub struct DocumentStore {
    pub documents: Vec<Document>,
}

// --- Document commands ---

#[tauri::command]
pub fn load_document(
    path: String,
    store: State<'_, Mutex<DocumentStore>>,
) -> Result<Document, String> {
    let file_path = std::path::Path::new(&path);
    let name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let content = parser::parse_file(file_path)?;
    let id = uuid::Uuid::new_v4().to_string();

    let doc = Document {
        id: id.clone(),
        name: name.clone(),
        content: Some(content),
    };

    let mut store = store.lock().map_err(|e| format!("Lock error: {}", e))?;
    store.documents.push(doc.clone());

    // Return without content to keep payload small
    Ok(Document {
        id,
        name,
        content: None,
    })
}

#[tauri::command]
pub fn list_documents(store: State<'_, Mutex<DocumentStore>>) -> Result<Vec<Document>, String> {
    let store = store.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(store
        .documents
        .iter()
        .map(|d| Document {
            id: d.id.clone(),
            name: d.name.clone(),
            content: None,
        })
        .collect())
}

#[tauri::command]
pub fn remove_document(id: String, store: State<'_, Mutex<DocumentStore>>) -> Result<(), String> {
    let mut store = store.lock().map_err(|e| format!("Lock error: {}", e))?;
    store.documents.retain(|d| d.id != id);
    Ok(())
}

// --- Settings commands ---

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<settings::AppSettings, String> {
    settings::load(&app)
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    settings: settings::AppSettings,
) -> Result<(), String> {
    settings::save(&app, &settings)
}

// --- Chat command ---

#[tauri::command]
pub async fn chat(
    app: AppHandle,
    store: State<'_, Mutex<DocumentStore>>,
    messages: Vec<api::ChatMessage>,
    selected_doc_ids: Vec<String>,
) -> Result<String, String> {
    let app_settings = settings::load(&app)?;

    // Gather document texts
    let doc_texts: Vec<(String, String)> = {
        let store = store.lock().map_err(|e| format!("Lock error: {}", e))?;
        store
            .documents
            .iter()
            .filter(|d| selected_doc_ids.contains(&d.id))
            .filter_map(|d| {
                d.content
                    .as_ref()
                    .map(|c| (d.name.clone(), c.clone()))
            })
            .collect()
    };

    // Build full message list with system prompt
    let system_prompt = api::build_system_prompt(&doc_texts);
    let mut full_messages = vec![api::ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
    }];
    full_messages.extend(messages);

    api::chat_completion_stream(&app, &app_settings, full_messages).await
}

// --- Voice commands ---

#[tauri::command]
pub fn get_voice_model_status(app: AppHandle) -> Result<voice::VoiceModelStatus, String> {
    voice::check_models(&app)
}

#[tauri::command]
pub async fn download_voice_models(app: AppHandle) -> Result<(), String> {
    voice::download_models(&app).await
}

#[tauri::command]
pub fn transcribe(
    app: AppHandle,
    whisper_state: State<'_, Mutex<voice::WhisperState>>,
    audio_data: Vec<u8>,
) -> Result<String, String> {
    let model_path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("models")
        .join("whisper")
        .join("ggml-base.en.bin");

    if !model_path.exists() {
        return Err("Whisper model not found. Please download voice models in Settings.".to_string());
    }

    let mut state = whisper_state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    voice::transcribe_audio(&mut state, &model_path, audio_data)
}

#[tauri::command]
pub async fn speak(app: AppHandle, text: String) -> Result<Vec<u8>, String> {
    let app_settings = settings::load(&app)?;
    api::text_to_speech(&app_settings, &text).await
}
