#[cfg(target_arch = "wasm32")]
use crate::api;
use serde::Deserialize;
#[cfg(target_arch = "wasm32")]
use serde::de::DeserializeOwned;
#[cfg(target_arch = "wasm32")]
use shared::ApiResponse;
use shared::{
    ExecutionUploadDocumentResponse, KycDocumentItem, UpsertKycDocumentResponse,
    UpsertLoadDocumentResponse,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
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
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = stloadsUploadLoadDocument)]
    async fn stloads_upload_load_document(
        url: &str,
        token: &str,
        document_name: &str,
        document_type: &str,
        input_id: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = stloadsOpenProtectedDocument)]
    async fn stloads_open_protected_document(url: &str, token: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = stloadsDownloadProtectedDocument)]
    async fn stloads_download_protected_document(
        url: &str,
        token: &str,
        file_name: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = stloadsHashSelectedFile)]
    async fn stloads_hash_selected_file(input_id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = stloadsQueueOfflineExecutionSubmission)]
    fn stloads_queue_offline_execution_submission(
        leg_id: u64,
        submission_type: &str,
        payload: &str,
    ) -> JsValue;

    #[wasm_bindgen(catch, js_name = stloadsQueueOfflineExecutionDocumentUpload)]
    async fn stloads_queue_offline_execution_document_upload(
        leg_id: u64,
        document_name: &str,
        document_type: &str,
        input_id: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = stloadsReplayOfflineExecutionSubmissions)]
    async fn stloads_replay_offline_execution_submissions(
        url: &str,
        token: &str,
        leg_id: u64,
    ) -> Result<JsValue, JsValue>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalFileHashResult {
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub hash: String,
}

pub fn kyc_upload_input_id() -> &'static str {
    "kyc-document-upload"
}

pub fn profile_kyc_upload_input_id() -> &'static str {
    "profile-kyc-document-upload"
}

pub fn profile_kyc_replace_input_id(document_id: u64) -> String {
    format!("profile-kyc-document-replace-{}", document_id)
}

pub fn profile_kyc_verify_input_id(document_id: u64) -> String {
    format!("profile-kyc-document-verify-{}", document_id)
}

pub fn upload_input_id(load_id: u64) -> String {
    format!("load-document-upload-{}", load_id)
}

pub fn execution_upload_input_id(leg_id: u64) -> String {
    format!("execution-document-upload-{}", leg_id)
}

#[cfg(target_arch = "wasm32")]
pub fn queue_offline_execution_submission(
    leg_id: u64,
    submission_type: &str,
    payload: &str,
) -> Result<String, String> {
    stloads_queue_offline_execution_submission(leg_id, submission_type, payload)
        .as_string()
        .ok_or_else(|| "Offline execution queue did not return a text payload.".into())
}

#[cfg(target_arch = "wasm32")]
pub async fn queue_offline_execution_document_upload(
    leg_id: u64,
    document_name: &str,
    document_type: &str,
    input_id: &str,
) -> Result<String, String> {
    let raw = stloads_queue_offline_execution_document_upload(
        leg_id,
        document_name,
        document_type,
        input_id,
    )
    .await
    .map_err(|error| format!("Offline execution document queue failed: {:?}", error))?;

    raw.as_string()
        .ok_or_else(|| "Offline execution document queue did not return a text payload.".into())
}

