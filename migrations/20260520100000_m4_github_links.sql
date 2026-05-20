-- M4 GitHub correlation tables (additive).
-- Parent: https://github.com/zyx-corporation/kotonoha-management/issues/105
-- Issue: https://github.com/zyx-corporation/kotonoha-core/issues/32
-- Spec: https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/30_m4_github_integration_spec_draft.md

BEGIN;

-- Logical GitHub repo binding (project_id reserved for M6 Team Mode).
CREATE TABLE github_repository_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    owner TEXT NOT NULL,
    repo TEXT NOT NULL,
    project_id UUID,
    default_branch TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT github_repository_links_owner_repo_unique UNIQUE (owner, repo)
);

CREATE INDEX idx_github_repository_links_project ON github_repository_links (project_id)
WHERE
    project_id IS NOT NULL;

COMMENT ON TABLE github_repository_links IS 'M4: owner/repo anchor; GitHub holds canonical text, DB holds IDs and correlation.';
COMMENT ON COLUMN github_repository_links.project_id IS 'M6: optional workspace project FK (not enforced in M4).';

-- MeaningDelta ↔ GitHub Issue number (per repository).
CREATE TABLE github_issue_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    repository_link_id UUID NOT NULL REFERENCES github_repository_links (id) ON DELETE CASCADE,
    meaning_delta_id UUID NOT NULL REFERENCES meaning_deltas (id) ON DELETE CASCADE,
    issue_number INTEGER NOT NULL CHECK (issue_number > 0),
    issue_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT github_issue_links_unique UNIQUE (
        repository_link_id,
        meaning_delta_id,
        issue_number
    )
);

CREATE INDEX idx_github_issue_links_delta ON github_issue_links (meaning_delta_id);
CREATE INDEX idx_github_issue_links_repo_issue ON github_issue_links (repository_link_id, issue_number);

COMMENT ON TABLE github_issue_links IS 'M4: correlate MeaningDelta with a GitHub Issue (number + optional URL).';

-- MeaningDelta and/or RDEAssessment ↔ Pull Request.
CREATE TABLE github_pull_request_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    repository_link_id UUID NOT NULL REFERENCES github_repository_links (id) ON DELETE CASCADE,
    meaning_delta_id UUID REFERENCES meaning_deltas (id) ON DELETE CASCADE,
    rde_assessment_id UUID REFERENCES rde_assessments (id) ON DELETE CASCADE,
    pr_number INTEGER NOT NULL CHECK (pr_number > 0),
    pr_url TEXT,
    head_sha TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT github_pull_request_links_target CHECK (
        meaning_delta_id IS NOT NULL
        OR rde_assessment_id IS NOT NULL
    )
);

CREATE UNIQUE INDEX idx_github_pr_links_delta ON github_pull_request_links (
    repository_link_id,
    pr_number,
    meaning_delta_id
)
WHERE
    meaning_delta_id IS NOT NULL;

CREATE UNIQUE INDEX idx_github_pr_links_rde ON github_pull_request_links (
    repository_link_id,
    pr_number,
    rde_assessment_id
)
WHERE
    rde_assessment_id IS NOT NULL;

CREATE INDEX idx_github_pr_links_head_sha ON github_pull_request_links (head_sha)
WHERE
    head_sha IS NOT NULL;

COMMENT ON TABLE github_pull_request_links IS 'M4: PR correlation; head_sha aligns with meaning_deltas.git_commit for listing.';

-- ReviewDecision ↔ posted GitHub review comment (after human posts).
CREATE TABLE github_review_comment_refs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    repository_link_id UUID NOT NULL REFERENCES github_repository_links (id) ON DELETE CASCADE,
    review_decision_id UUID NOT NULL REFERENCES review_decisions (id) ON DELETE CASCADE,
    github_comment_id BIGINT NOT NULL,
    comment_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
    CONSTRAINT github_review_comment_refs_unique UNIQUE (
        repository_link_id,
        review_decision_id,
        github_comment_id
    )
);

CREATE INDEX idx_github_review_comment_refs_decision ON github_review_comment_refs (review_decision_id);

COMMENT ON TABLE github_review_comment_refs IS 'M4: pointer to GitHub review comment after ReviewDecision is reflected on GH.';

COMMIT;
