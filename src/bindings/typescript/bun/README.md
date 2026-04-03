# Bun Binding

The Bun binding reuses the TypeScript-facing Node.js surface and keeps the
shared remote transport available for host-based deployments.

See [`index.ts`](./index.ts). That surface includes the local `CorsaUtils`
helper object plus the shared remote utility adapter exports.
