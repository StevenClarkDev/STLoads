use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use super::{loads_helpers::render_row, shared::tone_style};
use crate::{
    api, realtime,
    session::{self, use_auth},
};
use shared::{
    BookLoadLegRequest, BulkLoadImportCommitRequest, BulkLoadImportPreviewRequest,
    BulkLoadImportResponse, CarrierMatchScreen, CarrierNetworkScreen, LoadBoardFilters,
    LoadBoardScreen, RealtimeEventKind, RealtimeTopic, UpsertCarrierNetworkRequest,
};

#[component]
pub fn LoadBoardPage() -> impl IntoView {
    let auth = use_auth();
    let selected_tab = RwSignal::new("all".to_string());
    let filters = RwSignal::new(LoadBoardFilters {
        page: 1,
        per_page: 20,
        ..Default::default()
    });
    let screen = RwSignal::new(None::<LoadBoardScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let pending_leg_id = RwSignal::new(None::<u64>);
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_connected = RwSignal::new(false);
    let ws_handle = RwSignal::new(None::<AbortHandle>);
    let bulk_csv = RwSignal::new(String::new());
    let bulk_filename = RwSignal::new("load-import.csv".to_string());
    let bulk_import_result = RwSignal::new(None::<BulkLoadImportResponse>);
    let bulk_import_loading = RwSignal::new(false);
    let carrier_network = RwSignal::new(None::<CarrierNetworkScreen>);
    let carrier_network_loading = RwSignal::new(false);
    let carrier_matches = RwSignal::new(None::<CarrierMatchScreen>);
    let carrier_matches_loading = RwSignal::new(false);
    let matching_leg_id = RwSignal::new(None::<u64>);
    let network_carrier_user_id = RwSignal::new(String::new());
    let network_relationship_status = RwSignal::new("preferred".to_string());
    let network_group_key = RwSignal::new(String::new());
    let network_notes = RwSignal::new(String::new());
    let network_effective_to = RwSignal::new(String::new());

    Effect::new(move |_| {
        let tab = selected_tab.get();
        let current_filters = filters.get();
        let _refresh = refresh_nonce.get();
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();

        if !ready {
            return;
        }

        if !current_session.authenticated {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some("Sign in to view the Rust load board.".into()));
            return;
        }

        is_loading.set(true);
        let auth = auth;

        spawn_local(async move {
            match api::fetch_load_board_screen(&tab, &current_filters).await {
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

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();

        if !ready || !current_session.authenticated {
            carrier_network.set(None);
            return;
        }

        carrier_network_loading.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::fetch_carrier_network_screen().await {
                Ok(next) => {
                    carrier_network.set(Some(next));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                }
            }
            carrier_network_loading.set(false);
        });
    });

    Effect::new(move |_| {
        let current_session = auth.session.get();
        if !auth.session_ready.get() || !current_session.authenticated {
            if let Some(existing_handle) = ws_handle.get_untracked() {
                existing_handle.abort();
                ws_handle.set(None);
            }
            ws_connected.set(false);
            return;
        }

        let current_user_id = current_session.user.as_ref().map(|user| user.id);
        let auth = auth;

        if let Some(existing_handle) = ws_handle.get_untracked() {
            existing_handle.abort();
        }

        let handle = realtime::connect_realtime_listener(
            None,
            vec![RealtimeTopic::LoadBoard],
            move |event| match event.kind {
                RealtimeEventKind::LoadLegBooked
                | RealtimeEventKind::LegExecutionUpdated
                | RealtimeEventKind::OfferReviewed
                | RealtimeEventKind::PaymentsOperationsUpdated
                | RealtimeEventKind::TmsOperationsUpdated => {
                    refresh_nonce.update(|value| *value += 1);
                    action_message.set(Some(format!("Realtime update: {}", event.summary)));
                }
                RealtimeEventKind::SessionInvalidated => {
                    if event.actor_user_id == current_user_id {
                        if let Some(existing_handle) = ws_handle.get_untracked() {
                            existing_handle.abort();
                            ws_handle.set(None);
                        }
                        session::invalidate_session(
                            &auth,
                            "The current Rust session was invalidated; sign in again.",
                        );
                        ws_connected.set(false);
                    }
                }
                RealtimeEventKind::MessageSent
                | RealtimeEventKind::ConversationRead
                | RealtimeEventKind::ConversationPresenceChanged => {}
                _ => {}
            },
        );

        ws_connected.set(handle.is_some());
        ws_handle.set(handle);
    });

    let book_leg = move |leg_id: u64| {
        let current_session = auth.session.get();
        let can_self_book = current_session
            .user
            .as_ref()
            .map(|user| user.role_key == "carrier")
            .unwrap_or(false);

        if !can_self_book {
            action_message.set(Some(
                "Only authenticated carrier accounts can self-book a leg in this Rust load board slice.".into(),
            ));
            return;
        }

        pending_leg_id.set(Some(leg_id));
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            let response = api::book_load_leg(
                leg_id,
                &BookLoadLegRequest {
                    booked_amount: None,
                    idempotency_key: Some(format!("load-board-carrier-booking-{}", leg_id)),
                },
            )
            .await;

            match response {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
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

            pending_leg_id.set(None);
        });
    };

    let run_bulk_import = move |commit: bool| {
        let csv = bulk_csv.get_untracked();
        if csv.trim().is_empty() {
            action_message.set(Some("Paste CSV rows before running a bulk import.".into()));
            return;
        }

        bulk_import_loading.set(true);
        bulk_import_result.set(None);
        action_message.set(None);
        let auth = auth;
        let filename = Some(bulk_filename.get_untracked());

        spawn_local(async move {
            let response = if commit {
                api::commit_bulk_load_import(&BulkLoadImportCommitRequest {
                    csv,
                    filename,
                    idempotency_key: None,
                })
                .await
            } else {
                api::preview_bulk_load_import(&BulkLoadImportPreviewRequest { csv, filename }).await
            };

            match response {
                Ok(result) => {
                    action_message.set(Some(result.message.clone()));
                    if commit && result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                    bulk_import_result.set(Some(result));
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

            bulk_import_loading.set(false);
        });
    };

    let save_carrier_network = move || {
        let Some(carrier_user_id) = network_carrier_user_id.get().parse::<u64>().ok() else {
            action_message.set(Some(
                "Choose a carrier before saving the private network row.".into(),
            ));
            return;
        };
        let payload = UpsertCarrierNetworkRequest {
            id: None,
            carrier_user_id,
            relationship_status: network_relationship_status.get(),
            carrier_group_key: (!network_group_key.get().trim().is_empty())
                .then(|| network_group_key.get()),
            notes: (!network_notes.get().trim().is_empty()).then(|| network_notes.get()),
            effective_to: (!network_effective_to.get().trim().is_empty())
                .then(|| network_effective_to.get()),
        };

        carrier_network_loading.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::upsert_carrier_network(&payload).await {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    carrier_network.set(Some(response.screen));
                    if response.success {
                        network_carrier_user_id.set(String::new());
                        network_group_key.set(String::new());
                        network_notes.set(String::new());
                        network_effective_to.set(String::new());
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
            carrier_network_loading.set(false);
        });
    };

    let open_carrier_matches = move |leg_id: u64| {
        carrier_matches_loading.set(true);
        matching_leg_id.set(Some(leg_id));
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::fetch_carrier_matches(leg_id).await {
                Ok(screen) => {
                    carrier_matches.set(Some(screen));
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
            carrier_matches_loading.set(false);
        });
    };

    let can_self_book = Signal::derive(move || {
        auth.session
            .get()
            .user
            .map(|user| user.role_key == "carrier")
            .unwrap_or(false)
    });

    let can_view_profile = Signal::derive(move || {
        session::has_permission(&auth, "manage_loads")
            || session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_dispatch_desk")
    });

    let apply_text_filter = move |setter: fn(&mut LoadBoardFilters, String), value: String| {
        filters.update(|current| {
            setter(current, value);
            current.page = 1;
        });
    };

    view! {
        <article style="display:grid;gap:1.25rem;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div>
                    <p style=tone_style("info")>{move || screen.get().map(|value| value.role_label).unwrap_or_else(|| "Marketplace Workspace".into())}</p>
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Manage Loads".into())}</h2>
                    <p>
                        "Live load board."
                    </p>
                </div>
                {move || screen.get().and_then(|value| {
                    value.primary_action_label.zip(value.primary_action_href).map(|(label, href)| {
                        view! {
                            <a href=href style="padding:0.7rem 1rem;border-radius:0.9rem;background:#111827;color:white;text-decoration:none;">
                                {label}
                            </a>
                        }
                    }).map(IntoView::into_view)
                })}
            </section>

            {move || error_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">
                    {message}
                </section>
            })}

            {move || action_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #bfdbfe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                    {message}
                </section>
            })}

            {move || carrier_matches.get().map(|matches| {
                let row_count = matches.rows.len();
                let rows_empty = matches.rows.is_empty();
                let load_label = matches.load_label;
                let leg_id = matches.leg_id;
                let rows = matches.rows;
                let notes = matches.notes;
                view! {
                    <section style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #dbeafe;border-radius:1rem;background:#f8fbff;">
                        <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <strong>{format!("Carrier matches for {}", load_label)}</strong>
                                <p style="margin:0.25rem 0 0;color:#64748b;">{format!("Leg {} | {} ranked carrier candidates", leg_id, row_count)}</p>
                            </div>
                            <span style=tone_style(if carrier_matches_loading.get() { "warning" } else { "info" })>
                                {move || if carrier_matches_loading.get() { "Ranking" } else { "Ranked" }}
                            </span>
                        </div>
                        <div style="display:grid;gap:0.65rem;">
                            {rows.into_iter().take(8).map(|row| {
                                let tone = if row.eligible { "success" } else { "danger" };
                                let status = row.relationship_status.unwrap_or_else(|| "network-neutral".into());
                                view! {
                                    <div style="display:grid;gap:0.4rem;padding:0.8rem;border:1px solid #e5e7eb;border-radius:0.85rem;background:white;">
                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                                            <strong>{row.carrier_label}</strong>
                                            <div style="display:flex;gap:0.4rem;align-items:center;flex-wrap:wrap;">
                                                <span style=tone_style(tone)>{if row.eligible { "Eligible" } else { "Blocked" }}</span>
                                                <span style=tone_style("secondary")>{status}</span>
                                                <span style=tone_style("primary")>{format!("Score {}", row.score)}</span>
                                            </div>
                                        </div>
                                        <p style="margin:0;color:#475569;">{row.explanation.join(" | ")}</p>
                                        {(!row.blocked_reasons.is_empty()).then(|| view! {
                                            <p style="margin:0;color:#be123c;">{row.blocked_reasons.join(" | ")}</p>
                                        })}
                                    </div>
                                }
                            }).collect_view()}
                            {rows_empty.then(|| view! {
                                <p style="margin:0;color:#64748b;">"No carrier candidates were available for this load owner scope."</p>
                            })}
                            {notes.into_iter().map(|note| view! { <small style="color:#64748b;">{note}</small> }).collect_view()}
                        </div>
                    </section>
                }
            })}

            <section style="display:flex;gap:0.75rem;flex-wrap:wrap;">
                {move || screen.get().map(|value| {
                    value.tabs
                        .into_iter()
                        .map(|tab| {
                            let tab_key = tab.key.clone();
                            let style = if tab.is_active {
                                "padding:0.5rem 0.85rem;border-radius:999px;background:#111827;color:white;border:none;cursor:pointer;"
                            } else {
                                "padding:0.5rem 0.85rem;border-radius:999px;background:#f4f4f5;color:#111827;border:none;cursor:pointer;"
                            };
                            view! {
                                <button
                                    type="button"
                                    style=style
                                    on:click=move |_| {
                                        selected_tab.set(tab_key.clone());
                                        action_message.set(None);
                                    }
                                >
                                    {format!("{} ({})", tab.label, tab.count)}
                                </button>
                            }
                        })
                        .collect_view()
                })}
            </section>

            <section style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;">
                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                    <strong>"Search and saved views"</strong>
                    <button
                        type="button"
                        style="padding:0.5rem 0.8rem;border:1px solid #d1d5db;border-radius:0.75rem;background:#f8fafc;color:#111827;cursor:pointer;"
                        on:click=move |_| {
                            filters.set(LoadBoardFilters { page: 1, per_page: 20, ..Default::default() });
                            action_message.set(None);
                        }
                    >
                        "Reset"
                    </button>
                </div>
                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                    {move || screen.get().map(|value| value.saved_filters.into_iter().map(|saved| {
                        let saved_filters = saved.filters.clone();
                        let saved_name = saved.name.clone();
                        let button_label = if saved.is_default {
                            format!("{} default", saved.name)
                        } else {
                            saved.name
                        };
                        view! {
                            <button
                                type="button"
                                style="padding:0.45rem 0.7rem;border:1px solid #d1d5db;border-radius:999px;background:#f8fafc;color:#111827;cursor:pointer;"
                                on:click=move |_| {
                                    filters.set(LoadBoardFilters { page: 1, ..saved_filters.clone() });
                                    action_message.set(Some(format!("Applied saved view: {}", saved_name)));
                                }
                            >
                                {button_label}
                            </button>
                        }
                    }).collect_view())}
                </div>
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.75rem;align-items:end;">
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Origin"</span>
                        <input prop:value=move || filters.get().origin.unwrap_or_default() on:input=move |ev| apply_text_filter(|current, value| current.origin = (!value.trim().is_empty()).then_some(value), event_target_value(&ev)) placeholder="Dallas" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Destination"</span>
                        <input prop:value=move || filters.get().destination.unwrap_or_default() on:input=move |ev| apply_text_filter(|current, value| current.destination = (!value.trim().is_empty()).then_some(value), event_target_value(&ev)) placeholder="Joliet" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Pickup date"</span>
                        <input type="date" prop:value=move || filters.get().pickup_date.unwrap_or_default() on:input=move |ev| apply_text_filter(|current, value| current.pickup_date = (!value.trim().is_empty()).then_some(value), event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Delivery date"</span>
                        <input type="date" prop:value=move || filters.get().delivery_date.unwrap_or_default() on:input=move |ev| apply_text_filter(|current, value| current.delivery_date = (!value.trim().is_empty()).then_some(value), event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Equipment"</span>
                        <select prop:value=move || filters.get().equipment_id.map(|value| value.to_string()).unwrap_or_default() on:change=move |ev| {
                            let value = event_target_value(&ev);
                            filters.update(|current| {
                                current.equipment_id = value.parse::<u64>().ok();
                                current.page = 1;
                            });
                        } style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                            <option value="">"Any equipment"</option>
                            {move || screen.get().map(|value| value.equipment_options.into_iter().map(|option| view! { <option value={option.id.to_string()}>{option.label}</option> }).collect_view())}
                        </select>
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Commodity"</span>
                        <select prop:value=move || filters.get().commodity_type_id.map(|value| value.to_string()).unwrap_or_default() on:change=move |ev| {
                            let value = event_target_value(&ev);
                            filters.update(|current| {
                                current.commodity_type_id = value.parse::<u64>().ok();
                                current.page = 1;
                            });
                        } style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                            <option value="">"Any commodity"</option>
                            {move || screen.get().map(|value| value.commodity_options.into_iter().map(|option| view! { <option value={option.id.to_string()}>{option.label}</option> }).collect_view())}
                        </select>
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Min rate"</span>
                        <input type="number" step="0.01" prop:value=move || filters.get().min_rate.map(|value| value.to_string()).unwrap_or_default() on:input=move |ev| {
                            let value = event_target_value(&ev);
                            filters.update(|current| {
                                current.min_rate = value.parse::<f64>().ok();
                                current.page = 1;
                            });
                        } placeholder="1000" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Max rate"</span>
                        <input type="number" step="0.01" prop:value=move || filters.get().max_rate.map(|value| value.to_string()).unwrap_or_default() on:input=move |ev| {
                            let value = event_target_value(&ev);
                            filters.update(|current| {
                                current.max_rate = value.parse::<f64>().ok();
                                current.page = 1;
                            });
                        } placeholder="5000" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Customer/ref"</span>
                        <input prop:value=move || filters.get().customer.unwrap_or_default() on:input=move |ev| apply_text_filter(|current, value| current.customer = (!value.trim().is_empty()).then_some(value), event_target_value(&ev)) placeholder="PO or customer ref" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Visibility"</span>
                        <select prop:value=move || filters.get().visibility.unwrap_or_default() on:change=move |ev| apply_text_filter(|current, value| current.visibility = (!value.trim().is_empty()).then_some(value), event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                            <option value="">"Any visibility"</option>
                            <option value="public">"Public"</option>
                            <option value="private">"Private"</option>
                            <option value="contract">"Contract"</option>
                            <option value="internal">"Internal"</option>
                        </select>
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Compliance"</span>
                        <select prop:value=move || filters.get().compliance.unwrap_or_default() on:change=move |ev| apply_text_filter(|current, value| current.compliance = (!value.trim().is_empty()).then_some(value), event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                            <option value="">"Any compliance"</option>
                            <option value="hazmat">"Hazmat"</option>
                            <option value="temperature_controlled">"Temperature controlled"</option>
                            <option value="documents_required">"Docs required"</option>
                        </select>
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Rows"</span>
                        <select prop:value=move || filters.get().per_page.to_string() on:change=move |ev| {
                            let value = event_target_value(&ev).parse::<u64>().unwrap_or(20);
                            filters.update(|current| {
                                current.per_page = value.clamp(10, 100);
                                current.page = 1;
                            });
                        } style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                            <option value="10">"10"</option>
                            <option value="20">"20"</option>
                            <option value="50">"50"</option>
                            <option value="100">"100"</option>
                        </select>
                    </label>
                </div>
            </section>

            {move || carrier_network.get().and_then(|network| network.can_manage.then_some(network)).map(|network| {
                view! {
                    <section style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;">
                        <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <strong>"Private carrier network"</strong>
                                <p style="margin:0.25rem 0 0;color:#64748b;">{format!("Managed by {}", network.owner_label)}</p>
                            </div>
                            <span style=tone_style(if carrier_network_loading.get() { "warning" } else { "success" })>
                                {move || if carrier_network_loading.get() { "Saving" } else { "Ready" }}
                            </span>
                        </div>
                        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.75rem;align-items:end;">
                            <label style="display:grid;gap:0.3rem;">
                                <span>"Carrier"</span>
                                <select prop:value=move || network_carrier_user_id.get() on:change=move |ev| network_carrier_user_id.set(event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                                    <option value="">"Choose carrier"</option>
                                    {network.carrier_options.clone().into_iter().map(|option| view! {
                                        <option value={option.user_id.to_string()}>{option.label}</option>
                                    }).collect_view()}
                                </select>
                            </label>
                            <label style="display:grid;gap:0.3rem;">
                                <span>"Status"</span>
                                <select prop:value=move || network_relationship_status.get() on:change=move |ev| network_relationship_status.set(event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;background:white;">
                                    <option value="preferred">"Preferred"</option>
                                    <option value="approved">"Approved"</option>
                                    <option value="backup">"Backup"</option>
                                    <option value="blocked">"Blocked"</option>
                                </select>
                            </label>
                            <label style="display:grid;gap:0.3rem;">
                                <span>"Carrier group"</span>
                                <input prop:value=move || network_group_key.get() on:input=move |ev| network_group_key.set(event_target_value(&ev)) placeholder="primary, reefer, hazmat" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                            </label>
                            <label style="display:grid;gap:0.3rem;">
                                <span>"Expires"</span>
                                <input type="date" prop:value=move || network_effective_to.get() on:input=move |ev| network_effective_to.set(event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                            </label>
                            <label style="display:grid;gap:0.3rem;">
                                <span>"Notes"</span>
                                <input prop:value=move || network_notes.get() on:input=move |ev| network_notes.set(event_target_value(&ev)) placeholder="Routing guide, block reason, approval note" style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                            </label>
                            <button type="button" disabled=move || carrier_network_loading.get() on:click=move |_| save_carrier_network() style="padding:0.65rem 0.9rem;border:1px solid #111827;border-radius:0.75rem;background:#111827;color:white;cursor:pointer;">
                                "Save network row"
                            </button>
                        </div>
                        <div style="overflow:auto;">
                            <table style="width:100%;border-collapse:collapse;min-width:780px;">
                                <thead style="background:#f8fafc;">
                                    <tr>
                                        <th style="text-align:left;padding:0.65rem;">"Carrier"</th>
                                        <th style="text-align:left;padding:0.65rem;">"Status"</th>
                                        <th style="text-align:left;padding:0.65rem;">"Group"</th>
                                        <th style="text-align:left;padding:0.65rem;">"Effective"</th>
                                        <th style="text-align:left;padding:0.65rem;">"Notes"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {if network.rows.is_empty() {
                                        view! { <tr><td colspan="5" style="padding:0.75rem;color:#64748b;">"No private network relationships are configured yet."</td></tr> }.into_any()
                                    } else {
                                        network.rows.into_iter().map(|row| {
                                            let tone = if row.relationship_status == "blocked" { "danger" } else if row.relationship_status == "preferred" { "success" } else { "info" };
                                            view! {
                                                <tr style="border-top:1px solid #f1f5f9;">
                                                    <td style="padding:0.65rem;">{row.carrier_label}</td>
                                                    <td style="padding:0.65rem;"><span style=tone_style(tone)>{row.relationship_status}</span></td>
                                                    <td style="padding:0.65rem;">{row.carrier_group_key.unwrap_or_else(|| "-".into())}</td>
                                                    <td style="padding:0.65rem;">{format!("{} to {}", row.effective_from, row.effective_to.unwrap_or_else(|| "open".into()))}</td>
                                                    <td style="padding:0.65rem;">{row.notes.unwrap_or_else(|| "-".into())}</td>
                                                </tr>
                                            }
                                        }).collect_view().into_any()
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </section>
                }
            })}

            <section style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;">
                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                    <strong>"Bulk load import"</strong>
                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                        <button
                            type="button"
                            disabled=move || bulk_import_loading.get()
                            on:click=move |_| run_bulk_import(false)
                            style="padding:0.5rem 0.8rem;border:1px solid #d1d5db;border-radius:0.75rem;background:#f8fafc;color:#111827;cursor:pointer;"
                        >
                            {move || if bulk_import_loading.get() { "Working..." } else { "Preview" }}
                        </button>
                        <button
                            type="button"
                            disabled=move || bulk_import_loading.get()
                            on:click=move |_| run_bulk_import(true)
                            style="padding:0.5rem 0.8rem;border:1px solid #111827;border-radius:0.75rem;background:#111827;color:white;cursor:pointer;"
                        >
                            "Commit"
                        </button>
                    </div>
                </div>
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.75rem;align-items:start;">
                    <label style="display:grid;gap:0.3rem;">
                        <span>"Filename"</span>
                        <input prop:value=move || bulk_filename.get() on:input=move |ev| bulk_filename.set(event_target_value(&ev)) style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;" />
                    </label>
                    <label style="display:grid;gap:0.3rem;">
                        <span>"CSV rows"</span>
                        <textarea
                            rows="6"
                            prop:value=move || bulk_csv.get()
                            on:input=move |ev| bulk_csv.set(event_target_value(&ev))
                            placeholder="title,load_type_id,equipment_id,commodity_type_id,weight,weight_unit,pickup_address,pickup_city,pickup_country,delivery_address,delivery_city,delivery_country,pickup_date,delivery_date,price"
                            style="padding:0.65rem 0.75rem;border:1px solid #d1d5db;border-radius:0.75rem;font-family:monospace;"
                        ></textarea>
                    </label>
                </div>
                {move || bulk_import_result.get().map(|result| {
                    view! {
                        <div style="display:grid;gap:0.6rem;">
                            <p style="margin:0;">
                                {format!(
                                    "{} total | {} valid | {} invalid | {} created",
                                    result.total_rows, result.valid_rows, result.invalid_rows, result.created_load_count
                                )}
                            </p>
                            {result.error_csv.map(|csv| view! {
                                <textarea rows="4" readonly prop:value=csv style="padding:0.65rem 0.75rem;border:1px solid #fecaca;border-radius:0.75rem;font-family:monospace;color:#991b1b;"></textarea>
                            })}
                        </div>
                    }
                })}
            </section>

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;">
                {move || screen.get().map(|value| {
                    value.metrics
                        .into_iter()
                        .map(|metric| {
                            view! {
                                <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                                    <strong>{metric.label}</strong>
                                    <div style="font-size:1.2rem;margin:0.4rem 0;">{metric.value}</div>
                                    <p style="margin:0;">{metric.note}</p>
                                </div>
                            }
                        })
                        .collect_view()
                })}
            </section>

            <section style="overflow:auto;border:1px solid #e5e7eb;border-radius:1rem;">
                <table style="width:100%;border-collapse:collapse;min-width:1080px;">
                    <thead style="background:#f8fafc;">
                        <tr>
                            <th style="text-align:left;padding:0.9rem;">"Load ID"</th>
                            <th style="text-align:left;padding:0.9rem;">"Origin"</th>
                            <th style="text-align:left;padding:0.9rem;">"Destination"</th>
                            <th style="text-align:left;padding:0.9rem;">"Pickup"</th>
                            <th style="text-align:left;padding:0.9rem;">"Delivery"</th>
                            <th style="text-align:left;padding:0.9rem;">"Status"</th>
                            <th style="text-align:left;padding:0.9rem;">"Board"</th>
                            <th style="text-align:left;padding:0.9rem;">"Bid"</th>
                            <th style="text-align:left;padding:0.9rem;">"Amount"</th>
                            <th style="text-align:left;padding:0.9rem;">"Payment"</th>
                            <th style="text-align:left;padding:0.9rem;">"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            if is_loading.get() && screen.get().is_none() {
                                view! {
                                    <tr>
                                        <td colspan="11" style="padding:1rem;">"Loading auth-scoped load board data from the Rust backend..."</td>
                                    </tr>
                                }.into_any()
                            } else {
                                screen
                                    .get()
                                    .map(|value| {
                                        value.rows
                                            .into_iter()
                                            .map(|row| render_row(
                                                row,
                                                pending_leg_id,
                                                book_leg,
                                                open_carrier_matches,
                                                matching_leg_id,
                                                can_self_book.get(),
                                                can_view_profile.get(),
                                            ))
                                            .collect_view()
                                            .into_any()
                                    })
                                    .unwrap_or_else(|| view! {
                                        <tr>
                                            <td colspan="11" style="padding:1rem;">"No load board data is available for this authenticated scope yet."</td>
                                        </tr>
                                    }.into_any())
                            }
                        }}
                    </tbody>
                </table>
            </section>

            <section style="display:grid;gap:0.35rem;">
                {move || screen.get().map(|value| {
                    view! {
                        <>
                            {value
                                .recommendation_notes
                                .into_iter()
                                .map(|note| view! { <p style="margin:0;">{note}</p> })
                                .collect_view()}
                            <div style="display:flex;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                                <button
                                    type="button"
                                    disabled=value.pagination.page <= 1
                                    on:click=move |_| filters.update(|current| current.page = current.page.saturating_sub(1).max(1))
                                    style="padding:0.45rem 0.75rem;border:1px solid #d1d5db;border-radius:0.7rem;background:#f8fafc;color:#111827;cursor:pointer;"
                                >
                                    "Previous"
                                </button>
                                <small>{format!("Page {} | {} total matching rows", value.pagination.page, value.pagination.total)}</small>
                                <button
                                    type="button"
                                    disabled=value.pagination.page * value.pagination.per_page >= value.pagination.total
                                    on:click=move |_| filters.update(|current| current.page += 1)
                                    style="padding:0.45rem 0.75rem;border:1px solid #d1d5db;border-radius:0.7rem;background:#f8fafc;color:#111827;cursor:pointer;"
                                >
                                    "Next"
                                </button>
                            </div>
                        </>
                    }
                })}
            </section>
        </article>
    }
}
