-- M1 semantic lineage tables (draft — non-normative DDL).
-- Parent: https://github.com/zyx-corporation/kotonoha-management/issues/97
-- Issue: https://github.com/zyx-corporation/kotonoha-core/issues/21
-- Concept: https://github.com/zyx-corporation/kotonoha-docs/blob/main/ja/paper/kotonoha_concept.md

BEGIN;

-- Logical document under review (spec fragment, design note, paper draft, etc.)
CREATE TABLE document_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    external_ref TEXT,
    title TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now ()
);

CREATE UNIQUE INDEX idx_document_objects_external_ref ON document_objects (external_ref)
WHERE
    external_ref IS NOT NULL;

COMMENT ON TABLE document_objects IS 'M1: logical Document Object; not a raw filesystem path.';

-- Point-in-time semantic snapshot
CREATE TABLE meaning_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    document_object_id UUID REFERENCES document_objects (id) ON DELETE SET NULL,
    git_commit TEXT,
    snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    source_context JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now ()
);

CREATE INDEX idx_meaning_states_document ON meaning_states (document_object_id);
CREATE INDEX idx_meaning_states_git_commit ON meaning_states (git_commit)
WHERE
    git_commit IS NOT NULL;

COMMENT ON TABLE meaning_states IS 'M1: MeaningState — intent/constraints/unresolved snapshot at a point in time.';

-- Semantic change (ΔM) anchored to Git content lineage
CREATE TABLE meaning_deltas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    document_object_id UUID REFERENCES document_objects (id) ON DELETE SET NULL,
    prior_meaning_state_id UUID REFERENCES meaning_states (id) ON DELETE SET NULL,
    new_meaning_state_id UUID REFERENCES meaning_states (id) ON DELETE SET NULL,
    agent_run_id UUID,
    git_commit TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_range_start INTEGER,
    line_range_end INTEGER,
    diff_ref TEXT,
    observation JSONB NOT NULL DEFAULT '{}'::jsonb,
    source_context JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT meaning_deltas_git_anchor CHECK (
        (
            line_range_start IS NOT NULL
            AND line_range_end IS NOT NULL
            AND line_range_end >= line_range_start
        )
        OR diff_ref IS NOT NULL
    )
);

CREATE INDEX idx_meaning_deltas_git_commit ON meaning_deltas (git_commit);
CREATE INDEX idx_meaning_deltas_file_path ON meaning_deltas (file_path);
CREATE INDEX idx_meaning_deltas_document ON meaning_deltas (document_object_id);
CREATE INDEX idx_meaning_deltas_observation_gin ON meaning_deltas USING gin (observation jsonb_path_ops);

COMMENT ON TABLE meaning_deltas IS 'M1: MeaningDelta (ΔM); git_commit + file_path required; line range OR diff_ref required.';

-- RDE evaluation of a meaning delta (M1 model; distinct from spec Phase-1 rde_documents rows)
CREATE TABLE rde_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    meaning_delta_id UUID NOT NULL REFERENCES meaning_deltas (id) ON DELETE CASCADE,
    payload JSONB NOT NULL,
    audit_correlation_id TEXT,
    rde_document_id UUID REFERENCES rde_documents (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now ()
);

CREATE INDEX idx_rde_assessments_meaning_delta ON rde_assessments (meaning_delta_id);
CREATE INDEX idx_rde_assessments_payload_gin ON rde_assessments USING gin (payload jsonb_path_ops);
CREATE INDEX idx_rde_assessments_correlation ON rde_assessments (audit_correlation_id)
WHERE
    audit_correlation_id IS NOT NULL;

COMMENT ON TABLE rde_assessments IS 'M1: RDEAssessment JSONB bound to a MeaningDelta; optional link to interchange-derived rde_documents.';

-- Human or institutional review outcome (not a substitute for human authority)
CREATE TABLE review_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    meaning_delta_id UUID NOT NULL REFERENCES meaning_deltas (id) ON DELETE CASCADE,
    rde_assessment_id UUID REFERENCES rde_assessments (id) ON DELETE SET NULL,
    decision TEXT NOT NULL,
    decided_by TEXT NOT NULL,
    rationale JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT review_decisions_decision CHECK (
        decision IN (
            'approve',
            'hold',
            'reject',
            'needs_revision'
        )
    )
);

CREATE INDEX idx_review_decisions_meaning_delta ON review_decisions (meaning_delta_id);
CREATE INDEX idx_review_decisions_decision ON review_decisions (decision);

COMMENT ON TABLE review_decisions IS 'M1: ReviewDecision — approve/hold/reject/needs_revision; records accountability, does not auto-approve.';

-- Minimal external agent work unit (ChatGPT app, Claude Code, etc.)
CREATE TABLE agent_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    agent_kind TEXT NOT NULL,
    external_ref TEXT,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now ()
);

CREATE INDEX idx_agent_runs_kind ON agent_runs (agent_kind);

COMMENT ON TABLE agent_runs IS 'M1 minimal AgentRun log; full capability model deferred.';

ALTER TABLE meaning_deltas
ADD CONSTRAINT meaning_deltas_agent_run_id_fkey FOREIGN KEY (agent_run_id) REFERENCES agent_runs (id) ON DELETE SET NULL;

COMMIT;
