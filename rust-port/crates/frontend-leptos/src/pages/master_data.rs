use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use shared::{
    CityUpsertRequest, CountryUpsertRequest, LocationUpsertRequest, MasterDataDeleteRequest,
    MasterDataMutationResponse, MasterDataRow, MasterDataScreen, SimpleCatalogUpsertRequest,
};

use crate::{
    api,
    session::{self, use_auth},
};

use super::admin_guard_view;

fn scroll_to_top_of_page() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            window.scroll_to_with_x_and_y(0.0, 0.0);
        }
    }
}

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#fff1f2;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;",
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

fn supports_write(kind: &str) -> bool {
    matches!(
        kind,
        "countries" | "cities" | "locations" | "load_types" | "equipments" | "commodity_types"
    )
}

fn supports_delete(kind: &str) -> bool {
    supports_write(kind)
}

fn writable_status_label(kind: &str) -> &'static str {
    match kind {
        "countries" => "Writable plus hard delete",
        "cities" => "Writable plus hard delete",
        "locations" | "load_types" | "equipments" | "commodity_types" => {
            "Writable plus safe archive"
        }
        _ => "Read only",
    }
}

fn delete_label(kind: &str) -> &'static str {
    match kind {
        "countries" | "cities" => "Delete",
        _ => "Archive",
    }
}

fn action_key(kind: &str, id: u64) -> String {
    format!("{}:{}", kind, id)
}

