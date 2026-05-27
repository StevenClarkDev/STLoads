
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

export function stloadsQueueOfflineExecutionSubmission(legId, submissionType, payload) {
  const key = `stloads.execution.offline.${legId}`;
  const existing = JSON.parse(window.localStorage.getItem(key) || '[]');
  let parsedPayload = payload || {};
  if (typeof parsedPayload === 'string') {
    try {
      parsedPayload = JSON.parse(parsedPayload);
    } catch (_) {
      parsedPayload = { value: parsedPayload };
    }
  }
  existing.push({
    id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
    legId,
    submissionType,
    payload: parsedPayload,
    capturedAt: new Date().toISOString(),
    status: 'pending'
  });
  window.localStorage.setItem(key, JSON.stringify(existing));
  return JSON.stringify({ pendingCount: existing.length });
}

export async function stloadsQueueOfflineExecutionDocumentUpload(
  legId,
  documentName,
  documentType,
  inputId
) {
  const input = document.getElementById(inputId);
  if (!input || !input.files || input.files.length === 0) {
    throw new Error('Choose a file before queueing an offline execution document.');
  }

  const file = input.files[0];
  const dataUrl = await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ''));
    reader.onerror = () => reject(reader.error || new Error('Could not read selected file.'));
    reader.readAsDataURL(file);
  });
  const payload = {
    document_name: documentName || documentType || 'Execution document',
    document_type: documentType || 'other',
    file_name: file.name || 'offline-document.bin',
    mime_type: file.type || null,
    bytes_base64: dataUrl
  };
  input.value = '';
  return stloadsQueueOfflineExecutionSubmission(legId, 'document_upload', payload);
}

export async function stloadsReplayOfflineExecutionSubmissions(url, token, legId) {
  const key = `stloads.execution.offline.${legId}`;
  const existing = JSON.parse(window.localStorage.getItem(key) || '[]');
  const remaining = [];
  let replayed = 0;
  let failed = 0;

  for (const item of existing) {
    if (item.status === 'replayed') {
      continue;
    }

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          client_submission_id: item.id,
          submission_type: item.submissionType,
          payload: item.payload || {},
          captured_at: item.capturedAt || null
        })
      });

      if (!response.ok) {
        throw new Error(`POST ${url} returned ${response.status}`);
      }

      const envelope = await response.json();
      if (envelope && envelope.data && envelope.data.success === false) {
        throw new Error(envelope.data.message || 'Offline submission was rejected by the server.');
      }

      replayed += 1;
    } catch (error) {
      failed += 1;
      remaining.push({ ...item, status: 'failed', lastError: String(error) });
    }
  }

  window.localStorage.setItem(key, JSON.stringify(remaining));
  return JSON.stringify({ replayed, failed, pendingCount: remaining.length });
}
