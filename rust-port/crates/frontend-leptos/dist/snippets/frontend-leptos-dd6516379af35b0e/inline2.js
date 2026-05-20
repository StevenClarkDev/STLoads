
export async function stloadsUploadLoadDocument(url, token, documentName, documentType, inputId) {
  const input = document.getElementById(inputId);
  if (!input || !input.files || input.files.length === 0) {
    throw new Error('Choose a file before uploading a load document.');
  }

  const file = input.files[0];
  const normalizedType = (documentType || '').trim().toLowerCase().replace(/[\s-]+/g, '_');
  const allowedTypes = new Set([
    'rate_confirmation',
    'bill_of_lading',
    'delivery_pod',
    'invoice',
    'lumper_receipt',
    'insurance_certificate',
    'carrier_packet',
    'customs_document',
    'blockchain',
    'pickup_bol',
    'pickup_photo',
    'delivery_photo',
    'other',
  ]);
  const allowedMime = new Set(['application/pdf', 'image/jpeg', 'image/png', 'text/plain']);
  if (!allowedTypes.has(normalizedType)) {
    throw new Error('Choose a production document type before uploading.');
  }
  if (file.size > 25 * 1024 * 1024) {
    throw new Error('Document uploads are limited to 25 MB.');
  }
  if (file.type && !allowedMime.has(file.type.toLowerCase())) {
    throw new Error(`Document MIME type ${file.type} is not allowed.`);
  }
  const form = new FormData();
  form.append('document_name', documentName || '');
  form.append('document_type', normalizedType);
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
