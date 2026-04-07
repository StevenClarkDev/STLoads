BEGIN;

-- Disposable PostgreSQL seed for the Rust smoke pass.
-- Rerun this before the smoke script so mutable flows reset to a known state.
-- Passwords are stored in plaintext intentionally for this disposable dataset because
-- the Rust login route currently falls back to plain equality when bcrypt verification fails.

-- Clean up the deterministic smoke records first so reruns are stable.
DELETE FROM conversation_presence WHERE conversation_id IN (9401);
DELETE FROM conversation_reads WHERE conversation_id IN (9401);
DELETE FROM messages WHERE id BETWEEN 9411 AND 9499;
DELETE FROM conversations WHERE id IN (9401);
DELETE FROM offers WHERE id IN (9501);
DELETE FROM escrows WHERE leg_id IN (9311, 9312, 9313);
DELETE FROM stloads_reconciliation_log WHERE handoff_id IN (9601);
DELETE FROM stloads_sync_errors WHERE id IN (9701);
DELETE FROM stloads_external_refs WHERE handoff_id IN (9601);
DELETE FROM stloads_handoff_events WHERE handoff_id IN (9601);
DELETE FROM stloads_handoffs WHERE id IN (9601);
DELETE FROM load_history WHERE id BETWEEN 9351 AND 9399;
DELETE FROM carrier_preferences WHERE id IN (9151);
DELETE FROM load_documents WHERE load_id IN (9301, 9302, 9303);
DELETE FROM load_legs WHERE id IN (9311, 9312, 9313);
DELETE FROM loads WHERE id IN (9301, 9302, 9303);
DELETE FROM locations WHERE id IN (9221, 9222, 9223, 9224, 9225, 9226);
DELETE FROM cities WHERE id IN (9211, 9212, 9213);
DELETE FROM countries WHERE id IN (9201, 9202);
DELETE FROM commodity_types WHERE id IN (9241);
DELETE FROM equipments WHERE id IN (9231);
DELETE FROM load_types WHERE id IN (9251);
DELETE FROM user_history WHERE id BETWEEN 9121 AND 9139;
DELETE FROM user_details WHERE id IN (9111, 9112);
DELETE FROM model_has_roles WHERE model_type = 'App\\Models\\User' AND model_id IN (9101, 9102, 9103);
DELETE FROM personal_access_tokens WHERE tokenable_type = 'App\\Models\\User' AND tokenable_id IN (9101, 9102, 9103);
DELETE FROM users WHERE id IN (9101, 9102, 9103);

INSERT INTO users (
    id, name, email, password, role_id, company_name, phone_no, status,
    email_verified_at, approved_at, stripe_connect_account_id, payouts_enabled,
    kyc_status, terms_agreed, created_at, updated_at
)
VALUES
    (
        9101,
        'Rust Smoke Admin',
        'admin.smoke@stloads.test',
        'AdminPass123!',
        1,
        'STLoads Ops',
        '+1-555-0101',
        1,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL,
        FALSE,
        NULL,
        TRUE,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        9102,
        'Rust Smoke Shipper',
        'shipper.smoke@stloads.test',
        'ShipperPass123!',
        2,
        'Acme Retail Imports',
        '+1-555-0102',
        1,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL,
        FALSE,
        NULL,
        TRUE,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        9103,
        'Rust Smoke Carrier',
        'carrier.smoke@stloads.test',
        'CarrierPass123!',
        3,
        'North Route Logistics',
        '+1-555-0103',
        1,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        'acct_smoke_carrier_9103',
        FALSE,
        'pending',
        TRUE,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    )
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    email = EXCLUDED.email,
    password = EXCLUDED.password,
    role_id = EXCLUDED.role_id,
    company_name = EXCLUDED.company_name,
    phone_no = EXCLUDED.phone_no,
    status = EXCLUDED.status,
    email_verified_at = EXCLUDED.email_verified_at,
    approved_at = EXCLUDED.approved_at,
    stripe_connect_account_id = EXCLUDED.stripe_connect_account_id,
    payouts_enabled = EXCLUDED.payouts_enabled,
    kyc_status = EXCLUDED.kyc_status,
    terms_agreed = EXCLUDED.terms_agreed,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO user_details (
    id, user_id, company_name, company_address, dot_number, mc_number,
    equipment_types, business_entity_id, created_at, updated_at
)
VALUES
    (
        9111,
        9102,
        'Acme Retail Imports',
        '100 Harbor Way, Newark, NJ',
        NULL,
        NULL,
        'Dry Van',
        'SHIPPER-ENTITY-9102',
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        9112,
        9103,
        'North Route Logistics',
        '250 Carrier Lane, Allentown, PA',
        'USDOT-9103',
        'MC-9103',
        'Dry Van',
        'CARRIER-ENTITY-9103',
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    )
ON CONFLICT (id) DO UPDATE SET
    company_name = EXCLUDED.company_name,
    company_address = EXCLUDED.company_address,
    dot_number = EXCLUDED.dot_number,
    mc_number = EXCLUDED.mc_number,
    equipment_types = EXCLUDED.equipment_types,
    business_entity_id = EXCLUDED.business_entity_id,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO user_history (id, user_id, admin_id, status, remarks, created_at, updated_at)
