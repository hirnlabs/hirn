# Signaling

Minimal Rust WebRTC signaling server and encrypted store-and-forward message relay for Hirn.

## Core Responsibilities

- **WebRTC Handshake**: Coordinates SDP offer/answer exchanges and ICE candidate gathering between peers across local and remote networks (evaluated against Matrix MSC3401 signaling specs).
- **Encrypted Message Relay**: Stores and forwards end-to-end encrypted envelope updates (`y-crdt` / Envelope streams) for offline devices to sync asynchronously upon reconnecting.
- **Peer Identity Verification**: Verifies peer identities via SHA-256 DTLS certificate fingerprints exchanged over secure signaling connections.
- **Configurable Endpoint**: Integrates natively with Desktop, Agent CLI, and Assistant clients using the `HIRN_RELAY_URL` environment variable or UI setting.

## Architecture & Self-Hosting

- **Minimal Rust Core**: Built with high-performance, asynchronous Rust (`axum`, `tokio`, `tokio-tungstenite`) for minimal CPU and memory overhead.
- **Tiny Docker Container**: Packaged via multi-stage builds into a minimal scratch/distroless image (~15MB container footprint).
- **Open Source & Self-Hostable**: 100% open source service easily deployable on any VPS, home server, or container cluster.
- **Licensing & Sync Tier**: Serves as the signaling anchor for commercial Hirn Sync subscriptions while remaining free and open for self-hosted community usage.

## Quick Start (Docker)

```bash
docker run -d \
  -p 8080:8080 \
  -e PORT=8080 \
  --name hirn-signaling \
  ghcr.io/hirnlabs/signaling:latest
```
