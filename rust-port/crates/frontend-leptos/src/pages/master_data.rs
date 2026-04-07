use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use shared::{LocationUpsertRequest, MasterDataScreen, SimpleCatalogUpsertRequest};

use crate::{
    api,
    session::{self, use_auth},
};

use super::admin_guard_view;

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        _ => "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;",
    }
}

fn parse_optional_u64(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        trimmed.parse::<u64>().ok()
    }
}

#[component]
pub fn MasterDataPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<MasterDataScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);
    let pending_action = RwSignal::new(None::<String>);

    let load_type_id = RwSignal::new(String::new());
    let load_type_name = RwSignal::new(String::new());
    let equipment_id = RwSignal::new(String::new());
    let equipment_name = RwSignal::new(String::new());
    let commodity_type_id = RwSignal::new(String::new());
    let commodity_type_name = RwSignal::new(String::new());
    let location_id = RwSignal::new(String::new());
    let location_name = RwSignal::new(String::new());
    let location_country_id = RwSignal::new(String::new());
    let location_city_id = RwSignal::new(String::new());

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_master_data")
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let _refresh = refresh_nonce.get();

        if !ready || !current_session.authenticated || !can_view.get() {
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_master_data_screen().await {
                Ok(next_screen) => {
                    screen.set(Some(next_screen));
                    error_message.set(None);
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

    let save_load_type = move |_| {
        if load_type_name.get().trim().is_empty() {
            action_message.set(Some("Enter a load type name before saving.".into()));
            return;
        }

        pending_action.set(Some("load type".into()));
        let auth = auth.clone();
        let request = SimpleCatalogUpsertRequest {
            id: parse_optional_u64(&load_type_id.get()),
            name: load_type_name.get(),
        };
        spawn_local(async move {
            match api::upsert_load_type(&request).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        load_type_id.set(String::new());
                        load_type_name.set(String::new());
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
            pending_action.set(None);
        });
    };

    let save_equipment = move |_| {
        if equipment_name.get().trim().is_empty() {
            action_message.set(Some("Enter an equipment name before saving.".into()));
            return;
        }

        pending_action.set(Some("equipment".into()));
        let auth = auth.clone();
        let request = SimpleCatalogUpsertRequest {
            id: parse_optional_u64(&equipment_id.get()),
            name: equipment_name.get(),
        };
        spawn_local(async move {
            match api::upsert_equipment(&request).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        equipment_id.set(String::new());
                        equipment_name.set(String::new());
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
            pending_action.set(None);
        });
    };

    let save_commodity_type = move |_| {
        if commodity_type_name.get().trim().is_empty() {
            action_message.set(Some("Enter a commodity type name before saving.".into()));
            return;
        }

        pending_action.set(Some("commodity type".into()));
        let auth = auth.clone();
        let request = SimpleCatalogUpsertRequest {
            id: parse_optional_u64(&commodity_type_id.get()),
            name: commodity_type_name.get(),
        };
        spawn_local(async move {
            match api::upsert_commodity_type(&request).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        commodity_type_id.set(String::new());
                        commodity_type_name.set(String::new());
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
            pending_action.set(None);
        });
    };

    let save_location = move |_| {
        if location_name.get().trim().is_empty() {
            action_message.set(Some("Enter a location name before saving.".into()));
            return;
        }

        pending_action.set(Some("location".into()));
        let auth = auth.clone();
        let request = LocationUpsertRequest {
            id: parse_optional_u64(&location_id.get()),
            name: location_name.get(),
            country_id: parse_optional_u64(&location_country_id.get()),
            city_id: parse_optional_u64(&location_city_id.get()),
        };
        spawn_local(async move {
            match api::upsert_location(&request).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        location_id.set(String::new());
                        location_name.set(String::new());
                        location_country_id.set(String::new());
                        location_city_id.set(String::new());
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
            pending_action.set(None);
        });
    };

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Master Data Catalog", &["access_admin_portal", "manage_master_data"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1.25rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Master Data Catalog".into())}</h2>
                                <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "Read-first master-data visibility for the Rust admin portal.".into())}</p>
                            </div>
                            <span style=tone_style(if pending_action.get().is_some() { "warning" } else { "success" })>
                                {move || pending_action.get().map(|value| format!("Saving {}...", value)).unwrap_or_else(|| "Write controls ready".into())}
                            </span>
                        </section>

                        {move || auth.session.get().user.map(|user| view! { <section style="padding:0.85rem 1rem;border:1px solid #dcfce7;border-radius:0.9rem;background:#f0fdf4;color:#166534;">{format!("Authenticated as {} ({})", user.name, user.role_label)}</section> })}
                        {move || action_message.get().map(|message| view! { <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">{message}</section> })}
                        {move || error_message.get().map(|message| view! { <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">{message}</section> })}

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;">
                            {move || {
                                if is_loading.get() && screen.get().is_none() {
                                    view! { <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"Loading master-data catalog from the Rust backend..."</div> }.into_any()
                                } else {
                                    screen.get().map(|data| {
                                        data.summary_cards.into_iter().map(|card| view! {
                                            <div style="padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fcfcfb;display:grid;gap:0.45rem;">
                                                <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;"><strong>{card.label}</strong><span style="padding:0.2rem 0.55rem;border-radius:999px;background:#eff6ff;color:#1d4ed8;">{card.total}</span></div>
                                                <p style="margin:0;">{card.note}</p>
                                                <small style="color:#64748b;">{card.admin_route}</small>
                                            </div>
                                        }).collect_view().into_any()
                                    }).unwrap_or_else(|| view! { <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"No master-data catalog is available yet."</div> }.into_any())
                                }
                            }}
                        </section>

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(250px,1fr));gap:1rem;align-items:start;">
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;"><strong>"Load type write flow"</strong><input prop:value=move || load_type_id.get() on:input=move |ev| load_type_id.set(event_target_value(&ev)) placeholder="Optional ID to edit" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><input prop:value=move || load_type_name.get() on:input=move |ev| load_type_name.set(event_target_value(&ev)) placeholder="Dry Van" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><button type="button" on:click=save_load_type disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">"Save load type"</button></div>
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;"><strong>"Equipment write flow"</strong><input prop:value=move || equipment_id.get() on:input=move |ev| equipment_id.set(event_target_value(&ev)) placeholder="Optional ID to edit" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><input prop:value=move || equipment_name.get() on:input=move |ev| equipment_name.set(event_target_value(&ev)) placeholder="Reefer" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><button type="button" on:click=save_equipment disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">"Save equipment"</button></div>
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;"><strong>"Commodity type write flow"</strong><input prop:value=move || commodity_type_id.get() on:input=move |ev| commodity_type_id.set(event_target_value(&ev)) placeholder="Optional ID to edit" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><input prop:value=move || commodity_type_name.get() on:input=move |ev| commodity_type_name.set(event_target_value(&ev)) placeholder="Consumer Goods" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><button type="button" on:click=save_commodity_type disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">"Save commodity type"</button></div>
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;"><strong>"Location write flow"</strong><input prop:value=move || location_id.get() on:input=move |ev| location_id.set(event_target_value(&ev)) placeholder="Optional ID to edit" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><input prop:value=move || location_name.get() on:input=move |ev| location_name.set(event_target_value(&ev)) placeholder="Dallas Yard" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><input prop:value=move || location_country_id.get() on:input=move |ev| location_country_id.set(event_target_value(&ev)) placeholder="Country ID" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><input prop:value=move || location_city_id.get() on:input=move |ev| location_city_id.set(event_target_value(&ev)) placeholder="City ID" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /><button type="button" on:click=save_location disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">"Save location"</button></div>
                        </section>

                        <section style="display:grid;gap:1rem;">
                            {move || screen.get().map(|data| {
                                data.sections.into_iter().map(|section| view! {
                                    <section style="border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;padding:1rem;display:grid;gap:0.8rem;">
                                        <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                            <div><strong>{section.label.clone()}</strong><div><small style="color:#64748b;">{section.admin_route.clone()}</small></div></div>
                                            <div style="display:flex;gap:0.5rem;flex-wrap:wrap;align-items:center;"><span style="padding:0.2rem 0.55rem;border-radius:999px;background:#f1f5f9;color:#334155;">{format!("{} rows", section.total)}</span><span style=tone_style(if matches!(section.key.as_str(), "locations" | "load_types" | "equipments" | "commodity_types") { "success" } else { "warning" })>{if matches!(section.key.as_str(), "locations" | "load_types" | "equipments" | "commodity_types") { "Writable via form" } else { "Read only" }}</span></div>
                                        </div>
                                        {if section.rows.is_empty() { view! { <p style="margin:0;">{section.empty_message}</p> }.into_any() } else { view! {
                                            <div style="overflow:auto;"><table style="width:100%;border-collapse:collapse;min-width:720px;"><thead style="background:#f8fafc;"><tr><th style="text-align:left;padding:0.75rem;">"ID"</th><th style="text-align:left;padding:0.75rem;">"Name"</th><th style="text-align:left;padding:0.75rem;">"Context"</th><th style="text-align:left;padding:0.75rem;">"Status"</th><th style="text-align:left;padding:0.75rem;">"Detail"</th></tr></thead><tbody>{section.rows.into_iter().map(|row| view! { <tr style="border-top:1px solid #f1f5f9;vertical-align:top;"><td style="padding:0.75rem;">{row.id}</td><td style="padding:0.75rem;"><strong>{row.primary_label}</strong></td><td style="padding:0.75rem;">{row.secondary_label.unwrap_or_else(|| "-".into())}</td><td style="padding:0.75rem;">{row.status_label}</td><td style="padding:0.75rem;">{row.detail}</td></tr> }).collect_view()}</tbody></table></div>
                                        }.into_any() }}
                                    </section>
                                }).collect_view()
                            })}
                        </section>

                        <section style="display:grid;gap:0.35rem;">{move || screen.get().map(|data| data.notes.into_iter().map(|note| view! { <p style="margin:0;">{note}</p> }).collect_view())}</section>
                    </article>
                }.into_any()
            }
        }}
    }
}
