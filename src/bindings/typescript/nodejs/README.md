# TypeScript Node.js Surface

The TypeScript-facing Node.js entrypoint lives here.

It re-exports the native `napi-rs` wrapper implemented under
[`../../nodejs/corsa_bind_node`](../../nodejs/corsa_bind_node)
and also exposes the shared remote transport helpers from
[`../typescript`](../typescript/README.md).

Use `CorsaUtils` for local Rust-backed helper calls, or `createRemoteCorsaUtils`
when you want the same shape over a remote host contract.
