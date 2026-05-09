-- Kotonoha / SLS — PostgreSQL initial schema (v0)
-- Aligns with kotonoha-spec interchange shapes and audit-trail-relationship (abstract).
-- Apply in order; destructive reset for development only.

BEGIN;

-- --- Lineage (semantic-lineage-model.md): addressable units with optional predecessor link
CREATE TABLE lineage_units (
    id TEXT PRIMARY KEY,
    prior_unit_id TEXT REFERENCES lineage_units (id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_lineage_units_prior ON lineage_units (prior_unit_id);

COMMENT ON TABLE lineage_units IS 'Minimal lineage units; id is URI/IRI-shaped string per kotonoha-spec Phase 1.';

-- --- RDE review output documents (full JSON document including rde_review_output wrapper)
CREATE TABLE rde_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    subject_ref TEXT NOT NULL,
    spec_version TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT rde_documents_spec_version_phase1 CHECK (spec_version = '0.1')
);

CREATE INDEX idx_rde_documents_subject ON rde_documents (subject_ref);
CREATE INDEX idx_rde_documents_payload_gin ON rde_documents USING gin (payload jsonb_path_ops);

COMMENT ON TABLE rde_documents IS 'Stores JSON payloads validated against kotonoha-spec docs/rde-review-output.md (Phase 1 bundle 0.1).';

-- --- Append-only audit stream (correlation_ref aligns with subject_ref / external IDs per deployment)
CREATE TABLE audit_events (
    id BIGSERIAL PRIMARY KEY,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    event_type TEXT NOT NULL,
    correlation_ref TEXT,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX idx_audit_events_correlation ON audit_events (correlation_ref);
CREATE INDEX idx_audit_events_occurred ON audit_events (occurred_at);

COMMENT ON TABLE audit_events IS 'Append-only audit records; correlate with RDE subject_ref via correlation_ref (audit-trail-relationship.md).';

COMMIT;
