use std::collections::BTreeMap;

use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;

use super::shared::{FIELD_LABEL_STYLE, MUTED_TEXT_STYLE, tone_style};
use crate::{
    api, document_upload,
    session::{self, use_auth},
};
use shared::{
    KycDocumentItem, SelfProfileScreen, UpdateCarrierCapacityRequest, UpdateSelfProfileRequest,
    UpsertKycDocumentRequest, VerifyKycDocumentRequest,
};

use super::profile_helpers::*;
use super::profile_kyc_helpers::render_kyc_workspace;

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
    let capacity_equipment_types = RwSignal::new(String::new());
    let capacity_lane_preferences = RwSignal::new(String::new());
    let capacity_operating_regions = RwSignal::new(String::new());
    let capacity_preferred_commodities = RwSignal::new(String::new());
    let capacity_service_levels = RwSignal::new(String::new());
    let capacity_certifications = RwSignal::new(String::new());
    let capacity_availability_status = RwSignal::new("available".to_string());
    let capacity_available_power_units = RwSignal::new("0".to_string());
    let capacity_insurance_limit_usd = RwSignal::new("0".to_string());
    let capacity_notes = RwSignal::new(String::new());
    let saving_capacity = RwSignal::new(false);

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
        let auth = auth;
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
                    if let Some(capacity) = next.carrier_capacity.as_ref() {
                        capacity_equipment_types
                            .set(join_capacity_values(&capacity.equipment_types));
                        capacity_lane_preferences
                            .set(join_capacity_values(&capacity.lane_preferences));
                        capacity_operating_regions
                            .set(join_capacity_values(&capacity.operating_regions));
                        capacity_preferred_commodities
                            .set(join_capacity_values(&capacity.preferred_commodities));
                        capacity_service_levels.set(join_capacity_values(&capacity.service_levels));
                        capacity_certifications.set(join_capacity_values(&capacity.certifications));
                        capacity_availability_status.set(capacity.availability_status.clone());
                        capacity_available_power_units
                            .set(capacity.available_power_units.to_string());
                        capacity_insurance_limit_usd
                            .set(format!("{:.0}", capacity.insurance_limit_usd));
                        capacity_notes.set(capacity.capacity_notes.clone().unwrap_or_default());
                    }
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
        let auth = auth;

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

    let save_capacity = move |ev: SubmitEvent| {
        ev.prevent_default();
        let power_units = capacity_available_power_units
            .get()
            .trim()
            .parse::<u32>()
            .unwrap_or(0);
        let insurance_limit = capacity_insurance_limit_usd
            .get()
            .trim()
            .parse::<f64>()
            .unwrap_or(0.0);
        let payload = UpdateCarrierCapacityRequest {
            equipment_types: split_capacity_values(capacity_equipment_types.get()),
            lane_preferences: split_capacity_values(capacity_lane_preferences.get()),
            operating_regions: split_capacity_values(capacity_operating_regions.get()),
            preferred_commodities: split_capacity_values(capacity_preferred_commodities.get()),
            service_levels: split_capacity_values(capacity_service_levels.get()),
            certifications: split_capacity_values(capacity_certifications.get()),
            availability_status: capacity_availability_status.get(),
            available_power_units: power_units,
            insurance_limit_usd: insurance_limit,
            capacity_notes: optional_capacity_notes(capacity_notes.get()),
        };

        saving_capacity.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::update_carrier_capacity(&payload).await {
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
            saving_capacity.set(false);
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
        let auth = auth;

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
        let auth = auth;
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
        let auth = auth;

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
        let auth = auth;

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
        let auth = auth;

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
                "This KYC row does not have a stored SHA-256 hash yet.".into(),
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
                            "Local SHA-256 verification failed for document #{}. The selected file does not match the stored hash.",
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
                    let required_documents = screen_value.required_documents.clone();
                    view! {
                        <>
                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;">
                                {fact_grid("Personal Facts", screen_value.personal_facts.clone())}
                                {fact_grid("Company Facts", screen_value.company_facts.clone())}
                            </section>

                            <form on:submit=submit style="display:grid;gap:1rem;padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fff;">
                                <div style=FIELD_LABEL_STYLE><strong>"Edit profile"</strong></div>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                    <label style=FIELD_LABEL_STYLE><span>"Name"</span><input type="text" prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev)) /></label>
                                    <label style=FIELD_LABEL_STYLE><span>"Email"</span><input type="email" prop:value=move || email.get() on:input=move |ev| email.set(event_target_value(&ev)) /></label>
                                    <label style=FIELD_LABEL_STYLE><span>"Phone"</span><input type="text" prop:value=move || phone_no.get() on:input=move |ev| phone_no.set(event_target_value(&ev)) /></label>
                                    <label style=FIELD_LABEL_STYLE><span>"Company"</span><input type="text" prop:value=move || company_name.get() on:input=move |ev| company_name.set(event_target_value(&ev)) /></label>
                                </div>
                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Address"</span>
                                    <textarea rows="2" prop:value=move || address.get() on:input=move |ev| address.set(event_target_value(&ev)) style="padding:0.75rem 0.85rem;border:1px solid #d6d3d1;border-radius:0.85rem;resize:vertical;" />
                                </label>

                                {show_carrier_fields.then(|| view! {
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                        <label style=FIELD_LABEL_STYLE><span>"DOT Number"</span><input type="text" prop:value=move || dot_number.get() on:input=move |ev| dot_number.set(event_target_value(&ev)) /></label>
                                        <label style=FIELD_LABEL_STYLE><span>"MC Number"</span><input type="text" prop:value=move || mc_number.get() on:input=move |ev| mc_number.set(event_target_value(&ev)) /></label>
                                    </div>
                                })}

                                {show_broker_fields.then(|| view! {
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                        <label style=FIELD_LABEL_STYLE><span>"MC/CBSA/USDOT"</span><input type="text" prop:value=move || mc_cbsa_usdot_no.get() on:input=move |ev| mc_cbsa_usdot_no.set(event_target_value(&ev)) /></label>
                                        <label style=FIELD_LABEL_STYLE><span>"UCR/HCC"</span><input type="text" prop:value=move || ucr_hcc_no.get() on:input=move |ev| ucr_hcc_no.set(event_target_value(&ev)) /></label>
                                    </div>
                                })}

                                <div style="display:grid;gap:0.35rem;padding-top:0.25rem;border-top:1px solid #e5e7eb;">
                                    <strong>"Change password"</strong>
                                    <small style=MUTED_TEXT_STYLE>"Leave both fields empty if you do not want to change the password."</small>
                                </div>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                    <label style=FIELD_LABEL_STYLE><span>"New password"</span><input type="password" prop:value=move || password.get() on:input=move |ev| password.set(event_target_value(&ev)) /></label>
                                    <label style=FIELD_LABEL_STYLE><span>"Confirm password"</span><input type="password" prop:value=move || password_confirmation.get() on:input=move |ev| password_confirmation.set(event_target_value(&ev)) /></label>
                                </div>

                                <div style="display:flex;justify-content:flex-end;">
                                    <button type="submit" disabled=move || saving.get() style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;">
                                        {move || if saving.get() { "Saving..." } else { "Save profile" }}
                                    </button>
                                </div>
                            </form>

                            {show_carrier_fields.then(|| {
                                let capacity = screen_value.carrier_capacity.clone();
                                view! {
                                    <form on:submit=save_capacity style="display:grid;gap:1rem;padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fff;">
                                        <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                            <div style=FIELD_LABEL_STYLE>
                                                <strong>"Carrier capacity profile"</strong>
                                                <small style=MUTED_TEXT_STYLE>"Structured capacity feeds eligibility, matching, and future tender controls."</small>
                                            </div>
                                            <span style=tone_style(capacity_tone(capacity.as_ref()))>
                                                {capacity.as_ref().map(|item| item.readiness_label.clone()).unwrap_or_else(|| "Capacity not configured".into())}
                                            </span>
                                        </div>

                                        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Equipment types"</span>
                                                <input type="text" placeholder="dry van, reefer" prop:value=move || capacity_equipment_types.get() on:input=move |ev| capacity_equipment_types.set(event_target_value(&ev)) />
                                            </label>
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Operating regions"</span>
                                                <input type="text" placeholder="tx, midwest, southeast" prop:value=move || capacity_operating_regions.get() on:input=move |ev| capacity_operating_regions.set(event_target_value(&ev)) />
                                            </label>
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Lane preferences"</span>
                                                <input type="text" placeholder="dallas to chicago, houston outbound" prop:value=move || capacity_lane_preferences.get() on:input=move |ev| capacity_lane_preferences.set(event_target_value(&ev)) />
                                            </label>
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Preferred commodities"</span>
                                                <input type="text" placeholder="consumer goods, food grade" prop:value=move || capacity_preferred_commodities.get() on:input=move |ev| capacity_preferred_commodities.set(event_target_value(&ev)) />
                                            </label>
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Service levels"</span>
                                                <input type="text" placeholder="standard, expedited" prop:value=move || capacity_service_levels.get() on:input=move |ev| capacity_service_levels.set(event_target_value(&ev)) />
                                            </label>
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Certifications"</span>
                                                <input type="text" placeholder="hazmat, twic, food grade" prop:value=move || capacity_certifications.get() on:input=move |ev| capacity_certifications.set(event_target_value(&ev)) />
                                            </label>
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Availability"</span>
                                                <select prop:value=move || capacity_availability_status.get() on:change=move |ev| capacity_availability_status.set(event_target_value(&ev))>
                                                    <option value="available">"Available"</option>
                                                    <option value="limited">"Limited"</option>
                                                    <option value="unavailable">"Unavailable"</option>
                                                    <option value="seasonal">"Seasonal"</option>
                                                    <option value="paused">"Paused"</option>
                                                </select>
                                            </label>
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Available power units"</span>
                                                <input type="number" min="0" prop:value=move || capacity_available_power_units.get() on:input=move |ev| capacity_available_power_units.set(event_target_value(&ev)) />
                                            </label>
                                            <label style=FIELD_LABEL_STYLE>
                                                <span>"Insurance limit USD"</span>
                                                <input type="number" min="0" step="1000" prop:value=move || capacity_insurance_limit_usd.get() on:input=move |ev| capacity_insurance_limit_usd.set(event_target_value(&ev)) />
                                            </label>
                                        </div>

                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"Capacity notes"</span>
                                            <textarea rows="2" prop:value=move || capacity_notes.get() on:input=move |ev| capacity_notes.set(event_target_value(&ev)) style="padding:0.75rem 0.85rem;border:1px solid #d6d3d1;border-radius:0.85rem;resize:vertical;" />
                                        </label>

                                        <div style="display:flex;justify-content:flex-end;">
                                            <button type="submit" disabled=move || saving_capacity.get() style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;">
                                                {move || if saving_capacity.get() { "Saving capacity..." } else { "Save capacity" }}
                                            </button>
                                        </div>
                                    </form>
                                }
                            })}

                            {render_kyc_workspace(required_documents, documents, upload_document_name, upload_document_type, uploading_document, editing_document_id, editing_document_name, editing_document_type, saving_document, replacing_document_id, local_verify_results, opening_document_id, verifying_document_id, local_verify_loading_id, deleting_document_id, reset_document_forms, upload_document, save_document_metadata, replace_document_file, start_edit_document, open_document, verify_document, verify_local_document, delete_document)}

                            <section style=FIELD_LABEL_STYLE>
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
