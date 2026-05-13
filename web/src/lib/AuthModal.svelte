<script>
  import { onMount } from 'svelte';

  export let onSubmit;
  export let error = null;
  export let loading = false;

  let code = '';
  let inputElement;

  onMount(() => {
    if (inputElement) inputElement.focus();
    document.body.style.overflow = 'hidden';
    return () => { document.body.style.overflow = ''; };
  });

  function handleSubmit(e) {
    e.preventDefault();
    if (code.trim()) {
      onSubmit(code.trim());
    }
  }
</script>

<div class="auth-overlay">
  <div class="auth-box">
    <div class="auth-icon">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
        <path d="M7 11V7a5 5 0 0110 0v4"/>
      </svg>
    </div>
    <h2 class="auth-title">Authentication Required</h2>
    <p class="auth-subtitle">Please enter the access code to continue.</p>
    
    <form class="auth-form" on:submit={handleSubmit}>
      <div class="input-group">
        <input 
          bind:this={inputElement}
          type="password" 
          bind:value={code} 
          placeholder="Access Code"
          disabled={loading}
          class:has-error={error}
        />
        {#if error}
          <div class="error-msg">{error}</div>
        {/if}
      </div>
      
      <button type="submit" class="btn btn-primary" disabled={loading || !code.trim()}>
        {#if loading}
          <span class="spinner"></span>
        {:else}
          Unlock
        {/if}
      </button>
    </form>
  </div>
</div>

<style>
  .auth-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    backdrop-filter: blur(12px);
    animation: fadeIn 0.2s ease;
    padding: 20px;
  }

  .auth-box {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 40px;
    max-width: 420px;
    width: 100%;
    text-align: center;
    animation: slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
  }

  .auth-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background: rgba(255, 107, 53, 0.1);
    color: var(--accent);
    margin-bottom: 24px;
  }

  .auth-title {
    font-size: 22px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 8px 0;
  }

  .auth-subtitle {
    font-size: 14px;
    color: var(--text-secondary);
    margin: 0 0 32px 0;
  }

  .auth-form {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .input-group {
    position: relative;
    text-align: left;
  }

  input {
    width: 100%;
    padding: 14px 16px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 16px;
    transition: all 0.2s;
    outline: none;
    font-family: var(--font-sans);
  }

  input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(255, 107, 53, 0.15);
  }

  input.has-error {
    border-color: var(--danger);
  }

  input.has-error:focus {
    box-shadow: 0 0 0 3px rgba(244, 63, 94, 0.15);
  }

  input:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .error-msg {
    color: var(--danger);
    font-size: 13px;
    margin-top: 8px;
  }

  .btn {
    width: 100%;
    padding: 14px;
    border-radius: var(--radius-sm);
    font-size: 16px;
    font-weight: 600;
    transition: all 0.2s;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 50px;
  }

  .btn-primary {
    background: linear-gradient(135deg, #ff6b35, #ff8c42);
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.1);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(255, 107, 53, 0.3);
  }

  .btn-primary:disabled {
    background: var(--border);
    color: var(--text-muted);
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @media (max-width: 480px) {
    .auth-box { padding: 32px 24px; }
    .auth-title { font-size: 20px; }
  }
</style>
