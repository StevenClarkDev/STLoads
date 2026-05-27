use crate::session;
use leptos::prelude::*;
use shared::{MasterDataMutationResponse, MasterDataRow, MasterDataScreen};

pub(super) fn scroll_to_top_of_page() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            window.scroll_to_with_x_and_y(0.0, 0.0);
        }
    }
}

pub(super) fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#fff1f2;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;",
        _ => "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;",
    }
}

pub(super) fn parse_optional_u64(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        trimmed.parse::<u64>().ok()
    }
}

pub(super) fn supports_write(kind: &str) -> bool {
    matches!(
        kind,
        "countries"
            | "cities"
            | "locations"
            | "load_types"
            | "equipments"
            | "commodity_types"
            | "service_levels"
            | "rejection_reasons"
            | "exception_reasons"
            | "trailer_types"
            | "hazmat_classes"
            | "accessorials"
            | "document_requirements"
            | "customer_configurations"
    )
}

pub(super) fn is_governed_catalog_kind(kind: &str) -> bool {
    matches!(
        kind,
        "service_levels"
            | "rejection_reasons"
            | "exception_reasons"
            | "trailer_types"
            | "hazmat_classes"
            | "accessorials"
            | "document_requirements"
            | "customer_configurations"
    )
}

pub(super) fn supports_delete(kind: &str) -> bool {
    supports_write(kind)
}

pub(super) fn writable_status_label(kind: &str) -> &'static str {
    match kind {
        "countries" => "Writable plus hard delete",
        "cities" => "Writable plus hard delete",
        "locations" | "load_types" | "equipments" | "commodity_types" => {
            "Writable plus safe archive"
        }
        "service_levels"
        | "rejection_reasons"
        | "exception_reasons"
        | "trailer_types"
        | "hazmat_classes"
        | "accessorials"
        | "document_requirements"
        | "customer_configurations" => "Governed write plus ledger",
        _ => "Read only",
    }
}

pub(super) fn delete_label(kind: &str) -> &'static str {
    match kind {
        "countries" | "cities" => "Delete",
        _ => "Archive",
    }
}

pub(super) fn action_key(kind: &str, id: u64) -> String {
    format!("{}:{}", kind, id)
}

pub(super) fn master_data_row_matches_query(kind: &str, row: &MasterDataRow, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }

    kind.to_ascii_lowercase().contains(&query)
        || row.id.to_string().contains(&query)
        || row.primary_label.to_ascii_lowercase().contains(&query)
        || row
            .secondary_label
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&query)
        || row.status_label.to_ascii_lowercase().contains(&query)
        || row.detail.to_ascii_lowercase().contains(&query)
}

pub(super) fn handle_master_data_result(
    auth: session::AuthContext,
    result: Result<MasterDataMutationResponse, String>,
    action_message: RwSignal<Option<String>>,
    refresh_nonce: RwSignal<u64>,
    clear_form: impl FnOnce(),
) {
    match result {
        Ok(response) => {
            action_message.set(Some(response.message));
            if response.success {
                clear_form();
                refresh_nonce.update(|value| *value += 1);
            }
        }
        Err(error) => {
            if error.contains("returned 401") {
                session::invalidate_session(&auth, "Your Rust session expired; sign in again.");
            }
            action_message.set(Some(error));
        }
    }
}

pub(super) fn render_clear_button(
    clear_form: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
) -> impl IntoView {
    view! {
        <button
            type="button"
            on:click=clear_form
            style="padding:0.45rem 0.8rem;border:1px solid #d1d5db;border-radius:0.8rem;background:white;cursor:pointer;"
        >
            "Clear"
        </button>
    }
}

pub(super) fn render_simple_panel(
    kind: &'static str,
    title: &'static str,
    placeholder: &'static str,
    id_signal: RwSignal<String>,
    name_signal: RwSignal<String>,
    save_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    clear_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    button_label: &'static str,
    is_active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div style=move || {
            if is_active.get() {
                "border:2px solid #60a5fa;border-radius:1rem;padding:1rem;background:#f8fbff;display:grid;gap:0.75rem;box-shadow:0 0 0 3px rgba(191,219,254,0.35);".to_string()
            } else {
                "border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;".to_string()
            }
        }>
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                <strong>{title}</strong>
                <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                    {move || is_active.get().then(|| view! {
                        <span style=tone_style("info")>{format!("Editing {}", kind.replace('_', " "))}</span>
                    })}
                    {render_clear_button(clear_action)}
                </div>
            </div>
            <input
                prop:value=move || id_signal.get()
                on:input=move |ev| id_signal.set(event_target_value(&ev))
                placeholder="Optional ID to edit"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <input
                prop:value=move || name_signal.get()
                on:input=move |ev| name_signal.set(event_target_value(&ev))
                placeholder=placeholder
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <button
                type="button"
                on:click=save_action
                style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;"
            >
                {button_label}
            </button>
        </div>
    }
}

