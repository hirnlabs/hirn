---
title: "server"
description: "Effiziente, skalierbare Modellbereitstellung über mehrere Geräte hinweg."
lead: "Hochskalierende Modell-Server-Infrastruktur. Der verteilte RPC-Server führt Inferenz effizient auf lokaler Hardware und vernetzten Geräten aus."
icon: "server"
highlights:
  - title: "Hoher Durchsatz"
    description: "Optimiert für dauerhaften Mehreingaben-Betrieb mit Caching, Batching und effizienten Ausführungspfaden."
  - title: "Plattformübergreifend"
    description: "Nativ optimiert für Windows, Linux und macOS, skaliert flexibel über unterschiedliche Hardware hinweg."
  - title: "Skalierende Ressourcen"
    description: "Teilt VRAM und Systemressourcen dynamisch zu, damit auch große Modelle praxistauglich lokal laufen."
---

## Serving-Infrastruktur statt Routing-Logik

**hirn_server** konzentriert sich auf schnelle und zuverlässige Modellausführung im großen Maßstab. Er sagt keine Absichten voraus, sondern führt die vom Router ausgewählten Anfragen aus.

Der Server stellt eine einheitliche RPC-Schnittstelle bereit und kann verschiedene Backends ansteuern:

- **llama.cpp:** Für hochoptimiertes CPU/GPU-Offloading auf Konsumenten-Hardware.
- **vLLM:** Für Durchsatz-orientiertes Serving auf größeren VRAM-Setups.
- **exo / cactus:** Für experimentelle und spezialisierte lokale Routing-Architekturen.

## Verteilt über mehrere Geräte

Das Ausführen großer Modelle erfordert oft mehr VRAM, als eine einzelne Maschine bieten kann. Der Server bündelt Kapazitäten über mehrere GPUs und vernetzte Geräte hinweg.

## Einfacher Betrieb

Das Ziel ist leistungsfähiges Serving ohne schwere Betriebsaufwände: konsistente RPC-Verträge, Backend-agnostische Ausführung und vorhersagbares Verhalten unter Last.

Kurz gesagt: Der Router entscheidet, *was* laufen soll; der Server entscheidet, *wie* es am effizientesten auf verfügbarer Hardware ausgeführt wird.
