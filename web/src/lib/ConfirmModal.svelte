<script>
  import { onMount, onDestroy } from 'svelte';

  export let title = 'Confirm';
  export let message = 'Are you sure?';
  export let confirmText = 'Delete';
  export let cancelText = 'Cancel';
  export let danger = true;
  export let onConfirm;
  export let onCancel;

  function handleKeydown(e) {
    if (e.key === 'Escape') onCancel();
  }

  function handleOverlayClick(e) {
    if (e.target === e.currentTarget) onCancel();
  }

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
<div class="confirm-overlay" on:click={handleOverlayClick}>
  <div class="confirm-box">
    <div class="confirm-icon" class:danger>
      {#if danger}
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
          <line x1="12" y1="9" x2="12" y2="13"/>
          <line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
      {:else}
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
      {/if}
    </div>
    <h3 class="confirm-title">{title}</h3>
    <p class="confirm-message">{message}</p>
    <div class="confirm-actions">
      <button class="btn btn-ghost" on:click={onCancel}>{cancelText}</button>
      <button class="btn" class:btn-danger={danger} class:btn-primary={!danger} on:click={onConfirm}>{confirmText}</button>
    </div>
  </div>
</div>

<style>
  .confirm-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 6000;
    backdrop-filter: blur(6px);
    animation: fadeIn 0.12s ease;
    padding: 20px;
  }

  .confirm-box {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 28px 32px;
    max-width: 400px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    text-align: center;
    animation: slideUp 0.2s ease;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }

  .confirm-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    margin-bottom: 16px;
    color: var(--text-muted);
    background: var(--bg-card);
  }

  .confirm-icon.danger {
    color: var(--danger);
    background: rgba(244, 63, 94, 0.1);
  }

  .confirm-title {
    font-size: 17px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 8px;
  }

  .confirm-message {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin-bottom: 24px;
    word-break: break-word;
  }

  .confirm-actions {
    display: flex;
    gap: 10px;
    justify-content: center;
  }

  .btn {
    padding: 9px 22px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
    min-width: 100px;
  }

  .btn-ghost {
    background: var(--bg-card);
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .btn-ghost:hover {
    background: var(--bg-card-hover);
    color: var(--text-primary);
    border-color: var(--border-hover);
  }

  .btn-danger {
    background: var(--danger);
    color: #fff;
  }

  .btn-danger:hover {
    filter: brightness(1.15);
    transform: translateY(-1px);
  }

  .btn-primary {
    background: linear-gradient(135deg, #ff6b35, #ff8c42);
    color: #fff;
  }

  .btn-primary:hover {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  @media (max-width: 768px) {
    .confirm-box { padding: 24px 20px; }
  }
</style>
