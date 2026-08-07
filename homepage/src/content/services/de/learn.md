---
title: "learn"
description: "Lokale Finetuning-Schleife für deinen persönlichen Router."
lead: "Dein Router sollte keine generischen Regeln nutzen; er sollte deinen spezifischen Workflow lernen. hirn_learn bietet die Tools zum Sammeln, Labeln und Finetunen deines lokalen Klassifizierers."
icon: "learn"
highlights:
  - title: "Kontinuierliches Lernen"
    description: "Je mehr du hirn nutzt, desto präziser sagt der Router voraus, wie viel Compute deine spezifischen Prompts benötigen."
  - title: "Privacy First"
    description: "Alle Finetuning-Daten bleiben lokal auf deiner Festplatte. Keinerlei Prompts, Präferenzen oder Codebasen verlassen dein System."
  - title: "Ein-Klick-Finetune"
    description: "Trainiere deinen Klassifizierer nativ in der Benutzeroberfläche mit einem Klick und wende die aktualisierten Modellgewichte sofort an."
---

## Datenerfassung

Während du arbeitest, protokolliert der Router jede Interaktion lokal im Hintergrund. Diese Protokolle werden ausschließlich auf deiner lokalen Festplatte unter `~/.hirn/logs/` gespeichert und niemals hochgeladen.

```json
{"input": "Datenlayer refactorn", "intent": "coding", "files": 12, "chosen_model": "gemini-3.5-flash"}
{"input": "Speicherleck analysieren", "intent": "debugging", "files": 3, "chosen_model": "fable-5"}
```

## Die Trainingsschleife

Mit einem für Function Calling optimierten Mini-Modell wie **Needle** kannst du das Verhalten des Routers schnell an deine persönlichen Vorlieben anpassen.

Starte die lokale **hirn UI**, um historische Protokolle zu prüfen, Fehlroutings zu korrigieren (z. B. festzulegen, dass bestimmte PR-Reviews immer Fable 5 nutzen sollen) und einen hochwertigen eigenen Datensatz aufzubauen.

Sobald du einen kuratierten Datensatz hast, bringt ein Ein-Klick-Finetuning-Prozess Needle bei, deine spezifischen Eingaben auf den korrekten Routing-Funktionsaufruf abzubilden, um Token-Kosten und Latenz dauerhaft zu optimieren.
