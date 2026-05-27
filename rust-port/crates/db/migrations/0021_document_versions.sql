CREATE TABLE IF NOT EXISTS load_document_versions (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES load_documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    document_name TEXT NOT NULL,
    document_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    storage_provider TEXT NOT NULL,
    uploaded_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    original_name TEXT NULL,
    mime_type TEXT NULL,
    file_size BIGINT NULL,
    hash TEXT NULL,
    hash_algorithm TEXT NULL,
    mock_blockchain_tx TEXT NULL,
    mock_blockchain_timestamp TIMESTAMP NULL,
    replacement_reason TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (document_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_load_document_versions_document
    ON load_document_versions (document_id, version_number DESC);

CREATE TABLE IF NOT EXISTS kyc_document_versions (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES kyc_documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    document_name TEXT NOT NULL,
    document_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    original_name TEXT NULL,
    mime_type TEXT NULL,
    file_size BIGINT NULL,
    hash TEXT NULL,
    hash_algorithm TEXT NULL,
    mock_blockchain_tx TEXT NULL,
    mock_blockchain_timestamp TIMESTAMP NULL,
    uploaded_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    replacement_reason TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (document_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_kyc_document_versions_document
    ON kyc_document_versions (document_id, version_number DESC);

CREATE TABLE IF NOT EXISTS leg_document_versions (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES leg_documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    document_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    meta JSONB NULL,
    uploaded_by_user_id BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    original_name TEXT NULL,
    mime_type TEXT NULL,
    file_size BIGINT NULL,
    replacement_reason TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (document_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_leg_document_versions_document
    ON leg_document_versions (document_id, version_number DESC);

INSERT INTO load_document_versions (
    document_id, version_number, document_name, document_type, file_path, storage_provider,
    uploaded_by_user_id, original_name, mime_type, file_size, hash, hash_algorithm,
    mock_blockchain_tx, mock_blockchain_timestamp, replacement_reason, created_at
)
SELECT
    id, 1, document_name, document_type, file_path, storage_provider,
    uploaded_by_user_id, original_name, mime_type, file_size, hash, hash_algorithm,
    mock_blockchain_tx, mock_blockchain_timestamp, 'backfilled current document at versioning rollout', created_at
FROM load_documents
ON CONFLICT (document_id, version_number) DO NOTHING;

INSERT INTO kyc_document_versions (
    document_id, version_number, document_name, document_type, file_path, original_name,
    mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp,
    uploaded_by_user_id, replacement_reason, created_at
)
SELECT
    id, 1, document_name, document_type, file_path, original_name,
    mime_type, file_size, hash, hash_algorithm, mock_blockchain_tx, mock_blockchain_timestamp,
    user_id, 'backfilled current document at versioning rollout', created_at
FROM kyc_documents
ON CONFLICT (document_id, version_number) DO NOTHING;

INSERT INTO leg_document_versions (
    document_id, version_number, document_type, file_path, meta, uploaded_by_user_id,
    original_name, mime_type, file_size, replacement_reason, created_at
)
SELECT
    id, 1, type, path, meta,
    CASE
        WHEN NULLIF(meta ->> 'uploaded_by', '') ~ '^[0-9]+$'
            THEN NULLIF(meta ->> 'uploaded_by', '')::bigint
        ELSE NULL
    END,
    NULLIF(meta ->> 'original_name', ''), NULLIF(meta ->> 'mime_type', ''),
    CASE
        WHEN NULLIF(meta ->> 'file_size', '') ~ '^[0-9]+$'
            THEN NULLIF(meta ->> 'file_size', '')::bigint
        ELSE NULL
    END,
    'backfilled current document at versioning rollout', created_at
FROM leg_documents
ON CONFLICT (document_id, version_number) DO NOTHING;
