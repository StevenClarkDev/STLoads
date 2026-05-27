use leptos::prelude::*;

use super::shared::{ROW_BORDER_STYLE, TABLE_CELL_STYLE, tone_style};
use crate::api::{ApiLifecycleScreen, SandboxGovernanceScreen};

pub(super) fn render_api_lifecycle(
    lifecycle_screen: RwSignal<Option<ApiLifecycleScreen>>,
) -> impl IntoView {
    view! {
        <section style="display:grid;gap:1rem;">
            <div>
                <h3 style="margin:0;">"API lifecycle"</h3>
                <p style="margin:0.25rem 0 0;color:#64748b;">
                    "Version support, sunset policy, customer notice windows, SDK strategy, and runnable sandbox examples."
                </p>
            </div>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;">
                {move || lifecycle_screen.get().map(|value| {
                    value.policies.into_iter().map(|row| {
                        let status_style = tone_style(&row.release_status);
                        view! {
                            <div style="border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:#ffffff;display:grid;gap:0.5rem;">
                                <strong>{format!("API {}", row.api_version)}</strong>
                                <span style=status_style>{row.release_status}</span>
                                <small>{format!("Released {} | notice {} days", row.released_on, row.minimum_notice_days)}</small>
                                <small>{row.sdk_strategy}</small>
                                <small>{row.emergency_breaking_change_policy}</small>
                            </div>
                        }
                    }).collect_view()
                })}
            </div>
            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                <table style="width:100%;border-collapse:collapse;min-width:720px;">
                    <thead>
                        <tr style="text-align:left;background:#f8fafc;">
                            <th style=TABLE_CELL_STYLE>"Example"</th>
                            <th style=TABLE_CELL_STYLE>"Surface"</th>
                            <th style=TABLE_CELL_STYLE>"Method"</th>
                            <th style=TABLE_CELL_STYLE>"Path"</th>
                            <th style=TABLE_CELL_STYLE>"Sandbox"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || lifecycle_screen.get().map(|value| {
                            value.examples.into_iter().map(|row| view! {
                                <tr style=ROW_BORDER_STYLE>
                                    <td style=TABLE_CELL_STYLE>{row.example_key}</td>
                                    <td style=TABLE_CELL_STYLE>{row.surface}</td>
                                    <td style=TABLE_CELL_STYLE><code>{row.method}</code></td>
                                    <td style=TABLE_CELL_STYLE><code>{row.path}</code></td>
                                    <td style=TABLE_CELL_STYLE>{if row.sandbox_runnable { "Runnable" } else { "Reference" }}</td>
                                </tr>
                            }).collect_view()
                        })}
                    </tbody>
                </table>
            </div>
            <div style="display:grid;gap:0.5rem;">
                {move || lifecycle_screen.get().map(|value| {
                    value.upgrade_paths.into_iter().map(|path| view! {
                        <small style="color:#475569;">{path}</small>
                    }).collect_view()
                })}
            </div>
        </section>
    }
}

pub(super) fn render_sandbox_governance(
    sandbox_screen: RwSignal<Option<SandboxGovernanceScreen>>,
    sandbox_reset_reason: RwSignal<String>,
    pending_action: RwSignal<Option<String>>,
    queue_reset: impl Fn(u64) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <section style="display:grid;gap:1rem;">
            <div>
                <h3 style="margin:0;">"Sandbox and demo governance"</h3>
                <p style="margin:0.25rem 0 0;color:#64748b;">
                    "Synthetic demo tenants, sandbox reset jobs, and hard blocks against production money movement, TMS pushes, and live notifications."
                </p>
            </div>
            <label style="display:grid;gap:0.35rem;max-width:520px;">
                <span>"Reset reason"</span>
                <input
                    type="text"
                    prop:value=move || sandbox_reset_reason.get()
                    on:input=move |ev| sandbox_reset_reason.set(event_target_value(&ev))
                    placeholder="Refresh demo data for customer integration testing"
                />
            </label>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1rem;">
                {move || sandbox_screen.get().map(|value| {
                    value.environments.into_iter().map(|row| {
                        let status_style = tone_style(&row.reset_status);
                        let environment_id = row.id;
                        view! {
                            <div style="border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:#ffffff;display:grid;gap:0.5rem;">
                                <strong>{row.display_name}</strong>
                                <code>{row.base_url}</code>
                                <span style=status_style>{row.reset_status}</span>
                                <small>{format!("Dataset {} | {}", row.seeded_dataset_version, row.data_classification)}</small>
                                <small>{format!("Payment blocked: {} | TMS blocked: {} | Notifications blocked: {}", row.production_payment_blocked, row.production_tms_push_blocked, row.production_notification_blocked)}</small>
                                <button type="button" class="shell-action secondary" on:click=move |_| queue_reset(environment_id) disabled=move || pending_action.get().is_some()>
                                    <i class="fas fa-undo"></i>
                                    <span>"Queue reset"</span>
                                </button>
                            </div>
                        }
                    }).collect_view()
                })}
            </div>
            <div style="display:grid;gap:0.5rem;">
                {move || sandbox_screen.get().map(|value| {
                    value.policy_notes.into_iter().map(|note| view! {
                        <small style="color:#475569;">{note}</small>
                    }).collect_view()
                })}
            </div>
            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                <table style="width:100%;border-collapse:collapse;min-width:620px;">
                    <thead>
                        <tr style="text-align:left;background:#f8fafc;">
                            <th style=TABLE_CELL_STYLE>"Reset job"</th>
                            <th style=TABLE_CELL_STYLE>"Status"</th>
                            <th style=TABLE_CELL_STYLE>"Reason"</th>
                            <th style=TABLE_CELL_STYLE>"Created"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || sandbox_screen.get().map(|value| {
                            value.reset_jobs.into_iter().map(|row| {
                                let status_style = tone_style(&row.job_status);
                                view! {
                                    <tr style=ROW_BORDER_STYLE>
                                        <td style=TABLE_CELL_STYLE>{format!("#{}", row.id)}</td>
                                        <td style=TABLE_CELL_STYLE><span style=status_style>{row.job_status}</span></td>
                                        <td style=TABLE_CELL_STYLE>{row.reset_reason.unwrap_or_else(|| "-".into())}</td>
                                        <td style=TABLE_CELL_STYLE>{row.created_at}</td>
                                    </tr>
                                }
                            }).collect_view()
                        })}
                    </tbody>
                </table>
            </div>
        </section>
    }
}
