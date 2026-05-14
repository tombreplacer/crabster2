<script>
  export let parts = [];
  export let onNavigate = () => {};
</script>

<nav class="breadcrumb">
  <button class="crumb root" on:click={() => onNavigate('')} aria-label="Home">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"/></svg>
  </button>
  {#each parts as part, i}
    <span class="sep">/</span>
    <button class="crumb" class:active={i === parts.length - 1} on:click={() => onNavigate(parts.slice(0, i + 1).join('/'))}>
      {part}
    </button>
  {/each}
</nav>

<style>
  .breadcrumb { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; padding: 10px 0; }
  .crumb {
    background: none; color: var(--text-secondary); font-size: 13px; font-weight: 500;
    padding: 4px 8px; border-radius: 6px; transition: all 0.15s; display: flex; align-items: center; gap: 4px;
  }
  .crumb:hover { background: var(--bg-card-hover); color: var(--text-primary); }
  .crumb.active { color: var(--text-primary); font-weight: 600; }
  .crumb.root { padding: 6px; }
  .sep { color: var(--text-muted); font-size: 12px; user-select: none; }
</style>
