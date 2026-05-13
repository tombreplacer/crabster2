<script>
  import { onMount } from 'svelte';
  import { fetchFiles, fetchServerInfo, uploadFiles, downloadUrl, deleteFile, createDir, formatSize, formatDate, getFileIconType } from './lib/api.js';
  import FileIcon from './lib/FileIcon.svelte';
  import Breadcrumb from './lib/Breadcrumb.svelte';
  import UploadZone from './lib/UploadZone.svelte';
  import Toast from './lib/Toast.svelte';

  let currentPath = '';
  let entries = [];
  let serverInfo = {};
  let loading = true;
  let error = null;
  let searchQuery = '';
  let sortKey = 'name';
  let sortAsc = true;
  let toasts = [];
  let contextMenu = null;
  let uploading = false;
  let uploadProgress = '';
  let showNewDirModal = false;
  let newDirName = '';

  $: pathParts = currentPath ? currentPath.split('/').filter(Boolean) : [];
  $: filtered = entries.filter(e => e.name.toLowerCase().includes(searchQuery.toLowerCase()));
  $: sorted = [...filtered].sort((a, b) => {
    if (a.is_dir !== b.is_dir) return b.is_dir - a.is_dir;
    let cmp = 0;
    if (sortKey === 'name') cmp = a.name.localeCompare(b.name);
    else if (sortKey === 'size') cmp = a.size - b.size;
    else if (sortKey === 'modified') cmp = (a.modified || '').localeCompare(b.modified || '');
    else if (sortKey === 'permissions') cmp = a.permissions.localeCompare(b.permissions);
    return sortAsc ? cmp : -cmp;
  });

  function toast(message, type = 'info') {
    const id = Date.now() + Math.random();
    toasts = [...toasts, { id, message, type }];
  }
  function removeToast(id) { toasts = toasts.filter(t => t.id !== id); }

  async function loadFiles(path) {
    loading = true; error = null;
    try {
      const data = await fetchFiles(path);
      entries = data.entries;
      currentPath = data.path;
      serverInfo.readonly = data.readonly;
      serverInfo.no_delete = data.no_delete;
    } catch (e) { error = e.message; toast(e.message, 'error'); }
    loading = false;
  }

  function navigate(path) { loadFiles(path); searchQuery = ''; closeContext(); }

  function handleEntryClick(entry) {
    if (entry.is_dir) {
      const newPath = currentPath ? `${currentPath}/${entry.name}` : entry.name;
      navigate(newPath);
    }
  }

  function handleSort(key) {
    if (sortKey === key) sortAsc = !sortAsc;
    else { sortKey = key; sortAsc = true; }
  }

  function handleDownload(entry) {
    const path = currentPath ? `${currentPath}/${entry.name}` : entry.name;
    const a = document.createElement('a');
    a.href = downloadUrl(path);
    a.download = entry.name;
    a.click();
    toast(`Downloading ${entry.name}`, 'info');
  }

  async function handleDelete(entry) {
    if (!confirm(`Delete "${entry.name}"?`)) return;
    try {
      const path = currentPath ? `${currentPath}/${entry.name}` : entry.name;
      await deleteFile(path);
      toast(`Deleted ${entry.name}`, 'success');
      loadFiles(currentPath);
    } catch (e) { toast(e.message, 'error'); }
  }

  async function handleUpload(e) {
    const files = e.detail;
    uploading = true;
    uploadProgress = `Uploading ${files.length} file(s)...`;
    try {
      const result = await uploadFiles(files, currentPath);
      toast(`Uploaded ${result.count} file(s)`, 'success');
      loadFiles(currentPath);
    } catch (e) { toast(e.message, 'error'); }
    uploading = false; uploadProgress = '';
  }

  async function handleCreateDir() {
    if (!newDirName.trim()) return;
    try {
      const path = currentPath ? `${currentPath}/${newDirName}` : newDirName;
      await createDir(path);
      toast(`Created folder "${newDirName}"`, 'success');
      showNewDirModal = false; newDirName = '';
      loadFiles(currentPath);
    } catch (e) { toast(e.message, 'error'); }
  }

  function handleContextMenu(e, entry) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, entry };
  }
  function closeContext() { contextMenu = null; }
  function copyPath(entry) {
    const path = currentPath ? `${currentPath}/${entry.name}` : entry.name;
    navigator.clipboard?.writeText(path);
    toast('Path copied', 'success');
    closeContext();
  }

  onMount(async () => {
    try { serverInfo = await fetchServerInfo(); } catch(e) {}
    loadFiles('');
    document.addEventListener('click', closeContext);
    return () => document.removeEventListener('click', closeContext);
  });
</script>