#[component]
pub fn MasterDataPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<MasterDataScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let search_query = RwSignal::new(String::new());
    let refresh_nonce = RwSignal::new(0_u64);
    let pending_action = RwSignal::new(None::<String>);
    let armed_delete = RwSignal::new(None::<String>);
    let active_editor = RwSignal::new(None::<String>);

    let country_id = RwSignal::new(String::new());
    let country_name = RwSignal::new(String::new());
    let country_iso_code = RwSignal::new(String::new());

    let city_id = RwSignal::new(String::new());
    let city_name = RwSignal::new(String::new());
    let city_country_id = RwSignal::new(String::new());

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

    let clear_country_form = move |_| {
        country_id.set(String::new());
        country_name.set(String::new());
        country_iso_code.set(String::new());
        if active_editor.get() == Some("countries".into()) {
            active_editor.set(None);
        }
    };

    let clear_city_form = move |_| {
        city_id.set(String::new());
        city_name.set(String::new());
        city_country_id.set(String::new());
        if active_editor.get() == Some("cities".into()) {
            active_editor.set(None);
        }
    };

    let clear_load_type_form = move |_| {
        load_type_id.set(String::new());
        load_type_name.set(String::new());
        if active_editor.get() == Some("load_types".into()) {
            active_editor.set(None);
        }
    };

    let clear_equipment_form = move |_| {
        equipment_id.set(String::new());
        equipment_name.set(String::new());
        if active_editor.get() == Some("equipments".into()) {
            active_editor.set(None);
        }
    };

    let clear_commodity_form = move |_| {
        commodity_type_id.set(String::new());
        commodity_type_name.set(String::new());
        if active_editor.get() == Some("commodity_types".into()) {
            active_editor.set(None);
        }
    };

    let clear_location_form = move |_| {
        location_id.set(String::new());
        location_name.set(String::new());
        location_country_id.set(String::new());
        location_city_id.set(String::new());
        if active_editor.get() == Some("locations".into()) {
            active_editor.set(None);
        }
    };

    let save_country = move |_| {
        if country_name.get().trim().is_empty() {
            action_message.set(Some("Enter a country name before saving.".into()));
            return;
        }

        pending_action.set(Some("country".into()));
        let auth = auth.clone();
        let request = CountryUpsertRequest {
            id: parse_optional_u64(&country_id.get()),
            name: country_name.get(),
            iso_code: {
                let trimmed = country_iso_code.get().trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            },
        };

        spawn_local(async move {
            handle_master_data_result(
                auth,
                api::upsert_country(&request).await,
                action_message,
                refresh_nonce,
                move || {
                    country_id.set(String::new());
                    country_name.set(String::new());
                    country_iso_code.set(String::new());
                },
            );
            pending_action.set(None);
        });
    };

    let save_city = move |_| {
        if city_name.get().trim().is_empty() {
            action_message.set(Some("Enter a city name before saving.".into()));
            return;
        }

        let Some(country_id_value) = parse_optional_u64(&city_country_id.get()) else {
            action_message.set(Some("Choose a country before saving this city.".into()));
            return;
        };

        pending_action.set(Some("city".into()));
        let auth = auth.clone();
        let request = CityUpsertRequest {
            id: parse_optional_u64(&city_id.get()),
            name: city_name.get(),
            country_id: country_id_value,
        };

        spawn_local(async move {
            handle_master_data_result(
                auth,
                api::upsert_city(&request).await,
                action_message,
                refresh_nonce,
                move || {
                    city_id.set(String::new());
                    city_name.set(String::new());
                    city_country_id.set(String::new());
                },
            );
            pending_action.set(None);
        });
    };

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
            handle_master_data_result(
                auth,
                api::upsert_load_type(&request).await,
                action_message,
                refresh_nonce,
                move || {
                    load_type_id.set(String::new());
                    load_type_name.set(String::new());
                },
            );
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
            handle_master_data_result(
                auth,
                api::upsert_equipment(&request).await,
                action_message,
                refresh_nonce,
                move || {
                    equipment_id.set(String::new());
                    equipment_name.set(String::new());
                },
            );
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
            handle_master_data_result(
                auth,
                api::upsert_commodity_type(&request).await,
                action_message,
                refresh_nonce,
                move || {
                    commodity_type_id.set(String::new());
                    commodity_type_name.set(String::new());
                },
            );
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
            handle_master_data_result(
                auth,
                api::upsert_location(&request).await,
                action_message,
                refresh_nonce,
                move || {
                    location_id.set(String::new());
                    location_name.set(String::new());
                    location_country_id.set(String::new());
                    location_city_id.set(String::new());
                },
            );
            pending_action.set(None);
        });
    };

    let run_delete = move |kind: String, row_id: u64| {
        let key = action_key(&kind, row_id);
        if armed_delete.get().as_deref() != Some(key.as_str()) {
            armed_delete.set(Some(key));
            action_message.set(Some(format!(
                "{} #{} is armed. Click {} again to confirm.",
                kind.replace('_', " "),
                row_id,
                delete_label(&kind).to_lowercase()
            )));
            return;
        }

        armed_delete.set(None);
        pending_action.set(Some(format!(
            "{} {}",
            delete_label(&kind).to_lowercase(),
            kind
        )));
        let auth = auth.clone();
        let payload = MasterDataDeleteRequest { id: row_id };

        spawn_local(async move {
            let result = match kind.as_str() {
                "countries" => api::delete_country(&payload).await,
                "cities" => api::delete_city(&payload).await,
                "load_types" => api::delete_load_type(&payload).await,
                "equipments" => api::delete_equipment(&payload).await,
                "commodity_types" => api::delete_commodity_type(&payload).await,
                "locations" => api::delete_location(&payload).await,
                _ => Err(format!("Delete flow is not supported for '{}'.", kind)),
            };

            handle_master_data_result(auth, result, action_message, refresh_nonce, || {});
            pending_action.set(None);
        });
    };

    let start_edit_row = move |kind: String, row: MasterDataRow| {
        active_editor.set(Some(kind.clone()));
        action_message.set(Some(format!(
            "Loaded {} #{} into the highlighted write form above. Review the populated fields, then click Save to finish the edit.",
            kind.replace('_', " "),
            row.id
        )));
        scroll_to_top_of_page();

        match kind.as_str() {
            "countries" => {
                country_id.set(row.id.to_string());
                country_name.set(row.primary_label);
                country_iso_code.set(row.secondary_label.unwrap_or_default());
            }
            "cities" => {
                city_id.set(row.id.to_string());
                city_name.set(row.primary_label);
                city_country_id.set(
                    row.country_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                );
            }
            "locations" => {
                location_id.set(row.id.to_string());
                location_name.set(row.primary_label);
                location_country_id.set(
                    row.country_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                );
                location_city_id.set(
                    row.city_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                );
            }
            "load_types" => {
                load_type_id.set(row.id.to_string());
                load_type_name.set(row.primary_label);
            }
            "equipments" => {
                equipment_id.set(row.id.to_string());
                equipment_name.set(row.primary_label);
            }
            "commodity_types" => {
                commodity_type_id.set(row.id.to_string());
                commodity_type_name.set(row.primary_label);
            }
            _ => {}
        }
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
                                <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "Admin write controls for the lookup data that keeps the Rust app moving.".into())}</p>
                            </div>
                            <div style="display:grid;gap:0.6rem;min-width:280px;">
                                <input
                                    type="text"
                                    placeholder="Search master data rows"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                    style="width:100%;padding:0.75rem 0.85rem;border:1px solid #d6d3d1;border-radius:0.9rem;"
                                />
                                <span style=move || tone_style(match pending_action.get() {
                                    Some(_) => "warning",
                                    None if armed_delete.get().is_some() => "danger",
                                    None => "success",
                                })>
                                    {move || {
                                        if let Some(value) = pending_action.get() {
                                            format!("Working on {}...", value)
                                        } else if let Some(value) = armed_delete.get() {
                                            format!("Confirm {}", value.replace(':', " #"))
                                        } else {
                                            "Write controls ready".into()
                                        }
                                    }}
                                </span>
                            </div>
                        </section>

                        {move || action_message.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">{message}</section>
                        })}
                        {move || error_message.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">{message}</section>
                        })}
                        {move || active_editor.get().map(|kind| view! {
                            <section style="padding:0.9rem 1rem;border:1px solid #bfdbfe;border-radius:0.95rem;background:#f8fbff;color:#1d4ed8;display:grid;gap:0.25rem;">
                                <strong>{format!("Editing {}", kind.replace('_', " "))}</strong>
                                <small>"The matching write panel is highlighted below so the loaded values are easier to spot."</small>
                            </section>
                        })}

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;align-items:start;">
                            {render_country_panel(country_id, country_name, country_iso_code, save_country, clear_country_form, Signal::derive(move || active_editor.get().as_deref() == Some("countries")))}
                            {render_city_panel(screen, city_id, city_name, city_country_id, save_city, clear_city_form, Signal::derive(move || active_editor.get().as_deref() == Some("cities")))}
                            {render_simple_panel("load_types", "Load type write flow", "Dry Van", load_type_id, load_type_name, save_load_type, clear_load_type_form, "Save load type", Signal::derive(move || active_editor.get().as_deref() == Some("load_types")))}
                            {render_simple_panel("equipments", "Equipment write flow", "Reefer", equipment_id, equipment_name, save_equipment, clear_equipment_form, "Save equipment", Signal::derive(move || active_editor.get().as_deref() == Some("equipments")))}
                            {render_simple_panel("commodity_types", "Commodity type write flow", "Consumer Goods", commodity_type_id, commodity_type_name, save_commodity_type, clear_commodity_form, "Save commodity type", Signal::derive(move || active_editor.get().as_deref() == Some("commodity_types")))}
                            {render_location_panel(
                                screen,
                                location_id,
                                location_name,
                                location_country_id,
                                location_city_id,
                                save_location,
                                clear_location_form,
                                Signal::derive(move || active_editor.get().as_deref() == Some("locations")),
                            )}
                        </section>

                        <section style="display:grid;gap:1rem;">
                            {move || screen.get().map(|data| {
                                data.sections.into_iter().map(|section| {
                                    let section_key = section.key.clone();
                                    let query = search_query.get();
                                    let rows = section
                                        .rows
                                        .clone()
                                        .into_iter()
                                        .filter(|row| master_data_row_matches_query(&section_key, row, &query))
                                        .collect::<Vec<_>>();
                                    let total = section.total;
                                    let empty_message = section.empty_message.clone();
                                    let label = section.label.clone();
                                    let admin_route = section.admin_route.clone();
                                    let status_tone = if supports_write(&section_key) { "success" } else { "warning" };

                                    view! {
                                        <section style="border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;padding:1rem;display:grid;gap:0.8rem;">
                                            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                                <div>
                                                    <strong>{label}</strong>
                                                    <div><small style="color:#64748b;">{admin_route}</small></div>
                                                </div>
                                                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;align-items:center;">
                                                    <span style="padding:0.2rem 0.55rem;border-radius:999px;background:#f1f5f9;color:#334155;">{format!("{} rows", total)}</span>
                                                    <span style=tone_style(status_tone)>{writable_status_label(&section_key)}</span>
                                                </div>
                                            </div>

                                            {if rows.is_empty() {
                                                view! { <p style="margin:0;">{empty_message}</p> }.into_any()
                                            } else {
                                                view! {
                                                    <div style="overflow:auto;">
                                                        <table style="width:100%;border-collapse:collapse;min-width:900px;">
                                                            <thead style="background:#f8fafc;">
                                                                <tr>
                                                                    <th style="text-align:left;padding:0.75rem;">"ID"</th>
                                                                    <th style="text-align:left;padding:0.75rem;">"Name"</th>
                                                                    <th style="text-align:left;padding:0.75rem;">"Context"</th>
                                                                    <th style="text-align:left;padding:0.75rem;">"Status"</th>
                                                                    <th style="text-align:left;padding:0.75rem;">"Detail"</th>
                                                                    <th style="text-align:left;padding:0.75rem;">"Actions"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                {rows.into_iter().map(|row| {
                                                                    let row_for_edit = row.clone();
                                                                    let section_key_for_edit = section_key.clone();
                                                                    let section_key_for_delete = section_key.clone();
                                                                    let row_id = row.id;
                                                                    let row_key = action_key(&section_key, row_id);
                                                                    let row_key_for_style = row_key.clone();
                                                                    let row_key_for_label = row_key.clone();
                                                                    let delete_text = delete_label(&section_key).to_string();

                                                                    view! {
                                                                        <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
                                                                            <td style="padding:0.75rem;">{row.id}</td>
                                                                            <td style="padding:0.75rem;"><strong>{row.primary_label.clone()}</strong></td>
                                                                            <td style="padding:0.75rem;">{row.secondary_label.clone().unwrap_or_else(|| "-".into())}</td>
                                                                            <td style="padding:0.75rem;">{row.status_label.clone()}</td>
                                                                            <td style="padding:0.75rem;">{row.detail.clone()}</td>
                                                                            <td style="padding:0.75rem;">
                                                                                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                                                                    {if row.editable && supports_write(&section_key_for_edit) {
                                                                                        view! {
                                                                                            <button type="button" on:click=move |_| start_edit_row(section_key_for_edit.clone(), row_for_edit.clone()) style="padding:0.45rem 0.8rem;border:1px solid #d1d5db;border-radius:0.8rem;background:white;cursor:pointer;">"Edit"</button>
                                                                                        }.into_any()
                                                                                    } else {
                                                                                        view! { <span style="color:#94a3b8;">"-"</span> }.into_any()
                                                                                    }}
                                                                                    {if supports_delete(&section_key_for_delete) {
                                                                                        view! {
                                                                                            <button
                                                                                                type="button"
                                                                                                on:click=move |_| run_delete(section_key_for_delete.clone(), row_id)
                                                                                                style=move || {
                                                                                                    if armed_delete.get().as_deref() == Some(row_key_for_style.as_str()) {
                                                                                                        "padding:0.45rem 0.8rem;border:1px solid #fecaca;border-radius:0.8rem;background:#fff1f2;color:#be123c;cursor:pointer;".to_string()
                                                                                                    } else {
                                                                                                        "padding:0.45rem 0.8rem;border:1px solid #d1d5db;border-radius:0.8rem;background:white;cursor:pointer;".to_string()
                                                                                                    }
                                                                                                }
                                                                                            >
                                                                                                {move || {
                                                                                                    if armed_delete.get().as_deref() == Some(row_key_for_label.as_str()) {
                                                                                                        format!("Confirm {}", delete_text)
                                                                                                    } else {
                                                                                                        delete_text.clone()
                                                                                                    }
                                                                                                }}
                                                                                            </button>
                                                                                        }.into_any()
                                                                                    } else {
                                                                                        view! { <span style="color:#94a3b8;">"Protected"</span> }.into_any()
                                                                                    }}
                                                                                </div>
                                                                            </td>
                                                                        </tr>
                                                                    }
                                                                }).collect_view()}
                                                            </tbody>
                                                        </table>
                                                    </div>
                                                }.into_any()
                                            }}
                                        </section>
                                    }
                                }).collect_view()
                            })}
                        </section>
                    </article>
                }.into_any()
            }
        }}
    }
}

fn master_data_row_matches_query(kind: &str, row: &MasterDataRow, query: &str) -> bool {
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

fn handle_master_data_result(
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

fn render_clear_button(
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

fn render_simple_panel(
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

fn render_country_panel(
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

fn render_city_panel(
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

fn render_location_panel(
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
