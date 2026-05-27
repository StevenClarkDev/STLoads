use leptos::{prelude::*, tachys::view::any_view::IntoAny};

use super::{
    load_profile_helpers::tone_style,
    shared::{
        FIELD_LABEL_STYLE, MUTED_TEXT_STYLE, TABLE_HEAD_CELL_STYLE, TABLE_HEADER_STYLE,
        human_file_size,
    },
};
use shared::{LoadDocumentRow, RequiredDocumentChecklistItem};

pub(super) fn render_load_documents_panel(
    load_id: u64,
    can_manage_documents: bool,
    required_documents: Vec<RequiredDocumentChecklistItem>,
    documents: Vec<LoadDocumentRow>,
    generating_documents: RwSignal<bool>,
    upload_document_name: RwSignal<String>,
    upload_document_type: RwSignal<String>,
    upload_input_id: String,
    is_uploading_document: RwSignal<bool>,
    document_name: RwSignal<String>,
    document_type: RwSignal<String>,
    file_path: RwSignal<String>,
    original_name: RwSignal<String>,
    mime_type: RwSignal<String>,
    file_size_input: RwSignal<String>,
    is_saving_document: RwSignal<bool>,
    opening_document_id: RwSignal<Option<u64>>,
    verifying_document_id: RwSignal<Option<u64>>,
    generate_standard_documents: impl Fn(u64) + Copy + Send + Sync + 'static,
    clear_upload_form: impl Fn() + Copy + Send + Sync + 'static,
    clear_document_form: impl Fn() + Copy + Send + Sync + 'static,
    upload_document: impl Fn() + Copy + Send + Sync + 'static,
    save_document: impl Fn() + Copy + Send + Sync + 'static,
    open_document: impl Fn(u64, String) + Copy + Send + Sync + 'static,
    download_document: impl Fn(u64, String, String) + Copy + Send + Sync + 'static,
    start_edit_document: impl Fn(LoadDocumentRow) + Copy + Send + Sync + 'static,
    verify_document: impl Fn(u64) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:1rem;overflow:auto;">
            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <strong>"Documents"</strong>
                {can_manage_documents.then(|| view! {
                    <div style="display:flex;gap:0.6rem;flex-wrap:wrap;">
                        <button
                            type="button"
                            style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:1px solid #d1d5db;background:#111827;color:white;cursor:pointer;"
                            disabled=move || generating_documents.get()
                                                    on:click=move |_| generate_standard_documents(load_id)
                        >
                            {move || if generating_documents.get() { "Generating..." } else { "Generate freight docs" }}
                        </button>
                        <button
                            type="button"
                            style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;cursor:pointer;"
                            on:click=move |_| { clear_upload_form(); clear_document_form(); }
                        >
                            "Reset forms"
                        </button>
                    </div>
                })}
            </div>

            {(!required_documents.is_empty()).then(|| view! {
                <div style="display:grid;gap:0.45rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                    <strong>"Required document checklist"</strong>
                    {required_documents.into_iter().map(|item| view! {
                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                            <span>{format!("{} - {}", item.label, item.lifecycle_state)}</span>
                            <small style=tone_style(&item.status_tone)>{item.status_label}</small>
                        </div>
                    }).collect_view()}
                </div>
            })}

            {can_manage_documents.then(|| view! {
                <form
                    style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fcfcfb;"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        upload_document();
                    }
                >
                    <strong>"Upload a document"</strong>
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                        <label style=FIELD_LABEL_STYLE>
                            <span>"Document name"</span>
                            <input type="text" prop:value=move || upload_document_name.get() on:input=move |ev| upload_document_name.set(event_target_value(&ev)) placeholder="Rate confirmation" />
                        </label>
                        <label style=FIELD_LABEL_STYLE>
                            <span>"Document type"</span>
                            <input type="text" prop:value=move || upload_document_type.get() on:input=move |ev| upload_document_type.set(event_target_value(&ev)) placeholder="rate_confirmation" />
                        </label>
                    </div>
                    <label style=FIELD_LABEL_STYLE>
                        <span>"Choose file"</span>
                        <input id=upload_input_id.clone() type="file" />
                    </label>
                    <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                        <button type="submit" style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;" disabled=move || is_uploading_document.get()>
                            {move || if is_uploading_document.get() { "Uploading..." } else { "Upload document" }}
                        </button>
                    </div>
                </form>
            })}

            {can_manage_documents.then(|| view! {
                <form
                    style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#f8fafc;"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        save_document();
                    }
                >
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                        <label style=FIELD_LABEL_STYLE>
                            <span>"Document name"</span>
                            <input type="text" prop:value=move || document_name.get() on:input=move |ev| document_name.set(event_target_value(&ev)) placeholder="Rate confirmation" />
                        </label>
                        <label style=FIELD_LABEL_STYLE>
                            <span>"Document type"</span>
                            <input type="text" prop:value=move || document_type.get() on:input=move |ev| document_type.set(event_target_value(&ev)) placeholder="rate_confirmation" />
                        </label>
                    </div>
                    <label style=FIELD_LABEL_STYLE>
                        <span>"Storage path or URL"</span>
                        <input type="text" prop:value=move || file_path.get() on:input=move |ev| file_path.set(event_target_value(&ev)) placeholder="ibm-cos://bucket/load-docs/rate-confirmation.pdf" />
                    </label>
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.85rem;">
                        <label style=FIELD_LABEL_STYLE>
                            <span>"Original file name"</span>
                            <input type="text" prop:value=move || original_name.get() on:input=move |ev| original_name.set(event_target_value(&ev)) placeholder="rate-confirmation.pdf" />
                        </label>
                        <label style=FIELD_LABEL_STYLE>
                            <span>"MIME type"</span>
                            <input type="text" prop:value=move || mime_type.get() on:input=move |ev| mime_type.set(event_target_value(&ev)) placeholder="application/pdf" />
                        </label>
                        <label style=FIELD_LABEL_STYLE>
                            <span>"File size (bytes)"</span>
                            <input type="number" min="0" step="1" prop:value=move || file_size_input.get() on:input=move |ev| file_size_input.set(event_target_value(&ev)) placeholder="1048576" />
                        </label>
                    </div>
                    <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                        <button type="submit" style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;" disabled=move || is_saving_document.get()>
                            {move || if is_saving_document.get() {
                                "Saving..."
                            } else {
                                "Save metadata"
                            }}
                        </button>
                    </div>
                </form>
            })}

            {documents.is_empty().then(|| view! {
                <p style="margin:0;">"No documents are attached yet. Upload the first one here to start the Rust-side document workflow."</p>
            })}

            {(!documents.is_empty()).then(|| view! {
                <table style="width:100%;border-collapse:collapse;min-width:720px;">
                    <thead style=TABLE_HEADER_STYLE>
                        <tr>
                            <th style=TABLE_HEAD_CELL_STYLE>"Name"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"File"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Content hash"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Uploaded"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {documents.into_iter().map(|document| {
                            let file_meta = format!("{} | {}", document.mime_type.clone().unwrap_or_else(|| "unknown mime".into()), human_file_size(document.file_size_bytes));
                            let blockchain_badge = document.blockchain_label.clone().map(|label| {
                                let tone = document.blockchain_tone.clone().unwrap_or_else(|| "secondary".into());
                                view! { <span style=tone_style(&tone)>{label}</span> }.into_any()
                            });
                            let can_edit_row = document.can_edit;
                            let can_verify_row = document.can_verify_blockchain;
                            let can_view_row = document.can_view_file && document.download_path.is_some();
                            let edit_row = document.clone();
                            let document_id = document.id;
                            let download_path = document.download_path.clone();
                            let file_name = document
                                .original_name
                                .clone()
                                .unwrap_or_else(|| document.file_label.clone());
                            let can_preview_row = document.mime_type.clone().map(|mime| {
                                mime.starts_with("image/") || mime.eq_ignore_ascii_case("application/pdf")
                            }).unwrap_or(false);
                            view! {
                                <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
                                    <td style="padding:0.75rem;display:grid;gap:0.3rem;">
                                        <strong>{document.document_name}</strong>
                                        <small>{document.document_type_label}</small>
                                    </td>
                                    <td style="padding:0.75rem;display:grid;gap:0.35rem;">
                                        <strong>{document.file_label}</strong>
                                        <small>{file_meta}</small>
                                        <small style="color:#64748b;word-break:break-all;">{document.source_path}</small>
                                        {document.uploaded_by_label.clone().map(|label| view! { <small style=MUTED_TEXT_STYLE>{label}</small> })}
                                    </td>
                                    <td style="padding:0.75rem;display:grid;gap:0.35rem;">
                                        {blockchain_badge.unwrap_or_else(|| view! { <span>"Hash not stored yet"</span> }.into_any())}
                                        {document.blockchain_hash_preview.clone().map(|preview| view! {
                                            <small style=MUTED_TEXT_STYLE>{format!("Hash: {}", preview)}</small>
                                        })}
                                    </td>
                                    <td style="padding:0.75rem;display:grid;gap:0.25rem;">
                                        <span>{document.uploaded_at_label}</span>
                                        <small style=MUTED_TEXT_STYLE>{document.version_history_label}</small>
                                    </td>
                                    <td style="padding:0.75rem;display:grid;gap:0.5rem;min-width:190px;">
                                        {can_view_row.then(|| {
                                            let preview_path = download_path.clone().unwrap_or_default();
                                            let download_path = download_path.clone().unwrap_or_default();
                                            let file_name = file_name.clone();
                                            view! {
                                                <div style=FIELD_LABEL_STYLE>
                                                    <button
                                                        type="button"
                                                        style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:none;background:#1d4ed8;color:white;cursor:pointer;"
                                                        disabled=move || opening_document_id.get() == Some(document_id)
                                                        on:click=move |_| open_document(document_id, preview_path.clone())
                                                    >
                                                        {move || if opening_document_id.get() == Some(document_id) {
                                                            "Opening...".to_string()
                                                        } else if can_preview_row {
                                                            "Preview".to_string()
                                                        } else {
                                                            "View file".to_string()
                                                        }}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;cursor:pointer;"
                                                        disabled=move || opening_document_id.get() == Some(document_id)
                                                        on:click=move |_| download_document(document_id, download_path.clone(), file_name.clone())
                                                    >
                                                        {move || if opening_document_id.get() == Some(document_id) { "Preparing..." } else { "Download" }}
                                                    </button>
                                                </div>
                                            }
                                        })}
                                        {can_edit_row.then(|| view! {
                                            <button
                                                type="button"
                                                style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;cursor:pointer;"
                                                on:click=move |_| start_edit_document(edit_row.clone())
                                            >
                                                "Edit row"
                                            </button>
                                        })}
                                        {can_verify_row.then(|| view! {
                                            <button
                                                type="button"
                                                style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:none;background:#0f766e;color:white;cursor:pointer;"
                                                disabled=move || verifying_document_id.get() == Some(document_id)
                                                on:click=move |_| verify_document(document_id)
                                            >
                                                {move || if verifying_document_id.get() == Some(document_id) { "Hashing..." } else { "Verify content hash" }}
                                            </button>
                                        })}
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            })}
        </section>
    }
}
