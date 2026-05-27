use std::collections::BTreeMap;

use leptos::{prelude::*, tachys::view::any_view::IntoAny};

use super::{
    profile_helpers::LocalVerifyOutcome,
    shared::{
        FIELD_LABEL_STYLE, MUTED_TEXT_STYLE, TABLE_HEAD_CELL_STYLE, TABLE_HEADER_STYLE,
        human_file_size, tone_style,
    },
};
use crate::document_upload;
use shared::{KycDocumentItem, RequiredDocumentChecklistItem};

pub(super) fn render_kyc_workspace(
    required_documents: Vec<RequiredDocumentChecklistItem>,
    documents: Vec<KycDocumentItem>,
    upload_document_name: RwSignal<String>,
    upload_document_type: RwSignal<String>,
    uploading_document: RwSignal<bool>,
    editing_document_id: RwSignal<Option<u64>>,
    editing_document_name: RwSignal<String>,
    editing_document_type: RwSignal<String>,
    saving_document: RwSignal<bool>,
    replacing_document_id: RwSignal<Option<u64>>,
    local_verify_results: RwSignal<BTreeMap<u64, LocalVerifyOutcome>>,
    opening_document_id: RwSignal<Option<u64>>,
    verifying_document_id: RwSignal<Option<u64>>,
    local_verify_loading_id: RwSignal<Option<u64>>,
    deleting_document_id: RwSignal<Option<u64>>,
    reset_document_forms: impl Fn() + Copy + Send + Sync + 'static,
    upload_document: impl Fn() + Copy + Send + Sync + 'static,
    save_document_metadata: impl Fn() + Copy + Send + Sync + 'static,
    replace_document_file: impl Fn() + Copy + Send + Sync + 'static,
    start_edit_document: impl Fn(KycDocumentItem) + Copy + Send + Sync + 'static,
    open_document: impl Fn(u64, String) + Copy + Send + Sync + 'static,
    verify_document: impl Fn(u64) + Copy + Send + Sync + 'static,
    verify_local_document: impl Fn(u64, Option<String>) + Copy + Send + Sync + 'static,
    delete_document: impl Fn(u64) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fff;display:grid;gap:1rem;">
            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style=FIELD_LABEL_STYLE>
                    <strong>"KYC revision workspace"</strong>
                    <small style=MUTED_TEXT_STYLE>
                        "This Rust document workbench replaces the heavier revision-oriented Blade workflow. Each change sends the profile back into admin review, just like the PHP flow."
                    </small>
                    <small style=MUTED_TEXT_STYLE>
                        "Hash-verification rows can verify a local file with SHA-256 in the browser."
                    </small>
                </div>
                <button
                    type="button"
                    style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;cursor:pointer;"
                    on:click=move |_| reset_document_forms()
                >
                    "Reset KYC forms"
                </button>
            </div>

            <form
                style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fcfcfb;"
                on:submit=move |ev| {
                    ev.prevent_default();
                    upload_document();
                }
            >
                <div style=FIELD_LABEL_STYLE>
                    <strong>"Add a new KYC row"</strong>
                    <small style=MUTED_TEXT_STYLE>"New rows require a file. Choose content hash to calculate SHA-256 as part of the upload step."</small>
                </div>
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                    <label style=FIELD_LABEL_STYLE>
                        <span>"Document name"</span>
                        <input type="text" prop:value=move || upload_document_name.get() on:input=move |ev| upload_document_name.set(event_target_value(&ev)) placeholder="Certificate of Insurance" />
                    </label>
                    <label style=FIELD_LABEL_STYLE>
                        <span>"Document type"</span>
                        <select prop:value=move || upload_document_type.get() on:change=move |ev| upload_document_type.set(event_target_value(&ev))>
                            <option value="standard">"Standard"</option>
                            <option value="Content hash">"Content hash"</option>
                        </select>
                    </label>
                </div>
                <label style=FIELD_LABEL_STYLE>
                    <span>"Choose file"</span>
                    <input id=document_upload::profile_kyc_upload_input_id() type="file" />
                </label>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <button type="submit" style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;" disabled=move || uploading_document.get()>
                        {move || if uploading_document.get() { "Uploading..." } else { "Add KYC row" }}
                    </button>
                    <small style=MUTED_TEXT_STYLE>"25 MB limit in the current Rust slice."</small>
                </div>
            </form>

            <form
                style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#f8fafc;"
                on:submit=move |ev| {
                    ev.prevent_default();
                    save_document_metadata();
                }
            >
                <div style=FIELD_LABEL_STYLE>
                    <strong>"Edit selected KYC row"</strong>
                    <small style=MUTED_TEXT_STYLE>{move || editing_document_id.get().map(|id| format!("Editing document #{}", id)).unwrap_or_else(|| "Choose a row below to edit metadata or replace its file.".into())}</small>
                </div>
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                    <label style=FIELD_LABEL_STYLE>
                        <span>"Document name"</span>
                        <input type="text" prop:value=move || editing_document_name.get() on:input=move |ev| editing_document_name.set(event_target_value(&ev)) placeholder="Certificate of Insurance" />
                    </label>
                    <label style=FIELD_LABEL_STYLE>
                        <span>"Document type"</span>
                        <select prop:value=move || editing_document_type.get() on:change=move |ev| editing_document_type.set(event_target_value(&ev))>
                            <option value="standard">"Standard"</option>
                            <option value="Content hash">"Content hash"</option>
                        </select>
                    </label>
                </div>
                <label style=FIELD_LABEL_STYLE>
                    <span>"Replace file"</span>
                    <input
                        id=move || editing_document_id.get().map(document_upload::profile_kyc_replace_input_id).unwrap_or_else(|| "profile-kyc-document-replace-idle".into())
                        type="file"
                    />
                </label>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <button type="submit" style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;" disabled=move || saving_document.get() || editing_document_id.get().is_none()>
                        {move || if saving_document.get() { "Saving..." } else { "Save row metadata" }}
                    </button>
                    <button
                        type="button"
                        style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#1d4ed8;color:white;cursor:pointer;"
                        disabled=move || replacing_document_id.get().is_some() || editing_document_id.get().is_none()
                        on:click=move |_| replace_document_file()
                    >
                        {move || if replacing_document_id.get().is_some() { "Replacing..." } else { "Replace file" }}
                    </button>
                </div>
            </form>

            {(!required_documents.is_empty()).then(|| view! {
                <div style="display:grid;gap:0.45rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                    <strong>"Required document checklist"</strong>
                    {required_documents.into_iter().map(|item| view! {
                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                            <span>{format!("{} - {}", item.label, item.requirement_scope)}</span>
                            <small style=tone_style(&item.status_tone)>{item.status_label}</small>
                        </div>
                    }).collect_view()}
                </div>
            })}

            {if documents.is_empty() {
                view! { <p style="margin:0;color:#64748b;">"No KYC documents are attached yet."</p> }.into_any()
            } else {
                view! {
                    <table style="width:100%;border-collapse:collapse;min-width:760px;">
                        <thead style=TABLE_HEADER_STYLE>
                            <tr>
                                <th style=TABLE_HEAD_CELL_STYLE>"Document"</th>
                                <th style=TABLE_HEAD_CELL_STYLE>"File"</th>
                                <th style=TABLE_HEAD_CELL_STYLE>"Content hash"</th>
                                <th style=TABLE_HEAD_CELL_STYLE>"Uploaded"</th>
                                <th style=TABLE_HEAD_CELL_STYLE>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {documents.into_iter().map(|document| {
                                let document_id = document.id;
                                let download_path = document.download_path.clone();
                                let verify_input_id = document_upload::profile_kyc_verify_input_id(document_id);
                                let stored_blockchain_hash = document.blockchain_hash.clone();
                                let file_meta = format!(
                                    "{} | {}",
                                    document.mime_type.clone().unwrap_or_else(|| "unknown mime".into()),
                                    human_file_size(document.file_size_bytes),
                                );
                                let blockchain_badge = document.blockchain_label.clone().map(|label| {
                                    let tone = document.blockchain_tone.clone().unwrap_or_else(|| "info".into());
                                    view! { <span style=tone_style(&tone)>{label}</span> }.into_any()
                                });
                                let edit_row = document.clone();
                                view! {
                                    <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
                                        <td style="padding:0.75rem;display:grid;gap:0.3rem;">
                                            <strong>{document.document_name}</strong>
                                            <small>{document.document_type.clone()}</small>
                                        </td>
                                        <td style="padding:0.75rem;display:grid;gap:0.35rem;">
                                            <strong>{document.file_label.clone()}</strong>
                                            <small>{file_meta}</small>
                                        </td>
                                        <td style="padding:0.75rem;display:grid;gap:0.35rem;">
                                            {blockchain_badge.unwrap_or_else(|| view! { <span>"Hash not stored yet"</span> }.into_any())}
                                            {document.blockchain_hash_preview.clone().map(|preview| view! {
                                                <small style=MUTED_TEXT_STYLE>{preview}</small>
                                            })}
                                            {move || local_verify_results.with(|rows| rows.get(&document_id).cloned()).map(|result| {
                                                let tone = if result.matches { "success" } else { "danger" };
                                                let summary = if result.matches {
                                                    format!("{} matched {}", result.file_name, result.hash_preview)
                                                } else {
                                                    format!("{} did not match {}", result.file_name, result.hash_preview)
                                                };
                                                view! {
                                                    <small style=tone_style(tone)>{summary}</small>
                                                }
                                            })}
                                        </td>
                                        <td style="padding:0.75rem;display:grid;gap:0.25rem;">
                                            <span>{document.uploaded_at_label.clone()}</span>
                                            <small style=MUTED_TEXT_STYLE>{document.version_history_label.clone()}</small>
                                        </td>
                                        <td style="padding:0.75rem;display:grid;gap:0.5rem;min-width:200px;">
                                            {document.can_view_file.then(|| {
                                                let path = download_path.clone().unwrap_or_default();
                                                view! {
                                                    <button
                                                        type="button"
                                                        style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:none;background:#1d4ed8;color:white;cursor:pointer;"
                                                        disabled=move || opening_document_id.get() == Some(document_id)
                                                        on:click=move |_| open_document(document_id, path.clone())
                                                    >
                                                        {move || if opening_document_id.get() == Some(document_id) { "Opening..." } else { "View file" }}
                                                    </button>
                                                }
                                            })}
                                            {document.can_edit.then(|| view! {
                                                <button
                                                    type="button"
                                                    style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;cursor:pointer;"
                                                    on:click=move |_| start_edit_document(edit_row.clone())
                                                >
                                                    "Edit row"
                                                </button>
                                            })}
                                            {document.can_verify_blockchain.then(|| view! {
                                                <button
                                                    type="button"
                                                    style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:none;background:#0f766e;color:white;cursor:pointer;"
                                                    disabled=move || verifying_document_id.get() == Some(document_id)
                                                    on:click=move |_| verify_document(document_id)
                                                >
                                                    {move || if verifying_document_id.get() == Some(document_id) { "Hashing..." } else { "Verify content hash" }}
                                                </button>
                                            })}
                                            {stored_blockchain_hash.clone().filter(|_| document.document_type.eq_ignore_ascii_case("blockchain")).map(|stored_hash| view! {
                                                <div style=FIELD_LABEL_STYLE>
                                                    <label
                                                        for=verify_input_id.clone()
                                                        style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:1px solid #0f766e;background:#ecfdf5;color:#065f46;cursor:pointer;text-align:center;"
                                                    >
                                                        {move || if local_verify_loading_id.get() == Some(document_id) { "Verifying local file..." } else { "Choose file to verify" }}
                                                    </label>
                                                    <input
                                                        id=verify_input_id.clone()
                                                        type="file"
                                                        style="display:none;"
                                                        on:change=move |_| verify_local_document(document_id, Some(stored_hash.clone()))
                                                    />
                                                </div>
                                            })}
                                            {document.can_delete.then(|| view! {
                                                <button
                                                    type="button"
                                                    style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:none;background:#be123c;color:white;cursor:pointer;"
                                                    disabled=move || deleting_document_id.get() == Some(document_id)
                                                    on:click=move |_| delete_document(document_id)
                                                >
                                                    {move || if deleting_document_id.get() == Some(document_id) { "Removing..." } else { "Delete row" }}
                                                </button>
                                            })}
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_any()
            }}
        </section>
    }
}
