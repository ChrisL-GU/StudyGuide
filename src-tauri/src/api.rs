use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

pub fn build_system_prompt(document_texts: &[(String, String)]) -> String {
    let mut prompt = String::from(
        "You are an intellectual reading companion. You guide a reader through study materials section by section, fostering deep, independent thinking. You never tell the reader what to think.

How to engage:
1. Present a passage or key section from the material, then pause and invite the reader to react before moving on. Walk through the text together — don't assume they've read ahead.
2. Ask what the reader thinks the author means, rather than explaining it yourself. If they ask you to summarize or interpret, turn it back: \"What do you think the author is getting at here?\"
3. Surface hidden assumptions — ask the reader to identify what the author takes for granted, what's left unsaid, or what worldview underlies the argument.
4. When the reader settles on an interpretation, play devil's advocate. Offer alternative readings or counterexamples: \"Could someone read this differently? What if the opposite were true?\"
5. Challenge completeness — if the reader's reasoning is sound but shallow, push further: \"That's a strong point. What would follow from that? Are there implications you haven't considered?\"
6. Connect across sections — draw the reader's attention to tensions, echoes, or contradictions between different parts of the text: \"How does this sit with what the author said earlier about...?\"
7. Acknowledge genuine insight. Not every response needs a challenge — when the reader sees something sharp, say so briefly and build on it.
8. Keep responses concise. One or two questions at a time. Let the reader do the heavy thinking.
9. Never lecture, never summarize unprompted, never present your own thesis. You are a sparring partner, not a professor.",
    );

    if !document_texts.is_empty() {
        prompt.push_str("\n\n--- STUDY MATERIALS ---\n");
        for (name, content) in document_texts {
            prompt.push_str(&format!("\n### Document: {}\n{}\n", name, content));
        }
        prompt.push_str("\n--- END STUDY MATERIALS ---");
    }

    prompt
}

pub async fn chat_completion_stream(
    app: &AppHandle,
    settings: &AppSettings,
    messages: Vec<ChatMessage>,
) -> Result<String, String> {
    if settings.api_key.is_empty() {
        return Err("API key is not configured. Please set it in Settings.".to_string());
    }

    let is_azure = !settings.api_version.is_empty();
    let base = settings.base_url.trim_end_matches('/');

    let url = if is_azure {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            base, settings.model, settings.api_version
        )
    } else {
        format!("{}/chat/completions", base)
    };

    let client = reqwest::Client::new();
    let body = serde_json::to_vec(&ChatRequest {
        model: settings.model.clone(),
        messages,
        stream: true,
    })
    .map_err(|e| format!("Failed to serialize request: {}", e))?;

    let mut last_err = String::new();
    let mut response = None;
    for attempt in 0u64..3 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(500 * attempt)).await;
        }
        let mut req = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body.clone());
        if is_azure {
            req = req.header("api-key", &settings.api_key);
        } else {
            req = req.header("Authorization", format!("Bearer {}", settings.api_key));
        }
        match req.send().await {
            Ok(resp) => {
                response = Some(resp);
                break;
            }
            Err(e) => {
                last_err = format!("API request failed: {}", e);
            }
        }
    }
    let response = response.ok_or(last_err)?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error ({}): {}", status, body));
    }

    // Read SSE stream
    let mut full_content = String::new();
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream read error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete lines
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if line.is_empty() || line.starts_with(':') {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    break;
                }
                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            full_content.push_str(content);
                            app.emit("chat-stream-chunk", content.clone()).ok();
                        }
                    }
                }
            }
        }
    }

    app.emit("chat-stream-done", ()).ok();
    Ok(full_content)
}

pub async fn text_to_speech(settings: &AppSettings, text: &str) -> Result<Vec<u8>, String> {
    // Use dedicated TTS settings, falling back to main settings
    let api_key = if settings.tts_api_key.is_empty() {
        &settings.api_key
    } else {
        &settings.tts_api_key
    };
    let base = if settings.tts_base_url.is_empty() {
        settings.base_url.trim_end_matches('/').to_string()
    } else {
        settings.tts_base_url.trim_end_matches('/').to_string()
    };
    let api_version = if settings.tts_api_version.is_empty() {
        &settings.api_version
    } else {
        &settings.tts_api_version
    };

    if api_key.is_empty() {
        return Err("API key is not configured. Please set it in Settings.".to_string());
    }

    let is_azure = !api_version.is_empty();

    let url = if is_azure {
        format!(
            "{}/openai/deployments/{}/audio/speech?api-version={}",
            base, settings.tts_model, api_version
        )
    } else {
        format!("{}/audio/speech", base)
    };

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": settings.tts_model,
        "input": text,
        "voice": settings.tts_voice,
        "response_format": "wav",
    });

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body);

    if is_azure {
        req = req.header("api-key", api_key);
    } else {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = req
        .send()
        .await
        .map_err(|e| format!("TTS request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("TTS error ({}): {}", status, body));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read TTS response: {}", e))?;

    Ok(bytes.to_vec())
}
