use leptos::prelude::*;

fn status_tone_class(tone: &str) -> &'static str {
    match tone {
        "success" | "active" | "approved" | "verified" => "success",
        "warning" | "pending" | "deferred" => "warning",
        "danger" | "failed" | "rejected" | "urgent" => "danger",
        "info" | "selected" => "info",
        _ => "secondary",
    }
}

#[component]
pub fn UiPageHeader(
    title: &'static str,
    eyebrow: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="ui-page-header">
            <div class="ui-page-header-copy">
                <p class="ui-eyebrow">{eyebrow}</p>
                <h2>{title}</h2>
                <div class="ui-page-header-body">{children()}</div>
            </div>
        </section>
    }
}

#[component]
pub fn UiPanel(children: Children) -> impl IntoView {
    view! {
        <section class="ui-panel">
            {children()}
        </section>
    }
}

#[component]
pub fn UiToolbar(children: Children) -> impl IntoView {
    view! {
        <div class="ui-toolbar" role="toolbar">
            {children()}
        </div>
    }
}

#[component]
pub fn UiFilterBar(children: Children) -> impl IntoView {
    view! {
        <section class="ui-filter-bar" aria-label="Filters">
            {children()}
        </section>
    }
}

#[component]
pub fn UiTableShell(children: Children) -> impl IntoView {
    view! {
        <div class="ui-table-shell">
            {children()}
        </div>
    }
}

#[component]
pub fn UiStatusPill(tone: &'static str, children: Children) -> impl IntoView {
    let class = format!("ui-status-pill {}", status_tone_class(tone));
    view! {
        <span class=class>{children()}</span>
    }
}

#[component]
pub fn UiBadge(tone: &'static str, children: Children) -> impl IntoView {
    let class = format!("ui-badge {}", status_tone_class(tone));
    view! {
        <span class=class>{children()}</span>
    }
}

#[component]
pub fn UiToast(tone: &'static str, children: Children) -> impl IntoView {
    let class = format!("ui-toast {}", status_tone_class(tone));
    view! {
        <section class=class role="status" aria-live="polite">
            {children()}
        </section>
    }
}

#[component]
pub fn UiModal(title: &'static str, open: bool, children: Children) -> impl IntoView {
    view! {
        {open.then(|| view! {
            <section class="ui-modal-backdrop" role="presentation">
                <div class="ui-modal" role="dialog" aria-modal="true" aria-label=title tabindex="-1">
                    <header class="ui-modal-header">
                        <h3>{title}</h3>
                    </header>
                    <div class="ui-modal-body">{children()}</div>
                </div>
            </section>
        })}
    }
}

#[component]
pub fn UiDrawer(title: &'static str, open: bool, children: Children) -> impl IntoView {
    view! {
        {open.then(|| view! {
            <aside class="ui-drawer" role="dialog" aria-modal="true" aria-label=title tabindex="-1">
                <header class="ui-drawer-header">
                    <h3>{title}</h3>
                </header>
                <div class="ui-drawer-body">{children()}</div>
            </aside>
        })}
    }
}

#[component]
pub fn UiTimeline(children: Children) -> impl IntoView {
    view! {
        <ol class="ui-timeline">
            {children()}
        </ol>
    }
}

#[component]
pub fn UiFileUploadFrame(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <section class="ui-file-upload" aria-label=label>
            {children()}
        </section>
    }
}

#[component]
pub fn UiMapPanel(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <section class="ui-map-panel" aria-label=label>
            {children()}
        </section>
    }
}

#[component]
pub fn UiMoneyInput(label: &'static str, value: String, children: Children) -> impl IntoView {
    view! {
        <label class="ui-money-input">
            <span>{label}</span>
            <div class="ui-money-input-control">
                <span aria-hidden="true">"$"</span>
                <input inputmode="decimal" value=value aria-label=label />
            </div>
            {children()}
        </label>
    }
}

#[component]
pub fn UiConfirmDialog(title: &'static str, open: bool, children: Children) -> impl IntoView {
    view! {
        {open.then(|| view! {
            <section class="ui-modal-backdrop" role="presentation">
                <div class="ui-confirm-dialog" role="alertdialog" aria-modal="true" aria-label=title tabindex="-1">
                    <h3>{title}</h3>
                    {children()}
                </div>
            </section>
        })}
    }
}

#[component]
pub fn UiFieldError(children: Children) -> impl IntoView {
    view! {
        <p class="ui-field-error" role="alert">
            {children()}
        </p>
    }
}
