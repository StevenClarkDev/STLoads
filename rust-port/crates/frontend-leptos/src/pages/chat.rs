use futures_util::future::AbortHandle;
use leptos::{ev, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use crate::{
    api, realtime,
    session::{self, use_auth},
};
use shared::{
    BookNowRequest, CarrierCancellationRequest, ChatSendMessageRequest, ChatWorkspaceScreen,
    CreateCounterofferRequest, CreateTenderInviteRequest, OfferReviewDecision, OfferReviewRequest,
    RealtimeEventKind, RealtimeTopic, RespondCounterofferRequest, RespondTenderInviteRequest,
    SubmitOfferRequest,
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
    let offer_amount = RwSignal::new(String::new());
    let counter_offer_id = RwSignal::new(String::new());
    let counter_amount = RwSignal::new(String::new());
    let counter_response_id = RwSignal::new(String::new());
    let tender_carrier_profile_id = RwSignal::new(String::new());
    let tender_response_id = RwSignal::new(String::new());
    let book_amount = RwSignal::new(String::new());
    let cancellation_award_id = RwSignal::new(String::new());
    let cancellation_reason = RwSignal::new(String::new());
    let marketplace_action_loading = RwSignal::new(false);
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

    let run_marketplace_action = move |label: &'static str, action: MarketplaceUiAction| {
        let Some(posting_id) = screen.get().and_then(|value| value.active_posting_id) else {
            action_message.set(Some(
                "No STLoads posting is linked to this conversation yet.".into(),
            ));
            return;
        };
        marketplace_action_loading.set(true);
        action_message.set(None);
        let auth = auth.clone();
        spawn_local(async move {
            let result = match action {
                MarketplaceUiAction::SubmitOffer { amount } => {
                    api::submit_marketplace_offer(
                        posting_id,
                        &SubmitOfferRequest {
                            amount,
                            currency: Some("USD".into()),
                            message: Some("Submitted from the Rust marketplace chat.".into()),
                            idempotency_key: None,
                        },
                    )
                    .await
                }
                MarketplaceUiAction::CreateCounteroffer { offer_id, amount } => {
                    api::create_marketplace_counteroffer(
                        offer_id,
                        &CreateCounterofferRequest {
                            amount,
                            currency: Some("USD".into()),
                            message: Some("Counteroffer from the Rust marketplace chat.".into()),
                            from_party_type: Some("shipper".into()),
                            to_party_type: Some("carrier".into()),
                        },
                    )
                    .await
                }
                MarketplaceUiAction::RespondCounteroffer {
                    counteroffer_id,
                    decision,
                } => {
                    api::respond_marketplace_counteroffer(
                        counteroffer_id,
                        &RespondCounterofferRequest {
                            decision,
                            note: Some("Counteroffer response from Rust chat.".into()),
                        },
                    )
                    .await
                }
                MarketplaceUiAction::CreateTender { carrier_profile_id } => {
                    api::create_marketplace_tender(
                        posting_id,
                        &CreateTenderInviteRequest {
                            carrier_profile_id,
                            tender_type: Some("direct".into()),
                            expires_minutes: Some(120),
                        },
                    )
                    .await
                }
                MarketplaceUiAction::RespondTender {
                    invite_id,
                    decision,
                } => {
                    api::respond_marketplace_tender(
                        invite_id,
                        &RespondTenderInviteRequest {
                            decision,
                            note: Some("Tender response from Rust chat.".into()),
                        },
                    )
                    .await
                }
                MarketplaceUiAction::BookNow { amount } => {
                    api::book_marketplace_posting(
                        posting_id,
                        &BookNowRequest {
                            carrier_profile_id: None,
                            offer_id: None,
                            tender_id: None,
                            amount,
                            currency: Some("USD".into()),
                            terms_accepted: true,
                            idempotency_key: None,
                        },
                    )
                    .await
                }
                MarketplaceUiAction::Cancel {
                    booking_award_id,
                    reason,
                } => {
                    api::request_marketplace_cancellation(
                        posting_id,
                        &CarrierCancellationRequest {
                            booking_award_id,
                            reason_code: "carrier_request".into(),
                            reason_detail: Some(reason),
                        },
                    )
                    .await
                }
            };

            match result {
                Ok(response) => {
                    action_message.set(Some(format!("{}: {}", label, response.message)));
                    if response.success {
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
            marketplace_action_loading.set(false);
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

                            <section style="display:grid;gap:0.75rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fcfcfb;">
                                <strong>"Marketplace actions"</strong>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Offer amount"</span>
                                        <input type="number" step="0.01" prop:value=move || offer_amount.get() on:input=move |ev| offer_amount.set(event_target_value(&ev)) />
                                        <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                            if let Some(amount) = parse_money(&offer_amount.get()) {
                                                run_marketplace_action("Offer", MarketplaceUiAction::SubmitOffer { amount });
                                            } else {
                                                action_message.set(Some("Enter a valid offer amount.".into()));
                                            }
                                        }>"Submit offer"</button>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Book-now amount"</span>
                                        <input type="number" step="0.01" prop:value=move || book_amount.get() on:input=move |ev| book_amount.set(event_target_value(&ev)) />
                                        <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                            run_marketplace_action("Book-now", MarketplaceUiAction::BookNow { amount: parse_money(&book_amount.get()) });
                                        }>"Book now"</button>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Counteroffer"</span>
                                        <input type="number" placeholder="Offer ID" prop:value=move || counter_offer_id.get() on:input=move |ev| counter_offer_id.set(event_target_value(&ev)) />
                                        <input type="number" step="0.01" placeholder="Amount" prop:value=move || counter_amount.get() on:input=move |ev| counter_amount.set(event_target_value(&ev)) />
                                        <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                            match (parse_id(&counter_offer_id.get()), parse_money(&counter_amount.get())) {
                                                (Some(offer_id), Some(amount)) => run_marketplace_action("Counteroffer", MarketplaceUiAction::CreateCounteroffer { offer_id, amount }),
                                                _ => action_message.set(Some("Enter a valid offer ID and counter amount.".into())),
                                            }
                                        }>"Create counter"</button>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Counter response"</span>
                                        <input type="number" placeholder="Counteroffer ID" prop:value=move || counter_response_id.get() on:input=move |ev| counter_response_id.set(event_target_value(&ev)) />
                                        <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                            <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                                if let Some(counteroffer_id) = parse_id(&counter_response_id.get()) {
                                                    run_marketplace_action("Counter accepted", MarketplaceUiAction::RespondCounteroffer { counteroffer_id, decision: "accept".into() });
                                                }
                                            }>"Accept"</button>
                                            <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                                if let Some(counteroffer_id) = parse_id(&counter_response_id.get()) {
                                                    run_marketplace_action("Counter rejected", MarketplaceUiAction::RespondCounteroffer { counteroffer_id, decision: "reject".into() });
                                                }
                                            }>"Reject"</button>
                                        </div>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Tender carrier profile"</span>
                                        <input type="number" prop:value=move || tender_carrier_profile_id.get() on:input=move |ev| tender_carrier_profile_id.set(event_target_value(&ev)) />
                                        <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                            if let Some(carrier_profile_id) = parse_signed_id(&tender_carrier_profile_id.get()) {
                                                run_marketplace_action("Tender", MarketplaceUiAction::CreateTender { carrier_profile_id });
                                            } else {
                                                action_message.set(Some("Enter a valid carrier profile ID.".into()));
                                            }
                                        }>"Create tender"</button>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Tender response"</span>
                                        <input type="number" placeholder="Invite ID" prop:value=move || tender_response_id.get() on:input=move |ev| tender_response_id.set(event_target_value(&ev)) />
                                        <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                            <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                                if let Some(invite_id) = parse_id(&tender_response_id.get()) {
                                                    run_marketplace_action("Tender accepted", MarketplaceUiAction::RespondTender { invite_id, decision: "accept".into() });
                                                }
                                            }>"Accept"</button>
                                            <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                                if let Some(invite_id) = parse_id(&tender_response_id.get()) {
                                                    run_marketplace_action("Tender declined", MarketplaceUiAction::RespondTender { invite_id, decision: "reject".into() });
                                                }
                                            }>"Decline"</button>
                                        </div>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Cancellation"</span>
                                        <input type="number" placeholder="Award ID" prop:value=move || cancellation_award_id.get() on:input=move |ev| cancellation_award_id.set(event_target_value(&ev)) />
                                        <input type="text" placeholder="Reason" prop:value=move || cancellation_reason.get() on:input=move |ev| cancellation_reason.set(event_target_value(&ev)) />
                                        <button type="button" disabled=move || marketplace_action_loading.get() on:click=move |_| {
                                            run_marketplace_action("Cancellation", MarketplaceUiAction::Cancel {
                                                booking_award_id: parse_signed_id(&cancellation_award_id.get()),
                                                reason: optional_reason(&cancellation_reason.get()),
                                            });
                                        }>"Request cancel"</button>
                                    </label>
                                </div>
                            </section>

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

#[derive(Clone)]
enum MarketplaceUiAction {
    SubmitOffer {
        amount: f64,
    },
    CreateCounteroffer {
        offer_id: u64,
        amount: f64,
    },
    RespondCounteroffer {
        counteroffer_id: u64,
        decision: String,
    },
    CreateTender {
        carrier_profile_id: i64,
    },
    RespondTender {
        invite_id: u64,
        decision: String,
    },
    BookNow {
        amount: Option<f64>,
    },
    Cancel {
        booking_award_id: Option<i64>,
        reason: String,
    },
}

fn parse_money(value: &str) -> Option<f64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|amount| *amount > 0.0)
}

fn parse_id(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok().filter(|id| *id > 0)
}

fn parse_signed_id(value: &str) -> Option<i64> {
    value.trim().parse::<i64>().ok().filter(|id| *id > 0)
}

fn optional_reason(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "Carrier requested cancellation from Rust chat.".into()
    } else {
        trimmed.into()
    }
}