VALUES
    (9121, 9102, 9101, 1, 'Smoke shipper approved for Rust/Postgres validation.', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (9122, 9103, 9101, 1, 'Smoke carrier approved for Rust/Postgres validation.', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET
    status = EXCLUDED.status,
    remarks = EXCLUDED.remarks,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO model_has_roles (role_id, model_type, model_id)
VALUES
    (1, 'App\\Models\\User', 9101),
    (2, 'App\\Models\\User', 9102),
    (3, 'App\\Models\\User', 9103)
ON CONFLICT DO NOTHING;

INSERT INTO countries (id, name, iso_code, created_at, updated_at)
VALUES
    (9201, 'United States', 'US', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (9202, 'Canada', 'CA', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    iso_code = EXCLUDED.iso_code,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO cities (id, country_id, name, created_at, updated_at)
VALUES
    (9211, 9201, 'Newark', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (9212, 9201, 'Chicago', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (9213, 9201, 'Dallas', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET
    country_id = EXCLUDED.country_id,
    name = EXCLUDED.name,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO locations (id, name, city_id, country_id, created_at, updated_at, deleted_at)
VALUES
    (9221, 'Port Newark Warehouse', 9211, 9201, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL),
    (9222, 'Chicago Crossdock', 9212, 9201, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL),
    (9223, 'Dallas Distribution Center', 9213, 9201, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL),
    (9224, 'Newark Offer Origin', 9211, 9201, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL),
    (9225, 'Chicago Offer Destination', 9212, 9201, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL),
    (9226, 'Dallas TMS Yard', 9213, 9201, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    city_id = EXCLUDED.city_id,
    country_id = EXCLUDED.country_id,
    deleted_at = NULL,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO equipments (id, name, created_at, updated_at, deleted_at)
VALUES (9231, 'Dry Van', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    deleted_at = NULL,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO commodity_types (id, name, created_at, updated_at, deleted_at)
VALUES (9241, 'General Freight', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    deleted_at = NULL,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO load_types (id, name, created_at, updated_at, deleted_at)
VALUES (9251, 'Full Truckload', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, NULL)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    deleted_at = NULL,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO loads (
    id, load_number, title, user_id, load_type_id, equipment_id, commodity_type_id,
    weight_unit, weight, special_instructions, is_hazardous, is_temperature_controlled,
    status, leg_count, created_at, updated_at, deleted_at
)
VALUES
    (
        9301,
        'SMOKE-LOAD-9301',
        'Open booking leg for booking and escrow smoke flow',
        9102,
        9251,
        9231,
        9241,
        'lbs',
        42000.00,
        'Keep available for direct carrier self-booking.',
        FALSE,
        FALSE,
        1,
        1,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL
    ),
    (
        9302,
        'SMOKE-LOAD-9302',
        'Negotiation leg for chat and offer review smoke flow',
        9102,
        9251,
        9231,
        9241,
        'lbs',
        38000.00,
        'Use this leg for conversation and offer review checks.',
        FALSE,
        FALSE,
        1,
        1,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL
    ),
    (
        9303,
        'SMOKE-LOAD-9303',
        'Seeded STLOADS handoff projection for admin smoke flow',
        9102,
        9251,
        9231,
        9241,
        'lbs',
        40000.00,
        'Used by the seeded published handoff and unresolved sync error.',
        FALSE,
        FALSE,
        1,
        1,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL
    )
ON CONFLICT (id) DO UPDATE SET
    load_number = EXCLUDED.load_number,
    title = EXCLUDED.title,
    user_id = EXCLUDED.user_id,
    load_type_id = EXCLUDED.load_type_id,
    equipment_id = EXCLUDED.equipment_id,
    commodity_type_id = EXCLUDED.commodity_type_id,
    weight_unit = EXCLUDED.weight_unit,
    weight = EXCLUDED.weight,
    special_instructions = EXCLUDED.special_instructions,
    is_hazardous = EXCLUDED.is_hazardous,
    is_temperature_controlled = EXCLUDED.is_temperature_controlled,
    status = EXCLUDED.status,
    leg_count = EXCLUDED.leg_count,
    deleted_at = NULL,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO load_legs (
    id, load_id, leg_no, leg_code, pickup_location_id, delivery_location_id,
    pickup_date, delivery_date, bid_status, price, status_id, booked_carrier_id,
    booked_at, booked_amount, accepted_offer_id, created_at, updated_at, deleted_at
)
VALUES
    (
        9311,
        9301,
        1,
        'SMOKE-LEG-9311',
        9221,
        9222,
        CURRENT_TIMESTAMP + INTERVAL '1 day',
        CURRENT_TIMESTAMP + INTERVAL '2 days',
        'open',
        2850.00,
        1,
        NULL,
        NULL,
        NULL,
        NULL,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL
    ),
    (
        9312,
        9302,
        1,
        'SMOKE-LEG-9312',
        9224,
        9225,
        CURRENT_TIMESTAMP + INTERVAL '3 days',
        CURRENT_TIMESTAMP + INTERVAL '4 days',
        'open',
        3100.00,
        1,
        NULL,
        NULL,
        NULL,
        NULL,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL
    ),
    (
        9313,
        9303,
        1,
        'SMOKE-LEG-9313',
        9226,
        9222,
        CURRENT_TIMESTAMP + INTERVAL '5 days',
        CURRENT_TIMESTAMP + INTERVAL '6 days',
        'open',
        3250.00,
        1,
        NULL,
        NULL,
        NULL,
        NULL,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP,
        NULL
    )
ON CONFLICT (id) DO UPDATE SET
    load_id = EXCLUDED.load_id,
    leg_no = EXCLUDED.leg_no,
    leg_code = EXCLUDED.leg_code,
    pickup_location_id = EXCLUDED.pickup_location_id,
    delivery_location_id = EXCLUDED.delivery_location_id,
    pickup_date = EXCLUDED.pickup_date,
    delivery_date = EXCLUDED.delivery_date,
    bid_status = EXCLUDED.bid_status,
    price = EXCLUDED.price,
    status_id = 1,
    booked_carrier_id = NULL,
    booked_at = NULL,
    booked_amount = NULL,
    accepted_offer_id = NULL,
    deleted_at = NULL,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO load_history (id, load_id, admin_id, status, remarks, created_at, updated_at)
VALUES
    (9351, 9301, 9101, 1, 'Open booking load reset for Rust smoke flow.', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (9352, 9302, 9101, 1, 'Negotiation load reset for Rust smoke flow.', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    (9353, 9303, 9101, 1, 'Seeded TMS projection reset for Rust smoke flow.', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET
    status = EXCLUDED.status,
    remarks = EXCLUDED.remarks,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO carrier_preferences (
    id, user_id, equipment_id, load_type_id, country_id, city_id,
    availability_days, max_weight_capacity, created_at, updated_at
)
VALUES
    (
        9151,
        9103,
        '9231',
        '9251',
        '9201',
        '9211,9212,9213',
        'Mon,Tue,Wed,Thu,Fri',
        45000.00,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    )
ON CONFLICT (id) DO UPDATE SET
    equipment_id = EXCLUDED.equipment_id,
    load_type_id = EXCLUDED.load_type_id,
    country_id = EXCLUDED.country_id,
    city_id = EXCLUDED.city_id,
    availability_days = EXCLUDED.availability_days,
    max_weight_capacity = EXCLUDED.max_weight_capacity,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO conversations (id, load_leg_id, shipper_id, carrier_id, created_at, updated_at)
VALUES (9401, 9312, 9102, 9103, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET
    load_leg_id = EXCLUDED.load_leg_id,
    shipper_id = EXCLUDED.shipper_id,
    carrier_id = EXCLUDED.carrier_id,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO messages (id, conversation_id, user_id, body, meta, created_at, updated_at)
VALUES
    (
        9411,
        9401,
        9102,
        'We seeded this conversation so the Rust chat workspace has a live thread on day one.',
        NULL,
        CURRENT_TIMESTAMP - INTERVAL '30 minutes',
        CURRENT_TIMESTAMP - INTERVAL '30 minutes'
    ),
    (
        9412,
        9401,
        9103,
        'Carrier side is ready for the first smoke-test reply.',
        NULL,
        CURRENT_TIMESTAMP - INTERVAL '15 minutes',
        CURRENT_TIMESTAMP - INTERVAL '15 minutes'
    )
ON CONFLICT (id) DO UPDATE SET
    conversation_id = EXCLUDED.conversation_id,
    user_id = EXCLUDED.user_id,
    body = EXCLUDED.body,
    meta = EXCLUDED.meta,
    created_at = EXCLUDED.created_at,
    updated_at = EXCLUDED.updated_at;

INSERT INTO offers (id, load_leg_id, carrier_id, conversation_id, amount, status_id, created_at, updated_at)
VALUES (9501, 9312, 9103, 9401, 2995.00, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET
    load_leg_id = EXCLUDED.load_leg_id,
    carrier_id = EXCLUDED.carrier_id,
    conversation_id = EXCLUDED.conversation_id,
    amount = EXCLUDED.amount,
    status_id = 1,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO stloads_handoffs (
    id, tms_load_id, tenant_id, external_handoff_id, load_id, status, tms_status, tms_status_at,
    party_type, freight_mode, equipment_type, commodity_description, weight, weight_unit,
    piece_count, is_hazardous, pickup_city, pickup_state, pickup_zip, pickup_country,
    pickup_address, pickup_window_start, pickup_window_end, dropoff_city, dropoff_state,
    dropoff_zip, dropoff_country, dropoff_address, dropoff_window_start, dropoff_window_end,
    board_rate, rate_currency, bid_type, quote_status, tender_posture, compliance_passed,
    readiness, pushed_by, push_reason, source_module, queued_at, published_at,
    retry_count, payload_version, raw_payload, created_at, updated_at
)
VALUES (
    9601,
    'TMS-SEED-LOAD-9601',
    'demo-tenant',
    'seed-handoff-9601',
    9303,
    'published',
    'dispatched',
    CURRENT_TIMESTAMP - INTERVAL '1 hour',
    'shipper',
    'truckload',
    'Dry Van',
    'Consumer goods',
    40000.00,
    'lbs',
    24,
    FALSE,
    'Dallas',
    'TX',
    '75201',
    'US',
    '100 Logistics Way, Dallas, TX',
    CURRENT_TIMESTAMP + INTERVAL '1 day',
    CURRENT_TIMESTAMP + INTERVAL '1 day 2 hours',
    'Chicago',
    'IL',
    '60601',
    'US',
    '400 Freight Ave, Chicago, IL',
    CURRENT_TIMESTAMP + INTERVAL '2 days',
    CURRENT_TIMESTAMP + INTERVAL '2 days 3 hours',
    3250.00,
    'USD',
    'Fixed',
    'open',
    'tendered',
    TRUE,
    'ready',
    'rust-smoke-seed',
    'Seeded published handoff for admin operations smoke flow.',
    'seed_script',
    CURRENT_TIMESTAMP - INTERVAL '2 hours',
    CURRENT_TIMESTAMP - INTERVAL '90 minutes',
    0,
    '1.0',
    '{"seeded": true, "scenario": "published_handoff"}'::jsonb,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
)
ON CONFLICT (id) DO UPDATE SET
    tms_load_id = EXCLUDED.tms_load_id,
    tenant_id = EXCLUDED.tenant_id,
    external_handoff_id = EXCLUDED.external_handoff_id,
    load_id = EXCLUDED.load_id,
    status = 'published',
    tms_status = EXCLUDED.tms_status,
    tms_status_at = EXCLUDED.tms_status_at,
    party_type = EXCLUDED.party_type,
    freight_mode = EXCLUDED.freight_mode,
    equipment_type = EXCLUDED.equipment_type,
    commodity_description = EXCLUDED.commodity_description,
    weight = EXCLUDED.weight,
    weight_unit = EXCLUDED.weight_unit,
    piece_count = EXCLUDED.piece_count,
    is_hazardous = EXCLUDED.is_hazardous,
    pickup_city = EXCLUDED.pickup_city,
    pickup_state = EXCLUDED.pickup_state,
    pickup_zip = EXCLUDED.pickup_zip,
    pickup_country = EXCLUDED.pickup_country,
    pickup_address = EXCLUDED.pickup_address,
    pickup_window_start = EXCLUDED.pickup_window_start,
    pickup_window_end = EXCLUDED.pickup_window_end,
    dropoff_city = EXCLUDED.dropoff_city,
    dropoff_state = EXCLUDED.dropoff_state,
    dropoff_zip = EXCLUDED.dropoff_zip,
    dropoff_country = EXCLUDED.dropoff_country,
    dropoff_address = EXCLUDED.dropoff_address,
    dropoff_window_start = EXCLUDED.dropoff_window_start,
    dropoff_window_end = EXCLUDED.dropoff_window_end,
    board_rate = EXCLUDED.board_rate,
    rate_currency = EXCLUDED.rate_currency,
    bid_type = EXCLUDED.bid_type,
    quote_status = EXCLUDED.quote_status,
    tender_posture = EXCLUDED.tender_posture,
    compliance_passed = EXCLUDED.compliance_passed,
    readiness = EXCLUDED.readiness,
    pushed_by = EXCLUDED.pushed_by,
    push_reason = EXCLUDED.push_reason,
    source_module = EXCLUDED.source_module,
    queued_at = EXCLUDED.queued_at,
    published_at = EXCLUDED.published_at,
    withdrawn_at = NULL,
    closed_at = NULL,
    retry_count = 0,
    payload_version = EXCLUDED.payload_version,
    raw_payload = EXCLUDED.raw_payload,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO stloads_handoff_events (
    id, handoff_id, event_type, performed_by, source_module,
    payload_snapshot, result, created_at, updated_at
)
VALUES (
    9611,
    9601,
    'published',
    'rust-smoke-seed',
    'seed_script',
    '{"seeded": true, "handoff_id": 9601}',
    'Published handoff created for admin smoke validation.',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
)
ON CONFLICT (id) DO UPDATE SET
    handoff_id = EXCLUDED.handoff_id,
    event_type = EXCLUDED.event_type,
    performed_by = EXCLUDED.performed_by,
    source_module = EXCLUDED.source_module,
    payload_snapshot = EXCLUDED.payload_snapshot,
    result = EXCLUDED.result,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO stloads_external_refs (id, handoff_id, ref_type, ref_value, ref_source, created_at, updated_at)
VALUES (
    9621,
    9601,
    'board_posting_id',
    'BOARD-SMOKE-9601',
    'seed_script',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
)
ON CONFLICT (id) DO UPDATE SET
    handoff_id = EXCLUDED.handoff_id,
    ref_type = EXCLUDED.ref_type,
    ref_value = EXCLUDED.ref_value,
    ref_source = EXCLUDED.ref_source,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO stloads_sync_errors (
    id, handoff_id, error_class, severity, title, detail, source_module, performed_by,
    resolved, resolved_at, resolved_by, resolution_note, created_at, updated_at
)
VALUES (
    9701,
    9601,
    'rate_mismatch',
    'warning',
    'Seeded rate mismatch for admin resolve flow.',
    'This unresolved sync error is intentionally present so the admin resolve route has a stable smoke target.',
    'seed_script',
    'rust-smoke-seed',
    FALSE,
    NULL,
    NULL,
    NULL,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
)
ON CONFLICT (id) DO UPDATE SET
    handoff_id = EXCLUDED.handoff_id,
    error_class = EXCLUDED.error_class,
    severity = EXCLUDED.severity,
    title = EXCLUDED.title,
    detail = EXCLUDED.detail,
    source_module = EXCLUDED.source_module,
    performed_by = EXCLUDED.performed_by,
    resolved = FALSE,
    resolved_at = NULL,
    resolved_by = NULL,
    resolution_note = NULL,
    updated_at = CURRENT_TIMESTAMP;

INSERT INTO stloads_reconciliation_log (
    id, handoff_id, action, tms_status_from, tms_status_to, stloads_status_from,
    stloads_status_to, detail, triggered_by, webhook_payload, created_at, updated_at
)
VALUES (
    9801,
    9601,
    'mismatch_detected',
    'dispatched',
    'dispatched',
    'published',
    'published',
    'Seeded reconciliation row for dashboard visibility.',
    'rust-smoke-seed',
    '{"seeded": true, "sync_error_id": 9701}'::jsonb,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
)
ON CONFLICT (id) DO UPDATE SET
    handoff_id = EXCLUDED.handoff_id,
    action = EXCLUDED.action,
    tms_status_from = EXCLUDED.tms_status_from,
    tms_status_to = EXCLUDED.tms_status_to,
    stloads_status_from = EXCLUDED.stloads_status_from,
    stloads_status_to = EXCLUDED.stloads_status_to,
    detail = EXCLUDED.detail,
    triggered_by = EXCLUDED.triggered_by,
    webhook_payload = EXCLUDED.webhook_payload,
    updated_at = CURRENT_TIMESTAMP;

SELECT setval(pg_get_serial_sequence('users', 'id'), COALESCE((SELECT MAX(id) FROM users), 1), true);
SELECT setval(pg_get_serial_sequence('user_details', 'id'), COALESCE((SELECT MAX(id) FROM user_details), 1), true);
SELECT setval(pg_get_serial_sequence('user_history', 'id'), COALESCE((SELECT MAX(id) FROM user_history), 1), true);
SELECT setval(pg_get_serial_sequence('countries', 'id'), COALESCE((SELECT MAX(id) FROM countries), 1), true);
SELECT setval(pg_get_serial_sequence('cities', 'id'), COALESCE((SELECT MAX(id) FROM cities), 1), true);
SELECT setval(pg_get_serial_sequence('locations', 'id'), COALESCE((SELECT MAX(id) FROM locations), 1), true);
SELECT setval(pg_get_serial_sequence('load_types', 'id'), COALESCE((SELECT MAX(id) FROM load_types), 1), true);
SELECT setval(pg_get_serial_sequence('equipments', 'id'), COALESCE((SELECT MAX(id) FROM equipments), 1), true);
SELECT setval(pg_get_serial_sequence('commodity_types', 'id'), COALESCE((SELECT MAX(id) FROM commodity_types), 1), true);
SELECT setval(pg_get_serial_sequence('loads', 'id'), COALESCE((SELECT MAX(id) FROM loads), 1), true);
SELECT setval(pg_get_serial_sequence('load_legs', 'id'), COALESCE((SELECT MAX(id) FROM load_legs), 1), true);
SELECT setval(pg_get_serial_sequence('load_history', 'id'), COALESCE((SELECT MAX(id) FROM load_history), 1), true);
SELECT setval(pg_get_serial_sequence('carrier_preferences', 'id'), COALESCE((SELECT MAX(id) FROM carrier_preferences), 1), true);
SELECT setval(pg_get_serial_sequence('offers', 'id'), COALESCE((SELECT MAX(id) FROM offers), 1), true);
SELECT setval(pg_get_serial_sequence('conversations', 'id'), COALESCE((SELECT MAX(id) FROM conversations), 1), true);
SELECT setval(pg_get_serial_sequence('messages', 'id'), COALESCE((SELECT MAX(id) FROM messages), 1), true);
SELECT setval(pg_get_serial_sequence('escrows', 'id'), COALESCE((SELECT MAX(id) FROM escrows), 1), true);
SELECT setval(pg_get_serial_sequence('stloads_handoffs', 'id'), COALESCE((SELECT MAX(id) FROM stloads_handoffs), 1), true);
SELECT setval(pg_get_serial_sequence('stloads_handoff_events', 'id'), COALESCE((SELECT MAX(id) FROM stloads_handoff_events), 1), true);
SELECT setval(pg_get_serial_sequence('stloads_external_refs', 'id'), COALESCE((SELECT MAX(id) FROM stloads_external_refs), 1), true);
SELECT setval(pg_get_serial_sequence('stloads_sync_errors', 'id'), COALESCE((SELECT MAX(id) FROM stloads_sync_errors), 1), true);
SELECT setval(pg_get_serial_sequence('stloads_reconciliation_log', 'id'), COALESCE((SELECT MAX(id) FROM stloads_reconciliation_log), 1), true);

COMMIT;
