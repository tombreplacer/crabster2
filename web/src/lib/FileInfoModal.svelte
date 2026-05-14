<script>
  import { onMount, onDestroy } from 'svelte';
  import { downloadUrl, formatSize, formatDate, getFileIconType } from './api.js';
  import FileIcon from './FileIcon.svelte';

  export let entry;
  export let filePath;
  export let onClose;
  export let onDelete = null;
  export let readonly = false;
  export let noDelete = false;

  $: dlUrl = downloadUrl(filePath);

  function handleKeydown(e) {
    if (e.key === 'Escape') onClose();
  }

  function handleOverlayClick(e) {
    if (e.target === e.currentTarget) onClose();
  }

  function copyPath() {
    navigator.clipboard?.writeText(filePath);
    copied = true;
    setTimeout(() => copied = false, 1500);
  }

  let copied = false;

  onMount(() => {
    document.addEventListener('keydown', handleKeydown);
    document.body.style.overflow = 'hidden';
  });

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeydown);
    document.body.style.overflow = '';
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="info-overlay" on:click={handleOverlayClick}>
  <div class="info-container">
    <button class="close-btn" on:click={onClose} title="Close">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
    </button>

    <div class="info-icon">
      <FileIcon type={getFileIconType(entry)} size={64} />
    </div>

    <h2 class="info-name">{entry.name}</h2>

    <div class="info-table">
      <div class="info-row">
        <span class="info-label">Type</span>
        <span class="info-value"><code>{entry.mime_type}</code></span>
      </div>
      <div class="info-row">
        <span class="info-label">Size</span>
        <span class="info-value">{formatSize(entry.size)}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Modified</span>
        <span class="info-value">{formatDate(entry.modified)}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Permissions</span>
        <span class="info-value"><code>{entry.permissions}</code></span>
      </div>
      {#if entry.owner && entry.owner !== 'N/A'}
        <div class="info-row">
          <span class="info-label">Owner</span>
          <span class="info-value">{entry.owner}{entry.group && entry.group !== 'N/A' ? ':' + entry.group : ''}</span>
        </div>
      {/if}
      <div class="info-row">
        <span class="info-label">Path</span>
        <span class="info-value path-value">
          <code>{filePath}</code>
          <button class="copy-btn" on:click={copyPath} title="Copy path">
            {#if copied}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg>
            {/if}
          </button>
        </span>
      </div>
    </div>

    <div class="info-actions">
      <a class="btn btn-primary" href={dlUrl} download={entry.name}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        Download
      </a>
      {#if !readonly && !noDelete && onDelete}
        <button class="btn btn-danger" on:click={() => { onDelete(entry); onClose(); }}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>
          Delete
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .info-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 5000;
    backdrop-filter: blur(8px);
    animation: fadeIn 0.15s ease;
    padding: 20px;
  }

  .info-container {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 32px;
    max-width: 460px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    position: relative;
    animation: slideUp 0.2s ease;
    box-shadow: 0 25px 80px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .close-btn {
    position: absolute;
    top: 12px;
    right: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--text-muted);
    transition: all 0.15s;
  }

  .close-btn:hover {
    color: var(--danger);
    background: rgba(244, 63, 94, 0.1);
  }

  .info-icon {
    margin-bottom: 16px;
    opacity: 0.8;
  }

  .info-name {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
    word-break: break-all;
    margin-bottom: 24px;
    max-width: 100%;
  }

  .info-table {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    margin-bottom: 24px;
  }

  .info-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    background: var(--bg-primary);
    gap: 12px;
  }

  .info-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    flex-shrink: 0;
  }

  .info-value {
    font-size: 13px;
    color: var(--text-primary);
    text-align: right;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .info-value code {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.04);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .path-value {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .copy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 4px;
    background: none;
    color: var(--text-muted);
    transition: all 0.15s;
    flex-shrink: 0;
  }

  .copy-btn:hover {
    background: var(--bg-card-hover);
    color: var(--text-primary);
  }

  .info-actions {
    display: flex;
    gap: 10px;
    width: 100%;
  }

  .btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 10px 20px;
    border-radius: var(--radius-sm);
    font-size: 14px;
    font-weight: 500;
    transition: all 0.15s;
    text-decoration: none;
    flex: 1;
  }

  .btn-primary {
    background: linear-gradient(135deg, #ff6b35, #ff8c42);
    color: #fff;
  }

  .btn-primary:hover {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  .btn-danger {
    background: rgba(244, 63, 94, 0.1);
    color: var(--danger);
    border: 1px solid rgba(244, 63, 94, 0.2);
  }

  .btn-danger:hover {
    background: rgba(244, 63, 94, 0.2);
    transform: translateY(-1px);
  }

  @media (max-width: 768px) {
    .info-container { padding: 24px 20px; }
    .info-name { font-size: 16px; }
  }
</style>
