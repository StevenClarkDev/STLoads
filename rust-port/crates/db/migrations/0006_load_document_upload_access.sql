ALTER TABLE load_documents
    ADD COLUMN IF NOT EXISTS uploaded_by_user_id BIGINT NULL,
    ADD COLUMN IF NOT EXISTS storage_provider VARCHAR(32) NOT NULL DEFAULT 'local';

UPDATE load_documents document
SET uploaded_by_user_id = load.user_id
FROM loads load
WHERE load.id = document.load_id
  AND document.uploaded_by_user_id IS NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'fk_load_documents_uploaded_by_user_id'
    ) THEN
        ALTER TABLE load_documents
            ADD CONSTRAINT fk_load_documents_uploaded_by_user_id
            FOREIGN KEY (uploaded_by_user_id) REFERENCES users(id) ON DELETE SET NULL;
    END IF;
END$$;

CREATE INDEX IF NOT EXISTS idx_load_documents_uploaded_by_user_id
    ON load_documents (uploaded_by_user_id);
CREATE INDEX IF NOT EXISTS idx_load_documents_storage_provider
    ON load_documents (storage_provider);

