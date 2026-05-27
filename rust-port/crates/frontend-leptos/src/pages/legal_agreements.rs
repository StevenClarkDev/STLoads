use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use shared::{AcceptLegalAgreementRequest, LegalAgreementScreen};

use crate::{
    api,
    session::{self, use_auth},
};

#[component]
pub fn LegalAgreementsPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<LegalAgreementScreen>);
    let feedback = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);

    let load_screen = move || {
        loading.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::fetch_legal_agreement_screen().await {
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
    };

    let accept = move |agreement_key: String, accept_for_organization: bool| {
        loading.set(true);
        let auth = auth;
        let payload = AcceptLegalAgreementRequest {
            agreement_key,
            accept_for_organization,
        };
        spawn_local(async move {
            match api::accept_legal_agreement(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
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
    };

    Effect::new(move |_| {
        if screen.get_untracked().is_none() {
            load_screen();
        }
    });

    view! {
        {move || {
            if !auth.session.get().authenticated {
                view! {
                    <main class="php-page-shell">
                        <section class="php-alert php-alert-warning">"Sign in before reviewing legal agreements."</section>
                    </main>
                }.into_any()
            } else {
                view! {
                    <main class="php-page-shell">
                        <section class="php-dashboard-header">
                            <div>
                                <p class="eyebrow">"Legal"</p>
                                <h1>"Legal Agreements"</h1>
                                <p>"Review required platform, privacy, tracking, payment, and operating terms before continuing freight workflows."</p>
                            </div>
                            <button type="button" class="php-btn php-btn-light" on:click=move |_| load_screen() disabled=move || loading.get()>
                                {move || if loading.get() { "Refreshing..." } else { "Refresh" }}
                            </button>
                        </section>

                        {move || feedback.get().map(|message| view! {
                            <section class="php-alert php-alert-warning">{message}</section>
                        })}

                        {move || screen.get().map(|screen| {
                            let missing_count = screen.missing_required.len();
                            view! {
                                <section class="php-card" style="border-radius:8px;">
                                    <p class="eyebrow">"Required Now"</p>
                                    <h2 style="margin:0;">{format!("{} agreements pending", missing_count)}</h2>
                                    <div style="display:grid;gap:0.75rem;margin-top:1rem;">
                                        <For
                                            each=move || screen.missing_required.clone()
                                            key=|item| item.id
                                            children=move |item| {
                                                let accept_for_org = item.requires_organization_acceptance;
                                                let agreement_key = item.agreement_key.clone();
                                                view! {
                                                    <article style="border:1px solid #e5e7eb;border-radius:8px;padding:0.9rem;display:grid;gap:0.45rem;">
                                                        <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                                            <div>
                                                                <h3 style="margin:0;font-size:1rem;">{item.title}</h3>
                                                                <p style="margin:0.25rem 0 0;color:#64748b;">{format!("{} version {}", item.agreement_key, item.version)}</p>
                                                                <p style="margin:0.25rem 0 0;color:#64748b;">{format!("Effective {}", item.effective_at)}</p>
                                                            </div>
                                                            <button
                                                                type="button"
                                                                class="php-btn php-btn-primary"
                                                                disabled=move || loading.get()
                                                                on:click=move |_| accept(agreement_key.clone(), accept_for_org)
                                                            >
                                                                "Accept"
                                                            </button>
                                                        </div>
                                                        {item.document_uri.map(|href| view! {
                                                            <a href=href target="_blank" rel="noreferrer">"Open agreement document"</a>
                                                        })}
                                                    </article>
                                                }
                                            }
                                        />
                                    </div>
                                    {if missing_count == 0 {
                                        view! { <p style="margin:1rem 0 0;color:#16a34a;">"All required legal agreements are accepted for the current account context."</p> }.into_any()
                                    } else {
                                        view! { <p style="margin:1rem 0 0;color:#64748b;">"Acceptance is stored with signer, timestamp, request, device evidence, and audit linkage."</p> }.into_any()
                                    }}
                                </section>
                            }
                        })}

                        {move || screen.get().map(|screen| view! {
                            <section class="php-card" style="border-radius:8px;overflow:auto;">
                                <p class="eyebrow">"Proofs"</p>
                                <h2 style="margin:0;">"Acceptance History"</h2>
                                <table class="php-table" style="margin-top:1rem;min-width:760px;">
                                    <thead>
                                        <tr>
                                            <th>"Agreement"</th>
                                            <th>"Version"</th>
                                            <th>"Signer"</th>
                                            <th>"Accepted"</th>
                                            <th>"Audit"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || screen.acceptance_proofs.clone()
                                            key=|proof| proof.id
                                            children=move |proof| view! {
                                                <tr>
                                                    <td>{proof.agreement_key}</td>
                                                    <td>{proof.version}</td>
                                                    <td>{format!("{} <{}>", proof.signer_name, proof.signer_email)}</td>
                                                    <td>{proof.accepted_at}</td>
                                                    <td>{proof.audit_event_id.map(|id| id.to_string()).unwrap_or_else(|| "-".into())}</td>
                                                </tr>
                                            }
                                        />
                                    </tbody>
                                </table>
                                {screen.notes.into_iter().map(|note| view! {
                                    <p style="margin:0.75rem 0 0;color:#64748b;font-size:0.88rem;">{note}</p>
                                }).collect_view()}
                            </section>
                        })}
                    </main>
                }.into_any()
            }
        }}
    }
}
