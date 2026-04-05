export interface Document {
  id: string;
  name: string;
  content?: string;
}

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface VoiceModelStatus {
  whisper_downloaded: boolean;
}

export interface AppSettings {
  api_key: string;
  base_url: string;
  model: string;
  api_version: string;
  tts_api_key: string;
  tts_base_url: string;
  tts_api_version: string;
  tts_model: string;
  tts_voice: string;
}
