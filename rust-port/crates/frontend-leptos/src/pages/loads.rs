use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;

use crate::{
    api, realtime,
    session::{self, use_auth},
};
use shared::{BookLoadLegRequest, LoadBoardRow, LoadBoardScreen, RealtimeEventKind, RealtimeTopic};

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
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_load_board_screen(&tab).await {
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

    view! {
        <article style="display:grid;gap:1.25rem;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div>
                    <p style=tone_style("info")>{move || screen.get().map(|value| value.role_label).unwrap_or_else(|| "Marketplace Workspace".into())}</p>
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Manage Loads".into())}</h2>
                    <p>
                        "The load board now reads a single app-level Rust auth session and only refreshes for scoped realtime updates."
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

            {move || auth.session.get().user.map(|user| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dcfce7;border-radius:0.9rem;background:#f0fdf4;color:#166534;">
                    {format!("Authenticated as {} ({})", user.name, user.role_label)}
                </section>
            })}

            {move || action_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                    {message}
                </section>
            })}

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
                                            .map(|row| render_row(row, pending_leg_id, book_leg, can_self_book.get(), can_view_profile.get()))
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
                            <small>{format!("Page {} of {} visible rows", value.pagination.page, value.pagination.total)}</small>
                        </>
                    }
                })}
            </section>
        </article>
    }
}

fn render_row(
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
    let show_book_button = can_self_book && booked_carrier_id.is_none();

    view! {
        <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
            <td style="padding:0.9rem;">
                <strong>{leg_code}</strong>
                {recommended_score.map(|score| view! { <div><small>{format!("match score {}", score)}</small></div> })}
            </td>
            <td style="padding:0.9rem;">{origin_label}</td>
            <td style="padding:0.9rem;">{destination_label}</td>
            <td style="padding:0.9rem;">{pickup_date_label}</td>
            <td style="padding:0.9rem;">{delivery_date_label}</td>
            <td style="padding:0.9rem;">
                <span style=tone_style(&status_tone)>{status_label}</span>
                {carrier_label.map(|carrier| view! { <div><small>{carrier}</small></div> })}
                {remarks_label.map(|remarks| view! { <div><small>{remarks}</small></div> })}
            </td>
            <td style="padding:0.9rem;">
                {stloads_label.clone().map(|label| {
                    let tone = stloads_tone.as_deref().unwrap_or("secondary");
                    view! {
                        <div style="display:grid;gap:0.35rem;">
                            <span style=tone_style(tone)>{label}</span>
                            {stloads_alert.clone().map(|alert| view! { <small>{alert}</small> })}
                        </div>
                    }
                })}
                {stloads_label.is_none().then(|| view! { <span>"Not posted"</span> })}
            </td>
            <td style="padding:0.9rem;">{bid_status_label}</td>
            <td style="padding:0.9rem;">{amount_label}</td>
            <td style="padding:0.9rem;">{payment_label}</td>
            <td style="padding:0.9rem;display:grid;gap:0.45rem;min-width:180px;">
                <strong>{primary_action_label}</strong>
                {can_view_profile.then(|| view! {
                    <A href=format!("/loads/{}", load_id) attr:style="color:#1d4ed8;text-decoration:none;">"View profile"</A>
                })}
                {show_book_button.then(|| view! {
                    <button
                        type="button"
                        style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:1px solid #111827;background:#111827;color:white;cursor:pointer;"
                        disabled=move || is_booking.get()
                        on:click=move |_| book_leg(leg_id)
                    >
                        {move || if is_booking.get() { "Booking..." } else { "Book this leg" }}
                    </button>
                })}
            </td>
        </tr>
    }
}
