# Architecture: Hirn Local-First

This document outlines the foundational architecture for the Hirn ecosystem, designed for extreme durability, offline resilience, and collaborative capabilities.

## 1. Local-First & CRDT Philosophy
Hirn follows the "Local-First" software principles. The system treats the local device as the primary source of truth, with synchronization as a background process.
- **Offline Resilience**: Users can perform all operations offline. Changes are queued and merged when connectivity is restored.
- **CRDT Substrate**: We utilize conflict-free replicated data types (CRDTs) to allow seamless, real-time collaboration without centralized locking or complex merge conflicts.

## 2. The Storage Hierarchy
We maintain a three-tier storage model to balance human usability, machine queryability, and collaborative synchronization:

### Tier 1: Canonical Storage (File-over-App)
The primary data is stored as **human-readable files** (Markdown, JSON, Plain Text).
- **Rationale**: User ownership is paramount. Files remain accessible to any system tool (`grep`, `vim`, `git`) regardless of whether Hirn is running.
- **Durability**: If the application layer fails, data remains intact.

### Tier 2: Collaboration Layer (CRDT)
A binary overlay layer (using `y-crdt` or `Automerge`) tracks live collaborative state.
- **Snapshots**: Periodic binary snapshots + log segments.
- **Compaction**: Append-only logs are compacted to maintain performance, but can be discarded/rebuilt because the Tier 1 Files are the source of truth.

### Tier 3: Queryable Indices
A multi-modal index composed of specialized storage engines, all derived from Tier 1 and Tier 2:
- **SQLite**: For structured, relational metadata, task tracking, and application configuration.
- **Grafeo**: A local-first graph database engine for knowledge linking, modeling entity relationships, and traversing complex connections.
- **Vector Database**: For semantic search, RAG (Retrieval-Augmented Generation), and high-dimensional similarity retrieval.

> **Note**: All indices are maintained in sync with your files, providing a high-performance query layer for your agents and tools.

## 3. The Sync Core (Rust)
The complexity of distribution is encapsulated in a unified Rust core, shared across all platforms (Flutter, Tauri/Rust, Web).

### The Envelope Pattern
All communication, whether CRDT synchronization or RPC, uses a standardized `Envelope`:
```rust
struct Envelope {
    id: uuid::Uuid,
    doc_id: Option<DocId>,       // The shard/doc identifier
    kind: EnvelopeKind,          // CRDTUpdate, RPCCall, RPCResult
    sender_id: ReplicaId,
    target_id: Option<ReplicaId>,
    payload: Vec<u8>,            // Binary data
    seq: u64,                    // Sequencing for ordering
    created_at: DateTime,
}
```

### Pluggable Transports
The Sync Core is transport-agnostic. It implements a trait that can be filled by:
- **Localhost**: Inter-process (Tauri ↔ Flutter ↔ CLI).
- **Relay**: WebSocket-based store-and-forward for offline delivery.
- **Tailscale/tsnet**: Encrypted P2P mesh.

## 4. Scaling
- **Sharding**: State is sharded per-document. Users only sync the state for the docs they are active in.
- **RPC Orchestration**: Remote function calls are treated as first-class messages in the envelope stream, allowing clients to trigger tasks or index rebuilds on remote peers.
