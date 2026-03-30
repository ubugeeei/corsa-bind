//! Orchestrator implementations and replicated-state data models.
//!
//! This module contains both the local worker pool and the experimental
//! distributed orchestration layer that mirrors cache and virtual-document state
//! through an in-process Raft core.

mod api;
mod distributed;
mod raft;
mod state;

/// Local worker-pool orchestrator with snapshot and result caches.
pub use api::ApiOrchestrator;
/// Distributed wrapper that replicates overlay and cache state.
pub use distributed::DistributedApiOrchestrator;
/// Raft topology and leadership state used by the distributed orchestrator.
pub use raft::{RaftCluster, RaftRole};
/// Serializable state mirrored across the distributed orchestrator cluster.
pub use state::{ReplicatedCacheEntry, ReplicatedCommand, ReplicatedSnapshot, ReplicatedState};
