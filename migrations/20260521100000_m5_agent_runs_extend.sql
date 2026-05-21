-- M5 AgentRun extension (additive).
-- Parent: https://github.com/zyx-corporation/kotonoha-management/issues/106
-- Issue: https://github.com/zyx-corporation/kotonoha-core/issues/33
-- Spec: https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md

BEGIN;

ALTER TABLE agent_runs
    ADD COLUMN capability_profile TEXT,
    ADD COLUMN parent_run_id UUID REFERENCES agent_runs (id) ON DELETE SET NULL,
    ADD COLUMN status TEXT NOT NULL DEFAULT 'started',
    ADD COLUMN output_artifact_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN denied_actions JSONB NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE agent_runs
    ADD CONSTRAINT agent_runs_status_check CHECK (
        status IN ('started', 'completed', 'failed', 'denied')
    );

CREATE INDEX idx_agent_runs_parent ON agent_runs (parent_run_id)
WHERE
    parent_run_id IS NOT NULL;

CREATE INDEX idx_agent_runs_status ON agent_runs (status);

COMMENT ON COLUMN agent_runs.capability_profile IS 'M5: allowed tool profile id (e.g. kotonoha-readonly, kotonoha-agent).';
COMMENT ON COLUMN agent_runs.parent_run_id IS 'M5: optional parent AgentRun for chained runs.';
COMMENT ON COLUMN agent_runs.status IS 'M5: started | completed | failed | denied.';
COMMENT ON COLUMN agent_runs.output_artifact_refs IS 'M5: JSON array of artifact pointers (not canonical text).';
COMMENT ON COLUMN agent_runs.denied_actions IS 'M5: JSON array of denied operation records for audit.';

COMMIT;
