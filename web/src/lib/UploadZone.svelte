<script>
  import { createEventDispatcher } from 'svelte';
  export let readonly = false;
  const dispatch = createEventDispatcher();
  let dragging = false;
  let fileInput;

  function onDrop(e) {
    e.preventDefault();
    dragging = false;
    if (readonly) return;
    const files = e.dataTransfer?.files;
    if (files?.length) dispatch('upload', Array.from(files));
  }
  function onDragOver(e) { e.preventDefault(); if (!readonly) dragging = true; }
  function onDragLeave() { dragging = false; }
  function onFileSelect(e) {
    const files = e.target.files;
    if (files?.length) dispatch('upload', Array.from(files));
    e.target.value = '';
  }
  function onClick() { if (!readonly) fileInput.click(); }
</script>

{#if !readonly}
<div class="upload-zone" class:dragging on:drop={onDrop} on:dragover={onDragOver} on:dragleave={onDragLeave} 
  on:click={onClick} role="button" tabindex="0" 
  on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && onClick()}>
  <input bind:this={fileInput} type="file" multiple hidden on:change={onFileSelect}/>
  <div class="upload-content">
    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
      <polyline points="17 8 12 3 7 8"/>
      <line x1="12" y1="3" x2="12" y2="15"/>
    </svg>
    <span class="label">{dragging ? 'Drop files here' : 'Drag & drop or click to upload'}</span>
  </div>
</div>
{/if}

<style>
  .upload-zone {
    border: 2px dashed var(--border); border-radius: var(--radius); padding: 24px;
    text-align: center; cursor: pointer; transition: all 0.2s ease;
    background: var(--bg-card); margin-bottom: 16px;
  }
  .upload-zone:hover, .upload-zone.dragging {
    border-color: var(--accent); background: rgba(255,107,53,0.05);
  }
  .upload-zone.dragging { transform: scale(1.01); box-shadow: 0 0 30px var(--accent-glow); }
  .upload-content { display: flex; flex-direction: column; align-items: center; gap: 8px; color: var(--text-secondary); }
  .upload-zone:hover .upload-content, .upload-zone.dragging .upload-content { color: var(--accent); }
  .label { font-size: 13px; font-weight: 500; }
</style>
