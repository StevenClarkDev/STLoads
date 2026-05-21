use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;

use crate::{
    api, realtime,
    session::{self, use_auth},
};
use shared::{
    BookLoadLegRequest, LoadBoardFilterState, LoadBoardRow, LoadBoardScreen, RealtimeEventKind,
    RealtimeTopic, SaveLoadBoardSearchRequest,
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
        <article style="display:grid;gap:0.85rem;min-height:calc(100vh - 150px);">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
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

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:0.6rem;">
                {move || screen.get().map(|value| {
                    value.metrics
                        .into_iter()
                        .map(|metric| {
                            view! {
                                <div style="padding:0.75rem 0.85rem;border:1px solid #e5e7eb;border-radius:0.6rem;background:#ffffff;">
                                    <small style="display:block;color:#64748b;font-weight:700;text-transform:uppercase;">{metric.label}</small>
                                    <strong style="font-size:1rem;">{metric.value}</strong>
                                </div>
                            }
                        })
                        .collect_view()
                })}
            </section>

            <details style="border:1px solid #dbe3ee;border-radius:0.65rem;background:#ffffff;">
                <summary style="cursor:pointer;list-style:none;padding:0.75rem 0.9rem;font-weight:700;display:flex;justify-content:space-between;gap:1rem;">
                    <span>"Search"</span>
                    <span style="color:#64748b;font-weight:500;">"Filters / saved views"</span>
                </summary>
                <section style="display:grid;gap:0.75rem;padding:0 0.9rem 0.9rem;">
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:0.6rem;">
                        {filter_input("Origin", draft_filters, |state, value| state.origin = value)}
                        {filter_input("Destination", draft_filters, |state, value| state.destination = value)}
                        {filter_input("Equipment", draft_filters, |state, value| state.equipment = value)}
                        {filter_input("Pickup from", draft_filters, |state, value| state.date_from = value)}
                        {filter_input("Pickup to", draft_filters, |state, value| state.date_to = value)}
                        {filter_input("Min rate", draft_filters, |state, value| state.min_rate = value)}
                        {filter_input("Max rate", draft_filters, |state, value| state.max_rate = value)}
                        {filter_input("Min weight", draft_filters, |state, value| state.min_weight = value)}
                        {filter_input("Max weight", draft_filters, |state, value| state.max_weight = value)}
                        <label style="display:grid;gap:0.25rem;font-size:0.78rem;font-weight:700;color:#475569;text-transform:uppercase;">
                            "Sort"
                            <select style="padding:0.5rem;border:1px solid #cbd5e1;border-radius:0.45rem;background:white;" on:change=move |ev| {
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

            <section style="overflow:auto;border:1px solid #dbe3ee;border-radius:0.65rem;background:#ffffff;max-height:calc(100vh - 360px);min-height:420px;">
                <table style="width:100%;border-collapse:separate;border-spacing:0;min-width:1120px;font-size:0.86rem;">
                    <thead style="background:#f8fafc;position:sticky;top:0;z-index:2;">
                        <tr>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;position:sticky;left:0;background:#f8fafc;z-index:3;">"Load"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Origin"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Destination"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Pickup"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Delivery"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Rate"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Status"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Board"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Fit"</th>
                            <th style="text-align:left;padding:0.65rem 0.75rem;border-bottom:1px solid #dbe3ee;">"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            if is_loading.get() && screen.get().is_none() {
                                view! {
                                    <tr>
                                        <td colspan="10" style="padding:1rem;">"Loading loads..."</td>
                                    </tr>
                                }.into_any()
                            } else {
                                screen
                                    .get()
                                    .map(|value| {
                                        if value.rows.is_empty() {
                                            view! { <tr><td colspan="10" style="padding:1rem;">"No loads found."</td></tr> }.into_any()
                                        } else {
                                            value.rows
                                                .into_iter()
                                                .enumerate()
                                                .map(|(index, row)| render_row(index, row, pending_leg_id, book_leg, can_self_book.get(), can_view_profile.get()))
                                                .collect_view()
                                                .into_any()
                                        }
                                    })
                                    .unwrap_or_else(|| view! {
                                        <tr>
                                            <td colspan="10" style="padding:1rem;">"No loads found."</td>
                                        </tr>
                                    }.into_any())
                            }
                        }}
                    </tbody>
                </table>
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
    pending_leg_id: RwSignal<Option<u64>>,
    book_leg: impl Fn(u64) + Copy + 'static,
    can_self_book: bool,
    can_view_profile: bool,
) -> impl IntoView {
    let LoadBoardRow {
        load_id,
        leg_id,
        leg_code,
        origin_label,
        destination_label,
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
    let show_book_button = can_self_book && booked_carrier_id.is_none();
    let origin_title = origin_label.clone();
    let destination_title = destination_label.clone();
    let origin_short = compact_location(&origin_label);
    let destination_short = compact_location(&destination_label);
    let code_short = compact_code(&leg_code);
    let fit_label = recommended_score
        .map(|score| format!("{}", score))
        .unwrap_or_else(|| "-".into());
    let row_background = if index % 2 == 0 { "#ffffff" } else { "#f8fafc" };
    let sticky_cell_style = format!(
        "padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;position:sticky;left:0;background:{};z-index:1;min-width:210px;max-width:230px;",
        row_background
    );

    view! {
        <tr style=format!("vertical-align:middle;background:{};", row_background)>
            <td style=sticky_cell_style title=leg_code.clone()>
                <strong style="display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{code_short}</strong>
                <small style="color:#64748b;">{format!("Leg {}", leg_id)}</small>
            </td>
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;max-width:180px;" title=origin_title>
                <span style="display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{origin_short}</span>
            </td>
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;max-width:180px;" title=destination_title>
                <span style="display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{destination_short}</span>
            </td>
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;white-space:nowrap;">{pickup_date_label}</td>
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;white-space:nowrap;">{delivery_date_label}</td>
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;white-space:nowrap;">
                <strong>{amount_label}</strong>
                <small style="display:block;color:#64748b;">{payment_label}</small>
            </td>
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;">
                <span style=tone_style(&status_tone)>{status_label}</span>
                {carrier_label.map(|carrier| view! { <div><small>{carrier}</small></div> })}
            </td>
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;">
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
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;white-space:nowrap;">
                <strong>{fit_label}</strong>
                <small style="display:block;color:#64748b;">{bid_status_label}</small>
            </td>
            <td style="padding:0.65rem 0.75rem;border-bottom:1px solid #edf2f7;display:grid;gap:0.35rem;min-width:145px;">
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
            </td>
        </tr>
    }
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

fn filter_input(
    label: &'static str,
    draft_filters: RwSignal<LoadBoardFilterState>,
    apply: fn(&mut LoadBoardFilterState, Option<String>),
) -> impl IntoView {
    view! {
        <label style="display:grid;gap:0.35rem;font-size:0.85rem;">
            {label}
            <input
                style="padding:0.55rem;border:1px solid #d4d4d8;border-radius:0.65rem;"
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
