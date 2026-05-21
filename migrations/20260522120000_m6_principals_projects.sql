-- M6 Team Mode: principals, projects, project_members + FK columns.
-- Parent: https://github.com/zyx-corporation/kotonoha-management/issues/138
-- Issue: https://github.com/zyx-corporation/kotonoha-core/issues/35 (M6-a)

BEGIN;

CREATE TABLE principals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    external_ref TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT principals_kind_check CHECK (
        kind IN (
            'human',
            'service',
            'agent_channel'
        )
    )
);

CREATE UNIQUE INDEX idx_principals_external_ref ON principals (external_ref)
WHERE
    external_ref IS NOT NULL;

CREATE INDEX idx_principals_kind ON principals (kind);

COMMENT ON TABLE principals IS 'M6: actor identity (human, service, agent_channel).';

CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now ()
);

COMMENT ON TABLE projects IS 'M6: workspace / project scope for shared MeaningDelta history.';

CREATE TABLE project_members (
    project_id UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    principal_id UUID NOT NULL REFERENCES principals (id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    PRIMARY KEY (project_id, principal_id),
    CONSTRAINT project_members_role_check CHECK (
        role IN (
            'owner',
            'reviewer',
            'viewer',
            'agent_runner'
        )
    )
);

CREATE INDEX idx_project_members_principal ON project_members (principal_id);

COMMENT ON TABLE project_members IS 'M6: RBAC membership (owner | reviewer | viewer | agent_runner).';

-- Fixed UUIDs for legacy backfill (see docs/agent-schema-m6.md).
INSERT INTO
    principals (id, kind, display_name, external_ref)
VALUES (
    '00000000-0000-4000-8000-000000000001'::uuid,
    'service',
    'Legacy default principal',
    'kotonoha.m6.legacy-default'
);

INSERT INTO
    projects (id, slug, name)
VALUES (
    '00000000-0000-4000-8000-000000000002'::uuid,
    'default',
    'Default project'
);

INSERT INTO
    project_members (project_id, principal_id, role)
VALUES (
    '00000000-0000-4000-8000-000000000002'::uuid,
    '00000000-0000-4000-8000-000000000001'::uuid,
    'owner'
);

ALTER TABLE agent_runs
ADD COLUMN principal_id UUID REFERENCES principals (id) ON DELETE RESTRICT;

ALTER TABLE meaning_deltas
ADD COLUMN project_id UUID REFERENCES projects (id) ON DELETE RESTRICT;

ALTER TABLE review_decisions
ADD COLUMN principal_id UUID REFERENCES principals (id) ON DELETE SET NULL;

UPDATE agent_runs
SET
    principal_id = '00000000-0000-4000-8000-000000000001'::uuid
WHERE
    principal_id IS NULL;

UPDATE meaning_deltas
SET
    project_id = '00000000-0000-4000-8000-000000000002'::uuid
WHERE
    project_id IS NULL;

UPDATE review_decisions
SET
    principal_id = '00000000-0000-4000-8000-000000000001'::uuid
WHERE
    principal_id IS NULL;

ALTER TABLE agent_runs
ALTER COLUMN principal_id
SET NOT NULL;

ALTER TABLE meaning_deltas
ALTER COLUMN project_id
SET NOT NULL;

ALTER TABLE review_decisions
ALTER COLUMN principal_id
SET NOT NULL;

CREATE INDEX idx_agent_runs_principal ON agent_runs (principal_id);

CREATE INDEX idx_meaning_deltas_project ON meaning_deltas (project_id);

CREATE INDEX idx_review_decisions_principal ON review_decisions (principal_id);

COMMENT ON COLUMN agent_runs.principal_id IS 'M6: executing principal (required after backfill).';

COMMENT ON COLUMN meaning_deltas.project_id IS 'M6: project scope for shared lineage.';

COMMENT ON COLUMN review_decisions.principal_id IS 'M6: human reviewer principal (complements decided_by text).';

COMMIT;
