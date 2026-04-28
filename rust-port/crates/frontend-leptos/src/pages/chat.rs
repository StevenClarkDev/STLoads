use futures_util::future::AbortHandle;
use leptos::{ev, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use crate::{
    api, realtime,
    session::{self, use_auth},
};
use shared::{
    ChatSendMessageRequest, ChatWorkspaceScreen, OfferReviewDecision, OfferReviewRequest,
    RealtimeEventKind, RealtimeTopic,
};

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => {
            "background:#e8fff3;padding:0.25rem 0.55rem;border-radius:999px;color:#0f766e;"
        }
        "warning" => {
            "background:#fff7dd;padding:0.25rem 0.55rem;border-radius:999px;color:#b45309;"
        }
        "danger" => "background:#ffe4e6;padding:0.25rem 0.55rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.55rem;border-radius:999px;color:#0369a1;",
        _ => "background:#f1f5f9;padding:0.25rem 0.55rem;border-radius:999px;color:#475569;",
    }
}

#[component]
pub fn ChatWorkspacePage() -> impl IntoView {
    let auth = use_auth();
    let selected_conversation_id = RwSignal::new(None::<u64>);
    let screen = RwSignal::new(None::<ChatWorkspaceScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let pending_offer_id = RwSignal::new(None::<u64>);
    let is_sending = RwSignal::new(false);
    let message_body = RwSignal::new(String::new());
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_connected = RwSignal::new(false);
    let ws_handle = RwSignal::new(None::<AbortHandle>);

    Effect::new(move |_| {
        let conversation_id = selected_conversation_id.get();
        let _refresh = refresh_nonce.get();
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();

        if !ready {
            return;
        }

        if !current_session.authenticated {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some("Sign in to open the Rust chat workspace.".into()));
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_chat_workspace_screen(conversation_id).await {
                Ok(next_screen) => {
                    error_message.set(None);
                    if selected_conversation_id.get().is_none() {
                        selected_conversation_id.set(next_screen.active_conversation_id);
                    }
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
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let conversation_id = selected_conversation_id.get();
        let current_user_id = current_session.user.as_ref().map(|user| user.id);

        if !ready || !current_session.authenticated {
            if let Some(existing_handle) = ws_handle.get_untracked() {
                existing_handle.abort();
                ws_handle.set(None);
            }
            ws_connected.set(false);
            return;
        }

        if let Some(existing_handle) = ws_handle.get_untracked() {
            existing_handle.abort();
        }

        let auth = auth.clone();
        let handle = realtime::connect_realtime_listener(
            conversation_id,
            vec![RealtimeTopic::Conversation],
            move |event| match event.kind {
                RealtimeEventKind::MessageSent
                | RealtimeEventKind::OfferReviewed
                | RealtimeEventKind::LoadLegBooked => {
                    if event.conversation_id.is_none()
                        || event.conversation_id == selected_conversation_id.get()
                    {
                        refresh_nonce.update(|value| *value += 1);
                        action_message.set(Some(format!("Realtime update: {}", event.summary)));
                    }
                }
                RealtimeEventKind::ConversationRead => {
                    if event.conversation_id == selected_conversation_id.get()
                        && event.subject_user_id != current_user_id
                    {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                RealtimeEventKind::ConversationPresenceChanged => {
                    if event.conversation_id == selected_conversation_id.get() {
                        refresh_nonce.update(|value| *value += 1);
                        action_message.set(Some(format!("Realtime update: {}", event.summary)));
                    }
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
                _ => {}
            },
        );

        ws_connected.set(handle.is_some());
        ws_handle.set(handle);
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let active_conversation_id = screen.get().and_then(|value| value.active_conversation_id);
        let latest_message_id = screen
            .get()
            .and_then(|value| value.messages.last().map(|message| message.id));

        if !ready || !current_session.authenticated {
            return;
        }

        let Some(conversation_id) = active_conversation_id else {
            return;
        };

        let auth = auth.clone();
        spawn_local(async move {
            if let Err(error) = api::mark_conversation_read(conversation_id).await {
                if error.contains("returned 401") {
                    session::invalidate_session(&auth, "Your Rust session expired; sign in again.");
                }
            }
        });

        let _ = latest_message_id;
    });

    let review_offer_action = move |offer_id: u64, decision: OfferReviewDecision| {
        let current_session = auth.session.get();
        let can_review = current_session
            .user
            .as_ref()
            .map(|user| user.role_key != "carrier")
            .unwrap_or(false);

        if !can_review {
            action_message.set(Some(
                "Carrier accounts cannot accept or decline offers from this Rust marketplace screen.".into(),
            ));
            return;
        }

        pending_offer_id.set(Some(offer_id));
        action_message.set(None);
        let auth = auth.clone();

        spawn_local(async move {
            match api::review_offer(offer_id, &OfferReviewRequest { decision }).await {
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

            pending_offer_id.set(None);
        });
    };

    let send_message_action = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        if !auth.session.get().authenticated {
            action_message.set(Some("Sign in before sending chat messages.".into()));
            return;
        }

        let Some(current_screen) = screen.get() else {
            action_message.set(Some("Chat data is still loading.".into()));
            return;
        };

        let Some(conversation_id) = current_screen.active_conversation_id else {
            action_message.set(Some(
                "There is no active conversation to send a message into.".into(),
            ));
            return;
        };

        let body = message_body.get().trim().to_string();
        if body.is_empty() {
            action_message.set(Some("Enter a message before sending.".into()));
            return;
        }

        is_sending.set(true);
        action_message.set(None);
        let auth = auth.clone();

        spawn_local(async move {
            match api::send_message(conversation_id, &ChatSendMessageRequest { body }).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        message_body.set(String::new());
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

            is_sending.set(false);
        });
    };

    let can_review_offers = Signal::derive(move || {
        auth.session
            .get()
            .user
            .map(|user| user.role_key != "carrier")
            .unwrap_or(false)
    });

    view! {
        <article style="display:grid;gap:1rem;">
            <section>
                <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Private Chat".into())}</h2>
                <p>
                    "The chat route now shares the app-level Rust session and reacts to scoped presence, read receipts, and conversation-only realtime events."
                </p>
            </section>

            {move || error_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">
                    {message}
                </section>
            })}

            <section style="display:grid;grid-template-columns:minmax(260px,320px) minmax(0,1fr);gap:1rem;align-items:start;">
                <aside style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#fcfcfb;display:grid;gap:0.75rem;">
                    <strong>"Recent conversations"</strong>
                    {move || {
                        if is_loading.get() && screen.get().is_none() {
                            view! { <p style="margin:0;">"Loading auth-scoped conversations from the Rust backend..."</p> }.into_any()
                        } else {
                            screen
                                .get()
                                .map(|value| {
                                    value.conversations
                                        .into_iter()
                                        .map(|item| {
                                            let conversation_id = item.id;
                                            let card_style = if item.is_active {
                                                "padding:0.85rem;border:1px solid #bfdbfe;border-radius:0.9rem;background:#eff6ff;display:grid;gap:0.35rem;cursor:pointer;text-align:left;"
                                            } else {
                                                "padding:0.85rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:white;display:grid;gap:0.35rem;cursor:pointer;text-align:left;"
                                            };
                                            view! {
                                                <button
                                                    type="button"
                                                    style=card_style
                                                    on:click=move |_| {
                                                        selected_conversation_id.set(Some(conversation_id));
                                                        action_message.set(None);
                                                    }
                                                >
                                                    <div style="display:flex;justify-content:space-between;gap:0.5rem;align-items:center;">
                                                        <strong>{item.participant_name}</strong>
                                                        {(item.unread_count > 0)
                                                            .then(|| view! { <span style=tone_style("info")>{item.unread_count}</span> })}
                                                    </div>
                                                    <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                                                        <small>{format!("{} | {}", item.participant_initials, item.load_leg_code)}</small>
                                                        {item.presence_label.clone().map(|label| {
                                                            let tone = item.presence_tone.clone().unwrap_or_else(|| "secondary".into());
                                                            view! { <span style=tone_style(&tone)>{label}</span> }
                                                        })}
                                                    </div>
                                                    <span>{item.last_message_preview}</span>
                                                    <small>{item.last_seen_label}</small>
                                                </button>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                })
                                .unwrap_or_else(|| view! { <p style="margin:0;">"No conversations are available for this account yet."</p> }.into_any())
                        }
                    }}
                </aside>

                <section style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:1rem;">
                    {move || screen.get().map(|value| view! {
                        <>
                            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                <div style="display:grid;gap:0.35rem;">
                                    <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                                        <strong>{value.active_participant.clone()}</strong>
                                        {value.active_participant_presence_label.clone().map(|label| {
                                            let tone = value.active_participant_presence_tone.clone().unwrap_or_else(|| "secondary".into());
                                            view! { <span style=tone_style(&tone)>{label}</span> }
                                        })}
                                    </div>
                                    <p style="margin:0;">{format!("Load {}", value.active_load_leg.clone())}</p>
                                    {value.active_participant_last_read_label.clone().map(|label| view! {
                                        <small style="color:#475569;">{label}</small>
                                    })}
                                </div>
                                <span style=tone_style(&value.smart_offer_tone)>{value.smart_offer_label.clone()}</span>
                            </div>

                            <div style="display:grid;gap:0.75rem;padding:0.5rem 0;min-height:220px;">
                                {value
                                    .messages
                                    .into_iter()
                                    .map(|message| {
                                        let bubble_style = if message.direction == "outgoing" {
                                            "margin-left:auto;max-width:70%;padding:0.8rem 1rem;border-radius:1rem;background:#111827;color:white;display:grid;gap:0.35rem;"
                                        } else {
                                            "max-width:70%;padding:0.8rem 1rem;border-radius:1rem;background:#f3f4f6;color:#111827;display:grid;gap:0.35rem;"
                                        };
                                        view! {
                                            <div style=bubble_style>
                                                <strong>{message.author_name}</strong>
                                                <span>{message.body}</span>
                                                <div style="display:flex;justify-content:space-between;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                                                    <small>{message.sent_at_label}</small>
                                                    {message.receipt_label.clone().map(|label| {
                                                        let tone = message.receipt_tone.clone().unwrap_or_else(|| "secondary".into());
                                                        view! { <span style=tone_style(&tone)>{label}</span> }
                                                    })}
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>

                            <form on:submit=send_message_action style="display:grid;gap:0.6rem;">
                                <textarea
                                    prop:value=move || message_body.get()
                                    on:input=move |ev| message_body.set(event_target_value(&ev))
                                    placeholder="Send an update to the active conversation"
                                    rows="4"
                                    style="padding:0.85rem 1rem;border:1px solid #d1d5db;border-radius:0.9rem;resize:vertical;"
                                />
                                <div style="display:flex;justify-content:flex-end;">
                                    <button
                                        type="submit"
                                        style="padding:0.65rem 1rem;border-radius:0.85rem;background:#111827;color:white;border:none;cursor:pointer;"
                                        disabled=move || is_sending.get()
                                    >
                                        {move || if is_sending.get() { "Sending..." } else { "Send message" }}
                                    </button>
                                </div>
                            </form>

                            <div style="display:grid;gap:0.75rem;">
                                <strong>"Offer history"</strong>
                                {value
                                    .offers
                                    .into_iter()
                                    .map(|offer| {
                                        let offer_id = offer.offer_id;
                                        let is_reviewing = Signal::derive(move || pending_offer_id.get() == Some(offer_id));
                                        let can_review_this_offer = can_review_offers.get() && offer.can_accept;
                                        view! {
                                            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;padding:0.8rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;flex-wrap:wrap;">
                                                <div>
                                                    <strong>{offer.amount_label}</strong>
                                                    <div><small>{offer.created_at_label}</small></div>
                                                </div>
                                                <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                                                    <span style=tone_style(&offer.status_tone)>{offer.status_label}</span>
                                                    {can_review_this_offer.then(|| view! {
                                                        <>
                                                            <button
                                                                type="button"
                                                                style="padding:0.5rem 0.8rem;border-radius:0.75rem;border:1px solid #0f766e;background:#ecfdf5;color:#0f766e;cursor:pointer;"
                                                                disabled=move || is_reviewing.get()
                                                                on:click=move |_| review_offer_action(offer_id, OfferReviewDecision::Accept)
                                                            >
                                                                "Accept"
                                                            </button>
                                                            <button
                                                                type="button"
                                                                style="padding:0.5rem 0.8rem;border-radius:0.75rem;border:1px solid #be123c;background:#fff1f2;color:#be123c;cursor:pointer;"
                                                                disabled=move || is_reviewing.get()
                                                                on:click=move |_| review_offer_action(offer_id, OfferReviewDecision::Decline)
                                                            >
                                                                {move || if is_reviewing.get() { "Working..." } else { "Decline" }}
                                                            </button>
                                                        </>
                                                    })}
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>

                            <section style="display:grid;gap:0.35rem;">
                                {value
                                    .notes
                                    .into_iter()
                                    .map(|note| view! { <p style="margin:0;">{note}</p> })
                                    .collect_view()}
                            </section>
                        </>
                    })}
                </section>
            </section>
        </article>
    }
}
