<script lang="ts">
  import { onMount } from 'svelte';
  import { pickAndLoadDocument, listDocuments, removeDocument } from '$lib/tauri';
  import {
    documents,
    selectedDocumentIds,
    addDocument,
    removeDocumentFromStore,
    toggleDocumentSelection,
  } from '$lib/stores.svelte';

  let loading = $state(false);
  let error = $state('');

  onMount(async () => {
    try {
      const docs = await listDocuments();
      documents.length = 0;
      documents.push(...docs);
    } catch {
      // Store is empty on first launch
    }
  });

  async function handleAddDocument() {
    loading = true;
    error = '';
    try {
      const doc = await pickAndLoadDocument();
      addDocument(doc);
    } catch (e: any) {
      if (e?.message !== 'No file selected') {
        error = e?.message || 'Failed to load document';
      }
    } finally {
      loading = false;
    }
  }

  async function handleRemove(id: string) {
    try {
      await removeDocument(id);
      removeDocumentFromStore(id);
    } catch (e: any) {
      error = e?.message || 'Failed to remove document';
    }
  }

  function getFileIcon(name: string): string {
    if (name.endsWith('.pdf')) return 'M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z';
    if (name.endsWith('.docx')) return 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z';
    return 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z';
  }

  function getFileType(name: string): string {
    const ext = name.split('.').pop()?.toUpperCase() || '';
    return ext;
  }
</script>

<div class="documents-page">
  <div class="page-header">
    <div>
      <h1>Documents</h1>
      <p class="subtitle">Add study materials to chat about</p>
    </div>
    <button class="add-btn" onclick={handleAddDocument} disabled={loading}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 5v14m-7-7h14" />
      </svg>
      {loading ? 'Loading...' : 'Add Document'}
    </button>
  </div>

  {#if error}
    <div class="error-banner">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      {error}
    </div>
  {/if}

  {#if documents.length === 0}
    <div class="empty-state">
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
      </div>
      <h3>No documents yet</h3>
      <p>Add PDF, DOCX, TXT, or Markdown files to start studying</p>
    </div>
  {:else}
    <div class="document-grid">
      {#each documents as doc (doc.id)}
        <div class="document-card" class:selected={selectedDocumentIds.has(doc.id)}>
          <button class="card-select" onclick={() => toggleDocumentSelection(doc.id)} title={selectedDocumentIds.has(doc.id) ? 'Deselect' : 'Select for chat'}>
            <div class="checkbox" class:checked={selectedDocumentIds.has(doc.id)}>
              {#if selectedDocumentIds.has(doc.id)}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                  <path d="M5 13l4 4L19 7" />
                </svg>
              {/if}
            </div>
          </button>
          <div class="card-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d={getFileIcon(doc.name)} />
            </svg>
          </div>
          <div class="card-info">
            <span class="doc-name">{doc.name}</span>
            <span class="doc-type">{getFileType(doc.name)}</span>
          </div>
          <button class="remove-btn" onclick={() => handleRemove(doc.id)} title="Remove document">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .documents-page {
    max-width: 720px;
  }

  .page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 1.5rem;
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

  .add-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.55rem 1rem;
    background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    transition: all 0.15s ease;
    box-shadow: 0 1px 3px rgba(99, 102, 241, 0.3);
  }

  .add-btn svg {
    width: 16px;
    height: 16px;
  }

  .add-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
  }

  .add-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.2);
    color: #fca5a5;
    padding: 0.65rem 0.85rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    font-size: 0.85rem;
  }

  .error-banner svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    color: #ef4444;
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
    width: 64px;
    height: 64px;
    border-radius: 16px;
    background: rgba(99, 102, 241, 0.08);
    border: 1px solid rgba(99, 102, 241, 0.15);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 1.25rem;
  }

  .empty-icon svg {
    width: 32px;
    height: 32px;
    color: #6366f1;
  }

  .empty-state h3 {
    margin: 0 0 0.35rem;
    color: #e2e8f0;
    font-size: 1.05rem;
    font-weight: 600;
  }

  .empty-state p {
    margin: 0;
    color: #64748b;
    font-size: 0.85rem;
  }

  .document-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .document-card {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.65rem 0.85rem;
    background: rgba(148, 163, 184, 0.04);
    border-radius: 10px;
    border: 1px solid rgba(148, 163, 184, 0.08);
    transition: all 0.15s ease;
  }

  .document-card:hover {
    background: rgba(148, 163, 184, 0.07);
    border-color: rgba(148, 163, 184, 0.12);
  }

  .document-card.selected {
    background: rgba(99, 102, 241, 0.06);
    border-color: rgba(99, 102, 241, 0.2);
  }

  .card-select {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    flex-shrink: 0;
  }

  .checkbox {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    border: 1.5px solid #475569;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .checkbox.checked {
    background: #6366f1;
    border-color: #6366f1;
  }

  .checkbox svg {
    width: 12px;
    height: 12px;
    color: white;
  }

  .card-icon {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    background: rgba(99, 102, 241, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .card-icon svg {
    width: 16px;
    height: 16px;
    color: #818cf8;
  }

  .card-info {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .doc-name {
    font-weight: 500;
    color: #e2e8f0;
    font-size: 0.875rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .doc-type {
    font-size: 0.65rem;
    font-weight: 600;
    color: #64748b;
    background: rgba(148, 163, 184, 0.08);
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    letter-spacing: 0.03em;
    flex-shrink: 0;
  }

  .remove-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.3rem;
    border-radius: 6px;
    color: #475569;
    transition: all 0.15s ease;
    flex-shrink: 0;
    opacity: 0;
  }

  .document-card:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn svg {
    width: 16px;
    height: 16px;
  }

  .remove-btn:hover {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
  }
</style>
