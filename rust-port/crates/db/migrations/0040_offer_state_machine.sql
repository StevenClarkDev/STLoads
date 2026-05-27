INSERT INTO offer_status_master (id, name, slug, description, sort_order, is_terminal)
VALUES
    (0, 'Declined', 'declined', 'Offer was declined by the load owner or carrier.', 0, TRUE),
    (1, 'Pending', 'pending', 'Offer submitted and awaiting decision.', 1, FALSE),
    (2, 'Countered', 'countered', 'Offer has an active counter awaiting response.', 2, FALSE),
    (3, 'Accepted', 'accepted', 'Offer accepted and used to book the leg.', 3, TRUE),
    (4, 'Withdrawn', 'withdrawn', 'Offer was withdrawn before acceptance.', 4, TRUE),
    (5, 'Expired', 'expired', 'Offer expired before a final decision.', 5, TRUE),
    (6, 'Superseded', 'superseded', 'Offer was replaced by a newer counteroffer or accepted offer.', 6, TRUE),
    (7, 'Cancelled', 'cancelled', 'Offer was cancelled before tendering could continue.', 7, TRUE)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    slug = EXCLUDED.slug,
    description = EXCLUDED.description,
    sort_order = EXCLUDED.sort_order,
    is_terminal = EXCLUDED.is_terminal;
