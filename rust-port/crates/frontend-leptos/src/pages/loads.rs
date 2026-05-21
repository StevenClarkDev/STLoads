use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;

use crate::{
    api, realtime,
    session::{self, use_auth},
};
use shared::{
    BookLoadLegRequest, LoadBoardFilterState, LoadBoardRow, LoadBoardScreen, RealtimeEventKind,
    RealtimeTopic, SaveLoadBoardSearchRequest, SubmitOfferRequest,
};

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.2rem 0.55rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.2rem 0.55rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#ffe4e6;padding:0.2rem 0.55rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.2rem 0.55rem;border-radius:999px;color:#0369a1;",
        "primary" => "background:#ede9fe;padding:0.2rem 0.55rem;border-radius:999px;color:#6d28d9;",
        "secondary" => {
            "background:#f1f5f9;padding:0.2rem 0.55rem;border-radius:999px;color:#475569;"
        }
        _ => "background:#e5e7eb;padding:0.2rem 0.55rem;border-radius:999px;color:#111827;",
    }
}

#[component]
pub fn LoadBoardPage() -> impl IntoView {
    let auth = use_auth();
    let selected_tab = RwSignal::new("all".to_string());
    let filters = RwSignal::new(LoadBoardFilterState::default());
    let draft_filters = RwSignal::new(LoadBoardFilterState::default());
    let saved_search_name = RwSignal::new(String::new());
    let saved_search_alert = RwSignal::new(false);
    let screen = RwSignal::new(None::<LoadBoardScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let pending_leg_id = RwSignal::new(None::<u64>);
    let pending_posting_id = RwSignal::new(None::<u64>);
    let selected_row = RwSignal::new(None::<LoadBoardRow>);
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_connected = RwSignal::new(false);
    let ws_handle = RwSignal::new(None::<AbortHandle>);

    Effect::new(move |_| {
        let tab = selected_tab.get();
        let active_filters = filters.get();
        let _refresh = refresh_nonce.get();
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();

        if !ready {
            return;
        }

        if !current_session.authenticated {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some("Sign in to view marketplace loads.".into()));
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_load_board_screen(&tab, &active_filters).await {
                Ok(next_screen) => {
                    error_message.set(None);
                    screen.set(Some(next_screen));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(&auth, "Your session expired; sign in again.");
                    }
                    error_message.set(Some(error));
                }
            }

            is_loading.set(false);
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
        let auth = auth.clone();

        if let Some(existing_handle) = ws_handle.get_untracked() {
            existing_handle.abort();
        }

        let handle = realtime::connect_realtime_listener(
            None,
            vec![RealtimeTopic::LoadBoard],
            move |event| match event.kind {
                RealtimeEventKind::LoadLegBooked
                | RealtimeEventKind::BookingAwarded
                | RealtimeEventKind::LegExecutionUpdated
                | RealtimeEventKind::OfferReviewed
                | RealtimeEventKind::OfferUpdated
                | RealtimeEventKind::PaymentsOperationsUpdated
                | RealtimeEventKind::PaymentUpdated
                | RealtimeEventKind::TmsOperationsUpdated
                | RealtimeEventKind::LoadBoardListingUpdated
                | RealtimeEventKind::LoadDocumentUpdated => {
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
                "Only authenticated carrier accounts can self-book marketplace freight.".into(),
            ));
            return;
        }

        pending_leg_id.set(Some(leg_id));
        action_message.set(None);
        let auth = auth.clone();

        spawn_local(async move {
            let response = api::book_load_leg(
                leg_id,
                &BookLoadLegRequest {
                    booked_amount: None,
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
                        session::invalidate_session(&auth, "Your session expired; sign in again.");
                    }
                    action_message.set(Some(error));
                }
            }

            pending_leg_id.set(None);
        });
    };

    let submit_bid = move |posting_id: u64, amount_label: String| {
        let current_session = auth.session.get();
        let can_bid = current_session
            .user
            .as_ref()
            .map(|user| user.role_key == "carrier")
            .unwrap_or(false);

        if !can_bid {
            action_message.set(Some(
                "Only carrier accounts can bid on marketplace loads.".into(),
            ));
            return;
        }

        let amount = parse_amount_label(&amount_label).unwrap_or(0.0);
        if amount <= 0.0 {
            action_message.set(Some(
                "This posting needs a valid rate before bidding.".into(),
            ));
            return;
        }

        pending_posting_id.set(Some(posting_id));
        action_message.set(None);
        let auth = auth.clone();

        spawn_local(async move {
            let response = api::submit_marketplace_offer(
                posting_id,
                &SubmitOfferRequest {
                    amount,
                    currency: Some("USD".into()),
                    message: Some("Carrier bid from Marketplace Loads.".into()),
                    idempotency_key: None,
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
                        session::invalidate_session(&auth, "Your session expired; sign in again.");
                    }
                    action_message.set(Some(error));
                }
            }

            pending_posting_id.set(None);
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

    let apply_filters = move |_| {
        filters.set(draft_filters.get());
        action_message.set(None);
    };

    let clear_filters = move |_| {
        let next = LoadBoardFilterState::default();
        draft_filters.set(next.clone());
        filters.set(next);
        action_message.set(None);
    };

    let save_search = move |_| {
        let name = saved_search_name.get();
        let active_filters = filters.get();
        let alert_enabled = saved_search_alert.get();
        if name.trim().is_empty() {
            action_message.set(Some("Name this search before saving it.".into()));
            return;
        }
        spawn_local(async move {
            match api::save_load_board_search(&SaveLoadBoardSearchRequest {
                name,
                alert_enabled,
                filters: active_filters,
            })
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

    view! {
        <article style="display:grid;gap:0.95rem;min-height:calc(100vh - 150px);background:#eef2f7;padding:0.95rem;border-radius:0.95rem;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;padding:1rem;border:1px solid #cbd5e1;border-radius:0.85rem;background:#ffffff;box-shadow:0 10px 24px rgba(15,23,42,0.06);">
                <div>
                    <small style="display:block;color:#64748b;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;">{move || screen.get().map(|value| value.role_label).unwrap_or_else(|| "Marketplace".into())}</small>
                    <h2 style="margin:0.25rem 0 0;">{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Marketplace Loads".into())}</h2>
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

            <section style="display:flex;gap:0.55rem;flex-wrap:wrap;padding:0.7rem;border:1px solid #d8dee8;border-radius:0.85rem;background:#ffffff;">
                {move || screen.get().map(|value| {
                    value.tabs
                        .into_iter()
                        .map(|tab| {
                            let tab_key = tab.key.clone();
                            let style = if tab.is_active {
                                "padding:0.55rem 0.85rem;border-radius:0.65rem;background:#0f172a;color:white;border:1px solid #0f172a;cursor:pointer;font-weight:800;"
                            } else {
                                "padding:0.55rem 0.85rem;border-radius:0.65rem;background:#f8fafc;color:#111827;border:1px solid #cbd5e1;cursor:pointer;font-weight:700;"
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

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:0.75rem;">
                {move || screen.get().map(|value| {
                    value.metrics
                        .into_iter()
                        .map(|metric| {
                            view! {
                                <div style="padding:0.85rem 0.95rem;border:1px solid #cbd5e1;border-left:4px solid #2563eb;border-radius:0.75rem;background:#ffffff;box-shadow:0 8px 18px rgba(15,23,42,0.04);">
                                    <small style="display:block;color:#475569;font-weight:800;text-transform:uppercase;letter-spacing:0.04em;">{metric.label}</small>
                                    <strong style="font-size:1.15rem;color:#0f172a;">{metric.value}</strong>
                                    <small style="display:block;color:#64748b;">{metric.note}</small>
                                </div>
                            }
                        })
                        .collect_view()
                })}
            </section>

            <details style="border:1px solid #d8dee8;border-radius:0.85rem;background:#ffffff;box-shadow:0 10px 24px rgba(15,23,42,0.05);">
                <summary style="cursor:pointer;list-style:none;padding:0.85rem 1rem;font-weight:800;display:flex;justify-content:space-between;gap:1rem;border-bottom:1px solid #e2e8f0;">
                    <span>"Search"</span>
                    <span style="color:#64748b;font-weight:500;">"Filters / saved views"</span>
                </summary>
                <section style="display:grid;gap:0.85rem;padding:1rem;background:#f8fafc;border-radius:0 0 0.85rem 0.85rem;">
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:0.7rem;">
                        {filter_input("Origin", draft_filters, |state, value| state.origin = value)}
                        {filter_input("Destination", draft_filters, |state, value| state.destination = value)}
                        {filter_input("Mode", draft_filters, |state, value| state.mode = value)}
                        {filter_input("Equipment", draft_filters, |state, value| state.equipment = value)}
                        {filter_input("Pickup from", draft_filters, |state, value| state.date_from = value)}
                        {filter_input("Pickup to", draft_filters, |state, value| state.date_to = value)}
                        {filter_input("Min rate", draft_filters, |state, value| state.min_rate = value)}
                        {filter_input("Max rate", draft_filters, |state, value| state.max_rate = value)}
                        {filter_input("Min weight", draft_filters, |state, value| state.min_weight = value)}
                        {filter_input("Max weight", draft_filters, |state, value| state.max_weight = value)}
                        <label style="display:grid;gap:0.25rem;font-size:0.78rem;font-weight:700;color:#475569;text-transform:uppercase;">
                            "Sort"
                            <select style="padding:0.6rem;border:1px solid #94a3b8;border-radius:0.55rem;background:white;" on:change=move |ev| {
                                let value = event_target_value(&ev);
                                draft_filters.update(|state| state.sort = if value.is_empty() { None } else { Some(value) });
                            }>
                                <option value="">"Pickup"</option>
                                <option value="distance">"Distance"</option>
                                <option value="rate_desc">"Rate high"</option>
                                <option value="rate_asc">"Rate low"</option>
                                <option value="rpm_desc">"RPM"</option>
                                <option value="match_score">"Fit"</option>
                            </select>
                        </label>
                    </div>
                    <div style="display:flex;gap:0.65rem;align-items:center;flex-wrap:wrap;">
                        <label style="display:flex;gap:0.35rem;align-items:center;"><input type="checkbox" on:change=move |ev| draft_filters.update(|state| state.hazmat = Some(event_target_checked(&ev))) />"Hazmat"</label>
                        <label style="display:flex;gap:0.35rem;align-items:center;"><input type="checkbox" on:change=move |ev| draft_filters.update(|state| state.temperature_controlled = Some(event_target_checked(&ev))) />"Temp"</label>
                        <button type="button" style="padding:0.5rem 0.9rem;border-radius:0.5rem;border:1px solid #111827;background:#111827;color:white;cursor:pointer;" on:click=move |_| {
                            draft_filters.update(|state| state.page = Some("1".into()));
                            apply_filters(());
                        }>"Search"</button>
                        <button type="button" style="padding:0.5rem 0.9rem;border-radius:0.5rem;border:1px solid #cbd5e1;background:white;cursor:pointer;" on:click=clear_filters>"Reset"</button>
                        <input placeholder="View name" prop:value=move || saved_search_name.get() on:input=move |ev| saved_search_name.set(event_target_value(&ev)) style="padding:0.5rem;border:1px solid #cbd5e1;border-radius:0.45rem;min-width:180px;" />
                        <label style="display:flex;gap:0.35rem;align-items:center;"><input type="checkbox" prop:checked=move || saved_search_alert.get() on:change=move |ev| saved_search_alert.set(event_target_checked(&ev)) />"Alert"</label>
                        <button type="button" style="padding:0.5rem 0.9rem;border-radius:0.5rem;border:1px solid #1d4ed8;background:#eff6ff;color:#1d4ed8;cursor:pointer;" on:click=save_search>"Save View"</button>
                    </div>
                    {move || screen.get().map(|value| {
                        if value.saved_searches.is_empty() {
                            view! { <small style="color:#64748b;">"No saved views."</small> }.into_any()
                        } else {
                            view! {
                                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                    {value.saved_searches.into_iter().map(|item| view! {
                                        <span style="padding:0.3rem 0.5rem;border:1px solid #e5e7eb;border-radius:999px;background:#f8fafc;font-size:0.82rem;">
                                            {format!("{}{}", item.name, if item.alert_enabled { " / alert" } else { "" })}
                                        </span>
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    })}
                </section>
            </details>

            {move || action_message.get().map(|message| view! {
                <section style="padding:0.7rem 0.9rem;border:1px solid #bfdbfe;border-radius:0.65rem;background:#eff6ff;color:#1e3a8a;">
                    {message}
                </section>
            })}

            <section style="display:grid;grid-template-columns:minmax(0,1fr) minmax(320px,380px);gap:0.9rem;align-items:start;">
                <div style="overflow:auto;border:1px solid #b7c2d0;border-radius:0.85rem;background:#ffffff;max-height:calc(100vh - 360px);min-height:470px;box-shadow:0 14px 32px rgba(15,23,42,0.08);">
                <table style="width:100%;border-collapse:separate;border-spacing:0;min-width:1240px;font-size:0.84rem;">
                    <thead style="background:#f1f5f9;position:sticky;top:0;z-index:2;">
                        <tr>
                            <th style=header_cell_sticky()>"Load"</th>
                            <th style=header_cell()>"Origin"</th>
                            <th style=header_cell()>"Destination"</th>
                            <th style=header_cell()>"Mode"</th>
                            <th style=header_cell()>"Pickup"</th>
                            <th style=header_cell()>"Delivery"</th>
                            <th style=header_cell()>"Rate"</th>
                            <th style=header_cell()>"Status"</th>
                            <th style=header_cell()>"Board"</th>
                            <th style=header_cell()>"Fit"</th>
                            <th style=header_cell()>"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            if is_loading.get() && screen.get().is_none() {
                                view! {
                                    <tr>
                                        <td colspan="11" style="padding:1rem;">"Loading loads..."</td>
                                    </tr>
                                }.into_any()
                            } else {
                                screen
                                    .get()
                                    .map(|value| {
                                        if value.rows.is_empty() {
                                            view! { <tr><td colspan="11" style="padding:1rem;">"No loads found."</td></tr> }.into_any()
                                        } else {
                                            value.rows
                                                .into_iter()
                                                .enumerate()
                                                .map(|(index, row)| render_row(index, row, selected_row, pending_leg_id, pending_posting_id, book_leg, submit_bid, can_self_book.get(), can_view_profile.get()))
                                                .collect_view()
                                                .into_any()
                                        }
                                    })
                                    .unwrap_or_else(|| view! {
                                        <tr>
                                            <td colspan="11" style="padding:1rem;">"No loads found."</td>
                                        </tr>
                                    }.into_any())
                            }
                        }}
                    </tbody>
                </table>
                </div>
                {move || render_detail_drawer(selected_row.get(), pending_leg_id, pending_posting_id, book_leg, submit_bid, can_self_book.get(), can_view_profile.get(), selected_row)}
            </section>

            <section style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                {move || screen.get().map(|value| {
                    view! {
                        <>
                            <small>{format!("Page {} of {}", value.pagination.page, value.pagination.total.max(1))}</small>
                            <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                                <button type="button" style="padding:0.45rem 0.75rem;border:1px solid #d4d4d8;border-radius:0.65rem;background:white;cursor:pointer;" disabled=move || {
                                    screen.get().map(|value| value.pagination.page <= 1).unwrap_or(true)
                                } on:click=move |_| {
                                    draft_filters.update(|state| {
                                        let current = state.page.as_deref().and_then(|value| value.parse::<u64>().ok()).unwrap_or(1);
                                        state.page = Some(current.saturating_sub(1).max(1).to_string());
                                    });
                                    filters.set(draft_filters.get());
                                }>"Previous"</button>
                                <button type="button" style="padding:0.45rem 0.75rem;border:1px solid #d4d4d8;border-radius:0.65rem;background:white;cursor:pointer;" disabled=move || {
                                    screen.get().map(|value| value.pagination.page.saturating_mul(value.pagination.per_page) >= value.pagination.total).unwrap_or(true)
                                } on:click=move |_| {
                                    draft_filters.update(|state| {
                                        let current = state.page.as_deref().and_then(|value| value.parse::<u64>().ok()).unwrap_or(1);
                                        state.page = Some(current.saturating_add(1).to_string());
                                    });
                                    filters.set(draft_filters.get());
                                }>"Next"</button>
                            </div>
                        </>
                    }
                })}
            </section>
        </article>
    }
}

fn render_row(
    index: usize,
    row: LoadBoardRow,
    selected_row: RwSignal<Option<LoadBoardRow>>,
    pending_leg_id: RwSignal<Option<u64>>,
    pending_posting_id: RwSignal<Option<u64>>,
    book_leg: impl Fn(u64) + Copy + 'static,
    submit_bid: impl Fn(u64, String) + Copy + 'static,
    can_self_book: bool,
    can_view_profile: bool,
) -> impl IntoView {
    let row_for_select = row.clone();
    let LoadBoardRow {
        load_id,
        leg_id,
        posting_id,
        leg_code,
        origin_label,
        destination_label,
        mode_label,
        pickup_date_label,
        delivery_date_label,
        status_label,
        status_tone,
        stloads_label,
        stloads_tone,
        stloads_alert,
        remarks_label: _remarks_label,
        carrier_label,
        booked_carrier_id,
        bid_status_label,
        amount_label,
        payment_label,
        recommended_score,
        primary_action_label,
    } = row;

    let is_booking = Signal::derive(move || pending_leg_id.get() == Some(leg_id));
    let is_bidding = Signal::derive(move || pending_posting_id.get() == posting_id);
    let show_book_button = can_self_book && booked_carrier_id.is_none();
    let show_bid_button = can_self_book && booked_carrier_id.is_none() && posting_id.is_some();
    let origin_title = origin_label.clone();
    let destination_title = destination_label.clone();
    let origin_short = compact_location(&origin_label);
    let destination_short = compact_location(&destination_label);
    let code_short = compact_code(&leg_code);
    let amount_for_bid = amount_label.clone();
    let fit_label = recommended_score
        .map(|score| format!("{}", score))
        .unwrap_or_else(|| "-".into());
    let row_background = if index % 2 == 0 { "#ffffff" } else { "#f8fafc" };
    let sticky_cell_style = format!(
        "padding:0.7rem 0.75rem;border-right:1px solid #d8dee8;border-bottom:1px solid #d8dee8;position:sticky;left:0;background:{};z-index:1;min-width:220px;max-width:245px;box-shadow:6px 0 14px rgba(15,23,42,0.04);",
        row_background
    );

    view! {
        <tr
            style=format!("vertical-align:middle;background:{};cursor:pointer;", row_background)
            on:click=move |_| selected_row.set(Some(row_for_select.clone()))
        >
            <td style=sticky_cell_style title=leg_code.clone()>
                <A href=format!("/loads/{}", load_id) attr:style="display:block;color:#0f172a;text-decoration:none;">
                    <strong style="display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{code_short}</strong>
                </A>
                <small style="color:#64748b;">{format!("Leg {}", leg_id)}</small>
            </td>
            <td style=body_cell("max-width:190px;") title=origin_title>
                <span style="display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{origin_short}</span>
            </td>
            <td style=body_cell("max-width:190px;") title=destination_title>
                <span style="display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{destination_short}</span>
            </td>
            <td style=body_cell("white-space:nowrap;")>
                <span style="display:inline-flex;align-items:center;padding:0.25rem 0.5rem;border-radius:0.5rem;border:1px solid #2563eb;background:#eff6ff;color:#1d4ed8;font-weight:800;font-size:0.78rem;">{mode_label}</span>
            </td>
            <td style=body_cell("white-space:nowrap;")>{pickup_date_label}</td>
            <td style=body_cell("white-space:nowrap;")>{delivery_date_label}</td>
            <td style="padding:0.7rem 0.75rem;border-right:1px solid #d8dee8;border-bottom:1px solid #d8dee8;white-space:nowrap;background:#f8fafc;">
                <strong style="font-size:0.95rem;color:#0f172a;">{amount_label}</strong>
                <small style="display:block;color:#64748b;">{payment_label}</small>
            </td>
            <td style=body_cell("")>
                <span style=tone_style(&status_tone)>{status_label}</span>
                {carrier_label.map(|carrier| view! { <div><small>{carrier}</small></div> })}
            </td>
            <td style=body_cell("")>
                {stloads_label.clone().map(|label| {
                    let tone = stloads_tone.as_deref().unwrap_or("secondary");
                    view! {
                        <div style="display:grid;gap:0.25rem;">
                            <span style=tone_style(tone)>{label}</span>
                            {stloads_alert.clone().map(|alert| view! { <small>{alert}</small> })}
                        </div>
                    }
                })}
                {stloads_label.is_none().then(|| view! { <span>"Not posted"</span> })}
            </td>
            <td style=body_cell("white-space:nowrap;")>
                <strong>{fit_label}</strong>
                <small style="display:block;color:#64748b;">{bid_status_label}</small>
            </td>
            <td style="padding:0.7rem 0.75rem;border-right:1px solid #d8dee8;border-bottom:1px solid #d8dee8;display:grid;gap:0.35rem;min-width:155px;background:#f8fafc;">
                <small style="font-weight:700;color:#334155;">{primary_action_label}</small>
                {can_view_profile.then(|| view! {
                    <A href=format!("/loads/{}", load_id) attr:style="color:#1d4ed8;text-decoration:none;">"View profile"</A>
                })}
                {show_book_button.then(|| view! {
                    <button
                        type="button"
                        style="padding:0.45rem 0.65rem;border-radius:0.5rem;border:1px solid #111827;background:#111827;color:white;cursor:pointer;"
                        disabled=move || is_booking.get()
                        on:click=move |_| book_leg(leg_id)
                    >
                        {move || if is_booking.get() { "Booking..." } else { "Book" }}
                    </button>
                })}
                {show_bid_button.then(|| {
                    let posting_id = posting_id.unwrap_or_default();
                    view! {
                        <button
                            type="button"
                            style="padding:0.45rem 0.65rem;border-radius:0.5rem;border:1px solid #2563eb;background:#eff6ff;color:#1d4ed8;cursor:pointer;"
                            disabled=move || is_bidding.get()
                            on:click=move |_| submit_bid(posting_id, amount_for_bid.clone())
                        >
                            {move || if is_bidding.get() { "Bidding..." } else { "Bid" }}
                        </button>
                    }
                })}
                {(!can_self_book && posting_id.is_some()).then(|| view! {
                    <A href="/chat" attr:style="color:#1d4ed8;text-decoration:none;">"Review bids"</A>
                })}
            </td>
        </tr>
    }
}

fn render_detail_drawer(
    row: Option<LoadBoardRow>,
    pending_leg_id: RwSignal<Option<u64>>,
    pending_posting_id: RwSignal<Option<u64>>,
    book_leg: impl Fn(u64) + Copy + 'static,
    submit_bid: impl Fn(u64, String) + Copy + 'static,
    can_self_book: bool,
    can_view_profile: bool,
    selected_row: RwSignal<Option<LoadBoardRow>>,
) -> impl IntoView {
    row.map(|row| {
        let LoadBoardRow {
            load_id,
            leg_id,
            posting_id,
            leg_code,
            origin_label,
            destination_label,
            mode_label,
            pickup_date_label,
            delivery_date_label,
            status_label,
            status_tone,
            stloads_label,
            stloads_tone,
            stloads_alert,
            remarks_label,
            carrier_label,
            booked_carrier_id,
            bid_status_label,
            amount_label,
            payment_label,
            recommended_score,
            primary_action_label,
        } = row;

        let is_booking = Signal::derive(move || pending_leg_id.get() == Some(leg_id));
        let is_bidding = Signal::derive(move || pending_posting_id.get() == posting_id);
        let show_book_button = can_self_book && booked_carrier_id.is_none();
        let show_bid_button = can_self_book && booked_carrier_id.is_none() && posting_id.is_some();
        let amount_for_bid = amount_label.clone();
        let board_label = stloads_label.unwrap_or_else(|| "Not posted".into());
        let board_tone = stloads_tone.unwrap_or_else(|| "secondary".into());
        let fit_label = recommended_score
            .map(|score| score.to_string())
            .unwrap_or_else(|| "-".into());
        let summary_label = route_summary_label(
            &mode_label,
            &origin_label,
            &destination_label,
            &pickup_date_label,
            &delivery_date_label,
        );

        view! {
            <aside style="position:sticky;top:0;border:1px solid #b7c2d0;border-radius:0.85rem;background:#ffffff;box-shadow:0 14px 32px rgba(15,23,42,0.08);overflow:hidden;">
                <header style="display:flex;justify-content:space-between;gap:0.75rem;align-items:flex-start;padding:0.9rem 1rem;border-bottom:1px solid #d8dee8;background:#f8fafc;">
                    <div style="display:grid;gap:0.2rem;">
                        <small style="color:#64748b;font-weight:800;text-transform:uppercase;letter-spacing:0.06em;">"Load summary"</small>
                        <strong style="font-size:1rem;color:#0f172a;word-break:break-word;">{compact_code(&leg_code)}</strong>
                    </div>
                    <button type="button" style="border:1px solid #cbd5e1;background:white;border-radius:0.45rem;padding:0.3rem 0.5rem;cursor:pointer;" on:click=move |_| selected_row.set(None)>"Close"</button>
                </header>
                <section style="display:grid;gap:0.85rem;padding:1rem;">
                    <div style="display:grid;gap:0.45rem;">
                        <span style=tone_style(&status_tone)>{status_label}</span>
                        <span style=tone_style(&board_tone)>{board_label}</span>
                    </div>
                    <p style="margin:0;color:#334155;line-height:1.45;">{summary_label}</p>
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.65rem;">
                        {detail_cell("Mode", mode_label)}
                        {detail_cell("Fit", fit_label)}
                        {detail_cell("Pickup", pickup_date_label)}
                        {detail_cell("Delivery", delivery_date_label)}
                        {detail_cell("Rate", amount_label)}
                        {detail_cell("Payment", payment_label)}
                    </div>
                    <div style="display:grid;gap:0.45rem;">
                        <small style="color:#64748b;font-weight:800;text-transform:uppercase;letter-spacing:0.05em;">"Origin"</small>
                        <strong>{origin_label}</strong>
                    </div>
                    <div style="display:grid;gap:0.45rem;">
                        <small style="color:#64748b;font-weight:800;text-transform:uppercase;letter-spacing:0.05em;">"Destination"</small>
                        <strong>{destination_label}</strong>
                    </div>
                    <div style="display:grid;gap:0.45rem;">
                        <small style="color:#64748b;font-weight:800;text-transform:uppercase;letter-spacing:0.05em;">"Bid status"</small>
                        <strong>{bid_status_label}</strong>
                    </div>
                    {carrier_label.map(|carrier| view! {
                        <div style="display:grid;gap:0.45rem;">
                            <small style="color:#64748b;font-weight:800;text-transform:uppercase;letter-spacing:0.05em;">"Carrier"</small>
                            <strong>{carrier}</strong>
                        </div>
                    })}
                    {remarks_label.map(|remarks| view! {
                        <div style="padding:0.75rem;border:1px solid #e2e8f0;border-radius:0.65rem;background:#f8fafc;color:#334155;">
                            {remarks}
                        </div>
                    })}
                    {stloads_alert.map(|alert| view! {
                        <div style="padding:0.75rem;border:1px solid #fde68a;border-radius:0.65rem;background:#fffbeb;color:#92400e;">
                            {alert}
                        </div>
                    })}
                    <div style="display:grid;gap:0.55rem;border-top:1px solid #e2e8f0;padding-top:0.9rem;">
                        <small style="font-weight:800;color:#334155;">{primary_action_label}</small>
                        {can_view_profile.then(|| view! {
                            <A href=format!("/loads/{}", load_id) attr:style="display:block;text-align:center;padding:0.55rem 0.75rem;border-radius:0.55rem;background:#0f172a;color:white;text-decoration:none;font-weight:800;">"View full load"</A>
                        })}
                        {show_book_button.then(|| view! {
                            <button
                                type="button"
                                style="padding:0.55rem 0.75rem;border-radius:0.55rem;border:1px solid #111827;background:#111827;color:white;cursor:pointer;font-weight:800;"
                                disabled=move || is_booking.get()
                                on:click=move |_| book_leg(leg_id)
                            >
                                {move || if is_booking.get() { "Booking..." } else { "Book load" }}
                            </button>
                        })}
                        {show_bid_button.then(|| {
                            let posting_id = posting_id.unwrap_or_default();
                            view! {
                                <button
                                    type="button"
                                    style="padding:0.55rem 0.75rem;border-radius:0.55rem;border:1px solid #2563eb;background:#eff6ff;color:#1d4ed8;cursor:pointer;font-weight:800;"
                                    disabled=move || is_bidding.get()
                                    on:click=move |_| submit_bid(posting_id, amount_for_bid.clone())
                                >
                                    {move || if is_bidding.get() { "Submitting..." } else { "Submit bid" }}
                                </button>
                            }
                        })}
                        {(!can_self_book && posting_id.is_some()).then(|| view! {
                            <A href="/chat" attr:style="display:block;text-align:center;padding:0.55rem 0.75rem;border-radius:0.55rem;border:1px solid #bfdbfe;background:#eff6ff;color:#1d4ed8;text-decoration:none;font-weight:800;">"Review bids"</A>
                        })}
                    </div>
                </section>
            </aside>
        }.into_any()
    }).unwrap_or_else(|| {
        view! {
            <aside style="position:sticky;top:0;border:1px dashed #b7c2d0;border-radius:0.85rem;background:#ffffff;min-height:320px;display:grid;place-items:center;padding:1rem;color:#64748b;text-align:center;">
                <div>
                    <strong style="display:block;color:#334155;">"Select a load"</strong>
                    <span>"Open the full summary and actions here."</span>
                </div>
            </aside>
        }.into_any()
    })
}

fn parse_amount_label(value: &str) -> Option<f64> {
    let cleaned = value
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    cleaned.parse::<f64>().ok()
}

fn compact_location(value: &str) -> String {
    let parts = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .take(2)
        .collect::<Vec<_>>();

    if parts.is_empty() {
        value.trim().chars().take(42).collect()
    } else {
        parts.join(", ")
    }
}

fn compact_code(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() <= 22 {
        return trimmed.into();
    }

    format!("{}...", trimmed.chars().take(22).collect::<String>())
}

fn route_summary_label(
    mode: &str,
    origin: &str,
    destination: &str,
    pickup: &str,
    delivery: &str,
) -> String {
    format!("{mode} / {origin} -> {destination} / {pickup} to {delivery}")
}

fn header_cell() -> &'static str {
    "text-align:left;padding:0.7rem 0.75rem;border-right:1px solid #b7c2d0;border-bottom:1px solid #b7c2d0;text-transform:uppercase;font-size:0.72rem;letter-spacing:0.04em;color:#334155;white-space:nowrap;"
}

fn header_cell_sticky() -> &'static str {
    "text-align:left;padding:0.7rem 0.75rem;border-right:1px solid #b7c2d0;border-bottom:1px solid #b7c2d0;position:sticky;left:0;background:#f1f5f9;z-index:3;text-transform:uppercase;font-size:0.72rem;letter-spacing:0.04em;color:#334155;white-space:nowrap;"
}

fn body_cell(extra: &'static str) -> String {
    format!(
        "padding:0.7rem 0.75rem;border-right:1px solid #d8dee8;border-bottom:1px solid #d8dee8;background:inherit;{}",
        extra
    )
}

fn detail_cell(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div style="display:grid;gap:0.25rem;padding:0.65rem;border:1px solid #e2e8f0;border-radius:0.6rem;background:#f8fafc;min-width:0;">
            <small style="color:#64748b;font-weight:800;text-transform:uppercase;letter-spacing:0.05em;">{label}</small>
            <strong style="color:#0f172a;overflow:hidden;text-overflow:ellipsis;">{value}</strong>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_summary_keeps_mode_and_dates_visible() {
        let summary = route_summary_label("Rail", "Dallas, TX", "Chicago, IL", "May 22", "May 24");

        assert_eq!(
            summary,
            "Rail / Dallas, TX -> Chicago, IL / May 22 to May 24"
        );
    }
}

fn filter_input(
    label: &'static str,
    draft_filters: RwSignal<LoadBoardFilterState>,
    apply: fn(&mut LoadBoardFilterState, Option<String>),
) -> impl IntoView {
    view! {
        <label style="display:grid;gap:0.35rem;font-size:0.78rem;font-weight:800;color:#475569;text-transform:uppercase;letter-spacing:0.04em;">
            {label}
            <input
                style="padding:0.6rem;border:1px solid #94a3b8;border-radius:0.55rem;background:#ffffff;color:#0f172a;font-size:0.88rem;text-transform:none;letter-spacing:0;"
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    draft_filters.update(|state| {
                        apply(state, if value.trim().is_empty() { None } else { Some(value.clone()) });
                    });
                }
            />
        </label>
    }
}
