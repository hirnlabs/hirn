# Data

The Data module manages the persistence layer for the Hirn ecosystem, implementing the "File-over-App" and "Local-First" storage hierarchy.

## Storage Strategies

- **Persistent Store**: SQLite-based schemas for user configuration, session history and tool data.
- **Vector DB**: Vector embeddings storage for semantic search and retrieval (RAG).
- **Graph DB**: Graph-based storage for relationships and connections between entities.
- **Cache**: high throughput in-memory stores for high-frequency data access.
- **File System**: The canonical source of truth for most user data (like notes, tasks, documents), stored as human-readable markdown files on disk.

## The Storage Hierarchy

- **Canonical Storage (Tier 1)**: The primary source of truth. User data (notes, tasks, documents) is stored as **human-readable files** on disk. This ensures user data longevity and interoperability with other system tools.
- **Collaboration Layer (Tier 2)**: Persistent binary overlays (snapshots + op logs) for CRDT-based state management. This data is derived from Tier 1 but optimized for merging and real-time collaboration.
- **Queryable Index (Tier 3)**: A multi-modal index composed of specialized, local-file-based storage engines. These databases (SQLite, Grafeo for graphs, and local Vector DBs) are fully modifiable by both the user and agents, enabling high-performance relational queries, complex knowledge traversal, and RAG-capable semantic search.

## Key Features

- **Performance & Sync**: The query index (Tier 3) and collaborative state (Tier 2) are kept in sync with the canonical files (Tier 1), ensuring high-speed access while maintaining file-based durability.
- **Compaction**: The module handles log-structured compaction, periodically flattening CRDT logs into snapshots to ensure performance remains stable as data grows.
- **SDK Bindings**: The `sdk/` module uses the Data module as its primary persistence engine, providing tools with a seamless interface to read/write files and execute queries against the index.