<div class="app">
  <header class="header">
    <div class="header-inner">
      <div class="logo">
        <svg width="32" height="32" viewBox="0 0 100 100">
          <circle cx="50" cy="55" r="30" fill="none" stroke="#ff6b35" stroke-width="3"/>
          <ellipse cx="50" cy="50" rx="35" ry="20" fill="none" stroke="#ff8c42" stroke-width="2.5"/>
          <circle cx="40" cy="45" r="4" fill="#ff6b35"/><circle cx="60" cy="45" r="4" fill="#ff6b35"/>
          <path d="M20 40 Q10 25 5 15" stroke="#ff6b35" stroke-width="2.5" fill="none" stroke-linecap="round"/>
          <path d="M80 40 Q90 25 95 15" stroke="#ff6b35" stroke-width="2.5" fill="none" stroke-linecap="round"/>
          <path d="M22 50 Q8 50 2 55" stroke="#ff8c42" stroke-width="2" fill="none" stroke-linecap="round"/>
          <path d="M78 50 Q92 50 98 55" stroke="#ff8c42" stroke-width="2" fill="none" stroke-linecap="round"/>
          <path d="M25 58 Q12 65 8 75" stroke="#ff8c42" stroke-width="2" fill="none" stroke-linecap="round"/>
          <path d="M75 58 Q88 65 92 75" stroke="#ff8c42" stroke-width="2" fill="none" stroke-linecap="round"/>
        </svg>
        <h1>Crabster</h1>
        {#if serverInfo.version}<span class="version">v{serverInfo.version}</span>{/if}
      </div>
      <div class="header-info">
        {#if serverInfo.root}<span class="server-path" title={serverInfo.root}>📂 {serverInfo.root}</span>{/if}
        {#if serverInfo.readonly}<span class="badge badge-warn">Read-Only</span>{/if}
      </div>
    </div>
  </header>

  <main class="main">
    <div class="toolbar">
      <Breadcrumb parts={pathParts} onNavigate={navigate}/>
      <div class="toolbar-actions">
        <div class="search-box">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input type="text" placeholder="Filter files..." bind:value={searchQuery}/>
        </div>
        {#if !serverInfo.readonly}
          <button class="btn btn-ghost" on:click={() => { showNewDirModal = true; }} title="New folder">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/><line x1="12" y1="11" x2="12" y2="17"/><line x1="9" y1="14" x2="15" y2="14"/></svg>
          </button>
        {/if}
        <button class="btn btn-ghost" on:click={() => loadFiles(currentPath)} title="Refresh">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/></svg>
        </button>
      </div>
    </div>

    <UploadZone readonly={serverInfo.readonly} on:upload={handleUpload}/>

    {#if uploading}
      <div class="upload-bar"><div class="upload-spinner"></div>{uploadProgress}</div>
    {/if}

    {#if loading}
      <div class="loading"><div class="spinner"></div><span>Loading...</span></div>
    {:else if error}
      <div class="error-msg">⚠️ {error}</div>
    {:else}
      <div class="table-wrap">
        <table class="file-table">
          <thead>
            <tr>
              <th class="th-name" on:click={() => handleSort('name')}>Name {sortKey==='name' ? (sortAsc?'↑':'↓') : ''}</th>
              <th class="th-size" on:click={() => handleSort('size')}>Size {sortKey==='size' ? (sortAsc?'↑':'↓') : ''}</th>
              <th class="th-perm" on:click={() => handleSort('permissions')}>Permissions {sortKey==='permissions' ? (sortAsc?'↑':'↓') : ''}</th>
              <th class="th-owner">Owner</th>
              <th class="th-mod" on:click={() => handleSort('modified')}>Modified {sortKey==='modified' ? (sortAsc?'↑':'↓') : ''}</th>
              <th class="th-actions">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each sorted as entry, i}
              <tr class="file-row" style="animation-delay:{i*20}ms"
                on:dblclick={() => handleEntryClick(entry)}
                on:contextmenu={(e) => handleContextMenu(e, entry)}>
                <td class="td-name">
                  <FileIcon type={getFileIconType(entry)} size={18}/>
                  <button class="name-btn" class:is-dir={entry.is_dir} on:click={() => handleEntryClick(entry)}>
                    {entry.name}
                  </button>
                  {#if entry.is_symlink}<span class="badge badge-sm">symlink</span>{/if}
                </td>
                <td class="td-size">{entry.is_dir ? '—' : formatSize(entry.size)}</td>
                <td class="td-perm"><code>{entry.permissions}</code></td>
                <td class="td-owner">{entry.owner}{entry.group && entry.group !== 'N/A' ? ':' + entry.group : ''}</td>
                <td class="td-mod">{formatDate(entry.modified)}</td>
                <td class="td-actions">
                  {#if !entry.is_dir}
                    <button class="action-btn" title="Download" on:click={() => handleDownload(entry)}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
                    </button>
                  {/if}
                  {#if !serverInfo.readonly && !serverInfo.no_delete}
                    <button class="action-btn action-danger" title="Delete" on:click={() => handleDelete(entry)}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>
                    </button>
                  {/if}
                </td>
              </tr>
            {:else}
              <tr><td colspan="6" class="empty">No files found</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
      <div class="status-bar">{sorted.length} item{sorted.length !== 1 ? 's' : ''} · {sorted.filter(e=>e.is_dir).length} folders · {sorted.filter(e=>!e.is_dir).length} files</div>
    {/if}
  </main>

  {#if contextMenu}
    <div class="ctx-menu" style="left:{contextMenu.x}px;top:{contextMenu.y}px" on:click|stopPropagation>
      {#if !contextMenu.entry.is_dir}
        <button on:click={() => { handleDownload(contextMenu.entry); closeContext(); }}>⬇ Download</button>
      {/if}
      <button on:click={() => copyPath(contextMenu.entry)}>📋 Copy path</button>
      {#if !serverInfo.readonly && !serverInfo.no_delete}
        <button class="ctx-danger" on:click={() => { handleDelete(contextMenu.entry); closeContext(); }}>🗑 Delete</button>
      {/if}
    </div>
  {/if}

  {#if showNewDirModal}
    <div class="modal-overlay" on:click={() => showNewDirModal = false}>
      <div class="modal" on:click|stopPropagation>
        <h3>New Folder</h3>
        <input type="text" bind:value={newDirName} placeholder="Folder name" autofocus
          on:keydown={(e) => e.key === 'Enter' && handleCreateDir()}/>
        <div class="modal-actions">
          <button class="btn btn-ghost" on:click={() => showNewDirModal = false}>Cancel</button>
          <button class="btn btn-primary" on:click={handleCreateDir}>Create</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="toast-container">
    {#each toasts as t (t.id)}
      <Toast message={t.message} type={t.type} onClose={() => removeToast(t.id)}/>
    {/each}
  </div>
</div>

<style>
  .app { min-height: 100vh; display: flex; flex-direction: column; }

  .header { background: var(--bg-secondary); border-bottom: 1px solid var(--border); padding: 0 24px; position: sticky; top: 0; z-index: 100; backdrop-filter: blur(20px); }
  .header-inner { max-width: 1400px; margin: 0 auto; display: flex; align-items: center; justify-content: space-between; height: 60px; }
  .logo { display: flex; align-items: center; gap: 12px; }
  .logo h1 { font-size: 20px; font-weight: 700; background: linear-gradient(135deg, #ff6b35, #ff8c42); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
  .version { font-size: 11px; color: var(--text-muted); background: var(--bg-card); padding: 2px 8px; border-radius: 20px; }
  .header-info { display: flex; align-items: center; gap: 12px; }
  .server-path { font-size: 12px; color: var(--text-secondary); font-family: var(--mono); max-width: 400px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .badge { font-size: 11px; padding: 3px 10px; border-radius: 20px; font-weight: 600; }
  .badge-warn { background: rgba(251,191,36,0.15); color: var(--warning); }
  .badge-sm { font-size: 9px; padding: 1px 6px; border-radius: 10px; background: rgba(192,132,252,0.15); color: #c084fc; margin-left: 6px; }

  .main { max-width: 1400px; width: 100%; margin: 0 auto; padding: 16px 24px 40px; flex: 1; }

  .toolbar { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 12px; flex-wrap: wrap; }
  .toolbar-actions { display: flex; align-items: center; gap: 8px; }
  .search-box { display: flex; align-items: center; gap: 8px; background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 6px 12px; transition: border-color 0.15s; }
  .search-box:focus-within { border-color: var(--accent); }
  .search-box input { background: none; border: none; color: var(--text-primary); font-size: 13px; outline: none; width: 180px; }
  .search-box input::placeholder { color: var(--text-muted); }
  .search-box svg { color: var(--text-muted); flex-shrink: 0; }

  .btn { padding: 8px 16px; border-radius: var(--radius-sm); font-size: 13px; font-weight: 500; transition: all 0.15s; display: flex; align-items: center; gap: 6px; }
  .btn-ghost { background: var(--bg-card); color: var(--text-secondary); border: 1px solid var(--border); }
  .btn-ghost:hover { background: var(--bg-card-hover); color: var(--text-primary); border-color: var(--border-hover); }
  .btn-primary { background: linear-gradient(135deg, #ff6b35, #ff8c42); color: #fff; }
  .btn-primary:hover { filter: brightness(1.1); transform: translateY(-1px); }

  .upload-bar { display: flex; align-items: center; gap: 10px; padding: 10px 16px; background: rgba(255,107,53,0.08); border: 1px solid rgba(255,107,53,0.2); border-radius: var(--radius-sm); margin-bottom: 12px; font-size: 13px; color: var(--accent); }
  .upload-spinner { width: 16px; height: 16px; border: 2px solid rgba(255,107,53,0.3); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.6s linear infinite; }

  .loading { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 60px; color: var(--text-secondary); }
  .spinner { width: 24px; height: 24px; border: 2px solid var(--border); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
  .error-msg { padding: 20px; background: rgba(244,63,94,0.08); border: 1px solid rgba(244,63,94,0.2); border-radius: var(--radius-sm); color: var(--danger); font-size: 14px; }

  .table-wrap { overflow-x: auto; border: 1px solid var(--border); border-radius: var(--radius); background: var(--bg-card); }
  .file-table { width: 100%; border-collapse: collapse; font-size: 13px; }
  .file-table thead { position: sticky; top: 0; z-index: 1; }
  .file-table th { padding: 10px 14px; text-align: left; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted); background: var(--bg-secondary); cursor: pointer; user-select: none; border-bottom: 1px solid var(--border); white-space: nowrap; }
  .file-table th:hover { color: var(--text-secondary); }
  .file-row { animation: fadeIn 0.2s ease both; transition: background 0.1s; }
  .file-row:hover { background: var(--bg-card-hover); }
  .file-row td { padding: 8px 14px; border-bottom: 1px solid rgba(255,255,255,0.03); }
  .td-name { display: flex; align-items: center; gap: 10px; min-width: 250px; }
  .name-btn { background: none; color: var(--text-primary); font-size: 13px; text-align: left; padding: 0; transition: color 0.15s; }
  .name-btn:hover { color: var(--accent); }
  .name-btn.is-dir { font-weight: 500; cursor: pointer; }
  .name-btn.is-dir:hover { color: var(--accent); text-decoration: underline; }
  .td-size { color: var(--text-secondary); font-family: var(--mono); font-size: 12px; white-space: nowrap; }
  .td-perm code { font-family: var(--mono); font-size: 12px; color: var(--text-secondary); background: rgba(255,255,255,0.04); padding: 2px 6px; border-radius: 4px; }
  .td-owner { color: var(--text-secondary); font-size: 12px; white-space: nowrap; }
  .td-mod { color: var(--text-muted); font-family: var(--mono); font-size: 12px; white-space: nowrap; }
  .td-actions { display: flex; gap: 4px; }
  .action-btn { background: none; color: var(--text-muted); padding: 4px 6px; border-radius: 4px; transition: all 0.15s; }
  .action-btn:hover { background: var(--bg-card-hover); color: var(--text-primary); }
  .action-danger:hover { color: var(--danger); background: rgba(244,63,94,0.1); }
  .empty { text-align: center; padding: 40px; color: var(--text-muted); }
  .th-name { min-width: 250px; }
  .th-size { width: 100px; }
  .th-perm { width: 120px; }
  .th-owner { width: 120px; }
  .th-mod { width: 160px; }
  .th-actions { width: 80px; }

  .status-bar { padding: 10px 0; font-size: 12px; color: var(--text-muted); text-align: right; }

  .ctx-menu { position: fixed; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 4px; min-width: 160px; z-index: 1000; box-shadow: 0 10px 40px rgba(0,0,0,0.5); backdrop-filter: blur(20px); animation: fadeIn 0.1s ease; }
  .ctx-menu button { display: flex; align-items: center; gap: 8px; width: 100%; padding: 8px 12px; background: none; color: var(--text-primary); font-size: 13px; border-radius: 6px; transition: background 0.1s; text-align: left; }
  .ctx-menu button:hover { background: var(--bg-card-hover); }
  .ctx-danger { color: var(--danger) !important; }

  .modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 2000; backdrop-filter: blur(4px); animation: fadeIn 0.15s ease; }
  .modal { background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius); padding: 24px; min-width: 360px; animation: slideUp 0.2s ease; }
  .modal h3 { font-size: 16px; margin-bottom: 16px; }
  .modal input { width: 100%; padding: 10px 14px; background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 14px; outline: none; margin-bottom: 16px; }
  .modal input:focus { border-color: var(--accent); }
  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; }

  .toast-container { position: fixed; bottom: 20px; right: 20px; display: flex; flex-direction: column; gap: 8px; z-index: 3000; }

  @media (max-width: 768px) {
    .header-inner { flex-direction: column; height: auto; padding: 12px 0; gap: 8px; }
    .server-path { max-width: 250px; }
    .main { padding: 12px 16px 40px; }
    .toolbar { flex-direction: column; align-items: stretch; }
    .search-box input { width: 100%; }
    .th-perm, .td-perm, .th-owner, .td-owner { display: none; }
  }
</style>
