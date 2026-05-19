
export async function stloadsUploadLoadDocument(url, token, documentName, documentType, inputId) {
  const input = document.getElementById(inputId);
  if (!input || !input.files || input.files.length === 0) {
    throw new Error('Choose a file before uploading a load document.');
  }

  const file = input.files[0];
  const form = new FormData();
  form.append('document_name', documentName || '');
  form.append('document_type', documentType || '');
  form.append('file', file, file.name || 'document.bin');

  const headers = {};
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(url, {
    method: 'POST',
    headers,
    body: form,
  });

  const text = await response.text();
  if (!response.ok) {
    throw new Error(`POST ${url} returned ${response.status} ${text}`);
  }

  input.value = '';
  return text;
}

export async function stloadsOpenProtectedDocument(url, token) {
  const headers = {};
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(url, {
    method: 'GET',
    headers,
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`GET ${url} returned ${response.status} ${text}`);
  }

  const blob = await response.blob();
  const objectUrl = URL.createObjectURL(blob);
  window.open(objectUrl, '_blank', 'noopener,noreferrer');
  window.setTimeout(() => URL.revokeObjectURL(objectUrl), 60000);
  return true;
}

export async function stloadsDownloadProtectedDocument(url, token, fileName) {
  const headers = {};
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(url, {
    method: 'GET',
    headers,
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`GET ${url} returned ${response.status} ${text}`);
  }

  const blob = await response.blob();
  const objectUrl = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = objectUrl;
  anchor.download = fileName || 'document.bin';
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(objectUrl), 60000);
  return true;
}

export async function stloadsHashSelectedFile(inputId) {
  const input = document.getElementById(inputId);
  if (!input || !input.files || input.files.length === 0) {
    throw new Error('Choose a file before verifying a blockchain document.');
  }

  const file = input.files[0];
  const buffer = await file.arrayBuffer();
  const digest = await crypto.subtle.digest('SHA-256', buffer);
  const hash = Array.from(new Uint8Array(digest))
    .map((value) => value.toString(16).padStart(2, '0'))
    .join('');

  input.value = '';
  return JSON.stringify({
    fileName: file.name || 'document.bin',
    hash,
  });
}
