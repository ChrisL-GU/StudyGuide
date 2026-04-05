import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type { Document, ChatMessage, AppSettings, VoiceModelStatus } from './types';

export async function pickAndLoadDocument(): Promise<Document> {
  const path = await open({
    multiple: false,
    filters: [
      {
        name: 'Documents',
        extensions: ['pdf', 'docx', 'txt', 'md'],
      },
    ],
  });

  if (!path) {
    throw new Error('No file selected');
  }

  return await invoke<Document>('load_document', { path });
}

export async function listDocuments(): Promise<Document[]> {
  return await invoke<Document[]>('list_documents');
}

export async function removeDocument(id: string): Promise<void> {
  await invoke('remove_document', { id });
}

export async function getSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>('get_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  await invoke('save_settings', { settings });
}

export async function sendChatStream(
  messages: ChatMessage[],
  selectedDocIds: string[],
  onChunk: (chunk: string) => void
): Promise<string> {
  const unlistenChunk: UnlistenFn = await listen<string>('chat-stream-chunk', (event) => {
    onChunk(event.payload);
  });

  try {
    const fullText = await invoke<string>('chat', {
      messages,
      selectedDocIds,
    });
    return fullText;
  } finally {
    unlistenChunk();
  }
}

export async function getVoiceModelStatus(): Promise<VoiceModelStatus> {
  return await invoke<VoiceModelStatus>('get_voice_model_status');
}

export async function downloadVoiceModels(): Promise<void> {
  await invoke('download_voice_models');
}

export async function transcribeAudio(audioData: Uint8Array): Promise<string> {
  return await invoke<string>('transcribe', {
    audioData: Array.from(audioData),
  });
}

export async function speakText(text: string): Promise<Uint8Array> {
  const data = await invoke<number[]>('speak', { text });
  return new Uint8Array(data);
}
