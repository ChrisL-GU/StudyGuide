import type { Document, ChatMessage, AppSettings } from './types';

export let documents = $state<Document[]>([]);
export let chatMessages = $state<ChatMessage[]>([]);
export let selectedDocumentIds = $state<Set<string>>(new Set());

export let settings = $state<AppSettings>({
  api_key: '',
  base_url: 'https://api.openai.com/v1',
  model: 'gpt-4o',
  api_version: '',
  tts_api_key: '',
  tts_base_url: '',
  tts_api_version: '',
  tts_model: 'tts',
  tts_voice: 'alloy',
});

export function addDocument(doc: Document) {
  documents.push(doc);
  selectedDocumentIds.add(doc.id);
}

export function removeDocumentFromStore(id: string) {
  const idx = documents.findIndex((d) => d.id === id);
  if (idx !== -1) documents.splice(idx, 1);
  selectedDocumentIds.delete(id);
}

export function toggleDocumentSelection(id: string) {
  if (selectedDocumentIds.has(id)) {
    selectedDocumentIds.delete(id);
  } else {
    selectedDocumentIds.add(id);
  }
}

export function clearChat() {
  chatMessages.length = 0;
}