#[cfg(target_arch = "wasm32")]
pub async fn replay_offline_execution_submissions(leg_id: u64) -> Result<String, String> {
    let raw = stloads_replay_offline_execution_submissions(
        &api::api_href(&format!("/execution/legs/{}/offline-submissions", leg_id)),
        &api::auth_token().unwrap_or_default(),
        leg_id,
    )
    .await
    .map_err(|error| format!("Offline execution replay failed: {:?}", error))?;

    raw.as_string()
        .ok_or_else(|| "Offline execution replay did not return a text payload.".into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn replay_offline_execution_submissions(_leg_id: u64) -> Result<String, String> {
    Err("Offline replay is only available in the browser build.".into())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn queue_offline_execution_submission(
    _leg_id: u64,
    _submission_type: &str,
    _payload: &str,
) -> Result<String, String> {
    Err("Offline queue is only available in the browser build.".into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn queue_offline_execution_document_upload(
    _leg_id: u64,
    _document_name: &str,
    _document_type: &str,
    _input_id: &str,
) -> Result<String, String> {
    Err("Offline document queue is only available in the browser build.".into())
}

#[cfg(target_arch = "wasm32")]
pub async fn upload_kyc_document(
    document_name: &str,
    document_type: &str,
    input_id: &str,
) -> Result<KycDocumentItem, String> {
    let raw = stloads_upload_load_document(
        &api::api_href("/auth/onboarding/documents/upload"),
        &api::auth_token().unwrap_or_default(),
        document_name,
        document_type,
        input_id,
    )
    .await
    .map_err(|error| format!("KYC document upload failed: {:?}", error))?;

    let payload = raw
        .as_string()
        .ok_or_else(|| "KYC document upload response was not returned as text.".to_string())?;
    decode_envelope::<KycDocumentItem>(&payload)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upload_kyc_document(
    _document_name: &str,
    _document_type: &str,
    _input_id: &str,
) -> Result<KycDocumentItem, String> {
    Err("Binary document upload is only available in the browser build of the Rust UI.".into())
}

#[cfg(target_arch = "wasm32")]
pub async fn upload_profile_kyc_document(
    document_name: &str,
    document_type: &str,
    input_id: &str,
) -> Result<UpsertKycDocumentResponse, String> {
    let raw = stloads_upload_load_document(
        &api::api_href("/auth/profile/documents/upload"),
        &api::auth_token().unwrap_or_default(),
        document_name,
        document_type,
        input_id,
    )
    .await
    .map_err(|error| format!("Profile KYC upload failed: {:?}", error))?;

    let payload = raw
        .as_string()
        .ok_or_else(|| "Profile KYC upload response was not returned as text.".to_string())?;
    decode_envelope::<UpsertKycDocumentResponse>(&payload)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upload_profile_kyc_document(
    _document_name: &str,
    _document_type: &str,
    _input_id: &str,
) -> Result<UpsertKycDocumentResponse, String> {
    Err("Binary document upload is only available in the browser build of the Rust UI.".into())
}

#[cfg(target_arch = "wasm32")]
pub async fn replace_profile_kyc_document(
    document_id: u64,
    document_name: &str,
    document_type: &str,
    input_id: &str,
) -> Result<UpsertKycDocumentResponse, String> {
    let raw = stloads_upload_load_document(
        &api::api_href(&format!("/auth/profile/documents/{}/upload", document_id)),
        &api::auth_token().unwrap_or_default(),
        document_name,
        document_type,
        input_id,
    )
    .await
    .map_err(|error| format!("Profile KYC replacement failed: {:?}", error))?;

    let payload = raw
        .as_string()
        .ok_or_else(|| "Profile KYC replacement response was not returned as text.".to_string())?;
    decode_envelope::<UpsertKycDocumentResponse>(&payload)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn replace_profile_kyc_document(
    _document_id: u64,
    _document_name: &str,
    _document_type: &str,
    _input_id: &str,
) -> Result<UpsertKycDocumentResponse, String> {
    Err("Binary document upload is only available in the browser build of the Rust UI.".into())
}

#[cfg(target_arch = "wasm32")]
pub async fn upload_load_document(
    load_id: u64,
    document_name: &str,
    document_type: &str,
    input_id: &str,
) -> Result<UpsertLoadDocumentResponse, String> {
    let raw = stloads_upload_load_document(
        &api::api_href(&format!("/dispatch/loads/{}/documents/upload", load_id)),
        &api::auth_token().unwrap_or_default(),
        document_name,
        document_type,
        input_id,
    )
    .await
    .map_err(|error| format!("Load document upload failed: {:?}", error))?;

    let payload = raw
        .as_string()
        .ok_or_else(|| "Load document upload response was not returned as text.".to_string())?;
    decode_envelope::<UpsertLoadDocumentResponse>(&payload)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upload_load_document(
    _load_id: u64,
    _document_name: &str,
    _document_type: &str,
    _input_id: &str,
) -> Result<UpsertLoadDocumentResponse, String> {
    Err("Binary document upload is only available in the browser build of the Rust UI.".into())
}

#[cfg(target_arch = "wasm32")]
pub async fn upload_execution_document(
    leg_id: u64,
    document_name: &str,
    document_type: &str,
    input_id: &str,
) -> Result<ExecutionUploadDocumentResponse, String> {
    let raw = stloads_upload_load_document(
        &api::api_href(&format!("/execution/legs/{}/documents/upload", leg_id)),
        &api::auth_token().unwrap_or_default(),
        document_name,
        document_type,
        input_id,
    )
    .await
    .map_err(|error| format!("Execution document upload failed: {:?}", error))?;

    let payload = raw.as_string().ok_or_else(|| {
        "Execution document upload response was not returned as text.".to_string()
    })?;
    decode_envelope::<ExecutionUploadDocumentResponse>(&payload)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upload_execution_document(
    _leg_id: u64,
    _document_name: &str,
    _document_type: &str,
    _input_id: &str,
) -> Result<ExecutionUploadDocumentResponse, String> {
    Err("Binary document upload is only available in the browser build of the Rust UI.".into())
}

#[cfg(target_arch = "wasm32")]
pub async fn open_protected_document(path: &str) -> Result<(), String> {
    stloads_open_protected_document(&api::api_href(path), &api::auth_token().unwrap_or_default())
        .await
        .map(|_| ())
        .map_err(|error| format!("Protected document open failed: {:?}", error))
}

#[cfg(target_arch = "wasm32")]
pub async fn download_protected_document(path: &str, file_name: &str) -> Result<(), String> {
    stloads_download_protected_document(
        &api::api_href(path),
        &api::auth_token().unwrap_or_default(),
        file_name,
    )
    .await
    .map(|_| ())
    .map_err(|error| format!("Protected document download failed: {:?}", error))
}

#[cfg(target_arch = "wasm32")]
pub async fn hash_selected_file(input_id: &str) -> Result<LocalFileHashResult, String> {
    let raw = stloads_hash_selected_file(input_id)
        .await
        .map_err(|error| format!("Local blockchain verification failed: {:?}", error))?;

    let payload = raw.as_string().ok_or_else(|| {
        "Local blockchain verification did not return a text payload.".to_string()
    })?;

    serde_json::from_str::<LocalFileHashResult>(&payload)
        .map_err(|error| format!("Local blockchain verification decoding failed: {}", error))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn hash_selected_file(_input_id: &str) -> Result<LocalFileHashResult, String> {
    Err("Local blockchain verification is only available in the browser build.".into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn open_protected_document(_path: &str) -> Result<(), String> {
    Err("Protected document viewing is only available in the browser build of the Rust UI.".into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn download_protected_document(_path: &str, _file_name: &str) -> Result<(), String> {
    Err("Protected document download is only available in the browser build of the Rust UI.".into())
}

#[cfg(target_arch = "wasm32")]
fn decode_envelope<T>(raw: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let envelope = serde_json::from_str::<ApiResponse<T>>(raw)
        .map_err(|error| format!("Failed to decode document upload response: {}", error))?;
    Ok(envelope.data)
}