pub(super) fn render_governed_panel(
    governed_kind: RwSignal<String>,
    governed_id: RwSignal<String>,
    governed_code: RwSignal<String>,
    governed_label: RwSignal<String>,
    governed_description: RwSignal<String>,
    governed_requires_approval: RwSignal<bool>,
    save_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    clear_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    is_active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div style=move || {
            if is_active.get() {
                "border:2px solid #60a5fa;border-radius:1rem;padding:1rem;background:#f8fbff;display:grid;gap:0.75rem;box-shadow:0 0 0 3px rgba(191,219,254,0.35);".to_string()
            } else {
                "border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;".to_string()
            }
        }>
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                <strong>"Governed catalog"</strong>
                <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                    {move || is_active.get().then(|| view! { <span style=tone_style("info")>"Editing governed row"</span> })}
                    {render_clear_button(clear_action)}
                </div>
            </div>
            <select
                prop:value=move || governed_kind.get()
                on:change=move |ev| governed_kind.set(event_target_value(&ev))
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
            >
                <option value="service_levels">"Service levels"</option>
                <option value="rejection_reasons">"Rejection reasons"</option>
                <option value="exception_reasons">"Exception reasons"</option>
                <option value="trailer_types">"Trailer types"</option>
                <option value="hazmat_classes">"Hazmat classes"</option>
                <option value="accessorials">"Accessorials"</option>
            </select>
            <input
                prop:value=move || governed_id.get()
                on:input=move |ev| governed_id.set(event_target_value(&ev))
                placeholder="Optional ID to edit"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <input
                prop:value=move || governed_code.get()
                on:input=move |ev| governed_code.set(event_target_value(&ev))
                placeholder="code_like_this"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <input
                prop:value=move || governed_label.get()
                on:input=move |ev| governed_label.set(event_target_value(&ev))
                placeholder="Display label"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <textarea
                prop:value=move || governed_description.get()
                on:input=move |ev| governed_description.set(event_target_value(&ev))
                placeholder="Description and operational meaning"
                style="min-height:80px;padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;"
            ></textarea>
            <label style="display:flex;gap:0.55rem;align-items:center;">
                <input
                    type="checkbox"
                    prop:checked=move || governed_requires_approval.get()
                    on:change=move |ev| governed_requires_approval.set(event_target_checked(&ev))
                />
                <span>"Requires approval"</span>
            </label>
            <button
                type="button"
                on:click=save_action
                style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;"
            >
                "Save governed row"
            </button>
        </div>
    }
}

pub(super) fn render_document_requirement_panel(
    doc_rule_id: RwSignal<String>,
    doc_rule_key: RwSignal<String>,
    doc_rule_label: RwSignal<String>,
    doc_rule_scope: RwSignal<String>,
    doc_rule_role: RwSignal<String>,
    doc_rule_lifecycle: RwSignal<String>,
    doc_rule_type: RwSignal<String>,
    doc_rule_blocks: RwSignal<bool>,
    doc_rule_requires_approval: RwSignal<bool>,
    save_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    clear_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    is_active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div style=move || {
            if is_active.get() {
                "border:2px solid #60a5fa;border-radius:1rem;padding:1rem;background:#f8fbff;display:grid;gap:0.75rem;box-shadow:0 0 0 3px rgba(191,219,254,0.35);".to_string()
            } else {
                "border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;".to_string()
            }
        }>
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                <strong>"Document requirement"</strong>
                <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                    {move || is_active.get().then(|| view! { <span style=tone_style("info")>"Editing document rule"</span> })}
                    {render_clear_button(clear_action)}
                </div>
            </div>
            <input prop:value=move || doc_rule_id.get() on:input=move |ev| doc_rule_id.set(event_target_value(&ev)) placeholder="Optional ID to edit" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || doc_rule_key.get() on:input=move |ev| doc_rule_key.set(event_target_value(&ev)) placeholder="rule_key" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || doc_rule_label.get() on:input=move |ev| doc_rule_label.set(event_target_value(&ev)) placeholder="Rate confirmation" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || doc_rule_scope.get() on:input=move |ev| doc_rule_scope.set(event_target_value(&ev)) placeholder="load, onboarding, execution" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || doc_rule_role.get() on:input=move |ev| doc_rule_role.set(event_target_value(&ev)) placeholder="Optional role, e.g. carrier" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || doc_rule_lifecycle.get() on:input=move |ev| doc_rule_lifecycle.set(event_target_value(&ev)) placeholder="booking, submit_onboarding, complete_delivery" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || doc_rule_type.get() on:input=move |ev| doc_rule_type.set(event_target_value(&ev)) placeholder="document_type_key" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <label style="display:flex;gap:0.55rem;align-items:center;">
                <input type="checkbox" prop:checked=move || doc_rule_blocks.get() on:change=move |ev| doc_rule_blocks.set(event_target_checked(&ev)) />
                <span>"Blocks transition"</span>
            </label>
            <label style="display:flex;gap:0.55rem;align-items:center;">
                <input type="checkbox" prop:checked=move || doc_rule_requires_approval.get() on:change=move |ev| doc_rule_requires_approval.set(event_target_checked(&ev)) />
                <span>"Requires approval"</span>
            </label>
            <button type="button" on:click=save_action style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">
                "Save document rule"
            </button>
        </div>
    }
}

