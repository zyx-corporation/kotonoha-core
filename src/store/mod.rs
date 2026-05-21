//! Optional persistence adapters.

#[cfg(feature = "postgres")]
pub mod agent_runs;

#[cfg(feature = "postgres")]
pub mod github_links;

#[cfg(feature = "postgres")]
pub mod principals;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub use agent_runs::{AgentRunRow, AgentRunStatus, DeniedActionRecord, StartAgentRunInput};
#[cfg(feature = "postgres")]
pub use github_links::{
    GithubIssueLinkRow, GithubPullRequestLinkRow, GithubRepoRef, GithubRepositoryLinkRow,
    GithubReviewCommentRefRow,
};
#[cfg(feature = "postgres")]
pub use principals::{LegacyDefaults, PrincipalKind, PrincipalRow, ProjectMemberRole, ProjectRow};
