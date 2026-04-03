# corsa-bind

Rust and Node bindings for `typescript-go` over stdio.

`corsa-bind` gives this repository a small, focused goal:

- talk to upstream `typescript-go` through supported API/LSP entry points
- keep the upstream checkout pinned by exact commit for reproducibility
- expose the hot path in Rust, with Node bindings on top
- build type-aware tooling such as `corsa-oxlint` without forking upstream

> [!WARNING]
> The project is still `0.x`.
> Core Rust and Node bindings are usable, but some upstream-facing surfaces are
> still experimental and distributed orchestration remains feature-gated.

> [!IMPORTANT]
> This repository does not maintain a fork of `typescript-go`.
> `ref/typescript-go` is treated as a managed upstream checkout and verified
> against [`tsgo_ref.lock.toml`](./tsgo_ref.lock.toml).

## What You Get

- `corsa_bind_client`: typed Rust client for the `tsgo` stdio API
- `corsa_bind_lsp`: Rust LSP client with virtual-document support
- `corsa_bind_orchestrator`: local worker pooling and cache reuse
- `@corsa-bind/node`: native Node bindings built with `napi-rs`
- `corsa-oxlint`: type-aware Oxlint helpers powered by `tsgo`
- `corsa_bind_ref`: tooling for syncing and verifying the pinned upstream repo

## Quick Start

Repository tasks are run through `vp` (Vite+).

Requirements:

- Rust toolchain
- Node `24`
- Go version compatible with [`ref/typescript-go/go.mod`](./ref/typescript-go/go.mod)

Sync the pinned upstream checkout:

```bash
vp run -w sync_ref
vp run -w verify_ref
```

Install dependencies, build, and run tests:

```bash
vp install
vp run -w build
vp test
```

Build the real pinned `tsgo` binary when you want real-upstream tests or examples:

```bash
vp run -w build_tsgo
```

## Common Tasks

```bash
vp run -w build
vp test
vp run -w examples_smoke
vp run -w examples_real
vp run -w bench_native
vp run -w bench_ts
```

## Examples

Examples live in [`examples/`](./examples/README.md).

- smoke examples: `vp run -w examples_smoke`
- real pinned-`tsgo` examples: `vp run -w examples_real`
- experimental distributed Rust example: `vp run -w examples_rust_experimental`

## Upstream Tracking

`typescript-go` moves quickly, so this repo treats upstream tracking as a first-class part of development.

- exact pin metadata lives in [`tsgo_ref.lock.toml`](./tsgo_ref.lock.toml)
- managed checkout lives in `ref/typescript-go`
- dirty or branch-attached upstream state fails verification
- update workflow and policy are documented in [`docs/tsgo_dependency.md`](./docs/tsgo_dependency.md)

## Project Notes

- default API transport is msgpack over stdio
- unstable upstream endpoints such as `printNode` are opt-in
- published npm packages expect a caller-managed `typescript-go` executable
- the distributed layer is still behind the `experimental-distributed` cargo feature

## More Docs

- architecture and workspace tour: [`docs/project_guide.md`](./docs/project_guide.md)
- production and release posture: [`docs/production_readiness.md`](./docs/production_readiness.md)
- support and compatibility policy: [`docs/support_policy.md`](./docs/support_policy.md)
- CI and local reproduction notes: [`docs/ci_guide.md`](./docs/ci_guide.md)
- benchmarking notes: [`docs/benchmarking_guide.md`](./docs/benchmarking_guide.md)
- performance snapshots: [`docs/performance.md`](./docs/performance.md)
- release workflow: [`docs/release_guide.md`](./docs/release_guide.md)
- supply-chain policy: [`docs/supply_chain_policy.md`](./docs/supply_chain_policy.md)
- Node package details: [`npm/corsa_bind_node/README.md`](./npm/corsa_bind_node/README.md)
- `corsa-oxlint` details: [`npm/corsa_oxlint/README.md`](./npm/corsa_oxlint/README.md)
