const BASE = '';

export async function fetchFiles(path = '') {
  const res = await fetch(`${BASE}/api/files?path=${encodeURIComponent(path)}`);
  const data = await res.json();
  if (!res.ok) {
    if (data.needsAuth) throw { needsAuth: true };
    throw new Error(data.error || res.statusText);
  }
  return data;
}

export async function fetchServerInfo() {
  const res = await fetch(`${BASE}/api/info`);
  const data = await res.json();
  if (!res.ok) {
    if (data.needsAuth) throw { needsAuth: true };
    throw new Error(data.error || res.statusText);
  }
  return data;
}

export async function login(code) {
  const res = await fetch(`${BASE}/api/auth`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ code })
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.error || res.statusText);
  return data;
}

export async function uploadFiles(files, path = '') {
  const formData = new FormData();
  for (const file of files) {
    formData.append('files', file, file.name);
  }
  const res = await fetch(`${BASE}/api/upload?path=${encodeURIComponent(path)}`, {
    method: 'POST',
    body: formData,
  });
  if (!res.ok) throw new Error((await res.json()).error || res.statusText);
  return res.json();
}

export function downloadUrl(path) {
  return `${BASE}/api/download/${encodeURIComponent(path)}`;
}

export function previewUrl(path) {
  return `${BASE}/api/preview/${encodeURIComponent(path)}`;
}

export function isPreviewable(entry) {
  if (entry.is_dir) return false;
  const mime = entry.mime_type || '';
  if (mime.startsWith('image/')) return true;
  if (mime.startsWith('video/')) return true;
  if (mime.startsWith('audio/')) return true;
  if (mime.includes('pdf')) return true;
  if (mime.startsWith('text/')) return true;
  if (mime.includes('json') || mime.includes('xml') || mime.includes('javascript')
      || mime.includes('css') || mime.includes('yaml') || mime.includes('toml')
      || mime.includes('markdown') || mime.includes('x-sh') || mime.includes('x-python')
      || mime.includes('x-ruby') || mime.includes('x-perl') || mime.includes('x-httpd-php')) return true;
  return false;
}

export function getPreviewType(entry) {
  const mime = entry.mime_type || '';
  if (mime.startsWith('image/')) return 'image';
  if (mime.startsWith('video/')) return 'video';
  if (mime.startsWith('audio/')) return 'audio';
  if (mime.includes('pdf')) return 'pdf';
  return 'text';
}

export async function deleteFile(path) {
  const res = await fetch(`${BASE}/api/delete/${encodeURIComponent(path)}`, {
    method: 'DELETE',
  });
  if (!res.ok) throw new Error((await res.json()).error || res.statusText);
  return res.json();
}

export async function createDir(path) {
  const res = await fetch(`${BASE}/api/mkdir?path=${encodeURIComponent(path)}`, {
    method: 'POST',
  });
  if (!res.ok) throw new Error((await res.json()).error || res.statusText);
  return res.json();
}

export function formatSize(bytes) {
  if (bytes === 0) return '—';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

export function formatDate(iso) {
  if (!iso) return '—';
  const d = new Date(iso);
  const pad = n => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function getFileIconType(entry) {
  if (entry.is_dir) return 'folder';
  if (entry.is_symlink) return 'link';
  const mime = entry.mime_type || '';
  const name = entry.name.toLowerCase();
  if (mime.startsWith('image/')) return 'image';
  if (mime.startsWith('video/')) return 'video';
  if (mime.startsWith('audio/')) return 'audio';
  if (mime.startsWith('text/') || mime.includes('json') || mime.includes('xml') || mime.includes('javascript') || mime.includes('css')) return 'code';
  if (mime.includes('pdf')) return 'pdf';
  if (mime.includes('zip') || mime.includes('tar') || mime.includes('gzip') || mime.includes('rar') || mime.includes('7z') || name.endsWith('.gz') || name.endsWith('.bz2') || name.endsWith('.xz')) return 'archive';
  if (mime.includes('executable') || mime.includes('x-sharedlib') || name.endsWith('.exe') || name.endsWith('.sh') || name.endsWith('.bat')) return 'executable';
  return 'file';
}
