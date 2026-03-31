# Production Readiness Guide

This document is the short operational checklist for running `tsgo-rs` in production-style environments.

## Scope

The current production target is:

- local Rust and Node API clients
- LSP stdio integrations
- local worker orchestration and cache reuse

The following remains experimental:

- distributed orchestration
- the in-process Raft replication layer
- upstream endpoints called out as unstable by this repository

## Default Safety Controls

The default runtime configuration now includes:

- per-request timeout: `30s`
- graceful shutdown timeout: `2s`
- bounded outbound queue capacity: `256`
- unstable upstream endpoints disabled by default

These defaults can be overridden through:

- `ApiSpawnConfig`
- `LspSpawnConfig`
- `ApiOrchestratorConfig`

## Recommended Settings

For long-lived services:

- keep `request_timeout` enabled
- reduce `outbound_capacity` if you prefer earlier backpressure
- tune `max_cached_snapshots` and `max_cached_results` to fit process memory budgets
- leave unstable upstream endpoints disabled unless you have a concrete need and a rollback plan

For editor-like integrations:

- use stable cache keys for snapshots
- prewarm a small worker fleet instead of spawning per request
- treat the distributed orchestrator as experimental unless you are actively developing it

## Release Checklist

- `vp check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `vp run -w test`
- `vp run -w bench_verify`
- `vp run -w verify_ref`

## Cross-Platform Expectations

The main quality workflow is intended to stay green on:

- Linux
- macOS
- Windows

The pinned real-`tsgo` integration and benchmark verification currently remain concentrated in the dedicated `tsgo-ref` job.
