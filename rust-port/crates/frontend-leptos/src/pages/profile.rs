use std::collections::BTreeMap;

use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;

use crate::{
    api, document_upload,
    session::{self, use_auth},
};
use shared::{
    KycDocumentItem, SelfProfileFact, SelfProfileScreen, UpdateSelfProfileRequest,
    UpsertKycDocumentRequest, VerifyKycDocumentRequest,
};

fn fact_grid(title: &'static str, facts: Vec<SelfProfileFact>) -> impl IntoView {
    view! {
        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fff;display:grid;gap:0.65rem;">
            <strong>{title}</strong>
            {if facts.is_empty() {
                view! { <p style="margin:0;color:#64748b;">"No profile facts are recorded yet."</p> }.into_any()
            } else {
                facts.into_iter().map(|fact| view! {
                    <p style="margin:0;"><strong>{fact.label}</strong>" : "{fact.value}</p>
                }).collect_view().into_any()
            }}
        </section>
    }
}

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => {
            "background:#e8fff3;padding:0.25rem 0.55rem;border-radius:999px;color:#0f766e;"
        }
        "warning" => {
            "background:#fff7dd;padding:0.25rem 0.55rem;border-radius:999px;color:#b45309;"
        }
        "danger" => "background:#ffe4e6;padding:0.25rem 0.55rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.55rem;border-radius:999px;color:#0369a1;",
        _ => "background:#f1f5f9;padding:0.25rem 0.55rem;border-radius:999px;color:#475569;",
    }
}

fn human_file_size(bytes: Option<u64>) -> String {
    match bytes {
        Some(value) if value >= 1024 * 1024 => format!("{:.1} MB", value as f64 / 1024.0 / 1024.0),
        Some(value) if value >= 1024 => format!("{:.1} KB", value as f64 / 1024.0),
        Some(value) => format!("{} B", value),
        None => "Size not recorded".into(),
    }
}

