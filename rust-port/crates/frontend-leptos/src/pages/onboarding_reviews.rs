use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use crate::{
    api, document_upload,
    session::{self, use_auth},
};
use shared::{AdminOnboardingReviewScreen, ReviewOnboardingRequest};

use super::admin_guard_view;

#[component]
pub fn OnboardingReviewPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<AdminOnboardingReviewScreen>);
    let loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let action_loading_user_id = RwSignal::new(None::<u64>);
    let refresh_nonce = RwSignal::new(0_u64);

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_users")
    });

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();
        if !auth.session_ready.get() || !auth.session.get().authenticated || !can_view.get() {
            return;
        }

        loading.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::fetch_admin_onboarding_reviews().await {
                Ok(next) => {
                    screen.set(Some(next));
                    feedback.set(None);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    });

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Onboarding Reviews", &["access_admin_portal", "manage_users"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1rem;">
                        <section>
                            <h2>"Onboarding Reviews"</h2>
                            <p>"Approve, reject, or return accounts for revision from the Rust admin surface."</p>
                        </section>

                        {move || feedback.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;white-space:pre-wrap;">
                                {message}
                            </section>
                        })}

                        {move || if loading.get() && screen.get().is_none() {
                            view! { <p>"Loading onboarding review queue from the Rust backend..."</p> }.into_any()
                        } else if let Some(screen_data) = screen.get() {
                            view! {
                                <>
                                    <section style="display:grid;gap:0.35rem;">
                                        <strong>{screen_data.summary}</strong>
                                        {screen_data.notes.into_iter().map(|note| view! { <p style="margin:0;">{note}</p> }).collect_view()}
                                    </section>
                                    <section style="display:grid;gap:1rem;">
                                        {screen_data.users.into_iter().map(|user| {
                                            let feedback = feedback;
                                            let action_loading_user_id = action_loading_user_id;
                                            let refresh_nonce = refresh_nonce;
                                            view! {
                                                <article style="padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fcfcfb;display:grid;gap:0.75rem;">
                                                    <div style="display:flex;justify-content:space-between;gap:1rem;flex-wrap:wrap;align-items:flex-start;">
                                                        <div style="display:grid;gap:0.2rem;">
                                                            <strong>{format!("{} ({})", user.name, user.role_label)}</strong>
                                                            <span>{user.email}</span>
                                                            <small>{format!("{} | Submitted {} | {} document(s)", user.status_label, user.submitted_at_label, user.document_count)}</small>
                                                        </div>
                                                        <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                                            <button
                                                                type="button"
                                                                disabled=move || action_loading_user_id.get() == Some(user.user_id)
                                                                on:click={
                                                                    let user_id = user.user_id;
                                                                    move |_| run_review_action(user_id, "approve", None, feedback, action_loading_user_id, refresh_nonce)
                                                                }
                                                                style="padding:0.55rem 0.8rem;border:none;border-radius:0.75rem;background:#166534;color:white;cursor:pointer;"
                                                            >"Approve"</button>
                                                            <button
                                                                type="button"
                                                                disabled=move || action_loading_user_id.get() == Some(user.user_id)
                                                                on:click={
                                                                    let user_id = user.user_id;
                                                                    move |_| run_review_action(user_id, "revision", Some("Please revise and resubmit from the Rust onboarding flow.".into()), feedback, action_loading_user_id, refresh_nonce)
                                                                }
                                                                style="padding:0.55rem 0.8rem;border:none;border-radius:0.75rem;background:#b45309;color:white;cursor:pointer;"
                                                            >"Request Revision"</button>
                                                            <button
                                                                type="button"
                                                                disabled=move || action_loading_user_id.get() == Some(user.user_id)
                                                                on:click={
                                                                    let user_id = user.user_id;
                                                                    move |_| run_review_action(user_id, "reject", Some("Rejected from the Rust admin review queue.".into()), feedback, action_loading_user_id, refresh_nonce)
                                                                }
                                                                style="padding:0.55rem 0.8rem;border:none;border-radius:0.75rem;background:#be123c;color:white;cursor:pointer;"
                                                            >"Reject"</button>
                                                        </div>
                                                    </div>
                                                    {user.company_name.clone().map(|value| view! { <p style="margin:0;">{format!("Company: {}", value)}</p> })}
                                                    {user.company_address.clone().map(|value| view! { <p style="margin:0;">{format!("Address: {}", value)}</p> })}
                                                    <section style="display:grid;gap:0.4rem;">
                                                        <strong>"KYC Documents"</strong>
                                                        {if user.documents.is_empty() {
                                                            view! { <small>"No KYC documents uploaded yet."</small> }.into_any()
                                                        } else {
                                                            user.documents.into_iter().map(|document| view! {
                                                                <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.65rem 0.8rem;border:1px solid #e5e7eb;border-radius:0.85rem;">
                                                                    <div style="display:grid;gap:0.15rem;">
                                                                        <strong>{document.document_name}</strong>
                                                                        <small>{format!("{} | {}", document.document_type, document.uploaded_at_label)}</small>
                                                                    </div>
                                                                    {document.download_path.clone().map(|path: String| view! {
                                                                        <button type="button" on:click=move |_| {
                                                                            let path = path.clone();
                                                                            spawn_local(async move {
                                                                                let _ = document_upload::open_protected_document(&path).await;
                                                                            });
                                                                        } style="padding:0.45rem 0.7rem;border:1px solid #cbd5e1;border-radius:0.75rem;background:white;cursor:pointer;">"View file"</button>
                                                                    })}
                                                                </div>
                                                            }).collect_view().into_any()
                                                        }}
                                                    </section>
                                                </article>
                                            }
                                        }).collect_view()}
                                    </section>
                                </>
                            }.into_any()
                        } else {
                            view! { <p>"No onboarding review data is available yet."</p> }.into_any()
                        }}
                    </article>
                }.into_any()
            }
        }}
    }
}

fn run_review_action(
    user_id: u64,
    decision: &'static str,
    remarks: Option<String>,
    feedback: RwSignal<Option<String>>,
    action_loading_user_id: RwSignal<Option<u64>>,
    refresh_nonce: RwSignal<u64>,
) {
    action_loading_user_id.set(Some(user_id));
    spawn_local(async move {
        let result = api::review_onboarding_user(
            user_id,
            &ReviewOnboardingRequest {
                decision: decision.into(),
                remarks,
            },
        )
        .await;

        match result {
            Ok(response) => {
                feedback.set(Some(response.message));
                if response.success {
                    refresh_nonce.update(|value| *value += 1);
                }
            }
            Err(error) => feedback.set(Some(error)),
        }

        action_loading_user_id.set(None);
    });
}
