-- ============================================================================
-- BharatChat Sovereign Platform - Local PostgreSQL Relational Schema
-- Jurisdiction: INDIA_LOCALIZED_MUMBAI_BLR
-- Zero Third-Party Cloud Dependencies
-- ============================================================================

-- 1. Identities (W3C DID Ledger)
CREATE TABLE IF NOT EXISTS identities (
    did VARCHAR(128) PRIMARY KEY,
    verifying_key_hex VARCHAR(64) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'ACTIVE',
    registered_at BIGINT NOT NULL,
    revoked_at BIGINT,
    data_jurisdiction VARCHAR(64) NOT NULL DEFAULT 'INDIA_LOCALIZED_MUMBAI_BLR'
);

CREATE INDEX IF NOT EXISTS idx_identities_status ON identities(status);

-- 2. DPDP Act 2023 Consent Records
CREATE TABLE IF NOT EXISTS consent_records (
    id SERIAL PRIMARY KEY,
    principal_did VARCHAR(128) NOT NULL REFERENCES identities(did) ON DELETE CASCADE,
    fiduciary_did VARCHAR(128) NOT NULL,
    scope VARCHAR(64) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    granted_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    UNIQUE (principal_did, fiduciary_did, scope)
);

CREATE INDEX IF NOT EXISTS idx_consent_active ON consent_records(principal_did, is_active);

-- 3. Broadcast Channels (One-to-Many Public/Private)
CREATE TABLE IF NOT EXISTS broadcast_channels (
    channel_id VARCHAR(64) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    admin_did VARCHAR(128) NOT NULL REFERENCES identities(did) ON DELETE CASCADE,
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    created_at BIGINT NOT NULL,
    subscriber_count BIGINT NOT NULL DEFAULT 0
);

-- 4. Channel Posts (Tamper-evident Ed25519-signed)
CREATE TABLE IF NOT EXISTS channel_posts (
    post_id VARCHAR(64) PRIMARY KEY,
    channel_id VARCHAR(64) NOT NULL REFERENCES broadcast_channels(channel_id) ON DELETE CASCADE,
    author_did VARCHAR(128) NOT NULL REFERENCES identities(did),
    content_cid VARCHAR(128) NOT NULL,
    text_content TEXT NOT NULL,
    signature_hex VARCHAR(128) NOT NULL,
    reactions JSONB NOT NULL DEFAULT '{}'::jsonb,
    published_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_posts_channel ON channel_posts(channel_id, published_at DESC);

-- 5. Viral Misinformation Friction Ledger
CREATE TABLE IF NOT EXISTS forwarding_ledger (
    message_cid VARCHAR(128) PRIMARY KEY,
    first_sender_did VARCHAR(128) NOT NULL,
    hop_count INT NOT NULL DEFAULT 0,
    is_frequently_forwarded BOOLEAN NOT NULL DEFAULT FALSE,
    last_forwarded_at BIGINT NOT NULL
);

-- 6. IT Rules 2021 Statutory Abuse Grievance Tickets
CREATE TABLE IF NOT EXISTS abuse_grievance_tickets (
    ticket_id VARCHAR(64) PRIMARY KEY,
    reporter_did VARCHAR(128) NOT NULL,
    category VARCHAR(64) NOT NULL,
    message_cid VARCHAR(128) NOT NULL,
    accused_did VARCHAR(128) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'ACKNOWLEDGED_WITHIN_24H',
    acknowledged_at BIGINT NOT NULL,
    resolution_deadline BIGINT NOT NULL,
    resolved_at BIGINT
);

CREATE INDEX IF NOT EXISTS idx_tickets_status ON abuse_grievance_tickets(status);

-- 7. Conversational Commerce & NPCI UPI 2.0 Dynamic Orders
CREATE TABLE IF NOT EXISTS commerce_orders (
    order_id VARCHAR(64) PRIMARY KEY,
    customer_did VARCHAR(128) NOT NULL REFERENCES identities(did),
    merchant_did VARCHAR(128) NOT NULL REFERENCES identities(did),
    items JSONB NOT NULL,
    subtotal_inr NUMERIC(12, 2) NOT NULL,
    tax_inr NUMERIC(12, 2) NOT NULL,
    total_amount_inr NUMERIC(12, 2) NOT NULL,
    payee_vpa VARCHAR(128) NOT NULL,
    payee_name VARCHAR(128) NOT NULL,
    upi_intent_uri TEXT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'AwaitingPayment',
    created_at BIGINT NOT NULL
);

-- 8. Off-Chain Chunked Media Registry
CREATE TABLE IF NOT EXISTS media_chunks (
    chunk_cid VARCHAR(128) PRIMARY KEY,
    owner_did VARCHAR(128) NOT NULL REFERENCES identities(did) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    mime_type VARCHAR(64) NOT NULL,
    byte_size BIGINT NOT NULL,
    nonce_hex VARCHAR(32) NOT NULL,
    stored_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_media_owner ON media_chunks(owner_did);
