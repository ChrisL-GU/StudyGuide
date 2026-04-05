<script lang="ts">
  import { onMount } from 'svelte';
  import { getSettings, saveSettings, getVoiceModelStatus, downloadVoiceModels } from '$lib/tauri';
  import { settings } from '$lib/stores.svelte';
  import { voiceState } from '$lib/voiceState.svelte';
  import type { VoiceModelStatus } from '$lib/types';

  let saving = $state(false);
  let message = $state('');
  let messageType = $state<'success' | 'error'>('success');
  let voiceStatus = $state<VoiceModelStatus | null>(null);
  let downloading = $state(false);
  let ttsExpanded = $state(false);

  onMount(async () => {
    try {
      const loaded = await getSettings();
      settings.api_key = loaded.api_key;
      settings.base_url = loaded.base_url;
      settings.model = loaded.model;
      settings.api_version = loaded.api_version ?? '';
      settings.tts_api_key = loaded.tts_api_key ?? '';
      settings.tts_base_url = loaded.tts_base_url ?? '';
      settings.tts_api_version = loaded.tts_api_version ?? '';
      settings.tts_model = loaded.tts_model ?? 'tts';
      settings.tts_voice = loaded.tts_voice ?? 'alloy';
      // Auto-expand TTS section if TTS has custom settings
      if (settings.tts_api_key || settings.tts_base_url || settings.tts_api_version) {
        ttsExpanded = true;
      }
    } catch {
      // Use defaults
    }
    try {
      voiceStatus = await getVoiceModelStatus();
      voiceState.modelsReady = !!voiceStatus?.whisper_downloaded;
    } catch {
      // Voice not available
    }
  });

  async function handleDownloadModels() {
    downloading = true;
    message = '';
    try {
      await downloadVoiceModels();
      voiceStatus = await getVoiceModelStatus();
      voiceState.modelsReady = !!voiceStatus?.whisper_downloaded;
      message = 'Voice models downloaded successfully.';
      messageType = 'success';
    } catch (e: any) {
      message = e?.toString() || 'Failed to download models';
      messageType = 'error';
    } finally {
      downloading = false;
    }
  }

  async function handleSave() {
    saving = true;
    message = '';
    try {
      await saveSettings({
        api_key: settings.api_key,
        base_url: settings.base_url,
        model: settings.model,
        api_version: settings.api_version,
        tts_api_key: settings.tts_api_key,
        tts_base_url: settings.tts_base_url,
        tts_api_version: settings.tts_api_version,
        tts_model: settings.tts_model,
        tts_voice: settings.tts_voice,
      });
      message = 'Settings saved.';
      messageType = 'success';
    } catch (e: any) {
      message = e?.message || 'Failed to save settings';
      messageType = 'error';
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings-page">
  <div class="page-header">
    <h1>Settings</h1>
    <p class="subtitle">Configure your API and voice preferences</p>
  </div>

  <!-- Chat API Section -->
  <div class="section">
    <div class="section-header">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
      </svg>
      <h2>Chat API</h2>
    </div>

    <div class="field-group">
      <label class="field">
        <span class="field-label">API Key</span>
        <input type="password" bind:value={settings.api_key} placeholder="sk-..." />
      </label>

      <label class="field">
        <span class="field-label">Base URL</span>
        <input type="text" bind:value={settings.base_url} placeholder="https://api.openai.com/v1 or Azure endpoint" />
      </label>

      <div class="field-row">
        <label class="field">
          <span class="field-label">Model / Deployment</span>
          <input type="text" bind:value={settings.model} placeholder="gpt-4o" />
        </label>

        <label class="field">
          <span class="field-label">API Version <span class="hint">Azure only</span></span>
          <input type="text" bind:value={settings.api_version} placeholder="leave blank for OpenAI" />
        </label>
      </div>
    </div>
  </div>

  <!-- TTS Section -->
  <div class="section">
    <div class="section-header">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M15.536 8.464a5 5 0 010 7.072M17.95 6.05a8 8 0 010 11.9M11 5L6 9H2v6h4l5 4V5z" />
      </svg>
      <h2>Text-to-Speech</h2>
    </div>

    <div class="field-group">
      <div class="field-row">
        <label class="field">
          <span class="field-label">TTS Model / Deployment</span>
          <input type="text" bind:value={settings.tts_model} placeholder="tts" />
        </label>

        <label class="field">
          <span class="field-label">Voice</span>
          <div class="select-wrapper">
            <select bind:value={settings.tts_voice}>
              <option value="alloy">Alloy</option>
              <option value="echo">Echo</option>
              <option value="fable">Fable</option>
              <option value="nova">Nova</option>
              <option value="onyx">Onyx</option>
              <option value="shimmer">Shimmer</option>
            </select>
          </div>
        </label>
      </div>

      <button class="expand-btn" onclick={() => ttsExpanded = !ttsExpanded}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={ttsExpanded}>
          <path d="M9 5l7 7-7 7" />
        </svg>
        TTS-specific endpoint (if different from Chat API)
      </button>

      {#if ttsExpanded}
        <div class="expandable">
          <label class="field">
            <span class="field-label">TTS API Key</span>
            <input type="password" bind:value={settings.tts_api_key} placeholder="leave blank to use main API Key" />
          </label>

          <label class="field">
            <span class="field-label">TTS Base URL</span>
            <input type="text" bind:value={settings.tts_base_url} placeholder="leave blank to use main Base URL" />
          </label>

          <label class="field">
            <span class="field-label">TTS API Version</span>
            <input type="text" bind:value={settings.tts_api_version} placeholder="leave blank to use main API Version" />
          </label>
        </div>
      {/if}
    </div>
  </div>

  <!-- STT Section -->
  <div class="section">
    <div class="section-header">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M19 10v2a7 7 0 01-14 0v-2M12 19v3m-3 0h6M12 1a3 3 0 00-3 3v6a3 3 0 006 0V4a3 3 0 00-3-3z" />
      </svg>
      <h2>Speech-to-Text</h2>
    </div>

    <div class="field-group">
      <div class="model-card">
        <div class="model-info">
          <span class="model-name">Whisper</span>
          <span class="model-desc">Local speech recognition model (~142MB)</span>
        </div>
        <div class="model-status" class:ready={voiceStatus?.whisper_downloaded}>
          {#if voiceStatus?.whisper_downloaded}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M5 13l4 4L19 7" />
            </svg>
            Ready
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 8v4m0 4h.01" />
            </svg>
            Not installed
          {/if}
        </div>
      </div>

      {#if !voiceStatus?.whisper_downloaded}
        <button class="action-btn" onclick={handleDownloadModels} disabled={downloading}>
          {#if downloading}
            <span class="spinner"></span>
            Downloading...
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Download Whisper Model
          {/if}
        </button>
      {/if}
    </div>
  </div>

  <!-- Save -->
  {#if message}
    <div class="toast {messageType}">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        {#if messageType === 'success'}
          <path d="M5 13l4 4L19 7" />
        {:else}
          <path d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        {/if}
      </svg>
      {message}
    </div>
  {/if}

  <button class="save-btn" onclick={handleSave} disabled={saving}>
    {#if saving}
      <span class="spinner"></span>
      Saving...
    {:else}
      Save Settings
    {/if}
  </button>
</div>

<style>
  .settings-page {
    max-width: 560px;
  }

  .page-header {
    margin-bottom: 1.75rem;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: #f1f5f9;
    letter-spacing: -0.02em;
  }

  .subtitle {
    margin: 0.25rem 0 0;
    color: #64748b;
    font-size: 0.85rem;
  }

  .section {
    margin-bottom: 1.5rem;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .section-header svg {
    width: 16px;
    height: 16px;
    color: #818cf8;
  }

  h2 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: #e2e8f0;
    letter-spacing: -0.01em;
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    background: rgba(148, 163, 184, 0.04);
    border: 1px solid rgba(148, 163, 184, 0.08);
    border-radius: 12px;
    padding: 0.85rem;
  }

  .field-row {
    display: flex;
    gap: 0.65rem;
  }

  .field-row .field {
    flex: 1;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .field-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: #94a3b8;
    letter-spacing: 0.01em;
  }

  .hint {
    font-weight: 400;
    color: #475569;
  }

  input,
  select {
    padding: 0.5rem 0.65rem;
    border: 1px solid rgba(148, 163, 184, 0.12);
    border-radius: 7px;
    font-size: 0.825rem;
    font-family: inherit;
    background: rgba(15, 23, 42, 0.5);
    color: #e2e8f0;
    transition: all 0.15s ease;
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: rgba(99, 102, 241, 0.4);
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.08);
  }

  input::placeholder {
    color: #475569;
  }

  .select-wrapper {
    position: relative;
  }

  select {
    width: 100%;
    cursor: pointer;
    appearance: none;
    padding-right: 2rem;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%2364748b' stroke-width='2'%3E%3Cpath d='M19 9l-7 7-7-7'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.5rem center;
    background-size: 14px;
  }

  .expand-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0;
    background: none;
    border: none;
    color: #64748b;
    cursor: pointer;
    font-size: 0.75rem;
    font-family: inherit;
    transition: color 0.15s ease;
  }

  .expand-btn:hover {
    color: #94a3b8;
  }

  .expand-btn svg {
    width: 12px;
    height: 12px;
    transition: transform 0.2s ease;
  }

  .expand-btn svg.rotated {
    transform: rotate(90deg);
  }

  .expandable {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    padding-top: 0.25rem;
    border-top: 1px solid rgba(148, 163, 184, 0.06);
  }

  .model-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .model-info {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .model-name {
    font-size: 0.85rem;
    font-weight: 600;
    color: #e2e8f0;
  }

  .model-desc {
    font-size: 0.725rem;
    color: #64748b;
  }

  .model-status {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: #f59e0b;
  }

  .model-status svg {
    width: 14px;
    height: 14px;
  }

  .model-status.ready {
    color: #34d399;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    padding: 0.5rem 1rem;
    background: rgba(99, 102, 241, 0.1);
    border: 1px solid rgba(99, 102, 241, 0.2);
    border-radius: 8px;
    color: #a5b4fc;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 500;
    font-family: inherit;
    transition: all 0.15s ease;
  }

  .action-btn svg {
    width: 15px;
    height: 15px;
  }

  .action-btn:hover {
    background: rgba(99, 102, 241, 0.15);
    border-color: rgba(99, 102, 241, 0.3);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.55rem 0.75rem;
    border-radius: 8px;
    font-size: 0.8rem;
    margin-bottom: 0.75rem;
  }

  .toast svg {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  .toast.success {
    background: rgba(16, 185, 129, 0.08);
    border: 1px solid rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .toast.error {
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.15);
    color: #fca5a5;
  }

  .save-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    padding: 0.6rem 1.5rem;
    background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    font-family: inherit;
    transition: all 0.15s ease;
    box-shadow: 0 1px 3px rgba(99, 102, 241, 0.3);
  }

  .save-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
