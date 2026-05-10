-- Optional persistence for core interchange envelope JSON (`kotonoha.interchange.v1`).
-- Validation remains application-side (`interchange::validate_interchange_json`).

BEGIN;

CREATE TABLE interchange_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now ()
);

CREATE INDEX idx_interchange_documents_payload_gin ON interchange_documents USING gin (payload jsonb_path_ops);

COMMENT ON TABLE interchange_documents IS 'Stores full interchange envelopes (non-normative helper); see kotonoha-core interchange module.';

COMMIT;
