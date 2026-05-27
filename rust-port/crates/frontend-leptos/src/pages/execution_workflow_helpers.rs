use leptos::{prelude::*, tachys::view::any_view::IntoAny};

use crate::document_upload;

use super::{
    execution_helpers::{
        render_action_item, render_document_type_option, render_documents, render_status_items,
        render_tracking_points, tone_style, tracking_point_map_url, tracking_route_maps_url,
    },
    shared::{MUTED_TEXT_STYLE, PANEL_SCROLL_STYLE},
};
use shared::{
    ExecutionActionItem, ExecutionDocumentItem, ExecutionLegScreen, ExecutionTrackingPointItem,
    RequiredDocumentChecklistItem,
};

pub(super) fn render_closeout_documents_customer_and_finance(
    screen_value: ExecutionLegScreen,
    required_documents: Vec<RequiredDocumentChecklistItem>,
    documents: Vec<ExecutionDocumentItem>,
    document_count: usize,
    upload_document_name: RwSignal<String>,
    upload_document_type: RwSignal<String>,
    is_uploading_document: RwSignal<bool>,
    closeout_note: RwSignal<String>,
    is_saving_workflow: RwSignal<bool>,
    customer_tracking_expires_hours: RwSignal<String>,
    customer_tracking_revoke_reason: RwSignal<String>,
    telematics_provider: RwSignal<String>,
    route_distance: RwSignal<String>,
    route_duration: RwSignal<String>,
    finance_exception_description: RwSignal<String>,
    finance_exception_amount: RwSignal<String>,
    finance_exception_decision_type: RwSignal<String>,
    finance_exception_decision_status: RwSignal<String>,
    finance_exception_resolution_note: RwSignal<String>,
    upload_execution_document: impl Fn() + Copy + 'static,
    open_document: impl Fn(String) + Copy + 'static,
    save_closeout_approval: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    create_customer_tracking_link: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    revoke_customer_tracking_link: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    save_telematics: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    save_route_plan: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    save_finance_exception: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    decide_finance_exception: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
) -> impl IntoView {
    let document_type_options = screen_value.document_type_options.clone();
    let can_upload_documents = screen_value.can_upload_documents;
    let field_capture_strategy_label = screen_value.field_capture_strategy_label.clone();
    let offline_strategy_label = screen_value.offline_strategy_label.clone();
    let mobile_support_label = screen_value.mobile_support_label.clone();
    let upload_input_id = document_upload::execution_upload_input_id(screen_value.leg_id);
    let closeout_package_tone = screen_value.closeout_package_tone.clone();
    let closeout_ready = screen_value.closeout_ready;
    let closeout_package_label = screen_value.closeout_package_label.clone();
    let closeout_export_path = screen_value.closeout_export_path.clone();
    let closeout_checklist = screen_value.closeout_checklist.clone();
    let customer_tracking_path = screen_value.customer_tracking_path.clone();
    let offline_submission_status_label = screen_value.offline_submission_status_label.clone();
    let offline_submission_count = screen_value.offline_submission_count;
    let pending_offline_submission_count = screen_value.pending_offline_submission_count;
    let telematics_status_label = screen_value.telematics_status_label.clone();
    let route_plan_tone = screen_value.route_plan_tone.clone();
    let route_plan_label = screen_value.route_plan_label.clone();
    let claims_accessorial_items = screen_value.claims_accessorial_items.clone();

    view! {
        <>
            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
                <section style=PANEL_SCROLL_STYLE>
                    <div style="display:grid;gap:0.3rem;">
                        <strong>"Execution documents"</strong>
                        <small style=MUTED_TEXT_STYLE>{format!("{} execution document(s) currently attached.", document_count)}</small>
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
                    <div style="display:grid;gap:0.65rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                        <strong>"Mobile field capture"</strong>
                        <small style=MUTED_TEXT_STYLE>{field_capture_strategy_label}</small>
                        <small style=MUTED_TEXT_STYLE>{offline_strategy_label}</small>
                        <small style=MUTED_TEXT_STYLE>{mobile_support_label}</small>
                        <input
                            type="text"
                            placeholder="Optional custom document name"
                            prop:value=move || upload_document_name.get()
                            on:input=move |ev| upload_document_name.set(event_target_value(&ev))
                            disabled=move || !can_upload_documents || is_uploading_document.get()
                        />
                        <select
                            prop:value=move || upload_document_type.get()
                            on:change=move |ev| upload_document_type.set(event_target_value(&ev))
                            disabled=move || !can_upload_documents || is_uploading_document.get()
                        >
                            {document_type_options.into_iter().map(render_document_type_option).collect_view()}
                        </select>
                        <input
                            id=upload_input_id
                            type="file"
                            accept="image/*,application/pdf"
                            capture="environment"
                            disabled=move || !can_upload_documents || is_uploading_document.get()
                        />
                        <button
                            type="button"
                            style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#111827;color:white;cursor:pointer;justify-self:start;"
                            disabled=move || !can_upload_documents || is_uploading_document.get()
                            on:click=move |_| upload_execution_document()
                        >
                            {move || if is_uploading_document.get() { "Uploading..." } else if can_upload_documents { "Upload execution document" } else { "Upload locked" }}
                        </button>
                    </div>
                    {render_documents(documents, open_document)}
                </section>
            </section>

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
                <section style=PANEL_SCROLL_STYLE>
                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                        <strong>"Closeout package"</strong>
                        <span style=tone_style(&closeout_package_tone)>{if closeout_ready { "Release ready" } else { "Blocked" }}</span>
                    </div>
                    <small style=MUTED_TEXT_STYLE>{closeout_package_label}</small>
                    <textarea
                        rows="2"
                        placeholder="Closeout review note"
                        prop:value=move || closeout_note.get()
                        on:input=move |ev| closeout_note.set(event_target_value(&ev))
                        disabled=move || is_saving_workflow.get()
                    />
                    <button
                        type="button"
                        style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#0f766e;color:white;cursor:pointer;justify-self:start;"
                        disabled=move || is_saving_workflow.get()
                        on:click=save_closeout_approval
                    >
                        {move || if is_saving_workflow.get() { "Saving..." } else { "Approve POD closeout" }}
                    </button>
                    {closeout_export_path.map(|path| view! {
                        <a href=path target="_blank" rel="noopener noreferrer" style="justify-self:start;padding:0.45rem 0.75rem;border-radius:0.75rem;background:#f1f5f9;color:#0f172a;text-decoration:none;">
                            "Open closeout export"
                        </a>
                    })}
                    {render_status_items(closeout_checklist)}
                </section>
                <section style=PANEL_SCROLL_STYLE>
                    <strong>"Customer and integrations"</strong>
                    {customer_tracking_path.map(|path| view! {
                        <a href=path target="_blank" rel="noopener noreferrer" style="justify-self:start;padding:0.45rem 0.75rem;border-radius:0.75rem;background:#e0f2fe;color:#075985;text-decoration:none;">
                            "Open customer tracking"
                        </a>
                    }.into_any()).unwrap_or_else(|| view! {
                        <small style=MUTED_TEXT_STYLE>"Customer tracking link is not active for this leg."</small>
                    }.into_any())}
                    <div style="display:grid;gap:0.5rem;">
                        <label style="display:grid;gap:0.25rem;">
                            <small style="color:#475569;">"Tracking link expiration hours"</small>
                            <input
                                type="number"
                                min="1"
                                max="720"
                                prop:value=move || customer_tracking_expires_hours.get()
                                on:input=move |ev| customer_tracking_expires_hours.set(event_target_value(&ev))
                                disabled=move || is_saving_workflow.get()
                            />
                        </label>
                        <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                            <button
                                type="button"
                                style="padding:0.45rem 0.75rem;border-radius:0.75rem;border:none;background:#0369a1;color:white;cursor:pointer;"
                                disabled=move || is_saving_workflow.get()
                                on:click=create_customer_tracking_link
                            >
                                "Create/rotate tracking link"
                            </button>
                        </div>
                        <textarea
                            rows="2"
                            placeholder="Revocation reason"
                            prop:value=move || customer_tracking_revoke_reason.get()
                            on:input=move |ev| customer_tracking_revoke_reason.set(event_target_value(&ev))
                            disabled=move || is_saving_workflow.get()
                        />
                        <button
                            type="button"
                            style="padding:0.45rem 0.75rem;border-radius:0.75rem;border:1px solid #fecdd3;background:#fff1f2;color:#be123c;cursor:pointer;justify-self:start;"
                            disabled=move || is_saving_workflow.get()
                            on:click=revoke_customer_tracking_link
                        >
                            "Revoke customer tracking"
                        </button>
                    </div>
                    <div style="display:grid;gap:0.25rem;">
                        <strong>"Offline replay"</strong>
                        <small style=MUTED_TEXT_STYLE>{offline_submission_status_label}</small>
                        <small style=MUTED_TEXT_STYLE>{format!("{} total, {} pending", offline_submission_count, pending_offline_submission_count)}</small>
                    </div>
                    <div style="display:grid;gap:0.25rem;">
                        <strong>"Telematics"</strong>
                        <small style=MUTED_TEXT_STYLE>{telematics_status_label}</small>
                        <input
                            type="text"
                            placeholder="Provider key"
                            prop:value=move || telematics_provider.get()
                            on:input=move |ev| telematics_provider.set(event_target_value(&ev))
                            disabled=move || is_saving_workflow.get()
                        />
                        <button
                            type="button"
                            style="padding:0.45rem 0.75rem;border-radius:0.75rem;border:none;background:#475569;color:white;cursor:pointer;justify-self:start;"
                            disabled=move || is_saving_workflow.get()
                            on:click=save_telematics
                        >
                            "Save telematics decision"
                        </button>
                    </div>
                    <div style="display:grid;gap:0.25rem;">
                        <strong>"Route source"</strong>
                        <span style=tone_style(&route_plan_tone)>{route_plan_label}</span>
                        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(130px,1fr));gap:0.5rem;">
                            <input
                                type="number"
                                step="0.1"
                                placeholder="Miles"
                                prop:value=move || route_distance.get()
                                on:input=move |ev| route_distance.set(event_target_value(&ev))
                                disabled=move || is_saving_workflow.get()
                            />
                            <input
                                type="number"
                                step="1"
                                placeholder="Minutes"
                                prop:value=move || route_duration.get()
                                on:input=move |ev| route_duration.set(event_target_value(&ev))
                                disabled=move || is_saving_workflow.get()
                            />
                        </div>
                        <button
                            type="button"
                            style="padding:0.45rem 0.75rem;border-radius:0.75rem;border:none;background:#111827;color:white;cursor:pointer;justify-self:start;"
                            disabled=move || is_saving_workflow.get()
                            on:click=save_route_plan
                        >
                            "Save route plan"
                        </button>
                    </div>
                </section>
            </section>

            <section style=PANEL_SCROLL_STYLE>
                <strong>"Claims, detention, and accessorials"</strong>
                <div style="display:grid;gap:0.6rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                    <textarea
                        rows="2"
                        placeholder="Accessorial, detention, claim, or dispute details"
                        prop:value=move || finance_exception_description.get()
                        on:input=move |ev| finance_exception_description.set(event_target_value(&ev))
                        disabled=move || is_saving_workflow.get()
                    />
                    <input
                        type="number"
                        step="0.01"
                        placeholder="Amount"
                        prop:value=move || finance_exception_amount.get()
                        on:input=move |ev| finance_exception_amount.set(event_target_value(&ev))
                        disabled=move || is_saving_workflow.get()
                    />
                    <button
                        type="button"
                        style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#92400e;color:white;cursor:pointer;justify-self:start;"
                        disabled=move || is_saving_workflow.get()
                        on:click=save_finance_exception
                    >
                        "Record finance exception"
                    </button>
                </div>
                <div style="display:grid;gap:0.6rem;padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#f8fbff;">
                    <strong>"Resolve billing exception"</strong>
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:0.5rem;">
                        <select
                            prop:value=move || finance_exception_decision_type.get()
                            on:change=move |ev| finance_exception_decision_type.set(event_target_value(&ev))
                            disabled=move || is_saving_workflow.get()
                        >
                            <option value="accessorial">"Accessorial"</option>
                            <option value="detention">"Detention"</option>
                            <option value="damage">"Damage"</option>
                            <option value="shortage">"Shortage"</option>
                            <option value="late_delivery">"Late delivery"</option>
                            <option value="charge_dispute">"Charge dispute"</option>
                            <option value="service_failure">"Service failure"</option>
                        </select>
                        <select
                            prop:value=move || finance_exception_decision_status.get()
                            on:change=move |ev| finance_exception_decision_status.set(event_target_value(&ev))
                            disabled=move || is_saving_workflow.get()
                        >
                            <option value="approved">"Approve"</option>
                            <option value="rejected">"Reject"</option>
                            <option value="disputed">"Dispute"</option>
                            <option value="review">"Send to review"</option>
                            <option value="resolved">"Resolve"</option>
                        </select>
                    </div>
                    <textarea
                        rows="2"
                        placeholder="Resolution note for invoice, settlement, and support timeline"
                        prop:value=move || finance_exception_resolution_note.get()
                        on:input=move |ev| finance_exception_resolution_note.set(event_target_value(&ev))
                        disabled=move || is_saving_workflow.get()
                    />
                    <button
                        type="button"
                        style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#1d4ed8;color:white;cursor:pointer;justify-self:start;"
                        disabled=move || is_saving_workflow.get()
                        on:click=decide_finance_exception
                    >
                        "Apply finance decision"
                    </button>
                </div>
                {render_status_items(claims_accessorial_items)}
            </section>
        </>
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_execution_tracking_workspace(
    screen_value: ExecutionLegScreen,
    action_items: Vec<ExecutionActionItem>,
    tracking_points: Vec<ExecutionTrackingPointItem>,
    tracking_embed: Option<String>,
    latest_tracking_point: Option<ExecutionTrackingPointItem>,
    earliest_tracking_point: Option<ExecutionTrackingPointItem>,
    tracking_distance_label: String,
    tracking_window_label: String,
    tracking_health_label: String,
    tracking_health_tone: String,
    is_recording_consent: RwSignal<bool>,
    live_tracking_enabled: RwSignal<bool>,
    is_toggling_live_tracking: RwSignal<bool>,
    is_sending_location: RwSignal<bool>,
    pending_action_key: RwSignal<Option<String>>,
    action_note: RwSignal<String>,
    record_tracking_consent: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    start_live_tracking: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    stop_live_tracking: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    send_current_location: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    queue_offline_note: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    run_action: impl Fn(String) + Copy + 'static,
) -> impl IntoView {
    let tracking_consent_required = screen_value.tracking_consent_required;
    let tracking_consent_granted = screen_value.tracking_consent_granted;
    let tracking_consent_text = screen_value.tracking_consent_text.clone();
    let tracking_retention_label = screen_value.tracking_retention_label.clone();
    let customer_tracking_scope_label = screen_value.customer_tracking_scope_label.clone();
    let operator_mode_label = screen_value.operator_mode_label.clone();
    let live_tracking_note = screen_value.live_tracking_note.clone();
    let live_tracking_available = screen_value.live_tracking_available;
    let can_send_location_ping = screen_value.can_send_location_ping;
    let latest_location_label = screen_value.latest_location_label.clone();
    let latest_coordinate_label = screen_value.latest_coordinate_label.clone();
    let latest_map_url = screen_value.latest_map_url.clone();
    let tracking_health_tone_label = screen_value.tracking_health_tone.replace('_', " ");
    let delivery_completion_note = screen_value.delivery_completion_note.clone();
    let delivery_completion_ready = screen_value.delivery_completion_ready;
    let geofence_status_tone = screen_value.geofence_status_tone.clone();
    let geofence_status_label = screen_value.geofence_status_label.clone();
    let eta_risk_tone = screen_value.eta_risk_tone.clone();
    let eta_risk_label = screen_value.eta_risk_label.clone();

    view! {
        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
            <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;">
                {tracking_consent_required.then(|| view! {
                    <div style="padding:0.85rem 1rem;border:1px solid #fde68a;border-radius:0.9rem;background:#fffbeb;display:grid;gap:0.45rem;">
                        <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                            <strong>"Tracking consent"</strong>
                            <span style=tone_style(if tracking_consent_granted { "success" } else { "warning" })>
                                {if tracking_consent_granted { "Recorded" } else { "Required" }}
                            </span>
                        </div>
                        <small style="color:#92400e;">{tracking_consent_text}</small>
                        <small style=MUTED_TEXT_STYLE>{tracking_retention_label}</small>
                        <small style=MUTED_TEXT_STYLE>{format!("Customer scope: {}", customer_tracking_scope_label)}</small>
                        <button
                            type="button"
                            style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#92400e;color:white;cursor:pointer;justify-self:start;"
                            disabled=move || tracking_consent_granted || is_recording_consent.get()
                            on:click=record_tracking_consent
                        >
                            {move || if is_recording_consent.get() { "Recording..." } else if tracking_consent_granted { "Consent recorded" } else { "Accept tracking consent" }}
                        </button>
                    </div>
                })}
                <div style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;display:grid;gap:0.35rem;">
                    <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                        <div style="display:grid;gap:0.2rem;">
                            <strong>{operator_mode_label}</strong>
                            {live_tracking_note.map(|note| view! {
                                <small style="color:#1d4ed8;">{note}</small>
                            })}
                        </div>
                        <span style=move || tone_style(if live_tracking_enabled.get() { "success" } else if live_tracking_available { "warning" } else { "info" })>
                            {move || if live_tracking_enabled.get() {
                                "Tracking: ON".to_string()
                            } else if live_tracking_available {
                                "Tracking: Ready".to_string()
                            } else {
                                "Tracking: View only".to_string()
                            }}
                        </span>
                    </div>
                    {live_tracking_available.then(|| view! {
                        <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                            <button
                                type="button"
                                style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#166534;color:white;cursor:pointer;"
                                disabled=move || is_toggling_live_tracking.get() || live_tracking_enabled.get()
                                on:click=start_live_tracking
                            >
                                {move || if is_toggling_live_tracking.get() && !live_tracking_enabled.get() { "Starting..." } else { "Start live tracking" }}
                            </button>
                            <button
                                type="button"
                                style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#475569;color:white;cursor:pointer;"
                                disabled=move || is_toggling_live_tracking.get() || !live_tracking_enabled.get()
                                on:click=stop_live_tracking
                            >
                                {move || if is_toggling_live_tracking.get() && live_tracking_enabled.get() { "Stopping..." } else { "Stop live tracking" }}
                            </button>
                        </div>
                    })}
                </div>

                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                    <strong>"Execution actions"</strong>
                    <button
                        type="button"
                        style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#0f766e;color:white;cursor:pointer;"
                        disabled=move || is_sending_location.get() || !can_send_location_ping
                        on:click=send_current_location
                    >
                        {move || if is_sending_location.get() { "Sending GPS..." } else if can_send_location_ping { "Send current GPS" } else { "GPS locked" }}
                    </button>
                </div>
                <div style="display:grid;gap:0.35rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                    <strong>"Latest location"</strong>
                    <span>{latest_location_label.unwrap_or_else(|| "No location ping yet".into())}</span>
                    <small>{latest_coordinate_label.unwrap_or_else(|| "Waiting for the first GPS update".into())}</small>
                    <small style=MUTED_TEXT_STYLE>{tracking_window_label.clone()}</small>
                    <span style=tone_style(&tracking_health_tone)>{tracking_health_tone_label}</span>
                    <small style=MUTED_TEXT_STYLE>{tracking_health_label}</small>
                    {latest_map_url.map(|map_url| view! {
                        <a href=map_url target="_blank" rel="noopener noreferrer" style="justify-self:start;padding:0.45rem 0.75rem;border-radius:0.75rem;background:#e0f2fe;color:#075985;text-decoration:none;">
                            "Open latest point"
                        </a>
                    })}
                </div>
                <div style="display:grid;gap:0.4rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                    <strong>"Execution note"</strong>
                    <textarea
                        rows="3"
                        placeholder="Add carrier or operator context for the next execution step"
                        prop:value=move || action_note.get()
                        on:input=move |ev| action_note.set(event_target_value(&ev))
                        disabled=move || pending_action_key.get().is_some()
                    />
                    {delivery_completion_note.map(|note| view! {
                        <small style=move || if delivery_completion_ready { "color:#166534;" } else { "color:#b45309;" }>{note}</small>
                    })}
                    <button
                        type="button"
                        style="padding:0.45rem 0.75rem;border-radius:0.75rem;border:1px solid #cbd5e1;background:#f8fafc;color:#334155;cursor:pointer;justify-self:start;"
                        on:click=queue_offline_note
                    >
                        "Queue note offline"
                    </button>
                </div>
                <div style="display:grid;gap:0.75rem;">
                    {action_items.into_iter().map(|item| render_action_item(item, pending_action_key, run_action)).collect_view()}
                </div>
            </section>

            <section style=PANEL_SCROLL_STYLE>
                <strong>"Live location map"</strong>
                {tracking_embed.map(|embed_url| view! {
                    <iframe
                        src=embed_url
                        style="width:100%;min-height:320px;border:1px solid #e5e7eb;border-radius:0.95rem;background:#f8fafc;"
                    ></iframe>
                }.into_any()).unwrap_or_else(|| view! {
                    <section style="padding:1rem;border:1px dashed #cbd5e1;border-radius:0.95rem;background:#f8fafc;">
                        <strong>"No live map yet"</strong>
                    </section>
                }.into_any())}
                {latest_tracking_point.clone().map(|point| view! {
                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.75rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                        <div style="display:grid;gap:0.2rem;">
                            <strong>"Latest plotted point"</strong>
                            <small>{point.recorded_at_label.clone()}</small>
                            <small style=MUTED_TEXT_STYLE>{format!("{:.5}, {:.5}", point.lat, point.lng)}</small>
                        </div>
                        <a href=tracking_point_map_url(&point) target="_blank" rel="noopener noreferrer" style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#e0f2fe;color:#075985;text-decoration:none;">
                            "Open exact point"
                        </a>
                    </div>
                })}
                {earliest_tracking_point.clone().map(|point| view! {
                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.75rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                        <div style="display:grid;gap:0.2rem;">
                            <strong>"First plotted point"</strong>
                            <small>{point.recorded_at_label.clone()}</small>
                            <small style=MUTED_TEXT_STYLE>{format!("{:.5}, {:.5}", point.lat, point.lng)}</small>
                        </div>
                        <a href=tracking_point_map_url(&point) target="_blank" rel="noopener noreferrer" style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#f1f5f9;color:#0f172a;text-decoration:none;">
                            "Open starting point"
                        </a>
                    </div>
                })}
                {earliest_tracking_point.zip(latest_tracking_point).map(|(start, end)| view! {
                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.75rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                        <div style="display:grid;gap:0.2rem;">
                            <strong>"Route handoff"</strong>
                        </div>
                        <a href=tracking_route_maps_url(&start, &end) target="_blank" rel="noopener noreferrer" style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#ede9fe;color:#5b21b6;text-decoration:none;">
                            "Open route span"
                        </a>
                    </div>
                })}
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.75rem;">
                    <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                        <strong>"Tracking session"</strong>
                        <small style=MUTED_TEXT_STYLE>{tracking_window_label}</small>
                    </div>
                    <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                        <strong>"Approx. distance"</strong>
                        <small style=MUTED_TEXT_STYLE>{tracking_distance_label}</small>
                    </div>
                    <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                        <strong>"Tracking health"</strong>
                        <span style=tone_style(&tracking_health_tone)>{screen_value.tracking_health_tone.replace('_', " ")}</span>
                    </div>
                    <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                        <strong>"Geofence"</strong>
                        <span style=tone_style(&geofence_status_tone)>{geofence_status_label.unwrap_or_else(|| "Waiting for stop coordinates".into())}</span>
                    </div>
                    <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                        <strong>"ETA risk"</strong>
                        <span style=tone_style(&eta_risk_tone)>{eta_risk_label.unwrap_or_else(|| "Waiting for appointment data".into())}</span>
                    </div>
                </div>
                <strong>"Tracking points"</strong>
                {render_tracking_points(tracking_points)}
            </section>
        </section>
    }
}
