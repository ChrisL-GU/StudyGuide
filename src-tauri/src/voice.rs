use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperState {
    pub context: Option<WhisperContext>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VoiceModelStatus {
    pub whisper_downloaded: bool,
}

fn models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("models");
    Ok(dir)
}

fn whisper_model_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(models_dir(app)?.join("whisper").join("ggml-base.en.bin"))
}

pub fn check_models(app: &AppHandle) -> Result<VoiceModelStatus, String> {
    Ok(VoiceModelStatus {
        whisper_downloaded: whisper_model_path(app)?.exists(),
    })
}

pub async fn download_models(app: &AppHandle) -> Result<(), String> {
    let client = reqwest::Client::new();

    let whisper_path = whisper_model_path(app)?;
    if !whisper_path.exists() {
        let dir = whisper_path.parent().unwrap();
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create whisper dir: {}", e))?;

        app.emit("model-download-progress", "Downloading Whisper model...")
            .ok();

        let url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";
        let bytes = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to download Whisper model: {}", e))?
            .bytes()
            .await
            .map_err(|e| format!("Failed to read Whisper download: {}", e))?;

        fs::write(&whisper_path, &bytes)
            .map_err(|e| format!("Failed to save Whisper model: {}", e))?;
    }

    app.emit("model-download-progress", "Whisper model ready!")
        .ok();
    Ok(())
}

pub fn transcribe_audio(
    whisper_state: &mut WhisperState,
    whisper_model_path: &PathBuf,
    audio_pcm: Vec<u8>,
) -> Result<String, String> {
    // Lazy-load the model
    if whisper_state.context.is_none() {
        let ctx = WhisperContext::new_with_params(
            whisper_model_path.to_str().unwrap(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;
        whisper_state.context = Some(ctx);
    }

    let ctx = whisper_state.context.as_ref().unwrap();

    // Convert bytes back to f32 samples
    if audio_pcm.len() % 4 != 0 {
        return Err("Invalid audio data length".to_string());
    }
    let samples: Vec<f32> = audio_pcm
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    if samples.is_empty() {
        return Err("No audio data provided".to_string());
    }

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_suppress_non_speech_tokens(true);

    let mut state = ctx
        .create_state()
        .map_err(|e| format!("Failed to create Whisper state: {}", e))?;

    state
        .full(params, &samples)
        .map_err(|e| format!("Whisper inference failed: {}", e))?;

    let num_segments = state
        .full_n_segments()
        .map_err(|e| format!("Failed to get segments: {}", e))?;
    let mut text = String::new();
    for i in 0..num_segments {
        if let Ok(segment) = state.full_get_segment_text(i) {
            text.push_str(&segment);
        }
    }

    Ok(text.trim().to_string())
}
