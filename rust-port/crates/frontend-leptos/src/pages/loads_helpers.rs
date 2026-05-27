use leptos::prelude::*;
use leptos_router::components::A;
use shared::LoadBoardRow;

use super::shared::tone_style;
pub(super) fn render_row(
    row: LoadBoardRow,
    pending_leg_id: RwSignal<Option<u64>>,
    book_leg: impl Fn(u64) + Copy + 'static,
    open_carrier_matches: impl Fn(u64) + Copy + 'static,
    matching_leg_id: RwSignal<Option<u64>>,
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
    let is_matching = Signal::derive(move || matching_leg_id.get() == Some(leg_id));
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
                {can_view_profile.then(|| view! {
                    <button
                        type="button"
                        style="padding:0.5rem 0.75rem;border:1px solid #bfdbfe;border-radius:0.75rem;background:#eff6ff;color:#1d4ed8;cursor:pointer;"
                        disabled=move || is_matching.get()
                        on:click=move |_| open_carrier_matches(leg_id)
                    >
                        {move || if is_matching.get() { "Ranking..." } else { "Matches" }}
                    </button>
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
