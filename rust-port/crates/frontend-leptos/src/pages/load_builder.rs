use leptos::{ev, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params_map},
};

use crate::{
    api, google_places,
    session::{self, use_auth},
};
use shared::LoadBuilderScreen;

use super::load_builder_helpers::{
    FIELD_INPUT_STYLE, FIELD_LABEL_STYLE, FIELD_SELECT_STYLE, FIELD_TEXTAREA_STYLE, LoadLegDraft,
    build_create_load_request, default_leg, delivery_address_input_id, delivery_city_input_id,
    delivery_country_input_id, delivery_latitude_input_id, delivery_longitude_input_id,
    delivery_place_id_input_id, google_maps_api_key, initialize_form, pickup_address_input_id,
    pickup_city_input_id, pickup_country_input_id, pickup_latitude_input_id,
    pickup_longitude_input_id, pickup_place_id_input_id,
};
#[component]
pub fn LoadBuilderPage() -> impl IntoView {
    let navigate = use_navigate();
    let params = use_params_map();
    let auth = use_auth();
    let screen = RwSignal::new(None::<LoadBuilderScreen>);
    let is_loading = RwSignal::new(false);
    let is_submitting = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let google_status = RwSignal::new(None::<String>);
    let google_ready = RwSignal::new(false);

    let title = RwSignal::new(String::new());
    let load_type_id = RwSignal::new(String::new());
    let equipment_id = RwSignal::new(String::new());
    let commodity_type_id = RwSignal::new(String::new());
    let freight_mode = RwSignal::new("FTL".to_string());
    let visibility = RwSignal::new("public".to_string());
    let service_level = RwSignal::new(String::new());
    let customer_reference = RwSignal::new(String::new());
    let po_number = RwSignal::new(String::new());
    let pickup_appointment_ref = RwSignal::new(String::new());
    let delivery_appointment_ref = RwSignal::new(String::new());
    let facility_contact_name = RwSignal::new(String::new());
    let facility_contact_phone = RwSignal::new(String::new());
    let facility_contact_email = RwSignal::new(String::new());
    let appointment_window_start = RwSignal::new(String::new());
    let appointment_window_end = RwSignal::new(String::new());
    let accessorial_flags = RwSignal::new(String::new());
    let weight_unit = RwSignal::new("LBS".to_string());
    let weight = RwSignal::new(String::new());
    let temperature_data = RwSignal::new(String::new());
    let container_data = RwSignal::new(String::new());
    let securement_data = RwSignal::new(String::new());
    let special_instructions = RwSignal::new(String::new());
    let is_hazardous = RwSignal::new(false);
    let is_temperature_controlled = RwSignal::new(false);
    let legs = RwSignal::new(Vec::<LoadLegDraft>::new());

    let editing_load_id = Memo::new(move |_| {
        params.with(|map| {
            map.get("load_id")
                .and_then(|value| value.parse::<u64>().ok())
        })
    });

    let can_manage_loads = Signal::derive(move || session::has_permission(&auth, "manage_loads"));

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let editing_load_id = editing_load_id.get();

        if !ready {
            return;
        }

        if !current_session.authenticated {
            screen.set(None);
            error_message.set(Some(if editing_load_id.is_some() {
                "Sign in before editing loads from the Rust builder.".into()
            } else {
                "Sign in before creating loads from the Rust builder.".into()
            }));
            is_loading.set(false);
            return;
        }

        if !can_manage_loads.get() {
            screen.set(None);
            error_message.set(Some(if editing_load_id.is_some() {
                "The authenticated session does not have load update access in this Rust slice."
                    .into()
            } else {
                "The authenticated session does not have load creation access in this Rust slice."
                    .into()
            }));
            is_loading.set(false);
            return;
        }

        is_loading.set(true);
        let auth = auth;

        spawn_local(async move {
            match api::fetch_load_builder_screen(editing_load_id).await {
                Ok(next_screen) => {
                    error_message.set(None);
                    initialize_form(
                        &next_screen,
                        title,
                        load_type_id,
                        equipment_id,
                        commodity_type_id,
                        freight_mode,
                        visibility,
                        service_level,
                        customer_reference,
                        po_number,
                        pickup_appointment_ref,
                        delivery_appointment_ref,
                        facility_contact_name,
                        facility_contact_phone,
                        facility_contact_email,
                        appointment_window_start,
                        appointment_window_end,
                        accessorial_flags,
                        weight_unit,
                        weight,
                        temperature_data,
                        container_data,
                        securement_data,
                        special_instructions,
                        is_hazardous,
                        is_temperature_controlled,
                        legs,
                    );
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

    Effect::new(move |_| {
        let Some(_screen) = screen.get() else {
            return;
        };

        let leg_count = legs.with(|items| items.len());
        if leg_count == 0 {
            return;
        }

        let Some(api_key) = google_maps_api_key() else {
            google_ready.set(false);
            google_status.set(Some("Google address autocomplete is waiting for GOOGLE_MAPS_API_KEY in frontend runtime config. Manual typing still works, but IBM deployment should provide the browser key with domain restrictions.".into()));
            return;
        };

        spawn_local(async move {
            match google_places::ensure_loaded(&api_key).await {
                Ok(()) => {
                    google_ready.set(true);
                    google_status.set(Some("Google address autocomplete is active. If device GPS is available, predictions are biased toward the current area; if GPS is off, Google returns general suggestions instead.".into()));

                    for index in 0..leg_count {
                        let _ = google_places::attach_address_autocomplete(
                            &pickup_address_input_id(index),
                            &pickup_city_input_id(index),
                            &pickup_country_input_id(index),
                            &pickup_place_id_input_id(index),
                            &pickup_latitude_input_id(index),
                            &pickup_longitude_input_id(index),
                        )
                        .await;

                        let _ = google_places::attach_address_autocomplete(
                            &delivery_address_input_id(index),
                            &delivery_city_input_id(index),
                            &delivery_country_input_id(index),
                            &delivery_place_id_input_id(index),
                            &delivery_latitude_input_id(index),
                            &delivery_longitude_input_id(index),
                        )
                        .await;
                    }
                }
                Err(error) => {
                    google_ready.set(false);
                    google_status.set(Some(error));
                }
            }
        });
    });

    let add_leg = move |_| {
        let Some(screen_value) = screen.get() else {
            action_message.set(Some(
                "The load builder master data has not loaded yet, so a new leg cannot be added."
                    .into(),
            ));
            return;
        };

        legs.update(|items| items.push(default_leg(&screen_value)));
        action_message.set(None);
    };

    let submit = Callback::new(move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let editing_load_id = editing_load_id.get();

        if !can_manage_loads.get() {
            action_message.set(Some(if editing_load_id.is_some() {
                "The authenticated session does not have load update access in this Rust slice."
                    .into()
            } else {
                "The authenticated session does not have load creation access in this Rust slice."
                    .into()
            }));
            return;
        }

        let payload = match build_create_load_request(
            &title.get(),
            &load_type_id.get(),
            &equipment_id.get(),
            &commodity_type_id.get(),
            &freight_mode.get(),
            &visibility.get(),
            &service_level.get(),
            &customer_reference.get(),
            &po_number.get(),
            &pickup_appointment_ref.get(),
            &delivery_appointment_ref.get(),
            &facility_contact_name.get(),
            &facility_contact_phone.get(),
            &facility_contact_email.get(),
            &appointment_window_start.get(),
            &appointment_window_end.get(),
            &accessorial_flags.get(),
            &weight_unit.get(),
            &weight.get(),
            &temperature_data.get(),
            &container_data.get(),
            &securement_data.get(),
            &special_instructions.get(),
            is_hazardous.get(),
            is_temperature_controlled.get(),
            &legs.get(),
        ) {
            Ok(payload) => payload,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };

        is_submitting.set(true);
        action_message.set(None);
        let auth = auth;
        let navigate = navigate.clone();

        spawn_local(async move {
            let result = match editing_load_id {
                Some(load_id) => api::update_load(load_id, &payload).await,
                None => api::create_load(&payload).await,
            };

            match result {
                Ok(response) => {
                    action_message.set(Some(response.message.clone()));
                    if response.success {
                        if let Some(load_id) = response.load_id {
                            navigate(&format!("/loads/{}", load_id), Default::default());
                        } else if let Some(screen_value) = screen.get_untracked() {
                            initialize_form(
                                &screen_value,
                                title,
                                load_type_id,
                                equipment_id,
                                commodity_type_id,
                                freight_mode,
                                visibility,
                                service_level,
                                customer_reference,
                                po_number,
                                pickup_appointment_ref,
                                delivery_appointment_ref,
                                facility_contact_name,
                                facility_contact_phone,
                                facility_contact_email,
                                appointment_window_start,
                                appointment_window_end,
                                accessorial_flags,
                                weight_unit,
                                weight,
                                temperature_data,
                                container_data,
                                securement_data,
                                special_instructions,
                                is_hazardous,
                                is_temperature_controlled,
                                legs,
                            );
                        }
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

            is_submitting.set(false);
        });
    });

    view! {
        <article style="display:grid;gap:1.25rem;max-width:1100px;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style=FIELD_LABEL_STYLE>
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Create Load".into())}</h2>
                    <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "Rust builder for Google-address load creation.".into())}</p>
                </div>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <A href="/loads" attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#f4f4f5;color:#111827;text-decoration:none;">"Back to load board"</A>
                    <span style="padding:0.35rem 0.65rem;border-radius:999px;background:#eff6ff;color:#1d4ed8;">
                        {move || if is_submitting.get() { "Saving..." } else if google_ready.get() { "Google address mode" } else { "Address setup pending" }}
                    </span>
                </div>
            </section>

            {move || error_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">
                    {message}
                </section>
            })}

            {move || {
                if is_loading.get() && screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"Loading Rust load-builder master data..."</section>
                    }.into_any()
                } else if screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"The load builder is waiting for a valid authenticated Rust session."</section>
                    }.into_any()
                } else {
                    view! {
                        <form on:submit=move |ev| submit.run(ev) style="display:grid;gap:1.25rem;">
                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                <label style="display:grid;gap:0.35rem;grid-column:span 2;">
                                    <span>"Load title"</span>
                                    <input type="text" prop:value=move || title.get() on:input=move |ev| title.set(event_target_value(&ev)) placeholder="Dallas to Joliet produce reload" style=FIELD_INPUT_STYLE />
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Load type"</span>
                                    <select prop:value=move || load_type_id.get() on:change=move |ev| load_type_id.set(event_target_value(&ev)) style=FIELD_SELECT_STYLE>
                                        <option value="">"Select load type"</option>
                                        {move || screen.get().map(|value| value.load_type_options.into_iter().map(|option| view! { <option value={option.id.to_string()}>{option.label}</option> }).collect_view())}
                                    </select>
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Equipment"</span>
                                    <select prop:value=move || equipment_id.get() on:change=move |ev| equipment_id.set(event_target_value(&ev)) style=FIELD_SELECT_STYLE>
                                        <option value="">"Select equipment"</option>
                                        {move || screen.get().map(|value| value.equipment_options.into_iter().map(|option| view! { <option value={option.id.to_string()}>{option.label}</option> }).collect_view())}
                                    </select>
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Commodity type"</span>
                                    <select prop:value=move || commodity_type_id.get() on:change=move |ev| commodity_type_id.set(event_target_value(&ev)) style=FIELD_SELECT_STYLE>
                                        <option value="">"Select commodity type"</option>
                                        {move || screen.get().map(|value| value.commodity_type_options.into_iter().map(|option| view! { <option value={option.id.to_string()}>{option.label}</option> }).collect_view())}
                                    </select>
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Freight mode"</span>
                                    <select prop:value=move || freight_mode.get() on:change=move |ev| freight_mode.set(event_target_value(&ev)) style=FIELD_SELECT_STYLE>
                                        <option value="FTL">"FTL"</option>
                                        <option value="LTL">"LTL"</option>
                                        <option value="intermodal">"Intermodal"</option>
                                        <option value="drayage">"Drayage"</option>
                                        <option value="cross_border" disabled=true>"Cross-border deferred"</option>
                                        <option value="freight_forwarding" disabled=true>"Freight forwarding deferred"</option>
                                        <option value="mixed_mode" disabled=true>"Mixed mode deferred"</option>
                                    </select>
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Visibility"</span>
                                    <select prop:value=move || visibility.get() on:change=move |ev| visibility.set(event_target_value(&ev)) style=FIELD_SELECT_STYLE>
                                        <option value="public">"Public"</option>
                                        <option value="private">"Private"</option>
                                        <option value="contract">"Contract"</option>
                                        <option value="internal">"Internal only"</option>
                                    </select>
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Service level"</span>
                                    <input type="text" prop:value=move || service_level.get() on:input=move |ev| service_level.set(event_target_value(&ev)) placeholder="Standard, expedited, team, guaranteed" style=FIELD_INPUT_STYLE />
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Customer reference"</span>
                                    <input type="text" prop:value=move || customer_reference.get() on:input=move |ev| customer_reference.set(event_target_value(&ev)) placeholder="Customer load or tender ID" style=FIELD_INPUT_STYLE />
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"PO number"</span>
                                    <input type="text" prop:value=move || po_number.get() on:input=move |ev| po_number.set(event_target_value(&ev)) placeholder="PO, sales order, or shipper ref" style=FIELD_INPUT_STYLE />
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Weight unit"</span>
                                    <select prop:value=move || weight_unit.get() on:change=move |ev| weight_unit.set(event_target_value(&ev)) style=FIELD_SELECT_STYLE>
                                        {move || screen.get().map(|value| value.weight_units.into_iter().map(|unit| view! { <option value={unit.clone()}>{unit.clone()}</option> }).collect_view())}
                                    </select>
                                </label>

                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Weight"</span>
                                    <input type="number" step="0.01" min="0" prop:value=move || weight.get() on:input=move |ev| weight.set(event_target_value(&ev)) placeholder="42000" style=FIELD_INPUT_STYLE />
                                </label>
                            </section>

                            <section style="display:grid;gap:0.85rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;padding:1rem;">
                                <strong>"Appointments and facility contact"</strong>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Pickup appointment ref"</span>
                                        <input type="text" prop:value=move || pickup_appointment_ref.get() on:input=move |ev| pickup_appointment_ref.set(event_target_value(&ev)) placeholder="Pickup confirmation" style=FIELD_INPUT_STYLE />
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Delivery appointment ref"</span>
                                        <input type="text" prop:value=move || delivery_appointment_ref.get() on:input=move |ev| delivery_appointment_ref.set(event_target_value(&ev)) placeholder="Delivery confirmation" style=FIELD_INPUT_STYLE />
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Window start"</span>
                                        <input type="datetime-local" prop:value=move || appointment_window_start.get() on:input=move |ev| appointment_window_start.set(event_target_value(&ev)) style=FIELD_INPUT_STYLE />
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Window end"</span>
                                        <input type="datetime-local" prop:value=move || appointment_window_end.get() on:input=move |ev| appointment_window_end.set(event_target_value(&ev)) style=FIELD_INPUT_STYLE />
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Contact name"</span>
                                        <input type="text" prop:value=move || facility_contact_name.get() on:input=move |ev| facility_contact_name.set(event_target_value(&ev)) placeholder="Dock or facility contact" style=FIELD_INPUT_STYLE />
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Contact phone"</span>
                                        <input type="tel" prop:value=move || facility_contact_phone.get() on:input=move |ev| facility_contact_phone.set(event_target_value(&ev)) placeholder="+1 555 0100" style=FIELD_INPUT_STYLE />
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Contact email"</span>
                                        <input type="email" prop:value=move || facility_contact_email.get() on:input=move |ev| facility_contact_email.set(event_target_value(&ev)) placeholder="shipping@example.com" style=FIELD_INPUT_STYLE />
                                    </label>
                                </div>
                            </section>

                            <section style="display:grid;gap:0.85rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;padding:1rem;">
                                <strong>"Special handling"</strong>
                                <div style="display:flex;gap:1rem;flex-wrap:wrap;">
                                    <label style="display:flex;gap:0.5rem;align-items:center;">
                                        <input type="checkbox" prop:checked=move || is_hazardous.get() on:change=move |ev| is_hazardous.set(event_target_checked(&ev)) />
                                        <span>"Hazardous"</span>
                                    </label>
                                    <label style="display:flex;gap:0.5rem;align-items:center;">
                                        <input type="checkbox" prop:checked=move || is_temperature_controlled.get() on:change=move |ev| is_temperature_controlled.set(event_target_checked(&ev)) />
                                        <span>"Temperature controlled"</span>
                                    </label>
                                </div>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Accessorials"</span>
                                        <textarea prop:value=move || accessorial_flags.get() on:input=move |ev| accessorial_flags.set(event_target_value(&ev)) placeholder="Detention, lumper, liftgate, inside delivery, or JSON" style=FIELD_TEXTAREA_STYLE></textarea>
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Temperature details"</span>
                                        <textarea prop:value=move || temperature_data.get() on:input=move |ev| temperature_data.set(event_target_value(&ev)) placeholder="34-38 F continuous, pre-cool, pulp checks, or JSON" style=FIELD_TEXTAREA_STYLE></textarea>
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Container details"</span>
                                        <textarea prop:value=move || container_data.get() on:input=move |ev| container_data.set(event_target_value(&ev)) placeholder="Container size, chassis, seal, genset, or JSON" style=FIELD_TEXTAREA_STYLE></textarea>
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Securement details"</span>
                                        <textarea prop:value=move || securement_data.get() on:input=move |ev| securement_data.set(event_target_value(&ev)) placeholder="Straps, chains, tarps, dunnage, edge protectors, or JSON" style=FIELD_TEXTAREA_STYLE></textarea>
                                    </label>
                                </div>
                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Special instructions"</span>
                                    <textarea prop:value=move || special_instructions.get() on:input=move |ev| special_instructions.set(event_target_value(&ev)) placeholder="Appointment notes, lumper details, securement reminders, or customer-specific instructions." style="min-height:120px;padding:0.9rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;"></textarea>
                                </label>
                            </section>

                            <section style="display:grid;gap:0.85rem;">
                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                    <div>
                                        <strong>"Load legs"</strong>
                                    </div>
                                    <button type="button" on:click=add_leg style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">"Add leg"</button>
                                </div>

                                {move || {
                                    let current_legs = legs.get();
                                    let removable = current_legs.len() > 1;
                                    current_legs.into_iter().enumerate().map(|(index, leg)| {
                                        let pickup_address_id = pickup_address_input_id(index);
                                        let pickup_city_id = pickup_city_input_id(index);
                                        let pickup_country_id = pickup_country_input_id(index);
                                        let pickup_place_id_id = pickup_place_id_input_id(index);
                                        let pickup_latitude_id = pickup_latitude_input_id(index);
                                        let pickup_longitude_id = pickup_longitude_input_id(index);
                                        let delivery_address_id = delivery_address_input_id(index);
                                        let delivery_city_id = delivery_city_input_id(index);
                                        let delivery_country_id = delivery_country_input_id(index);
                                        let delivery_place_id_id = delivery_place_id_input_id(index);
                                        let delivery_latitude_id = delivery_latitude_input_id(index);
                                        let delivery_longitude_id = delivery_longitude_input_id(index);
                                        view! {
                                            <section style="display:grid;gap:0.85rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;padding:1rem;">
                                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                                    <strong>{format!("Leg {}", index + 1)}</strong>
                                                    {removable.then(|| view! {
                                                        <button
                                                            type="button"
                                                            on:click=move |_| {
                                                                legs.update(|items| {
                                                                    if items.len() > 1 && index < items.len() {
                                                                        items.remove(index);
                                                                    }
                                                                });
                                                            }
                                                            style="padding:0.45rem 0.75rem;border:1px solid #fecaca;border-radius:0.75rem;background:#fff1f2;color:#be123c;cursor:pointer;"
                                                        >
                                                            "Remove"
                                                        </button>
                                                    })}
                                                </div>

                                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                                    <label style="display:grid;gap:0.35rem;grid-column:span 2;">
                                                        <span>"Pickup address"</span>
                                                        <input id=pickup_address_id type="text" prop:value=leg.pickup_location_address.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.pickup_location_address = value.clone();
                                                                }
                                                            });
                                                        } placeholder="Search pickup address with Google" style=FIELD_INPUT_STYLE />
                                                    </label>

                                                    <label style="display:grid;gap:0.35rem;grid-column:span 2;">
                                                        <span>"Delivery address"</span>
                                                        <input id=delivery_address_id type="text" prop:value=leg.delivery_location_address.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.delivery_location_address = value.clone();
                                                                }
                                                            });
                                                        } placeholder="Search delivery address with Google" style=FIELD_INPUT_STYLE />
                                                    </label>

                                                    <input id=pickup_city_id type="hidden" prop:value=leg.pickup_city.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_city = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=pickup_country_id type="hidden" prop:value=leg.pickup_country.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_country = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=pickup_place_id_id type="hidden" prop:value=leg.pickup_place_id.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_place_id = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=pickup_latitude_id type="hidden" prop:value=leg.pickup_latitude.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_latitude = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=pickup_longitude_id type="hidden" prop:value=leg.pickup_longitude.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_longitude = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_city_id type="hidden" prop:value=leg.delivery_city.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_city = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_country_id type="hidden" prop:value=leg.delivery_country.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_country = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_place_id_id type="hidden" prop:value=leg.delivery_place_id.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_place_id = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_latitude_id type="hidden" prop:value=leg.delivery_latitude.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_latitude = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_longitude_id type="hidden" prop:value=leg.delivery_longitude.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_longitude = value.clone();
                                                            }
                                                        });
                                                    } />

                                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;grid-column:span 2;">
                                                        <div style="padding:0.75rem;border:1px solid #e5e7eb;border-radius:0.85rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                                            <small style="color:#64748b;">"Pickup city / country"</small>
                                                            <strong>{if leg.pickup_city.is_empty() && leg.pickup_country.is_empty() { "Waiting for Google selection".to_string() } else { format!("{} {}{}", leg.pickup_city, if leg.pickup_city.is_empty() || leg.pickup_country.is_empty() { "" } else { "-" }, leg.pickup_country) }}</strong>
                                                        </div>
                                                        <div style="padding:0.75rem;border:1px solid #e5e7eb;border-radius:0.85rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                                            <small style="color:#64748b;">"Delivery city / country"</small>
                                                            <strong>{if leg.delivery_city.is_empty() && leg.delivery_country.is_empty() { "Waiting for Google selection".to_string() } else { format!("{} {}{}", leg.delivery_city, if leg.delivery_city.is_empty() || leg.delivery_country.is_empty() { "" } else { "-" }, leg.delivery_country) }}</strong>
                                                        </div>
                                                    </div>

                                                    <label style=FIELD_LABEL_STYLE>
                                                        <span>"Pickup date"</span>
                                                        <input type="date" prop:value=leg.pickup_date.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.pickup_date = value.clone();
                                                                }
                                                            });
                                                        } style=FIELD_INPUT_STYLE />
                                                    </label>

                                                    <label style=FIELD_LABEL_STYLE>
                                                        <span>"Delivery date"</span>
                                                        <input type="date" prop:value=leg.delivery_date.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.delivery_date = value.clone();
                                                                }
                                                            });
                                                        } style=FIELD_INPUT_STYLE />
                                                    </label>

                                                    <label style=FIELD_LABEL_STYLE>
                                                        <span>"Bid status"</span>
                                                        <select prop:value=leg.bid_status.clone() on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.bid_status = value.clone();
                                                                }
                                                            });
                                                        } style=FIELD_SELECT_STYLE>
                                                            {move || screen.get().map(|value| value.bid_status_options.into_iter().map(|status| view! { <option value={status.clone()}>{status.clone()}</option> }).collect_view())}
                                                        </select>
                                                    </label>

                                                    <label style=FIELD_LABEL_STYLE>
                                                        <span>"Price"</span>
                                                        <input type="number" step="0.01" min="0" prop:value=leg.price.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.price = value.clone();
                                                                }
                                                            });
                                                        } placeholder="2450" style=FIELD_INPUT_STYLE />
                                                    </label>
                                                </div>
                                            </section>
                                        }
                                    }).collect_view()
                                }}
                            </section>

                            <section style="display:flex;justify-content:flex-end;gap:0.75rem;flex-wrap:wrap;">
                                <A href="/loads" attr:style="padding:0.75rem 1rem;border-radius:0.85rem;background:#f4f4f5;color:#111827;text-decoration:none;">"Cancel"</A>
                                <button type="submit" disabled=move || is_submitting.get() || !can_manage_loads.get() style="padding:0.75rem 1rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">
                                    {move || if is_submitting.get() { "Saving load...".to_string() } else { screen.get().map(|value| value.submit_label).unwrap_or_else(|| "Save load".into()) }}
                                </button>
                            </section>
                        </form>
                    }.into_any()
                }
            }}

        </article>
    }
}
