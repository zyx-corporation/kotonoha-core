-- M2 RDE assessment metadata (additive).
-- Parent: https://github.com/zyx-corporation/kotonoha-management/issues/103
-- Issue: https://github.com/zyx-corporation/kotonoha-core/issues/29

BEGIN;

ALTER TABLE rde_assessments
ADD COLUMN payload_schema_version TEXT,
ADD COLUMN source_kind TEXT,
ADD COLUMN validation_report JSONB;

ALTER TABLE rde_assessments
ADD CONSTRAINT rde_assessments_source_kind_check CHECK (
    source_kind IS NULL
    OR source_kind IN ('cli', 'llm', 'import', 'replay')
);

CREATE INDEX idx_rde_assessments_source_kind ON rde_assessments (source_kind)
WHERE
    source_kind IS NOT NULL;

COMMENT ON COLUMN rde_assessments.payload_schema_version IS 'M2: interchange spec_version or internal label at attach time.';
COMMENT ON COLUMN rde_assessments.source_kind IS 'M2: input channel — cli | llm | import | replay.';
COMMENT ON COLUMN rde_assessments.validation_report IS 'M2: machine-readable validation summary (strict flag, warnings, etc.).';

COMMIT;
