# Data Context

The persistence management subsystem managing the local-first storage hierarchy (File, SQLite, Vector, and Grafeo).

## Language

**Canonical Storage (Tier 1)**:
The primary source of truth stored as human-readable files (Markdown, JSON) directly on the file system.

**Collaboration Layer (Tier 2)**:
Binary CRDT overlay (y-crdt/Automerge) that tracks live collaborative state and coordinates sync.

**Queryable Indices (Tier 3)**:
A multi-modal index layer derived from Tier 1 and Tier 2, containing SQLite, Grafeo, and Vector databases.

**Grafeo**:
The local-first graph database engine for modeling entity relationships.