#[derive(Clone)]
struct LocalVerifyOutcome {
    file_name: String,
    hash_preview: String,
    matches: bool,
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<SelfProfileScreen>);
    let loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);

    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone_no = RwSignal::new(String::new());
    let address = RwSignal::new(String::new());
    let company_name = RwSignal::new(String::new());
    let dot_number = RwSignal::new(String::new());
    let mc_number = RwSignal::new(String::new());
    let mc_cbsa_usdot_no = RwSignal::new(String::new());
    let ucr_hcc_no = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirmation = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let upload_document_name = RwSignal::new(String::new());
    let upload_document_type = RwSignal::new("standard".to_string());
    let uploading_document = RwSignal::new(false);

    let editing_document_id = RwSignal::new(None::<u64>);
    let editing_document_name = RwSignal::new(String::new());
    let editing_document_type = RwSignal::new("standard".to_string());
    let saving_document = RwSignal::new(false);
    let replacing_document_id = RwSignal::new(None::<u64>);
    let verifying_document_id = RwSignal::new(None::<u64>);
    let deleting_document_id = RwSignal::new(None::<u64>);
    let opening_document_id = RwSignal::new(None::<u64>);
    let local_verify_loading_id = RwSignal::new(None::<u64>);
    let local_verify_results = RwSignal::new(BTreeMap::<u64, LocalVerifyOutcome>::new());

    let can_view = Signal::derive(move || auth.session.get().authenticated);

    let reset_document_forms = move || {
        upload_document_name.set(String::new());
        upload_document_type.set("standard".to_string());
        editing_document_id.set(None);
        editing_document_name.set(String::new());
        editing_document_type.set("standard".to_string());
    };

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();

        if !auth.session_ready.get() || !can_view.get() {
            return;
        }

        loading.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::fetch_self_profile_screen().await {
                Ok(next) => {
                    name.set(next.draft.name.clone());
                    email.set(next.draft.email.clone());
                    phone_no.set(next.draft.phone_no.clone().unwrap_or_default());
                    address.set(next.draft.address.clone().unwrap_or_default());
                    company_name.set(next.draft.company_name.clone().unwrap_or_default());
                    dot_number.set(next.draft.dot_number.clone().unwrap_or_default());
                    mc_number.set(next.draft.mc_number.clone().unwrap_or_default());
                    mc_cbsa_usdot_no.set(next.draft.mc_cbsa_usdot_no.clone().unwrap_or_default());
                    ucr_hcc_no.set(next.draft.ucr_hcc_no.clone().unwrap_or_default());
                    screen.set(Some(next));
                    feedback.set(None);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    });

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        saving.set(true);
        let auth = auth.clone();

        let payload = UpdateSelfProfileRequest {
            name: name.get(),
            email: email.get(),
            phone_no: optional_string(phone_no.get()),
            address: optional_string(address.get()),
            company_name: optional_string(company_name.get()),
            dot_number: optional_string(dot_number.get()),
            mc_number: optional_string(mc_number.get()),
            mc_cbsa_usdot_no: optional_string(mc_cbsa_usdot_no.get()),
            ucr_hcc_no: optional_string(ucr_hcc_no.get()),
            password: optional_string(password.get()),
            password_confirmation: optional_string(password_confirmation.get()),
        };

        spawn_local(async move {
            match api::update_self_profile(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message.clone()));
                    if response.success {
                        if let Some(session_state) = response.session {
                            auth.session.set(session_state);
                            auth.notice.set(Some(response.message));
                        }
                        password.set(String::new());
                        password_confirmation.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            saving.set(false);
        });
    };

    let start_edit_document = move |document: KycDocumentItem| {
        editing_document_id.set(Some(document.id));
        editing_document_name.set(document.document_name);
        editing_document_type.set(document.document_type);
        feedback.set(Some(
            "This KYC row is loaded into the Rust revision editor now.".into(),
        ));
    };

    let upload_document = move || {
        let document_name_value = upload_document_name.get();
        let document_type_value = upload_document_type.get();
        if document_name_value.trim().is_empty() {
            feedback.set(Some(
                "Enter a document name before uploading a new KYC file.".into(),
            ));
            return;
        }

        uploading_document.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            match document_upload::upload_profile_kyc_document(
                &document_name_value,
                &document_type_value,
                document_upload::profile_kyc_upload_input_id(),
            )
            .await
            {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        reset_document_forms();
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            uploading_document.set(false);
        });
    };

    let save_document_metadata = move || {
        let Some(document_id) = editing_document_id.get() else {
            feedback.set(Some(
                "Choose an existing KYC row before saving metadata edits.".into(),
            ));
            return;
        };

        let document_name_value = editing_document_name.get();
        let document_type_value = editing_document_type.get();
        if document_name_value.trim().is_empty() {
            feedback.set(Some(
                "Enter a document name before saving this KYC row.".into(),
            ));
            return;
        }

        saving_document.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::update_profile_kyc_document(
                document_id,
                &UpsertKycDocumentRequest {
                    document_name: document_name_value,
                    document_type: document_type_value,
                },
            )
            .await
            {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            saving_document.set(false);
        });
    };

    let replace_document_file = move || {
        let Some(document_id) = editing_document_id.get() else {
            feedback.set(Some(
                "Choose an existing KYC row before replacing its file.".into(),
            ));
            return;
        };

        let document_name_value = editing_document_name.get();
        let document_type_value = editing_document_type.get();
        if document_name_value.trim().is_empty() {
            feedback.set(Some(
                "Enter a document name before replacing this KYC file.".into(),
            ));
            return;
        }

        let input_id = document_upload::profile_kyc_replace_input_id(document_id);
        replacing_document_id.set(Some(document_id));
        let auth = auth.clone();

        spawn_local(async move {
            match document_upload::replace_profile_kyc_document(
                document_id,
                &document_name_value,
                &document_type_value,
                &input_id,
            )
            .await
            {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            replacing_document_id.set(None);
        });
    };

    let verify_document = move |document_id: u64| {
        verifying_document_id.set(Some(document_id));
        let auth = auth.clone();

        spawn_local(async move {
            match api::verify_profile_kyc_document(
                document_id,
                &VerifyKycDocumentRequest {
                    note: Some("Triggered from the Rust self-service revision workspace.".into()),
                },
            )
            .await
            {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            verifying_document_id.set(None);
        });
    };

    let delete_document = move |document_id: u64| {
        deleting_document_id.set(Some(document_id));
        let auth = auth.clone();

        spawn_local(async move {
            match api::delete_profile_kyc_document(document_id).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        if editing_document_id.get() == Some(document_id) {
                            editing_document_id.set(None);
                            editing_document_name.set(String::new());
                            editing_document_type.set("standard".to_string());
                        }
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            deleting_document_id.set(None);
        });
    };

    let open_document = move |document_id: u64, path: String| {
        opening_document_id.set(Some(document_id));
        spawn_local(async move {
            if let Err(error) = document_upload::open_protected_document(&path).await {
                feedback.set(Some(error));
            }
            opening_document_id.set(None);
        });
    };

    let verify_local_document = move |document_id: u64, stored_hash: Option<String>| {
        let Some(stored_hash) = stored_hash.filter(|value| !value.trim().is_empty()) else {
            feedback.set(Some(
                "This KYC row does not have a stored blockchain hash yet.".into(),
            ));
            return;
        };

        let input_id = document_upload::profile_kyc_verify_input_id(document_id);
        local_verify_loading_id.set(Some(document_id));

        spawn_local(async move {
            match document_upload::hash_selected_file(&input_id).await {
                Ok(result) => {
                    let matches = result.hash.eq_ignore_ascii_case(&stored_hash);
                    let hash_preview = if result.hash.len() > 24 {
                        format!("{}...", &result.hash[..24])
                    } else {
                        result.hash.clone()
                    };

                    local_verify_results.update(|rows| {
                        rows.insert(
                            document_id,
                            LocalVerifyOutcome {
                                file_name: result.file_name.clone(),
                                hash_preview,
                                matches,
                            },
                        );
                    });

                    feedback.set(Some(if matches {
                        format!(
                            "Local SHA-256 verification passed for document #{}.",
                            document_id
                        )
                    } else {
                        format!(
                            "Local SHA-256 verification failed for document #{}. The selected file does not match the anchored hash.",
                            document_id
                        )
                    }));
                }
                Err(error) => {
                    feedback.set(Some(error));
                }
            }

            local_verify_loading_id.set(None);
        });
    };

    view! {
        <article style="display:grid;gap:1rem;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.3rem;">
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "My Profile".into())}</h2>
                    <p>{move || screen.get().map(|value| format!("{} | {}", value.role_label, value.status_label)).unwrap_or_else(|| "Rust self-service profile".into())}</p>
                </div>
                <div style="display:flex;gap:0.6rem;flex-wrap:wrap;">
                    <A href="/" attr:style="padding:0.6rem 0.85rem;border-radius:0.8rem;background:#f4f4f5;color:#111827;text-decoration:none;">
                        "Back to dashboard"
                    </A>
                    <A href="/auth/onboarding" attr:style="padding:0.6rem 0.85rem;border-radius:0.8rem;background:#eef2ff;color:#312e81;text-decoration:none;">
                        "Open onboarding"
                    </A>
                </div>
            </section>

            {move || feedback.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;white-space:pre-wrap;">
                    {message}
                </section>
            })}

            {move || {
                if !auth.session_ready.get() {
                    view! { <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"Loading Rust session..."</section> }.into_any()
                } else if !can_view.get() {
                    view! { <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"Sign in before opening the Rust self-service profile."</section> }.into_any()
                } else if loading.get() && screen.get().is_none() {
                    view! { <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"Loading profile from the Rust backend..."</section> }.into_any()
                } else if let Some(screen_value) = screen.get() {
                    let show_carrier_fields = screen_value.role_key == "carrier";
                    let show_broker_fields = screen_value.role_key == "broker";
                    let documents = screen_value.documents.clone();
                    view! {
                        <>
                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;">
                                {fact_grid("Personal Facts", screen_value.personal_facts.clone())}
                                {fact_grid("Company Facts", screen_value.company_facts.clone())}
                            </section>

                            <form on:submit=submit style="display:grid;gap:1rem;padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fff;">
                                <div style="display:grid;gap:0.35rem;">
                                    <strong>"Edit profile"</strong>
                                    <small style="color:#64748b;">"This Rust screen replaces the old profile view and edit Blade pages with one self-serve workspace."</small>
                                </div>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                    <label style="display:grid;gap:0.35rem;"><span>"Name"</span><input type="text" prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev)) /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Email"</span><input type="email" prop:value=move || email.get() on:input=move |ev| email.set(event_target_value(&ev)) /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Phone"</span><input type="text" prop:value=move || phone_no.get() on:input=move |ev| phone_no.set(event_target_value(&ev)) /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Company"</span><input type="text" prop:value=move || company_name.get() on:input=move |ev| company_name.set(event_target_value(&ev)) /></label>
                                </div>
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Address"</span>
                                    <textarea rows="2" prop:value=move || address.get() on:input=move |ev| address.set(event_target_value(&ev)) style="padding:0.75rem 0.85rem;border:1px solid #d6d3d1;border-radius:0.85rem;resize:vertical;" />
                                </label>

                                {show_carrier_fields.then(|| view! {
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                        <label style="display:grid;gap:0.35rem;"><span>"DOT Number"</span><input type="text" prop:value=move || dot_number.get() on:input=move |ev| dot_number.set(event_target_value(&ev)) /></label>
                                        <label style="display:grid;gap:0.35rem;"><span>"MC Number"</span><input type="text" prop:value=move || mc_number.get() on:input=move |ev| mc_number.set(event_target_value(&ev)) /></label>
                                    </div>
                                })}

                                {show_broker_fields.then(|| view! {
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                        <label style="display:grid;gap:0.35rem;"><span>"MC/CBSA/USDOT"</span><input type="text" prop:value=move || mc_cbsa_usdot_no.get() on:input=move |ev| mc_cbsa_usdot_no.set(event_target_value(&ev)) /></label>
                                        <label style="display:grid;gap:0.35rem;"><span>"UCR/HCC"</span><input type="text" prop:value=move || ucr_hcc_no.get() on:input=move |ev| ucr_hcc_no.set(event_target_value(&ev)) /></label>
                                    </div>
                                })}

                                <div style="display:grid;gap:0.35rem;padding-top:0.25rem;border-top:1px solid #e5e7eb;">
                                    <strong>"Change password"</strong>
                                    <small style="color:#64748b;">"Leave both fields empty if you do not want to change the password."</small>
                                </div>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                    <label style="display:grid;gap:0.35rem;"><span>"New password"</span><input type="password" prop:value=move || password.get() on:input=move |ev| password.set(event_target_value(&ev)) /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Confirm password"</span><input type="password" prop:value=move || password_confirmation.get() on:input=move |ev| password_confirmation.set(event_target_value(&ev)) /></label>
                                </div>

                                <div style="display:flex;justify-content:flex-end;">
                                    <button type="submit" disabled=move || saving.get() style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;">
                                        {move || if saving.get() { "Saving..." } else { "Save profile" }}
                                    </button>
                                </div>
                            </form>

                            <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fff;display:grid;gap:1rem;">
                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                    <div style="display:grid;gap:0.35rem;">
                                        <strong>"KYC revision workspace"</strong>
                                        <small style="color:#64748b;">
                                            "This Rust document workbench replaces the heavier revision-oriented Blade workflow. Each change sends the profile back into admin review, just like the PHP flow."
                                        </small>
                                        <small style="color:#64748b;">
                                            "Blockchain rows can now verify a local file with SHA-256 in the browser, which closes one of the last big PHP-only profile behaviors."
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
                                    <div style="display:grid;gap:0.35rem;">
                                        <strong>"Add a new KYC row"</strong>
                                        <small style="color:#64748b;">"New rows require a file. Choose blockchain to anchor the row as part of the upload step."</small>
                                    </div>
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                        <label style="display:grid;gap:0.35rem;">
                                            <span>"Document name"</span>
                                            <input type="text" prop:value=move || upload_document_name.get() on:input=move |ev| upload_document_name.set(event_target_value(&ev)) placeholder="Certificate of Insurance" />
                                        </label>
                                        <label style="display:grid;gap:0.35rem;">
                                            <span>"Document type"</span>
                                            <select prop:value=move || upload_document_type.get() on:change=move |ev| upload_document_type.set(event_target_value(&ev))>
                                                <option value="standard">"Standard"</option>
                                                <option value="blockchain">"Blockchain"</option>
                                            </select>
                                        </label>
                                    </div>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Choose file"</span>
                                        <input id=document_upload::profile_kyc_upload_input_id() type="file" />
                                    </label>
                                    <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                                        <button type="submit" style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;" disabled=move || uploading_document.get()>
                                            {move || if uploading_document.get() { "Uploading..." } else { "Add KYC row" }}
                                        </button>
                                        <small style="color:#64748b;">"25 MB limit in the current Rust slice."</small>
                                    </div>
                                </form>

                                <form
                                    style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#f8fafc;"
                                    on:submit=move |ev| {
                                        ev.prevent_default();
                                        save_document_metadata();
                                    }
                                >
                                    <div style="display:grid;gap:0.35rem;">
                                        <strong>"Edit selected KYC row"</strong>
                                        <small style="color:#64748b;">{move || editing_document_id.get().map(|id| format!("Editing document #{}", id)).unwrap_or_else(|| "Choose a row below to edit metadata or replace its file.".into())}</small>
                                    </div>
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                        <label style="display:grid;gap:0.35rem;">
                                            <span>"Document name"</span>
                                            <input type="text" prop:value=move || editing_document_name.get() on:input=move |ev| editing_document_name.set(event_target_value(&ev)) placeholder="Certificate of Insurance" />
                                        </label>
                                        <label style="display:grid;gap:0.35rem;">
                                            <span>"Document type"</span>
                                            <select prop:value=move || editing_document_type.get() on:change=move |ev| editing_document_type.set(event_target_value(&ev))>
                                                <option value="standard">"Standard"</option>
                                                <option value="blockchain">"Blockchain"</option>
                                            </select>
                                        </label>
                                    </div>
                                    <label style="display:grid;gap:0.35rem;">
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

                                {if documents.is_empty() {
                                    view! { <p style="margin:0;color:#64748b;">"No KYC documents are attached yet."</p> }.into_any()
                                } else {
                                    view! {
                                        <table style="width:100%;border-collapse:collapse;min-width:760px;">
                                            <thead style="background:#f8fafc;">
                                                <tr>
                                                    <th style="text-align:left;padding:0.75rem;">"Document"</th>
                                                    <th style="text-align:left;padding:0.75rem;">"File"</th>
                                                    <th style="text-align:left;padding:0.75rem;">"Blockchain"</th>
                                                    <th style="text-align:left;padding:0.75rem;">"Uploaded"</th>
                                                    <th style="text-align:left;padding:0.75rem;">"Actions"</th>
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
                                                                {blockchain_badge.unwrap_or_else(|| view! { <span>"Not anchored yet"</span> }.into_any())}
                                                                {document.blockchain_hash_preview.clone().map(|preview| view! {
                                                                    <small style="color:#64748b;">{preview}</small>
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
                                                            <td style="padding:0.75rem;">{document.uploaded_at_label.clone()}</td>
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
                                                                        {move || if verifying_document_id.get() == Some(document_id) { "Anchoring..." } else { "Anchor to blockchain" }}
                                                                    </button>
                                                                })}
                                                                {stored_blockchain_hash.clone().filter(|_| document.document_type.eq_ignore_ascii_case("blockchain")).map(|stored_hash| view! {
                                                                    <div style="display:grid;gap:0.35rem;">
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

                            <section style="display:grid;gap:0.35rem;">
                                {screen_value.notes.into_iter().map(|note| view! { <p style="margin:0;">{note}</p> }).collect_view()}
                            </section>
                        </>
                    }.into_any()
                } else {
                    view! { <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"No Rust profile data is available yet."</section> }.into_any()
                }
            }}
        </article>
    }
}

fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
