use leptos::prelude::*;
use leptos_router::components::A;

use crate::{
    components::{UiBadge, UiPageHeader, UiPanel, UiStatusPill, UiToolbar},
    session::{self, use_auth},
};

#[derive(Clone, Copy)]
struct DashboardCard {
    label: &'static str,
    href: &'static str,
    icon: &'static str,
    status: &'static str,
    tone: &'static str,
    copy: &'static str,
}

#[derive(Clone, Copy)]
struct DashboardMetric {
    label: &'static str,
    value: &'static str,
    tone: &'static str,
}

#[derive(Clone, Copy)]
struct DashboardModel {
    eyebrow: &'static str,
    title: &'static str,
    summary: &'static str,
    metrics: &'static [DashboardMetric],
    primary_cards: &'static [DashboardCard],
    secondary_cards: &'static [DashboardCard],
}

const ADMIN_METRICS: &[DashboardMetric] = &[
    DashboardMetric {
        label: "Tenant controls",
        value: "Ready",
        tone: "success",
    },
    DashboardMetric {
        label: "Risk queues",
        value: "Open",
        tone: "warning",
    },
    DashboardMetric {
        label: "Audit trail",
        value: "Live",
        tone: "info",
    },
];

const ADMIN_PRIMARY: &[DashboardCard] = &[
    DashboardCard {
        label: "User directory",
        href: "/admin/users",
        icon: "fas fa-users",
        status: "Admin",
        tone: "info",
        copy: "Review users, roles, onboarding, and organization membership.",
    },
    DashboardCard {
        label: "Enterprise identity",
        href: "/admin/identity",
        icon: "fas fa-id-card",
        status: "SSO",
        tone: "success",
        copy: "Manage tenant domains, SSO routing, and SCIM provisioning.",
    },
    DashboardCard {
        label: "Audit search",
        href: "/admin/audit",
        icon: "fas fa-search",
        status: "Evidence",
        tone: "secondary",
        copy: "Find support, security, payment, and admin actions quickly.",
    },
];

const ADMIN_SECONDARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Access reviews",
        href: "/admin/access-reviews",
        icon: "fas fa-clipboard-check",
        status: "Review",
        tone: "warning",
        copy: "Run recertification and privileged-access decisions.",
    },
    DashboardCard {
        label: "Integrations",
        href: "/admin/integrations",
        icon: "fas fa-plug",
        status: "API",
        tone: "info",
        copy: "Monitor API lifecycle, sandbox governance, EDI, and TMS setup.",
    },
];

const SHIPPER_METRICS: &[DashboardMetric] = &[
    DashboardMetric {
        label: "Post freight",
        value: "Ready",
        tone: "success",
    },
    DashboardMetric {
        label: "Tracking",
        value: "Visible",
        tone: "info",
    },
    DashboardMetric {
        label: "Documents",
        value: "Governed",
        tone: "secondary",
    },
];

const SHIPPER_PRIMARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Create load",
        href: "/loads/new",
        icon: "fas fa-plus-circle",
        status: "Post",
        tone: "success",
        copy: "Build a shipment with stops, pricing, documents, and carrier rules.",
    },
    DashboardCard {
        label: "Load board",
        href: "/loads",
        icon: "fas fa-truck",
        status: "Loads",
        tone: "info",
        copy: "Track posted, booked, in-transit, and closeout freight.",
    },
    DashboardCard {
        label: "Messages",
        href: "/chat",
        icon: "fas fa-comments",
        status: "Live",
        tone: "secondary",
        copy: "Coordinate with carriers, brokers, and operations.",
    },
];

const SHIPPER_SECONDARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Notifications",
        href: "/notifications",
        icon: "fas fa-bell",
        status: "Events",
        tone: "warning",
        copy: "Review tender, tracking, document, and finance alerts.",
    },
    DashboardCard {
        label: "Profile",
        href: "/profile",
        icon: "fas fa-id-badge",
        status: "Account",
        tone: "secondary",
        copy: "Maintain company details, required records, and contacts.",
    },
];

