<script>
  export let type = 'file';
  export let size = 20;
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="icon icon-{type}">
  {#if type === 'folder'}
    <path d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" fill="rgba(255,170,50,0.2)" stroke="rgba(255,170,50,0.8)"/>
  {:else if type === 'image'}
    <rect x="3" y="3" width="18" height="18" rx="2" stroke="#a78bfa"/>
    <circle cx="8.5" cy="8.5" r="1.5" stroke="#a78bfa"/>
    <path d="M21 15l-5-5L5 21" stroke="#a78bfa"/>
  {:else if type === 'video'}
    <rect x="2" y="4" width="20" height="16" rx="2" stroke="#f472b6"/>
    <path d="M10 9l5 3-5 3V9z" fill="rgba(244,114,182,0.3)" stroke="#f472b6"/>
  {:else if type === 'audio'}
    <path d="M9 18V5l12-2v13" stroke="#fbbf24"/>
    <circle cx="6" cy="18" r="3" stroke="#fbbf24"/>
    <circle cx="18" cy="16" r="3" stroke="#fbbf24"/>
  {:else if type === 'code'}
    <polyline points="16 18 22 12 16 6" stroke="#60a5fa"/>
    <polyline points="8 6 2 12 8 18" stroke="#60a5fa"/>
    <line x1="14" y1="4" x2="10" y2="20" stroke="rgba(96,165,250,0.4)"/>
  {:else if type === 'pdf'}
    <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" stroke="#f43f5e"/>
    <polyline points="14 2 14 8 20 8" stroke="#f43f5e"/>
    <text x="8" y="16" font-size="6" fill="#f43f5e" stroke="none" font-weight="700">PDF</text>
  {:else if type === 'archive'}
    <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" stroke="#fbbf24"/>
    <polyline points="14 2 14 8 20 8" stroke="#fbbf24"/>
    <rect x="9" y="12" width="6" height="4" rx="1" stroke="#fbbf24" fill="rgba(251,191,36,0.2)"/>
  {:else if type === 'executable'}
    <rect x="3" y="3" width="18" height="18" rx="2" stroke="#4ade80"/>
    <path d="M8 12l3 3 5-6" stroke="#4ade80"/>
  {:else if type === 'link'}
    <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71" stroke="#c084fc"/>
    <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71" stroke="#c084fc"/>
  {:else}
    <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" stroke="var(--text-secondary)"/>
    <polyline points="14 2 14 8 20 8" stroke="var(--text-secondary)"/>
  {/if}
</svg>

<style>
  .icon { flex-shrink: 0; transition: transform 0.15s ease; }
  .icon-folder { color: #ffaa32; }
</style>
