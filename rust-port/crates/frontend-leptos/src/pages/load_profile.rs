use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    api, document_upload,
    session::{self, use_auth},
};
use shared::{
    EscrowFundRequest, EscrowHoldRequest, EscrowReleaseRequest, FacilityAppointmentRequest,
    GenerateFreightDocumentsRequest, LoadDocumentRow, LoadLifecycleActionRequest,
    LoadProfileLegRow, LoadProfileScreen, RateCalculationRequest, UpsertLoadDocumentRequest,
    VerifyLoadDocumentRequest,
};

use super::load_profile_documents_helpers::render_load_documents_panel;
use super::load_profile_helpers::{
    admin_profile_actions, render_admin_attention_lanes, render_admin_profile_summary,
    render_handoff, render_history, render_legs, tone_style,
};
use super::shared::{FIELD_LABEL_STYLE, MUTED_TEXT_STYLE, PANEL_SCROLL_STYLE};

#[component]
pub fn LoadProfilePage(#[prop(optional)] admin_mode: bool) -> impl IntoView {
    let auth = use_auth();
    let params = use_params_map();
    let load_id = Memo::new(move |_| {
        params.with(|map| {
            map.get("load_id")
                .and_then(|value| value.parse::<u64>().ok())
        })
    });

    let screen = RwSignal::new(None::<LoadProfileScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);

    let upload_document_name = RwSignal::new(String::new());
    let upload_document_type = RwSignal::new("rate_confirmation".to_string());
    let is_uploading_document = RwSignal::new(false);

    let editing_document_id = RwSignal::new(None::<u64>);
    let document_name = RwSignal::new(String::new());
    let document_type = RwSignal::new("rate_confirmation".to_string());
    let file_path = RwSignal::new(String::new());
    let original_name = RwSignal::new(String::new());
    let mime_type = RwSignal::new(String::new());
    let file_size_input = RwSignal::new(String::new());
    let is_saving_document = RwSignal::new(false);
    let verifying_document_id = RwSignal::new(None::<u64>);
    let generating_documents = RwSignal::new(false);
    let opening_document_id = RwSignal::new(None::<u64>);
    let admin_review_note = RwSignal::new(String::new());
    let admin_review_loading = RwSignal::new(false);
    let admin_finance_loading = RwSignal::new(false);
    let leg_finance_loading_id = RwSignal::new(None::<u64>);
    let rating_loading = RwSignal::new(false);
    let appointment_leg_id = RwSignal::new(String::new());
    let appointment_stop_type = RwSignal::new("pickup".to_string());
    let appointment_start = RwSignal::new(String::new());
    let appointment_end = RwSignal::new(String::new());
    let appointment_time_zone = RwSignal::new("UTC".to_string());
    let appointment_dock = RwSignal::new(String::new());
    let appointment_notes = RwSignal::new(String::new());
    let appointment_loading = RwSignal::new(false);
    let can_manage_payments = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_payments")
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let load_id = load_id.get();
        let _refresh = refresh_nonce.get();

        if !ready {
            return;
        }

        let Some(load_id) = load_id else {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some(
                "The requested Rust load profile URL is missing a valid load id.".into(),
            ));
            return;
        };

        if !current_session.authenticated {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some("Sign in before opening a Rust load profile.".into()));
            return;
        }

        is_loading.set(true);
        let auth = auth;

        spawn_local(async move {
            match api::fetch_load_profile_screen(load_id).await {
                Ok(next_screen) => {
                    error_message.set(None);
                    screen.set(Some(next_screen));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    error_message.set(Some(error));
                }
            }

            is_loading.set(false);
        });
    });

    let clear_upload_form = move || {
        upload_document_name.set(String::new());
        upload_document_type.set("rate_confirmation".to_string());
    };

    let clear_document_form = move || {
        editing_document_id.set(None);
        document_name.set(String::new());
        document_type.set("rate_confirmation".to_string());
        file_path.set(String::new());
        original_name.set(String::new());
        mime_type.set(String::new());
        file_size_input.set(String::new());
    };

    let start_edit_document = move |document: LoadDocumentRow| {
        editing_document_id.set(Some(document.id));
        document_name.set(document.document_name);
        document_type.set(document.document_type_key);
        file_path.set(document.source_path);
        original_name.set(document.original_name.unwrap_or_default());
        mime_type.set(document.mime_type.unwrap_or_default());
        file_size_input.set(
            document
                .file_size_bytes
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        action_message.set(Some(
            "Document row loaded into the Rust profile editor.".into(),
        ));
    };

    let upload_document = move || {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Load profile data is not ready yet, so the document upload could not start."
                    .into(),
            ));
            return;
        };

        let document_name_value = upload_document_name.get();
        let document_type_value = upload_document_type.get();
        if document_name_value.trim().is_empty() {
            action_message.set(Some(
                "Enter a document name before uploading a file.".into(),
            ));
            return;
        }
        if document_type_value.trim().is_empty() {
            action_message.set(Some(
                "Enter a document type before uploading a file.".into(),
            ));
            return;
        }

        let input_id = document_upload::upload_input_id(current_screen.load_id);
        is_uploading_document.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match document_upload::upload_load_document(
                current_screen.load_id,
                &document_name_value,
                &document_type_value,
                &input_id,
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        clear_upload_form();
                        clear_document_form();
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
                    action_message.set(Some(error));
                }
            }

            is_uploading_document.set(false);
        });
    };

    let save_document = move || {
        let Some(document_id) = editing_document_id.get() else {
            action_message.set(Some(
                "Choose an existing document row before saving metadata updates.".into(),
            ));
            return;
        };

        let payload = UpsertLoadDocumentRequest {
            document_name: document_name.get(),
            document_type: document_type.get(),
            file_path: file_path.get(),
            original_name: {
                let value = original_name.get();
                (!value.trim().is_empty()).then_some(value)
            },
            mime_type: {
                let value = mime_type.get();
                (!value.trim().is_empty()).then_some(value)
            },
            file_size: {
                let raw = file_size_input.get();
                if raw.trim().is_empty() {
                    None
                } else {
                    match raw.trim().parse::<i64>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            action_message
                                .set(Some("File size must be a whole number of bytes.".into()));
                            return;
                        }
                    }
                }
            },
        };

        is_saving_document.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::update_load_document(document_id, &payload).await {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        clear_document_form();
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
                    action_message.set(Some(error));
                }
            }

            is_saving_document.set(false);
        });
    };

    let verify_document = move |document_id: u64| {
        verifying_document_id.set(Some(document_id));
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::verify_load_document(
                document_id,
                &VerifyLoadDocumentRequest {
                    note: Some("Triggered from the Rust load profile.".into()),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
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
                    action_message.set(Some(error));
                }
            }

            verifying_document_id.set(None);
        });
    };

    let generate_standard_documents = move |load_id: u64| {
        generating_documents.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::generate_standard_freight_documents(
                load_id,
                &GenerateFreightDocumentsRequest {
                    template_keys: Vec::new(),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
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
                    action_message.set(Some(error));
                }
            }

            generating_documents.set(false);
        });
    };

    let open_document = move |document_id: u64, download_path: String| {
        opening_document_id.set(Some(document_id));
        action_message.set(None);

        spawn_local(async move {
            match document_upload::open_protected_document(&download_path).await {
                Ok(()) => {
                    action_message.set(Some(
                        "Opening the protected document in a new browser tab.".into(),
                    ));
                }
                Err(error) => {
                    action_message.set(Some(error));
                }
            }

            opening_document_id.set(None);
        });
    };

    let download_document = move |document_id: u64, download_path: String, file_name: String| {
        opening_document_id.set(Some(document_id));
        action_message.set(None);

        spawn_local(async move {
            match document_upload::download_protected_document(&download_path, &file_name).await {
                Ok(()) => {
                    action_message.set(Some(
                        "Downloading the protected document from the Rust profile.".into(),
                    ));
                }
                Err(error) => {
                    action_message.set(Some(error));
                }
            }

            opening_document_id.set(None);
        });
    };

    let run_leg_finance_action = move |leg: LoadProfileLegRow| {
        let Some(action_key) = leg.finance_action_key.clone() else {
            action_message.set(Some(
                "No direct finance action is available for this leg yet.".into(),
            ));
            return;
        };

        leg_finance_loading_id.set(Some(leg.leg_id));
        action_message.set(None);

        spawn_local(async move {
            let note = Some(format!(
                "Triggered from the Rust admin load profile for leg {}.",
                leg.leg_code
            ));
            let result = match action_key.as_str() {
                "fund" => {
                    api::fund_escrow(
                        leg.leg_id,
                        &EscrowFundRequest {
                            idempotency_key: None,
                            amount_cents: None,
                            currency: Some("USD".into()),
                            platform_fee_cents: None,
                            payment_intent_id: None,
                            charge_id: None,
                            transfer_group: None,
                            note,
                        },
                    )
                    .await
                }
                "hold" => {
                    api::hold_escrow(
                        leg.leg_id,
                        &EscrowHoldRequest {
                            idempotency_key: None,
                            note,
                        },
                    )
                    .await
                }
                "release" => {
                    api::release_escrow(
                        leg.leg_id,
                        &EscrowReleaseRequest {
                            idempotency_key: None,
                            transfer_id: None,
                            note,
                        },
                    )
                    .await
                }
                other => Err(format!("Unsupported finance action '{}'.", other)),
            };

            match result {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => action_message.set(Some(error)),
            }

            leg_finance_loading_id.set(None);
        });
    };

    let run_lifecycle_action = move |load_id: u64, action: String| {
        action_message.set(None);
        let template_name = (action == "template").then(|| {
            screen
                .get_untracked()
                .and_then(|value| value.load_number)
                .map(|value| format!("{} template", value))
                .unwrap_or_else(|| "Load template".into())
        });

        spawn_local(async move {
            match api::run_load_lifecycle_action(
                load_id,
                &LoadLifecycleActionRequest {
                    action,
                    reason: None,
                    template_name,
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => action_message.set(Some(error)),
            }
        });
    };

    let calculate_rating = move |load_id: u64| {
        rating_loading.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::calculate_load_rate(
                load_id,
                &RateCalculationRequest {
                    mileage_miles_override: None,
                    mileage_source_override: None,
                    manual_override_total: None,
                    manual_override_reason: None,
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
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
                    action_message.set(Some(error));
                }
            }

            rating_loading.set(false);
        });
    };

    let schedule_appointment = move |load_id: u64| {
        let leg_id = match appointment_leg_id.get().trim().parse::<u64>() {
            Ok(value) if value > 0 => value,
            _ => {
                action_message.set(Some("Choose a valid leg id before scheduling.".into()));
                return;
            }
        };
        if appointment_start.get().trim().is_empty() {
            action_message.set(Some("Appointment start is required.".into()));
            return;
        }

        appointment_loading.set(true);
        action_message.set(None);
        let auth = auth;
        let payload = FacilityAppointmentRequest {
            leg_id,
            stop_type: appointment_stop_type.get(),
            appointment_ref: None,
            appointment_start: appointment_start.get(),
            appointment_end: (!appointment_end.get().trim().is_empty())
                .then_some(appointment_end.get()),
            appointment_time_zone: (!appointment_time_zone.get().trim().is_empty())
                .then_some(appointment_time_zone.get()),
            dock_name: (!appointment_dock.get().trim().is_empty())
                .then_some(appointment_dock.get()),
            contact_name: None,
            contact_phone: None,
            contact_email: None,
            notes: (!appointment_notes.get().trim().is_empty()).then_some(appointment_notes.get()),
            reason: Some("Scheduled from Rust load profile.".into()),
        };

        spawn_local(async move {
            match api::schedule_facility_appointment(load_id, &payload).await {
                Ok(response) => {
                    action_message.set(Some(response.message));
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
                    action_message.set(Some(error));
                }
            }

            appointment_loading.set(false);
        });
    };

    let back_href = if admin_mode { "/admin/loads" } else { "/loads" };
    let profile_title = if admin_mode {
        "Admin Load Profile"
    } else {
        "Load Profile"
    };

    view! {
        <article style="display:grid;gap:1.25rem;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style=FIELD_LABEL_STYLE>
                    <h2>{move || screen.get().and_then(|value| value.load_number).unwrap_or_else(|| profile_title.into())}</h2>
                    <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "Rust load detail view".into())}</p>
                </div>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <A href=back_href attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#f4f4f5;color:#111827;text-decoration:none;">{if admin_mode { "Back to admin loads" } else { "Back to loads" }}</A>
                    <A href=move || screen.get().map(|value| format!("/loads/{}/edit", value.load_id)).unwrap_or_else(|| "/loads/new".into()) attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#fff7ed;color:#9a3412;text-decoration:none;">"Edit load"</A>
                    <button
                        type="button"
                        disabled=move || rating_loading.get() || screen.get().is_none()
                        on:click=move |_| {
                            if let Some(current) = screen.get() {
                                calculate_rating(current.load_id);
                            }
                        }
                        style="padding:0.7rem 1rem;border:none;border-radius:0.9rem;background:#0f766e;color:white;cursor:pointer;"
                    >
                        {move || if rating_loading.get() { "Calculating..." } else { "Calculate rate" }}
                    </button>
                    <A href="/loads/new" attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#111827;color:white;text-decoration:none;">"Create another load"</A>
                </div>
            </section>

            {move || error_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">{message}</section>
            })}

            {move || {
                if is_loading.get() && screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                            "Loading Rust load profile data..."
                        </section>
                    }.into_any()
                } else if let Some(screen_value) = screen.get() {
                    let documents = screen_value.documents.clone();
                    let required_documents = screen_value.required_documents.clone();
                    let can_manage_documents = screen_value.can_manage_documents;
                    let lifecycle_load_id = screen_value.load_id;
                    let upload_input_id = document_upload::upload_input_id(screen_value.load_id);
                    let has_pending_leg = screen_value.legs.iter().any(|leg| leg.status_code == 1);
                    let release_ready_leg_id = screen_value
                        .legs
                        .iter()
                        .find(|leg| leg.status_code == 10)
                        .map(|leg| leg.leg_id);
                    let has_release_ready_leg = release_ready_leg_id.is_some();
                    view! {
                        <>
                            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;">
                                <div style="display:grid;gap:0.25rem;">
                                    <strong>"Lifecycle"</strong>
                                    <span style=tone_style(match screen_value.lifecycle_status.as_str() {
                                        "published" => "success",
                                        "revised" => "primary",
                                        "cancelled" => "warning",
                                        "archived" => "dark",
                                        _ => "secondary",
                                    })>{format!("{} v{}", screen_value.lifecycle_status, screen_value.revision_number)}</span>
                                </div>
                                <div style="display:flex;gap:0.6rem;flex-wrap:wrap;">
                                    {screen_value.lifecycle_actions.clone().into_iter().map(|item| {
                                        let action_key = item.action.clone();
                                        let disabled_title = item.disabled_reason.clone().unwrap_or_default();
                                        view! {
                                            <button
                                                type="button"
                                                title=disabled_title
                                                disabled=!item.enabled
                                                on:click=move |_| run_lifecycle_action(lifecycle_load_id, action_key.clone())
                                                style=format!(
                                                    "padding:0.55rem 0.85rem;border-radius:0.8rem;border:1px solid #d1d5db;cursor:{};{}",
                                                    if item.enabled { "pointer" } else { "not-allowed" },
                                                    if item.enabled { tone_style(&item.tone) } else { "background:#f8fafc;color:#94a3b8;" }
                                                )
                                            >
                                                {item.label}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;">
                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                    <strong>"Facility appointment"</strong>
                                    <button
                                        type="button"
                                        disabled=move || appointment_loading.get()
                                        on:click=move |_| schedule_appointment(screen_value.load_id)
                                        style="padding:0.55rem 0.85rem;border:none;border-radius:0.8rem;background:#111827;color:white;cursor:pointer;"
                                    >
                                        {move || if appointment_loading.get() { "Scheduling..." } else { "Schedule" }}
                                    </button>
                                </div>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.75rem;align-items:end;">
                                    <label style="display:grid;gap:0.3rem;">
                                        <span>"Leg id"</span>
                                        <select prop:value=move || appointment_leg_id.get() on:change=move |ev| appointment_leg_id.set(event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                                            <option value="">"Choose leg"</option>
                                            {screen_value.legs.clone().into_iter().map(|leg| view! {
                                                <option value={leg.leg_id.to_string()}>{leg.leg_code}</option>
                                            }).collect_view()}
                                        </select>
                                    </label>
                                    <label style="display:grid;gap:0.3rem;">
                                        <span>"Stop"</span>
                                        <select prop:value=move || appointment_stop_type.get() on:change=move |ev| appointment_stop_type.set(event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                                            <option value="pickup">"Pickup"</option>
                                            <option value="delivery">"Delivery"</option>
                                        </select>
                                    </label>
                                    <label style="display:grid;gap:0.3rem;">
                                        <span>"Start"</span>
                                        <input type="datetime-local" prop:value=move || appointment_start.get() on:input=move |ev| appointment_start.set(event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                                    </label>
                                    <label style="display:grid;gap:0.3rem;">
                                        <span>"End"</span>
                                        <input type="datetime-local" prop:value=move || appointment_end.get() on:input=move |ev| appointment_end.set(event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                                    </label>
                                    <label style="display:grid;gap:0.3rem;">
                                        <span>"Time zone"</span>
                                        <input prop:value=move || appointment_time_zone.get() on:input=move |ev| appointment_time_zone.set(event_target_value(&ev)) placeholder="America/Chicago" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                                    </label>
                                    <label style="display:grid;gap:0.3rem;">
                                        <span>"Dock"</span>
                                        <input prop:value=move || appointment_dock.get() on:input=move |ev| appointment_dock.set(event_target_value(&ev)) placeholder="Dock 4" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                                    </label>
                                    <label style="display:grid;gap:0.3rem;">
                                        <span>"Notes"</span>
                                        <input prop:value=move || appointment_notes.get() on:input=move |ev| appointment_notes.set(event_target_value(&ev)) placeholder="Check-in, parking, lumper, warnings" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                                    </label>
                                </div>
                            </section>

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                {screen_value.info_fields.into_iter().map(|field| view! {
                                    <div class="wrap-safe" style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fcfcfb;display:grid;gap:0.35rem;">
                                        <small class="wrap-safe" style=MUTED_TEXT_STYLE>{field.label}</small>
                                        <strong class="wrap-safe">{field.value}</strong>
                                    </div>
                                }).collect_view()}
                            </section>

                            <section style="display:grid;gap:1rem;grid-template-columns:minmax(0,1fr);">
                                {admin_mode.then(|| admin_profile_actions(
                                    screen_value.load_id,
                                    has_pending_leg,
                                    has_release_ready_leg,
                                    release_ready_leg_id,
                                    admin_review_note,
                                    admin_review_loading,
                                    admin_finance_loading,
                                    action_message,
                                    refresh_nonce,
                                ))}
                                {admin_mode.then(|| render_admin_profile_summary(
                                    screen_value.load_id,
                                    screen_value.legs.clone(),
                                    screen_value.documents.clone(),
                                    screen_value.history.clone(),
                                    screen_value.stloads_handoff.clone(),
                                ))}
                                {admin_mode.then(|| render_admin_attention_lanes(
                                    screen_value.load_id,
                                    screen_value.legs.clone(),
                                    screen_value.documents.clone(),
                                    screen_value.stloads_handoff.clone(),
                                ))}
                                {render_handoff(screen_value.stloads_handoff.clone(), admin_mode)}
                            </section>

                            <section style="display:grid;gap:1rem;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));align-items:start;">
                                <section style=PANEL_SCROLL_STYLE>
                                    <strong>"Load legs"</strong>
                                    {render_legs(
                                        screen_value.legs.clone(),
                                        admin_mode,
                                        can_manage_payments.get(),
                                        leg_finance_loading_id,
                                        run_leg_finance_action,
                                    )}
                                </section>

                                {render_load_documents_panel(screen_value.load_id, can_manage_documents, required_documents, documents, generating_documents, upload_document_name, upload_document_type, upload_input_id.clone(), is_uploading_document, document_name, document_type, file_path, original_name, mime_type, file_size_input, is_saving_document, opening_document_id, verifying_document_id, generate_standard_documents, clear_upload_form, clear_document_form, upload_document, save_document, open_document, download_document, start_edit_document, verify_document)}
                            </section>

                            <section style=PANEL_SCROLL_STYLE>
                                <strong>"History"</strong>
                                {render_history(screen_value.history.clone())}
                            </section>

                        </>
                    }.into_any()
                } else {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                            "No Rust load profile data is available yet for this route."
                        </section>
                    }.into_any()
                }
            }}
        </article>
    }
}
