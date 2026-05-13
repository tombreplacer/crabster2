<script>
  import { onMount, onDestroy } from 'svelte';
  import { previewUrl, downloadUrl, getPreviewType, formatSize } from './api.js';

  export let entry;
  export let filePath;
  export let onClose;

  let textContent = '';
  let textLoading = false;
  let textError = '';

  $: type = getPreviewType(entry);
  $: url = previewUrl(filePath);
  $: dlUrl = downloadUrl(filePath);

  function handleKeydown(e) {
    if (e.key === 'Escape') onClose();
  }

  function handleOverlayClick(e) {
    if (e.target === e.currentTarget) onClose();
  }

  async function loadText() {
    textLoading = true;
    textError = '';
    try {
      const res = await fetch(url);
      if (!res.ok) throw new Error(res.statusText);
      const text = await res.text();
      // Limit display to 1MB of text
      textContent = text.length > 1_000_000 ? text.slice(0, 1_000_000) + '\n\n... (truncated)' : text;
    } catch (e) {
      textError = e.message;
    }
    textLoading = false;
  }

  onMount(() => {
    document.addEventListener('keydown', handleKeydown);
    if (type === 'text') loadText();
    // Prevent body scroll
    document.body.style.overflow = 'hidden';
  });

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeydown);
    document.body.style.overflow = '';
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="preview-overlay" on:click={handleOverlayClick}>
  <div class="preview-container">
    <div class="preview-header">
      <div class="preview-title">
        <span class="preview-name">{entry.name}</span>
        <span class="preview-meta">{entry.mime_type} · {formatSize(entry.size)}</span>
      </div>
      <div class="preview-actions">
        <a class="preview-btn" href={dlUrl} download={entry.name} title="Download">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        </a>
        <button class="preview-btn close-btn" on:click={onClose} title="Close">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    </div>
    <div class="preview-body">
      {#if type === 'image'}
        <img src={url} alt={entry.name} class="preview-image" />
      {:else if type === 'video'}
        <video controls autoplay class="preview-video" preload="metadata">
          <source src={url} type={entry.mime_type} />
          Your browser does not support video playback.
        </video>
      {:else if type === 'audio'}
        <div class="preview-audio-wrap">
          <div class="audio-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
          </div>
          <audio controls autoplay preload="metadata" style="width:100%;max-width:500px">
            <source src={url} type={entry.mime_type} />
          </audio>
        </div>
      {:else if type === 'pdf'}
        <iframe src={url} class="preview-pdf" title={entry.name}></iframe>
      {:else if type === 'text'}
        {#if textLoading}
          <div class="preview-loading"><div class="spinner"></div> Loading...</div>
        {:else if textError}
          <div class="preview-error">Failed to load: {textError}</div>
        {:else}
          <pre class="preview-text"><code>{textContent}</code></pre>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .preview-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 5000;
    backdrop-filter: blur(8px);
    animation: fadeIn 0.15s ease;
    padding: 20px;
  }

  .preview-container {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    max-width: 95vw;
    max-height: 95vh;
    width: 100%;
    overflow: hidden;
    animation: slideUp 0.2s ease;
    box-shadow: 0 25px 80px rgba(0, 0, 0, 0.6);
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .preview-title {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .preview-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .preview-meta {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--mono);
  }

  .preview-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .preview-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    background: var(--bg-card);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    transition: all 0.15s;
    text-decoration: none;
    cursor: pointer;
  }

  .preview-btn:hover {
    background: var(--bg-card-hover);
    color: var(--text-primary);
    border-color: var(--border-hover);
  }

  .close-btn:hover {
    color: var(--danger);
    border-color: rgba(244, 63, 94, 0.3);
    background: rgba(244, 63, 94, 0.1);
  }

  .preview-body {
    flex: 1;
    overflow: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 200px;
    background: var(--bg-primary);
  }

  .preview-image {
    max-width: 100%;
    max-height: calc(95vh - 70px);
    object-fit: contain;
    display: block;
  }

  .preview-video {
    max-width: 100%;
    max-height: calc(95vh - 70px);
    outline: none;
    background: #000;
  }

  .preview-audio-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    padding: 60px 40px;
  }

  .audio-icon {
    opacity: 0.6;
  }

  .preview-pdf {
    width: 100%;
    height: calc(95vh - 70px);
    border: none;
    background: #fff;
  }

  .preview-text {
    width: 100%;
    height: calc(95vh - 70px);
    margin: 0;
    padding: 20px 24px;
    overflow: auto;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--mono);
    font-size: 13px;
    line-height: 1.6;
    white-space: pre-wrap;
    word-wrap: break-word;
    tab-size: 4;
  }

  .preview-text code {
    font-family: inherit;
    color: inherit;
  }

  .preview-loading {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text-secondary);
    font-size: 14px;
    padding: 60px;
  }

  .preview-error {
    color: var(--danger);
    padding: 40px;
    font-size: 14px;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @media (max-width: 768px) {
    .preview-overlay { padding: 8px; }
    .preview-header { padding: 10px 14px; }
    .preview-name { font-size: 13px; }
  }
</style>