pub(super) fn render_customer_configuration_panel(
    customer_config_id: RwSignal<String>,
    customer_config_key: RwSignal<String>,
    customer_config_area: RwSignal<String>,
    customer_config_contract_id: RwSignal<String>,
    customer_config_lane_id: RwSignal<String>,
    customer_config_facility_id: RwSignal<String>,
    customer_config_carrier_group: RwSignal<String>,
    customer_config_refs: RwSignal<String>,
    customer_config_requires_approval: RwSignal<bool>,
    save_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    clear_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    is_active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div style=move || {
            if is_active.get() {
                "border:2px solid #60a5fa;border-radius:1rem;padding:1rem;background:#f8fbff;display:grid;gap:0.75rem;box-shadow:0 0 0 3px rgba(191,219,254,0.35);".to_string()
            } else {
                "border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;".to_string()
            }
        }>
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                <strong>"Customer configuration"</strong>
                <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                    {move || is_active.get().then(|| view! { <span style=tone_style("info")>"Editing customer configuration"</span> })}
                    {render_clear_button(clear_action)}
                </div>
            </div>
            <input prop:value=move || customer_config_id.get() on:input=move |ev| customer_config_id.set(event_target_value(&ev)) placeholder="Optional ID to edit" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || customer_config_key.get() on:input=move |ev| customer_config_key.set(event_target_value(&ev)) placeholder="customer_rule_key" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || customer_config_area.get() on:input=move |ev| customer_config_area.set(event_target_value(&ev)) placeholder="visibility, compliance, billing, notifications" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || customer_config_contract_id.get() on:input=move |ev| customer_config_contract_id.set(event_target_value(&ev)) placeholder="Optional contract ID" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || customer_config_lane_id.get() on:input=move |ev| customer_config_lane_id.set(event_target_value(&ev)) placeholder="Optional lane ID" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || customer_config_facility_id.get() on:input=move |ev| customer_config_facility_id.set(event_target_value(&ev)) placeholder="Optional facility ID" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || customer_config_carrier_group.get() on:input=move |ev| customer_config_carrier_group.set(event_target_value(&ev)) placeholder="Optional carrier group key" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <input prop:value=move || customer_config_refs.get() on:input=move |ev| customer_config_refs.set(event_target_value(&ev)) placeholder="Required references, comma separated" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
            <label style="display:flex;gap:0.55rem;align-items:center;">
                <input type="checkbox" prop:checked=move || customer_config_requires_approval.get() on:change=move |ev| customer_config_requires_approval.set(event_target_checked(&ev)) />
                <span>"Requires approval"</span>
            </label>
            <button type="button" on:click=save_action style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">
                "Save customer config"
            </button>
        </div>
    }
}

pub(super) fn render_country_panel(
    country_id: RwSignal<String>,
    country_name: RwSignal<String>,
    country_iso_code: RwSignal<String>,
    save_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    clear_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    is_active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div style=move || {
            if is_active.get() {
                "border:2px solid #60a5fa;border-radius:1rem;padding:1rem;background:#f8fbff;display:grid;gap:0.75rem;box-shadow:0 0 0 3px rgba(191,219,254,0.35);".to_string()
            } else {
                "border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;".to_string()
            }
        }>
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                <strong>"Countries management"</strong>
                <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                    {move || is_active.get().then(|| view! { <span style=tone_style("info")>"Editing country"</span> })}
                    {render_clear_button(clear_action)}
                </div>
            </div>
            <input
                prop:value=move || country_id.get()
                on:input=move |ev| country_id.set(event_target_value(&ev))
                placeholder="Optional ID to edit"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <input
                prop:value=move || country_name.get()
                on:input=move |ev| country_name.set(event_target_value(&ev))
                placeholder="United States"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <input
                prop:value=move || country_iso_code.get()
                on:input=move |ev| country_iso_code.set(event_target_value(&ev))
                placeholder="US"
                maxlength="8"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <button
                type="button"
                on:click=save_action
                style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;"
            >
                "Save country"
            </button>
        </div>
    }
}

