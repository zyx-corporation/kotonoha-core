//! Optional persistence adapters.

#[cfg(feature = "postgres")]
pub mod github_links;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub use github_links::{
    GithubIssueLinkRow, GithubPullRequestLinkRow, GithubRepoRef, GithubRepositoryLinkRow,
    GithubReviewCommentRefRow,
};
