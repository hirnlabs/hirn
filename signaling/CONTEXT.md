# Signaling Context

Minimal Rust WebRTC signaling server and encrypted store-and-forward message relay for Hirn.

## Language

**Signaling Server**:
The minimal Rust WebRTC handshake relay server (`axum` + `tokio`) facilitating peer-to-peer data channel negotiation and containerized as a tiny Docker image (~15MB). Evaluates Matrix Protocol MSC3401 signaling standards for peer discovery.

**Encrypted Relay Queue**:
An append-only store-and-forward queue holding encrypted binary envelopes for offline device synchronization, referencing Matrix Megolm/Olm E2EE event queue mechanics.