pub(super) fn render_city_panel(
    screen: RwSignal<Option<MasterDataScreen>>,
    city_id: RwSignal<String>,
    city_name: RwSignal<String>,
    city_country_id: RwSignal<String>,
    save_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    clear_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    is_active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div style=move || {
            if is_active.get() {
                "border:2px solid #60a5fa;border-radius:1rem;padding:1rem;background:#f8fbff;display:grid;gap:0.75rem;box-shadow:0 0 0 3px rgba(191,219,254,0.35);".to_string()
            } else {
                "border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;".to_string()
            }
        }>
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                <strong>"Cities management"</strong>
                <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                    {move || is_active.get().then(|| view! { <span style=tone_style("info")>"Editing city"</span> })}
                    {render_clear_button(clear_action)}
                </div>
            </div>
            <input
                prop:value=move || city_id.get()
                on:input=move |ev| city_id.set(event_target_value(&ev))
                placeholder="Optional ID to edit"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <input
                prop:value=move || city_name.get()
                on:input=move |ev| city_name.set(event_target_value(&ev))
                placeholder="Houston"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <select
                prop:value=move || city_country_id.get()
                on:change=move |ev| city_country_id.set(event_target_value(&ev))
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
            >
                <option value="">"Choose country"</option>
                {move || screen.get().map(|data| data.country_options.into_iter().map(|country| view! {
                    <option value={country.id.to_string()}>{country.label}</option>
                }).collect_view())}
            </select>
            <button
                type="button"
                on:click=save_action
                style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;"
            >
                "Save city"
            </button>
        </div>
    }
}

pub(super) fn render_location_panel(
    screen: RwSignal<Option<MasterDataScreen>>,
    location_id: RwSignal<String>,
    location_name: RwSignal<String>,
    location_country_id: RwSignal<String>,
    location_city_id: RwSignal<String>,
    save_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    clear_action: impl Fn(leptos::ev::MouseEvent) + Copy + 'static,
    is_active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div style=move || {
            if is_active.get() {
                "border:2px solid #60a5fa;border-radius:1rem;padding:1rem;background:#f8fbff;display:grid;gap:0.75rem;box-shadow:0 0 0 3px rgba(191,219,254,0.35);".to_string()
            } else {
                "border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;".to_string()
            }
        }>
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                <strong>"Location write flow"</strong>
                <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                    {move || is_active.get().then(|| view! { <span style=tone_style("info")>"Editing location"</span> })}
                    {render_clear_button(clear_action)}
                </div>
            </div>
            <input
                prop:value=move || location_id.get()
                on:input=move |ev| location_id.set(event_target_value(&ev))
                placeholder="Optional ID to edit"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <input
                prop:value=move || location_name.get()
                on:input=move |ev| location_name.set(event_target_value(&ev))
                placeholder="Dallas Yard"
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
            <select
                prop:value=move || location_country_id.get()
                on:change=move |ev| {
                    let next_value = event_target_value(&ev);
                    location_country_id.set(next_value.clone());
                    if let Some(selected_country_id) = parse_optional_u64(&next_value) {
                        let matches_country = screen.get().map(|data| {
                            data.city_options.into_iter().any(|city| {
                                city.country_id == selected_country_id
                                    && Some(city.id) == parse_optional_u64(&location_city_id.get())
                            })
                        }).unwrap_or(false);
                        if !matches_country {
                            location_city_id.set(String::new());
                        }
                    } else {
                        location_city_id.set(String::new());
                    }
                }
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
            >
                <option value="">"Choose country"</option>
                {move || screen.get().map(|data| data.country_options.into_iter().map(|country| view! {
                    <option value={country.id.to_string()}>{country.label}</option>
                }).collect_view())}
            </select>
            <select
                prop:value=move || location_city_id.get()
                on:change=move |ev| location_city_id.set(event_target_value(&ev))
                style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
            >
                <option value="">"Choose city"</option>
                {move || {
                    let selected_country = parse_optional_u64(&location_country_id.get());
                    screen.get().map(|data| {
                        data.city_options.into_iter().filter(|city| {
                            selected_country.map(|country_id| city.country_id == country_id).unwrap_or(true)
                        }).map(|city| view! {
                            <option value={city.id.to_string()}>{city.label}</option>
                        }).collect_view()
                    })
                }}
            </select>
            <button
                type="button"
                on:click=save_action
                style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;"
            >
                "Save location"
            </button>
        </div>
    }
}
