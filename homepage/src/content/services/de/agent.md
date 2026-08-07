---
title: "agent"
description: "Die Desktop-UI und der CLI-Agent basierend auf Goose."
lead: "Der CLI-Agent und Orchestration-Core, den du hinter den Kulissen bedienst. hirn_agent ist direkt auf Goose aufgebaut und verbindet sich über MCP v2 mit Tools, wodurch er zu einem leistungsfähigen lokalen Agenten wird."
icon: "agent"
highlights:
  - title: "MCP v2 Ökosystem"
    description: "Vollständige Unterstützung für den stateless MCP v2 Core, die MCP Apps Spezifikation für Rich-UIs und MCP Tasks für Hintergrund-Workflows."
  - title: "Natives Gefühl"
    description: "Keine trägen Web-Wrapper. Der Agent fühlt sich an wie ein nativer Bestandteil deines Betriebssystems, tief integriert in deine Entwicklertools."
  - title: "Hintergrund-Ausführung"
    description: "Läuft unauffällig in deiner Systemleiste, übernimmt langlaufende Aufgaben im Hintergrund und benachrichtigt dich bei Fertigstellung."
---

## In einem Befehl installieren

Installationsskripte erkennen dein Betriebssystem und deine Architektur, laden das passende Release-Artefakt herunter und legen alles unter `~/.hirn` ab.

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.ps1 | iex
```

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.sh | bash
```

## Nahtlose ModelRegistry-Integration

Durch die Nutzung von Gooses erweiterbarer `ModelRegistry` läuft die **hirn_router**-Erweiterung transparent im Hintergrund. Sie wechselt das aktive Modell unter der Haube pro Turn, sodass dein Chat-Verlauf und Kontextfenster ohne Unterbrechungen erhalten bleiben.

## Kuratierte Desktop-Plugins & MCP v2-Infrastruktur

Der Agent wird mit spezialisierten lokalen Plugins und Protokollstandards der nächsten Generation geliefert, die volle Kontrolle über deine Umgebung ermöglichen:

- **MCP v2 (Stateless Core):** Vollständig kompatibel mit der stateless Model Context Protocol Revision vom 28.07.2026. Macht protokollbasierte Sessions überflüssig und ermöglicht ultraschnelles HTTP-Gateway-Routing.
- **MCP Apps Spezifikation:** Alles im hirn-Ökosystem entwickelte Zeug ist 100% MCP v2-konform und nutzt die offizielle MCP Apps Spezifikation für interaktive, serverseitig gerenderte Interfaces.
- **MCP Tasks Erweiterung:** Native Unterstützung für die MCP Tasks Spezifikation zur Orchestrierung langlaufender Hintergrundaufgaben, asynchroner Workflows und reaktiver Benachrichtigungen.
- **Agent Control Protocol (ACP):** Standardisierte Schnittstelle zur Steuerung der Desktop-UI und Interaktion mit Anwendungen.
- **Lokaler Dateizugriff:** Sichere, berechtigungsgeprüfte Lese- und Schreibrechte für große lokale Codebasen.
- **Terminal-Ausführung:** Isolierte Terminal-Befehle, die vom Agenten direkt ausgeführt werden, um Tests zu starten, Skripte zu bauen oder Git zu verwalten.
