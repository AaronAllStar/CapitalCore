-- 0001_initial_schema.sql
-- EdgeArena Core Schema (Users, Roles, Permissions, User Roles, Audit Events)
-- Invariant #1 strictly enforced: No float/real/double precision columns.

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS audit_events (
    audit_id UUID PRIMARY KEY,
    event_id UUID NOT NULL,
    sequence BIGINT NOT NULL,
    decision VARCHAR(20) NOT NULL,
    reasons TEXT[] NOT NULL DEFAULT '{}',
    feature_versions JSONB NOT NULL DEFAULT '{}',
    rule_versions JSONB NOT NULL DEFAULT '{}',
    model_version VARCHAR(100),
    previous_hash VARCHAR(64) NOT NULL,
    record_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_event_id ON audit_events (event_id);
CREATE INDEX IF NOT EXISTS idx_audit_created_at ON audit_events (created_at);
CREATE INDEX IF NOT EXISTS idx_audit_sequence ON audit_events (sequence);
