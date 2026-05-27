use leptos::{ev, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_navigate};

use crate::{api, document_upload, session::use_auth};
use shared::{AuthOnboardingScreen, SubmitOnboardingRequest};

use crate::pages::auth_helpers::*;
#[component]
pub fn OnboardingPage() -> impl IntoView {
    let navigate = use_navigate();
    let auth = use_auth();
    let screen = RwSignal::new(None::<AuthOnboardingScreen>);
    let feedback = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let saving = RwSignal::new(false);

    let company_name = RwSignal::new(String::new());
    let company_address = RwSignal::new(String::new());
    let dot_number = RwSignal::new(String::new());
    let mc_number = RwSignal::new(String::new());
    let equipment_types = RwSignal::new(String::new());
    let business_entity_id = RwSignal::new(String::new());
    let facility_address = RwSignal::new(String::new());
    let fulfillment_contact_info = RwSignal::new(String::new());
    let fmcsa_broker_license_no = RwSignal::new(String::new());
    let mc_authority_number = RwSignal::new(String::new());
    let freight_forwarder_license = RwSignal::new(String::new());
    let customs_license = RwSignal::new(String::new());
    let kyc_document_name = RwSignal::new(String::new());
    let kyc_document_type = RwSignal::new("standard".to_string());
    let upload_in_progress = RwSignal::new(false);
    let refresh_nonce = RwSignal::new(0_u64);

    let load_screen = move || {
        let auth = auth;
        loading.set(true);
        spawn_local(async move {
            match api::fetch_onboarding_screen().await {
                Ok(response) => {
                    let pending = load_pending_onboarding_draft()
                        .filter(|draft| draft.role_key == response.role_key);
                    company_name.set(prefill_auth_field(
                        response.draft.company_name.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.company_name.clone()),
                    ));
                    company_address.set(prefill_auth_field(
                        response.draft.company_address.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.company_address.clone()),
                    ));
                    dot_number.set(prefill_auth_field(
                        response.draft.dot_number.clone(),
                        pending.as_ref().and_then(|draft| draft.dot_number.clone()),
                    ));
                    mc_number.set(prefill_auth_field(
                        response.draft.mc_number.clone(),
                        pending.as_ref().and_then(|draft| draft.mc_number.clone()),
                    ));
                    equipment_types.set(prefill_auth_field(
                        response.draft.equipment_types.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.equipment_types.clone()),
                    ));
                    business_entity_id.set(prefill_auth_field(
                        response.draft.business_entity_id.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.business_entity_id.clone()),
                    ));
                    facility_address.set(prefill_auth_field(
                        response.draft.facility_address.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.facility_address.clone()),
                    ));
                    fulfillment_contact_info.set(prefill_auth_field(
                        response.draft.fulfillment_contact_info.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.fulfillment_contact_info.clone()),
                    ));
                    fmcsa_broker_license_no.set(prefill_auth_field(
                        response.draft.fmcsa_broker_license_no.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.fmcsa_broker_license_no.clone()),
                    ));
                    mc_authority_number.set(prefill_auth_field(
                        response.draft.mc_authority_number.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.mc_authority_number.clone()),
                    ));
                    freight_forwarder_license.set(prefill_auth_field(
                        response.draft.freight_forwarder_license.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.freight_forwarder_license.clone()),
                    ));
                    customs_license.set(prefill_auth_field(
                        response.draft.customs_license.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.customs_license.clone()),
                    ));
                    if !response.notes.is_empty() {
                        feedback.set(Some(response.notes.join("\n")));
                    }
                    screen.set(Some(response));
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone().to_string()));
                    feedback.set(Some(error.to_string()));
                }
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();
        if auth.session_ready.get()
            && auth.session.get().authenticated
            && screen.get().is_none()
            && !loading.get()
        {
            load_screen();
        }
    });

    view! {
        <AuthArticle
            title=Signal::derive(|| "Onboarding".to_string())
            subtitle=Signal::derive(|| {
                "OTP-complete accounts continue here until the company profile is submitted for review.".to_string()
            })
        >
            <LocalNotice message=feedback />
            {move || if !auth.session_ready.get() || loading.get() {
                view! { <p>"Loading onboarding..."</p> }.into_any()
            } else if !auth.session.get().authenticated {
                view! {
                    <section style="display:grid;gap:0.75rem;">
                        <p>"Sign in first, then continue the onboarding flow from this page."</p>
                        <A href="/auth/login">"Go to login"</A>
                    </section>
                }.into_any()
            } else if let Some(screen_state) = screen.get() {
                let can_submit = screen_state.can_submit;
                let status_label = screen_state.status_label.clone();
                let requires_otp = screen_state.requires_otp;
                let is_carrier = screen_state.role_key == "carrier";
                let is_shipper = screen_state.role_key == "shipper";
                let is_broker = screen_state.role_key == "broker";
                let is_forwarder = screen_state.role_key == "freight_forwarder";
                let required_documents = screen_state.required_documents.clone();
                let submit_navigate = navigate.clone();
                let submit_auth = auth;
                view! {
                    <>
                        <section style="padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#f8fafc;display:grid;gap:0.35rem;">
                            <strong>{format!("{} onboarding", screen_state.role_label)}</strong>
                            <span>{format!("Status: {}", status_label)}</span>
                            <small>{if requires_otp { "OTP is still required before this form can be submitted." } else { "OTP continuity is satisfied for this account." }}</small>
                        </section>
                        <section style="display:grid;gap:0.75rem;padding:0.9rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#f8fbff;">
                            <strong>"KYC Documents"</strong>
                            {(!required_documents.is_empty()).then(|| view! {
                                <div style="display:grid;gap:0.45rem;padding:0.75rem;border:1px solid #bfdbfe;border-radius:0.75rem;background:white;">
                                    <strong>"Required document checklist"</strong>
                                    {required_documents.into_iter().map(|item| view! {
                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                                            <span>{format!("{} - {}", item.label, item.requirement_scope)}</span>
                                            <small style=tone_style(&item.status_tone)>{item.status_label}</small>
                                        </div>
                                    }).collect_view()}
                                </div>
                            })}
                            <div style="display:grid;grid-template-columns:2fr 1fr auto;gap:0.65rem;align-items:end;">
                                <TextField label="Document name" value=kyc_document_name input_type="text" placeholder="Government ID, proof of address, insurance" />
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Document type"</span>
                                    <select prop:value=move || kyc_document_type.get() on:change=move |ev| kyc_document_type.set(event_target_value(&ev)) style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;">
                                        <option value="standard">"Standard"</option>
                                        <option value="Content hash">"Content hash"</option>
                                    </select>
                                </label>
                                <div style="display:grid;gap:0.35rem;">
                                    <span>"Choose file"</span>
                                    <input id=document_upload::kyc_upload_input_id() type="file" style="padding:0.65rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;" />
                                </div>
                            </div>
                            <div style="display:flex;justify-content:flex-end;">
                                <button
                                    type="button"
                                    on:click=move |_| {
                                        let auth = auth;
                                        let document_name = kyc_document_name.get().trim().to_string();
                                        let document_type = kyc_document_type.get();
                                        if document_name.is_empty() {
                                            feedback.set(Some("Enter a KYC document name before uploading.".into()));
                                            return;
                                        }
                                        upload_in_progress.set(true);
                                        spawn_local(async move {
                                            match document_upload::upload_kyc_document(&document_name, &document_type, document_upload::kyc_upload_input_id()).await {
                                                Ok(document) => {
                                                    auth.notice.set(Some(format!("Uploaded {} successfully.", document.document_name)));
                                                    feedback.set(Some("KYC document uploaded. The onboarding screen is refreshing now.".into()));
                                                    kyc_document_name.set(String::new());
                                                    screen.set(None);
                                                    refresh_nonce.update(|value| *value += 1);
                                                }
                                                Err(error) => {
                                                    auth.notice.set(Some(error.clone()));
                                                    feedback.set(Some(error));
                                                }
                                            }
                                            upload_in_progress.set(false);
                                        });
                                    }
                                    style=button_style("#1d4ed8")
                                    disabled=move || upload_in_progress.get() || !can_submit
                                >
                                    {move || if upload_in_progress.get() { "Uploading..." } else if can_submit { "Upload KYC file" } else { "Read only" }}
                                </button>
                            </div>
                            <div style="display:grid;gap:0.5rem;">
                                {if screen_state.documents.is_empty() {
                                    view! { <small>"No KYC documents uploaded yet."</small> }.into_any()
                                } else {
                                    screen_state.documents.into_iter().map(|document| view! {
                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.65rem 0.8rem;border:1px solid #e5e7eb;border-radius:0.85rem;background:white;">
                                            <div style="display:grid;gap:0.15rem;">
                                                <strong>{document.document_name}</strong>
                                                <small>{format!("{} | {} | {}", document.document_type, document.version_history_label, document.uploaded_at_label)}</small>
                                            </div>
                                            {document.download_path.map(|path| view! {
                                                <button type="button" on:click=move |_| {
                                                    let path = path.clone();
                                                    spawn_local(async move {
                                                        let _ = document_upload::open_protected_document(&path).await;
                                                    });
                                                } style="padding:0.45rem 0.7rem;border:1px solid #cbd5e1;border-radius:0.75rem;background:white;cursor:pointer;">"View file"</button>
                                            })}
                                        </div>
                                    }).collect_view().into_any()
                                }}
                            </div>
                        </section>
                        <form on:submit=move |ev: ev::SubmitEvent| {
                            let navigate = submit_navigate.clone();
                            let auth = submit_auth;
                            ev.prevent_default();
                            let payload = SubmitOnboardingRequest {
                                company_name: company_name.get().trim().to_string(),
                                company_address: company_address.get().trim().to_string(),
                                dot_number: optional_string(dot_number.get()),
                                mc_number: optional_string(mc_number.get()),
                                equipment_types: optional_string(equipment_types.get()),
                                business_entity_id: optional_string(business_entity_id.get()),
                                facility_address: optional_string(facility_address.get()),
                                fulfillment_contact_info: optional_string(fulfillment_contact_info.get()),
                                fmcsa_broker_license_no: optional_string(fmcsa_broker_license_no.get()),
                                mc_authority_number: optional_string(mc_authority_number.get()),
                                freight_forwarder_license: optional_string(freight_forwarder_license.get()),
                                customs_license: optional_string(customs_license.get()),
                            };

                            saving.set(true);
                            feedback.set(None);
                            spawn_local(async move {
                                match api::submit_onboarding(&payload).await {
                                    Ok(response) => {
                                        if let Some(session_state) = response.session.clone() {
                                            auth.session.set(session_state);
                                            auth.session_ready.set(true);
                                        }
                                        auth.notice.set(Some(response.message.clone()));
                                        feedback.set(Some(response.message.clone()));
                                        if response.success {
                                            clear_pending_onboarding_draft();
                                            navigate(&response.next_step, Default::default());
                                        } else if response.next_step == "/auth/legal-agreements" {
                                            navigate(&response.next_step, Default::default());
                                        }
                                    }
                                    Err(error) => {
                                        auth.notice.set(Some(error.clone().to_string()));
                                        feedback.set(Some(error.to_string()));
                                    }
                                }
                                saving.set(false);
                            });
                        } style="display:grid;gap:0.85rem;">
                            <TextField label="Company name" value=company_name input_type="text" placeholder="Company name" />
                            <AddressAutocompleteField
                                label="Company address"
                                value=company_address
                                placeholder="Search company address"
                                input_id="onboarding-company-address"
                            />
                            {move || if is_carrier {
                                view! {
                                    <>
                                        <TextField label="DOT number" value=dot_number input_type="text" placeholder="DOT number" />
                                        <TextField label="MC number" value=mc_number input_type="text" placeholder="MC number" />
                                        <TextAreaField label="Equipment types" value=equipment_types placeholder="Dry van, reefer, flatbed" />
                                    </>
                                }.into_any()
                            } else { ().into_any() }}
                            {move || if is_shipper {
                                view! {
                                    <>
                                        <TextField label="Business entity ID" value=business_entity_id input_type="text" placeholder="Entity id" />
                                        <AddressAutocompleteField
                                            label="Facility address"
                                            value=facility_address
                                            placeholder="Search primary facility address"
                                            input_id="onboarding-facility-address"
                                        />
                                        <TextAreaField label="Fulfillment contact info" value=fulfillment_contact_info placeholder="Name, phone, email" />
                                    </>
                                }.into_any()
                            } else { ().into_any() }}
                            {move || if is_broker {
                                view! {
                                    <>
                                        <TextField label="FMCSA broker license" value=fmcsa_broker_license_no input_type="text" placeholder="Broker license number" />
                                        <TextField label="MC authority number" value=mc_authority_number input_type="text" placeholder="MC authority number" />
                                    </>
                                }.into_any()
                            } else { ().into_any() }}
                            {move || if is_forwarder {
                                view! {
                                    <>
                                        <TextField label="Freight forwarder license" value=freight_forwarder_license input_type="text" placeholder="Forwarder license" />
                                        <TextField label="Customs license" value=customs_license input_type="text" placeholder="Customs license" />
                                    </>
                                }.into_any()
                            } else { ().into_any() }}
                            <div style="display:flex;justify-content:space-between;align-items:center;gap:0.75rem;flex-wrap:wrap;">
                                <nav style="display:flex;gap:0.8rem;flex-wrap:wrap;font-size:0.95rem;">
                                    <A href="/auth/login">"Back to login"</A>
                                    <A href="/">"Dashboard"</A>
                                </nav>
                                <button type="submit" style=button_style("#0f766e") disabled=move || saving.get() || !can_submit>
                                    {move || if saving.get() { "Submitting..." } else if can_submit { "Submit onboarding" } else { "Read only" }}
                                </button>
                            </div>
                        </form>
                    </>
                }.into_any()
            } else {
                view! { <p>"Unable to load onboarding right now."</p> }.into_any()
            }}
        </AuthArticle>
    }
}
