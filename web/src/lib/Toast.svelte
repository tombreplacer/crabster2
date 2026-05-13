<script>
  import { onMount } from 'svelte';
  export let message = '';
  export let type = 'info'; // info, success, error
  export let duration = 3000;
  export let onClose = () => {};
  let visible = false;
  onMount(() => {
    requestAnimationFrame(() => visible = true);
    if (duration > 0) setTimeout(() => { visible = false; setTimeout(onClose, 300); }, duration);
  });
</script>

<div class="toast toast-{type}" class:visible>
  <div class="toast-icon">
    {#if type === 'success'}✓{:else if type === 'error'}✕{:else}ℹ{/if}
  </div>
  <span class="toast-msg">{message}</span>
  <button class="toast-close" on:click={() => { visible = false; setTimeout(onClose, 300); }}>×</button>
</div>

<style>
  .toast {
    display: flex; align-items: center; gap: 10px; padding: 12px 16px;
    border-radius: var(--radius-sm); backdrop-filter: blur(20px); min-width: 280px;
    opacity: 0; transform: translateX(40px); transition: all 0.3s ease;
    border: 1px solid var(--border); font-size: 13px;
  }
  .toast.visible { opacity: 1; transform: translateX(0); }
  .toast-info { background: rgba(96,165,250,0.1); border-color: rgba(96,165,250,0.3); }
  .toast-success { background: rgba(74,222,128,0.1); border-color: rgba(74,222,128,0.3); }
  .toast-error { background: rgba(244,63,94,0.1); border-color: rgba(244,63,94,0.3); }
  .toast-icon { font-size: 16px; font-weight: 700; }
  .toast-info .toast-icon { color: var(--info); }
  .toast-success .toast-icon { color: var(--success); }
  .toast-error .toast-icon { color: var(--danger); }
  .toast-msg { flex: 1; color: var(--text-primary); }
  .toast-close { background: none; color: var(--text-muted); font-size: 18px; padding: 0 4px; transition: color 0.15s; }
  .toast-close:hover { color: var(--text-primary); }
</style>
