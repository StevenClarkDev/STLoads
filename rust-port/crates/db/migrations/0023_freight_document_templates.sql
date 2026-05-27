CREATE TABLE IF NOT EXISTS freight_document_templates (
    id BIGSERIAL PRIMARY KEY,
    template_key TEXT NOT NULL,
    version TEXT NOT NULL,
    title TEXT NOT NULL,
    document_type_key TEXT NOT NULL,
    body_template TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (template_key, version)
);

CREATE INDEX IF NOT EXISTS idx_freight_document_templates_active
    ON freight_document_templates (template_key, is_active, created_at DESC);

CREATE TABLE IF NOT EXISTS generated_freight_documents (
    id BIGSERIAL PRIMARY KEY,
    load_id BIGINT NOT NULL REFERENCES loads(id) ON DELETE CASCADE,
    document_id BIGINT NOT NULL REFERENCES load_documents(id) ON DELETE CASCADE,
    template_key TEXT NOT NULL,
    template_version TEXT NOT NULL,
    generated_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    generation_status TEXT NOT NULL DEFAULT 'generated',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (load_id, template_key, template_version)
);

CREATE INDEX IF NOT EXISTS idx_generated_freight_documents_load
    ON generated_freight_documents (load_id, created_at DESC);

INSERT INTO freight_document_templates (
    template_key, version, title, document_type_key, body_template
) VALUES
    (
        'rate_confirmation',
        '2026-05-25',
        'Rate Confirmation',
        'rate_confirmation',
        'STLoads Rate Confirmation\nLoad: {{load_number}}\nTitle: {{load_title}}\nRoute: {{route_summary}}\nRate: {{rate_summary}}\nSpecial instructions: {{special_instructions}}\nGenerated: {{generated_at}}\n'
    ),
    (
        'bill_of_lading',
        '2026-05-25',
        'Bill Of Lading',
        'bill_of_lading',
        'STLoads Bill Of Lading\nLoad: {{load_number}}\nRoute: {{route_summary}}\nCommodity: {{commodity_summary}}\nWeight: {{weight_summary}}\nGenerated: {{generated_at}}\n'
    ),
    (
        'carrier_packet',
        '2026-05-25',
        'Carrier Packet',
        'carrier_packet',
        'STLoads Carrier Packet\nLoad: {{load_number}}\nCarrier: {{carrier_summary}}\nRequired evidence: authority, insurance certificate, W-9, operating agreement, payout setup.\nGenerated: {{generated_at}}\n'
    ),
    (
        'shipper_document_package',
        '2026-05-25',
        'Shipper Document Package',
        'shipper_package',
        'STLoads Shipper Document Package\nLoad: {{load_number}}\nCustomer package includes rate confirmation, BOL, carrier packet, POD/closeout evidence when available.\nGenerated: {{generated_at}}\n'
    )
ON CONFLICT (template_key, version) DO NOTHING;
