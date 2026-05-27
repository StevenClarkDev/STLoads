ALTER TABLE offers
    ADD COLUMN IF NOT EXISTS parent_offer_id BIGINT NULL,
    ADD COLUMN IF NOT EXISTS expires_at TIMESTAMP(6) NULL,
    ADD COLUMN IF NOT EXISTS tender_kind VARCHAR(30) NOT NULL DEFAULT 'spot_bid',
    ADD COLUMN IF NOT EXISTS decision_note TEXT NULL,
    ADD COLUMN IF NOT EXISTS rate_confirmation_ref VARCHAR(80) NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_offers_parent_offer_id'
    ) THEN
        ALTER TABLE offers
            ADD CONSTRAINT fk_offers_parent_offer_id
            FOREIGN KEY (parent_offer_id) REFERENCES offers(id) ON DELETE SET NULL;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_offers_tender_kind'
    ) THEN
        ALTER TABLE offers
            ADD CONSTRAINT chk_offers_tender_kind
            CHECK (tender_kind IN ('spot_bid', 'private_tender', 'counteroffer'));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_offers_parent_offer_id ON offers (parent_offer_id);
CREATE INDEX IF NOT EXISTS idx_offers_expires_at ON offers (expires_at);
CREATE INDEX IF NOT EXISTS idx_offers_tender_kind ON offers (tender_kind);