const CARRIER_METRICS: &[DashboardMetric] = &[
    DashboardMetric {
        label: "Available freight",
        value: "Open",
        tone: "success",
    },
    DashboardMetric {
        label: "Execution",
        value: "Mobile",
        tone: "info",
    },
    DashboardMetric {
        label: "Payout readiness",
        value: "Gated",
        tone: "warning",
    },
];

const CARRIER_PRIMARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Find loads",
        href: "/loads",
        icon: "fas fa-route",
        status: "Board",
        tone: "success",
        copy: "Review eligible freight, offers, and carrier network rules.",
    },
    DashboardCard {
        label: "Execution work",
        href: "/loads",
        icon: "fas fa-mobile-screen",
        status: "Driver",
        tone: "info",
        copy: "Open booked loads and continue pickup, delivery, POD, and closeout.",
    },
    DashboardCard {
        label: "Carrier profile",
        href: "/profile",
        icon: "fas fa-shield-alt",
        status: "Compliance",
        tone: "warning",
        copy: "Update documents, capacity, equipment, insurance, and authority data.",
    },
];

const CARRIER_SECONDARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Notifications",
        href: "/notifications",
        icon: "fas fa-bell",
        status: "Alerts",
        tone: "warning",
        copy: "Watch tender, tracking, POD, payment, and compliance events.",
    },
    DashboardCard {
        label: "Messages",
        href: "/chat",
        icon: "fas fa-comments",
        status: "Live",
        tone: "secondary",
        copy: "Coordinate dispatch, appointment, and document questions.",
    },
];

const BROKER_METRICS: &[DashboardMetric] = &[
    DashboardMetric {
        label: "Quote desk",
        value: "Open",
        tone: "success",
    },
    DashboardMetric {
        label: "Tendering",
        value: "Active",
        tone: "info",
    },
    DashboardMetric {
        label: "Margin guard",
        value: "Watched",
        tone: "warning",
    },
];

const BROKER_PRIMARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Quote desk",
        href: "/desk/quote",
        icon: "fas fa-columns",
        status: "Desk",
        tone: "success",
        copy: "Move freight through quote, tender, booking, execution, and exception lanes.",
    },
    DashboardCard {
        label: "Create load",
        href: "/loads/new",
        icon: "fas fa-plus-circle",
        status: "Post",
        tone: "info",
        copy: "Create customer freight with pricing, lane guide, and carrier controls.",
    },
    DashboardCard {
        label: "Load board",
        href: "/loads",
        icon: "fas fa-truck",
        status: "Board",
        tone: "secondary",
        copy: "Compare marketplace activity, carrier offers, and booked work.",
    },
];

const BROKER_SECONDARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Notifications",
        href: "/notifications",
        icon: "fas fa-bell",
        status: "Events",
        tone: "warning",
        copy: "Watch tender deadlines, tracking drift, documents, and finance holds.",
    },
    DashboardCard {
        label: "Messages",
        href: "/chat",
        icon: "fas fa-comments",
        status: "Live",
        tone: "secondary",
        copy: "Coordinate customers, carriers, and internal dispatch.",
    },
];

const FINANCE_METRICS: &[DashboardMetric] = &[
    DashboardMetric {
        label: "Approvals",
        value: "Queued",
        tone: "warning",
    },
    DashboardMetric {
        label: "Invoices",
        value: "Ready",
        tone: "info",
    },
    DashboardMetric {
        label: "Payout risk",
        value: "Held",
        tone: "danger",
    },
];

const FINANCE_PRIMARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Finance operations",
        href: "/admin/payments",
        icon: "fas fa-dollar-sign",
        status: "Payments",
        tone: "warning",
        copy: "Approve releases, review holds, invoices, settlements, and payout changes.",
    },
    DashboardCard {
        label: "Audit search",
        href: "/admin/audit",
        icon: "fas fa-search-dollar",
        status: "Evidence",
        tone: "secondary",
        copy: "Trace payment, refund, payout, and finance approval events.",
    },
    DashboardCard {
        label: "Notifications",
        href: "/notifications",
        icon: "fas fa-bell",
        status: "Alerts",
        tone: "info",
        copy: "Review payment holds, payout releases, and deliverability signals.",
    },
];

