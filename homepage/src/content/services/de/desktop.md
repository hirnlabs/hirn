---
title: "desktop"
description: "Tauri v2 & Svelte 5 Multi-Agent Desktop-GUI und Tool-Host für Windows, macOS und Linux."
lead: "Eine aufgeräumte, minimale Benutzeroberfläche inspiriert von Linear und Anytype. Entwickelt mit Tauri v2, Svelte 5 und TypeScript für Multi-Agenten ACP-Chat, sandboxed MCP App-Hosting, unterbrechungsfreie Ausführung und nahtlose CLI-Synchronisation."
icon: "desktop"
highlights:
  - title: "Multi-Agent ACP Chat"
    description: "Verbinde dich mit mehreren lokalen (hirn acp, goose, claude) und entfernten ACP-Agenten in einer zentralen Oberfläche."
  - title: "Tauri v2 + Svelte 5"
    description: "Reaktive Svelte 5 Runes ($state, $derived) in einer ultraschnellen nativen Desktop-Shell für flüssiges Session-Management."
  - title: "Kanonische CLI-Sync"
    description: "Zentraler Sitzungsspeicher unter ~/.hirn/sessions/ hält den Verlauf 100% synchron zwischen CLI und Desktop."
  - title: "Auto-Update"
    description: "Native Hintergrund-Updates beim Start via tauri-plugin-updater mit Ein-Klick-Neustart."
---

![Hirn Desktop Benutzeroberfläche](/assets/desktop-dark.png)

## Zentraler Multi-Agenten-Workspace

Hirn Desktop bietet eine übersichtliche, minimale Benutzeroberfläche zur simultanen Orchestrierung mehrerer Agent Client Protocol (ACP) Agenten-Sitzungen, egal ob lokal auf deinem Computer, im lokalen Netzwerk oder über verschlüsselte P2P-Verbindungen.

- **Unterbrechungsfreie Hintergrund-Ausführung:** Wechsle nahtlos zwischen Agenten-Sitzungen, während Hintergrund-Agenten weiterdenken, Antworten streamen oder Tools ausführen.
- **Sitzungsstatus-Indikatoren:** Echtzeit-Statusanzeigen für `idle`, `working` (pulsierender Status), `waiting_for_input` (Rechte-Freigaben) und `error`.
- **Kanonischer Sitzungsspeicher:** Alle Sitzungen werden direkt unter `~/.hirn/sessions/<session_id>.json` gespeichert und teilen eine einheitliche Datenquelle für CLI, Desktop und Web.

## Transportschicht-Architektur (Dual-Mode)

Angetrieben von einer abstrakten `AcpTransport`-Schnittstelle wechselt die Benutzeroberfläche nahtlos zwischen verschiedenen Ausführungsmodi:

```mermaid
graph TD
    UI["HIRN DESKTOP (Svelte 5)<br/>Sidebar Sessions & Central Chat View"]
    Session["AcpClientSession<br/>(ACP Initialize Handshake & Message Streaming)"]
    Tauri["TauriIpcTransport<br/>(Local Process / Rust tokio)"]
    WS["WebSocketTransport<br/>(Network / hirn serve)"]
    P2P["P2pWebRtcTransport<br/>(WebRTC P2P Sync)"]

    UI --> Session
    Session --> Tauri
    Session --> WS
    Session --> P2P
```

- **Tauri IPC Transport (`TauriIpcTransport`):** Führt lokale ACP-Agenten-Prozesse direkt über Rust `tokio::process`-Pipes (`stdin`/`stdout`) aus und überträgt Newline-Delimited JSON (NDJSON) über Tauri-IPC.
- **WebSocket Bridge (`WebSocketTransport`):** Verbindet sich mit `hirn serve` oder Netzwerk-Endpunkten über `ws://` / `wss://` für browserbasierte Workflows.
- **P2P WebRTC Sync (`P2pWebRtcTransport`):** Ermöglicht verschlüsselte Peer-to-Peer Agenten-Verbindungen mit Store-and-Forward Nachrichtenwarteschlangen über verschiedene Geräte hinweg.

## Isolierter MCP Tool Host (`hirn://`)

Hirn Desktop agiert als modularer Host für interaktive Werkzeug-Oberflächen (MCP Apps). Über eigene native URI-Schemata (`tauri::UriScheme`) werden lokale App-Pakete sicher unter `hirn://apps/<app-id>/index.html` ohne offene HTTP-Ports bereitgestellt.

## Installation & Deployment

### Native Desktop-Installer

Vorkompilierte Installer stehen auf GitHub Releases zur Verfügung:

- **Windows:** `.msi` oder `.exe` Installer.
- **macOS:** `.dmg` Disk-Image.
- **Linux:** `.AppImage` oder `.deb` Paket.

### Standalone Web Container (Docker)

Starte den Svelte 5 Web Client als leichtgewichtigen Container:

```bash
docker run -d \
  -p 80:80 \
  --name hirn-web \
  ghcr.io/hirnlabs/desktop-web:latest
```
