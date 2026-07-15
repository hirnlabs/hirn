# SDK

The Hirn SDK is the TypeScript-based interface for developers to build modular tools, agents, and applications that operate natively within the Hirn local-first ecosystem.

## Core Capabilities

- **Unified Data Access**: Provides bindings to the `data/` layer, abstracting the distinction between canonical file storage (Tier 1) and queryable indices and databases (Tier 3).
- **Collaboration & CRDTs**: Exposes hooks into the underlying CRDT engine, allowing tools to define collaborative structures that automatically sync across devices.
- **RPC & Inference**: Enables tools to perform remote procedure calls (RPC) and trigger model inference through the Rust-backed Sync Core.
- **P2P & Synchronization**: Simplifies encrypted, peer-to-peer communication and real-time state synchronization via the platform-agnostic transport layer.

## Architecture & Runtime

- **TypeScript Interface**: Provides the developer-facing API for tool building.
- **Backend Bridge (FFI)**: The SDK communicates with the **Rust Sync Core** via Foreign Function Interface (FFI). This ensures that heavy logic—CRDT merging, P2P networking, and model orchestration—runs in high-performance Rust, while the tools remain lightweight and portable across **Flutter** (mobile) and **Tauri** (desktop).