const FINANCE_SECONDARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Support search",
        href: "/admin/support",
        icon: "fas fa-headset",
        status: "Context",
        tone: "secondary",
        copy: "Find customer, carrier, load, document, and payment context.",
    },
    DashboardCard {
        label: "Access reviews",
        href: "/admin/access-reviews",
        icon: "fas fa-clipboard-check",
        status: "Control",
        tone: "warning",
        copy: "Confirm finance access remains approved and current.",
    },
];

const SUPPORT_METRICS: &[DashboardMetric] = &[
    DashboardMetric {
        label: "Search",
        value: "Scoped",
        tone: "success",
    },
    DashboardMetric {
        label: "Timeline",
        value: "Audited",
        tone: "info",
    },
    DashboardMetric {
        label: "Escalations",
        value: "Routed",
        tone: "warning",
    },
];

const SUPPORT_PRIMARY: &[DashboardCard] = &[
    DashboardCard {
        label: "Support search",
        href: "/admin/support",
        icon: "fas fa-headset",
        status: "Search",
        tone: "success",
        copy: "Find tenant-scoped users, loads, documents, payments, and timeline context.",
    },
    DashboardCard {
        label: "Audit search",
        href: "/admin/audit",
        icon: "fas fa-magnifying-glass",
        status: "Audit",
        tone: "info",
        copy: "Trace support actions, admin actions, and customer-impacting events.",
    },
    DashboardCard {
        label: "Notifications",
        href: "/notifications",
        icon: "fas fa-bell",
        status: "Comms",
        tone: "secondary",
        copy: "Review deliverability, branding, and operational communication status.",
    },
];

const SUPPORT_SECONDARY: &[DashboardCard] = &[
    DashboardCard {
        label: "User directory",
        href: "/admin/users",
        icon: "fas fa-users",
        status: "People",
        tone: "secondary",
        copy: "Open user records with role-safe admin access.",
    },
    DashboardCard {
        label: "Integrations",
        href: "/admin/integrations",
        icon: "fas fa-plug",
        status: "Triage",
        tone: "warning",
        copy: "Check API, TMS, sandbox, webhook, and EDI health context.",
    },
];

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth = use_auth();
    let today = "Tuesday, 26 May 2026";
    let can_admin = Signal::derive(move || session::has_permission(&auth, "access_admin_portal"));
    let can_finance = Signal::derive(move || session::has_permission(&auth, "manage_payments"));

    let dashboard_model = Signal::derive(move || {
        let session = auth.session.get();
        let role_key = session
            .user
            .as_ref()
            .map(|user| {
                user.organization_role_key
                    .clone()
                    .unwrap_or_else(|| user.role_key.clone())
            })
            .unwrap_or_else(|| "guest".into());
        model_for_role(&role_key, can_admin.get(), can_finance.get())
    });

    view! {
        <article class="php-grid">
            {move || {
                let model = dashboard_model.get();
                view! {
                    <UiPageHeader title=model.title eyebrow=model.eyebrow>
                        <p>{model.summary}</p>
                    </UiPageHeader>
                }
            }}

            <UiPanel>
                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                    <div>
                        <h3 style="margin:0;">{move || auth.session.get().user.as_ref().map(|user| format!("Welcome back, {}", user.name)).unwrap_or_else(|| "Welcome back".into())}</h3>
                        <p style="margin:0.25rem 0 0;color:#64748b;">{today}</p>
                    </div>
                    <UiToolbar>
                        <A href="/notifications" attr:class="shell-action secondary">
                            <i class="fas fa-bell"></i>
                            <span>"Notifications"</span>
                        </A>
                        <A href="/profile" attr:class="shell-action secondary">
                            <i class="fas fa-id-badge"></i>
                            <span>"Profile"</span>
                        </A>
                    </UiToolbar>
                </div>

                <section class="dashboard-grid" aria-label="Dashboard metrics">
                    {move || dashboard_model.get().metrics.iter().map(|metric| view! {
                        <div class="dashboard-card" aria-label=metric.label>
                            <UiStatusPill tone=metric.tone>{metric.value}</UiStatusPill>
                            <strong class="dashboard-card-title">{metric.label}</strong>
                        </div>
                    }).collect_view()}
                </section>
            </UiPanel>

            <section class="php-grid columns-2">
                <UiPanel>
                    <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                        <h3 style="margin:0;">"Primary work"</h3>
                        {move || {
                            auth.session.get().user.map(|user| {
                                let role_label = user.role_label;
                                view! { <UiBadge tone="secondary">{role_label}</UiBadge> }
                            })
                        }}
                    </div>
                    <div class="dashboard-grid">
                        {move || dashboard_model.get().primary_cards.iter().copied().map(dashboard_card_view).collect_view()}
                    </div>
                </UiPanel>

                <UiPanel>
                    <h3 style="margin:0;">"Follow-up work"</h3>
                    <div class="dashboard-grid">
                        {move || dashboard_model.get().secondary_cards.iter().copied().map(dashboard_card_view).collect_view()}
                    </div>
                </UiPanel>
            </section>
        </article>
    }
}

