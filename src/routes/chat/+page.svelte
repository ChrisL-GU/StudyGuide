<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { sendChatStream, transcribeAudio, speakText } from '$lib/tauri';
  import { startRecording, stopRecording, playWavBytes, stopPlayback } from '$lib/audio';
  import {
    chatMessages,
    documents,
    selectedDocumentIds,
    clearChat,
  } from '$lib/stores.svelte';
  import { voiceState } from '$lib/voiceState.svelte';
  import type { ChatMessage } from '$lib/types';

  let input = $state('');
  let loading = $state(false);
  let error = $state('');
  let messagesContainer: HTMLDivElement;
  let textareaEl: HTMLTextAreaElement;
  let spaceHeld = false;

  function resizeTextarea() {
    if (textareaEl) {
      textareaEl.style.height = 'auto';
      textareaEl.style.height = textareaEl.scrollHeight + 'px';
    }
  }

  function scrollToBottom() {
    if (messagesContainer) {
      requestAnimationFrame(() => {
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
      });
    }
  }

  async function handleSend() {
    const text = input.trim();
    if (!text || loading) return;

    error = '';
    input = '';
    requestAnimationFrame(resizeTextarea);

    const userMessage: ChatMessage = { role: 'user', content: text };
    chatMessages.push(userMessage);
    scrollToBottom();

    loading = true;
    try {
      const selectedIds = Array.from(selectedDocumentIds);
      const conversationHistory = chatMessages.map((m) => ({
        role: m.role,
        content: m.content,
      }));

      const assistantMessage: ChatMessage = { role: 'assistant', content: '' };
      chatMessages.push(assistantMessage);
      const msgIndex = chatMessages.length - 1;
      scrollToBottom();

      const fullText = await sendChatStream(
        conversationHistory,
        selectedIds,
        (chunk) => {
          chatMessages[msgIndex] = {
            ...chatMessages[msgIndex],
            content: chatMessages[msgIndex].content + chunk,
          };
          scrollToBottom();
        }
      );

      chatMessages[msgIndex] = { ...chatMessages[msgIndex], content: fullText };

      if (voiceState.ttsEnabled) {
        handleSpeak(fullText);
      }
    } catch (e: any) {
      error = e?.toString() || 'Failed to get response';
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  async function beginRecording() {
    if (voiceState.isRecording || voiceState.isTranscribing || loading) return;
    if (voiceState.isSpeaking) {
      stopPlayback();
      voiceState.isSpeaking = false;
    }
    try {
      await startRecording();
      voiceState.isRecording = true;
    } catch (e: any) {
      error = e?.toString() || 'Failed to start recording';
    }
  }

  async function endRecording() {
    if (!voiceState.isRecording) return;
    voiceState.isRecording = false;
    voiceState.isTranscribing = true;
    error = '';
    try {
      const audioData = await stopRecording();
      const text = await transcribeAudio(audioData);
      if (text) {
        input = input.trim() ? input.trimEnd() + '\n\n' + text : text;
        requestAnimationFrame(resizeTextarea);
      }
    } catch (e: any) {
      error = e?.toString() || 'Transcription failed';
    } finally {
      voiceState.isTranscribing = false;
    }
  }

  async function toggleRecording() {
    if (voiceState.isRecording) {
      await endRecording();
    } else {
      await beginRecording();
    }
  }

  // Global keyboard shortcuts: hold Space to record, Alt+Space to send
  function onGlobalKeydown(e: KeyboardEvent) {
    // Ignore if typing in an input/textarea (unless it's the space shortcut with no focus)
    const target = e.target as HTMLElement;
    const isTyping = target.tagName === 'TEXTAREA' || target.tagName === 'INPUT';

    if (e.key === 'Enter' && !e.shiftKey && !isTyping) {
      e.preventDefault();
      handleSend();
      return;
    }

    if (e.code === 'Space' && !e.altKey && !e.ctrlKey && !e.metaKey && !isTyping) {
      // Hold Space (not in textarea): start recording
      if (e.repeat) return; // ignore key repeat
      e.preventDefault();
      spaceHeld = true;
      beginRecording();
    }
  }

  function onGlobalKeyup(e: KeyboardEvent) {
    if (e.code === 'Space' && spaceHeld) {
      e.preventDefault();
      spaceHeld = false;
      endRecording();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', onGlobalKeydown);
    window.addEventListener('keyup', onGlobalKeyup);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', onGlobalKeydown);
    window.removeEventListener('keyup', onGlobalKeyup);
  });

  async function handleSpeak(text: string) {
    if (voiceState.isSpeaking) {
      stopPlayback();
      voiceState.isSpeaking = false;
      return;
    }
    voiceState.isSpeaking = true;
    try {
      const wavBytes = await speakText(text);
      await playWavBytes(wavBytes);
    } catch (e: any) {
      error = e?.toString() || 'TTS failed';
    } finally {
      voiceState.isSpeaking = false;
    }
  }

  const selectedCount = $derived(selectedDocumentIds.size);
  const selectedNames = $derived(
    documents
      .filter((d) => selectedDocumentIds.has(d.id))
      .map((d) => d.name)
  );
</script>

<div class="chat-page">
  <div class="chat-header">
    <div class="header-left">
      <h1>Chat</h1>
      {#if selectedCount > 0}
        <span class="doc-badge">{selectedCount} doc{selectedCount > 1 ? 's' : ''}</span>
      {/if}
    </div>
    <div class="header-controls">
      <button
        class="toggle-btn"
        class:active={voiceState.ttsEnabled}
        onclick={() => voiceState.ttsEnabled = !voiceState.ttsEnabled}
        title="Toggle read aloud"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          {#if voiceState.ttsEnabled}
            <path d="M15.536 8.464a5 5 0 010 7.072M17.95 6.05a8 8 0 010 11.9M11 5L6 9H2v6h4l5 4V5z" />
          {:else}
            <path d="M11 5L6 9H2v6h4l5 4V5zM23 9l-6 6M17 9l6 6" />
          {/if}
        </svg>
      </button>
      {#if chatMessages.length > 0}
        <button class="clear-btn" onclick={clearChat} title="Clear chat">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      {/if}
    </div>
  </div>

  {#if selectedCount > 0}
    <div class="context-bar">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
      </svg>
      Studying: {selectedNames.join(', ')}
    </div>
  {:else if documents.length > 0}
    <div class="context-bar warning">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
      </svg>
      No documents selected. Go to Documents to select study materials.
    </div>
  {:else}
    <div class="context-bar warning">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
      </svg>
      No documents loaded. Go to Documents to add study materials.
    </div>
  {/if}

  {#if error}
    <div class="error-banner">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      {error}
    </div>
  {/if}

  <div class="messages" bind:this={messagesContainer}>
    {#if chatMessages.length === 0}
      <div class="empty-state">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <path d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
        </div>
        <h3>Start a conversation</h3>
        <p>Ask questions about your study materials.<br />Your tutor will guide you with the Socratic method.</p>
      </div>
    {/if}

    {#each chatMessages as msg, i (i)}
      <div class="message {msg.role}">
        <div class="message-avatar">
          {#if msg.role === 'user'}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
            </svg>
          {/if}
        </div>
        <div class="message-body">
          <div class="message-meta">
            <span class="message-role">{msg.role === 'user' ? 'You' : 'Tutor'}</span>
            {#if msg.role === 'assistant' && msg.content}
              <button
                class="speak-btn"
                onclick={() => handleSpeak(msg.content)}
                title={voiceState.isSpeaking ? 'Stop' : 'Read aloud'}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  {#if voiceState.isSpeaking}
                    <rect x="6" y="4" width="4" height="16" /><rect x="14" y="4" width="4" height="16" />
                  {:else}
                    <path d="M15.536 8.464a5 5 0 010 7.072M11 5L6 9H2v6h4l5 4V5z" />
                  {/if}
                </svg>
              </button>
            {/if}
          </div>
          <div class="message-content">
            {#if msg.role === 'assistant' && !msg.content && loading}
              <span class="typing-indicator">
                <span></span><span></span><span></span>
              </span>
            {:else}
              {msg.content}
            {/if}
          </div>
        </div>
      </div>
    {/each}
  </div>

  <div class="input-area">
    <div class="input-wrapper">
      <button
        class="mic-btn"
        class:recording={voiceState.isRecording}
        class:transcribing={voiceState.isTranscribing}
        onclick={toggleRecording}
        disabled={voiceState.isTranscribing || loading}
        title={voiceState.isRecording ? 'Stop recording' : 'Start recording'}
      >
        {#if voiceState.isTranscribing}
          <span class="typing-indicator small"><span></span><span></span><span></span></span>
        {:else if voiceState.isRecording}
          <svg viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="2" /></svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 10v2a7 7 0 01-14 0v-2M12 19v3m-3 0h6M12 1a3 3 0 00-3 3v6a3 3 0 006 0V4a3 3 0 00-3-3z" />
          </svg>
        {/if}
      </button>
      <textarea
        bind:this={textareaEl}
        bind:value={input}
        onkeydown={handleKeydown}
        oninput={resizeTextarea}
        placeholder="Ask about your study materials..."
        disabled={loading}
        rows="1"
      ></textarea>
      <button class="send-btn" onclick={handleSend} disabled={loading || !input.trim()} title="Send message">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z" />
        </svg>
      </button>
    </div>
  </div>
</div>

<style>
  .chat-page {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 3rem);
    max-width: 800px;
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: #f1f5f9;
    letter-spacing: -0.02em;
  }

  .doc-badge {
    font-size: 0.7rem;
    font-weight: 600;
    color: #a5b4fc;
    background: rgba(99, 102, 241, 0.12);
    padding: 0.15rem 0.5rem;
    border-radius: 10px;
    letter-spacing: 0.01em;
  }

  .header-controls {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .toggle-btn,
  .clear-btn {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: none;
    background: rgba(148, 163, 184, 0.06);
    color: #64748b;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .toggle-btn svg,
  .clear-btn svg {
    width: 16px;
    height: 16px;
  }

  .toggle-btn:hover,
  .clear-btn:hover {
    background: rgba(148, 163, 184, 0.12);
    color: #e2e8f0;
  }

  .toggle-btn.active {
    background: rgba(99, 102, 241, 0.15);
    color: #818cf8;
  }

  .context-bar {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.5rem 0.75rem;
    background: rgba(99, 102, 241, 0.06);
    border: 1px solid rgba(99, 102, 241, 0.12);
    border-radius: 8px;
    font-size: 0.8rem;
    color: #a5b4fc;
    margin-bottom: 0.5rem;
  }

  .context-bar svg {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  .context-bar.warning {
    background: rgba(245, 158, 11, 0.06);
    border-color: rgba(245, 158, 11, 0.12);
    color: #fbbf24;
  }

  .context-bar.warning svg {
    color: #f59e0b;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    padding: 0.55rem 0.75rem;
    border-radius: 8px;
    margin-bottom: 0.5rem;
    font-size: 0.8rem;
  }

  .error-banner svg {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    color: #ef4444;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.5rem 0;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 1rem;
    text-align: center;
  }

  .empty-icon {
    width: 56px;
    height: 56px;
    border-radius: 14px;
    background: rgba(99, 102, 241, 0.08);
    border: 1px solid rgba(99, 102, 241, 0.12);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 1rem;
  }

  .empty-icon svg {
    width: 28px;
    height: 28px;
    color: #6366f1;
  }

  .empty-state h3 {
    margin: 0 0 0.3rem;
    color: #e2e8f0;
    font-size: 1rem;
    font-weight: 600;
  }

  .empty-state p {
    margin: 0;
    color: #64748b;
    font-size: 0.825rem;
    line-height: 1.6;
  }

  .message {
    display: flex;
    gap: 0.65rem;
    padding: 0.75rem 0.5rem;
    border-radius: 10px;
    transition: background 0.1s ease;
  }

  .message:hover {
    background: rgba(148, 163, 184, 0.03);
  }

  .message-avatar {
    width: 30px;
    height: 30px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .message-avatar svg {
    width: 16px;
    height: 16px;
  }

  .message.user .message-avatar {
    background: rgba(99, 102, 241, 0.12);
    color: #818cf8;
  }

  .message.assistant .message-avatar {
    background: rgba(16, 185, 129, 0.1);
    color: #34d399;
  }

  .message-body {
    flex: 1;
    min-width: 0;
  }

  .message-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.2rem;
  }

  .message-role {
    font-size: 0.75rem;
    font-weight: 600;
    color: #64748b;
  }

  .speak-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    color: #475569;
    transition: all 0.15s ease;
    opacity: 0;
  }

  .message:hover .speak-btn {
    opacity: 1;
  }

  .speak-btn svg {
    width: 14px;
    height: 14px;
  }

  .speak-btn:hover {
    color: #818cf8;
    background: rgba(99, 102, 241, 0.1);
  }

  .message-content {
    color: #cbd5e1;
    font-size: 0.875rem;
    line-height: 1.65;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .typing-indicator {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }

  .typing-indicator span {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #64748b;
    animation: typing 1.4s infinite;
  }

  .typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
  .typing-indicator span:nth-child(3) { animation-delay: 0.4s; }

  .typing-indicator.small span {
    width: 3px;
    height: 3px;
  }

  @keyframes typing {
    0%, 60%, 100% { opacity: 0.3; transform: scale(1); }
    30% { opacity: 1; transform: scale(1.2); }
  }

  .input-area {
    padding-top: 0.75rem;
  }

  .input-wrapper {
    display: flex;
    align-items: flex-end;
    gap: 0;
    background: rgba(148, 163, 184, 0.05);
    border: 1px solid rgba(148, 163, 184, 0.1);
    border-radius: 12px;
    padding: 0.35rem;
    transition: border-color 0.15s ease;
  }

  .input-wrapper:focus-within {
    border-color: rgba(99, 102, 241, 0.3);
  }

  .mic-btn {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    border: none;
    background: transparent;
    color: #64748b;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all 0.15s ease;
  }

  .mic-btn svg {
    width: 18px;
    height: 18px;
  }

  .mic-btn:hover {
    background: rgba(148, 163, 184, 0.1);
    color: #e2e8f0;
  }

  .mic-btn.recording {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
    animation: pulse 2s infinite;
  }

  .mic-btn.transcribing {
    opacity: 0.6;
    cursor: wait;
  }

  .mic-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  @keyframes pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.2); }
    50% { box-shadow: 0 0 0 6px rgba(239, 68, 68, 0); }
  }

  textarea {
    flex: 1;
    padding: 0.45rem 0.5rem;
    border: none;
    background: transparent;
    resize: none;
    font-family: inherit;
    font-size: 0.875rem;
    line-height: 1.5;
    color: #e2e8f0;
    outline: none;
    min-height: 34px;
    max-height: 120px;
  }

  textarea::placeholder {
    color: #475569;
  }

  .send-btn {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    border: none;
    background: #6366f1;
    color: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all 0.15s ease;
  }

  .send-btn svg {
    width: 15px;
    height: 15px;
  }

  .send-btn:hover {
    background: #4f46e5;
    transform: translateY(-1px);
  }

  .send-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
    transform: none;
  }
</style>
