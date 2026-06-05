//! **Kotonoha OSS core** — semantic lineage primitives and RDE interchange validation.
//!
//! Behaviour aligns with [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec).
//! See repository `docs/spec-traceability.md` for section mapping.

/// [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) bundle targeted by this crate release for interchange validation.
pub const TARGET_SPEC_BUNDLE: &str = "0.1";

pub mod context_pack;
pub mod git;
pub mod interchange;
pub mod lineage;
pub mod meta_rde;
pub mod observation_rde;
pub mod rde;
pub mod rde_attach;
pub mod rde_classifier;
pub mod rde_delta;
pub mod rde_evidence;
pub mod rde_impl;
pub mod rde_semantic;
pub mod semantic_lineage;

#[cfg(feature = "postgres")]
pub mod store;