fn dashboard_card_view(card: DashboardCard) -> impl IntoView {
    view! {
        <A href=card.href attr:class="dashboard-card">
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;">
                <i class=card.icon aria-hidden="true"></i>
                <UiStatusPill tone=card.tone>{card.status}</UiStatusPill>
            </div>
            <strong class="dashboard-card-title">{card.label}</strong>
            <p class="dashboard-card-copy">{card.copy}</p>
        </A>
    }
}

fn model_for_role(role_key: &str, can_admin: bool, can_finance: bool) -> DashboardModel {
    if role_key == "finance" || can_finance {
        return DashboardModel {
            eyebrow: "Finance",
            title: "Finance Command Center",
            summary: "Payment approvals, invoice settlement, payout risk, and finance evidence.",
            metrics: FINANCE_METRICS,
            primary_cards: FINANCE_PRIMARY,
            secondary_cards: FINANCE_SECONDARY,
        };
    }

    if role_key == "support" {
        return DashboardModel {
            eyebrow: "Support",
            title: "Support Command Center",
            summary: "Scoped search, issue timelines, audit evidence, and customer-impact triage.",
            metrics: SUPPORT_METRICS,
            primary_cards: SUPPORT_PRIMARY,
            secondary_cards: SUPPORT_SECONDARY,
        };
    }

    if can_admin || matches!(role_key, "owner" | "admin") {
        return DashboardModel {
            eyebrow: "Admin",
            title: "Admin Command Center",
            summary: "Tenant operations, identity, access, audit, integrations, and risk controls.",
            metrics: ADMIN_METRICS,
            primary_cards: ADMIN_PRIMARY,
            secondary_cards: ADMIN_SECONDARY,
        };
    }

    match role_key {
        "carrier" | "driver" => DashboardModel {
            eyebrow: "Carrier",
            title: "Carrier Command Center",
            summary: "Available freight, execution tasks, compliance readiness, and payout-critical alerts.",
            metrics: CARRIER_METRICS,
            primary_cards: CARRIER_PRIMARY,
            secondary_cards: CARRIER_SECONDARY,
        },
        "broker" | "freight_forwarder" => DashboardModel {
            eyebrow: "Broker / Forwarder",
            title: "Dispatch Command Center",
            summary: "Quote desk, tendering, booked freight, customer updates, and margin-sensitive work.",
            metrics: BROKER_METRICS,
            primary_cards: BROKER_PRIMARY,
            secondary_cards: BROKER_SECONDARY,
        },
        _ => DashboardModel {
            eyebrow: "Shipper",
            title: "Shipper Command Center",
            summary: "Post freight, track active loads, manage documents, and coordinate carrier work.",
            metrics: SHIPPER_METRICS,
            primary_cards: SHIPPER_PRIMARY,
            secondary_cards: SHIPPER_SECONDARY,
        },
    }
}
