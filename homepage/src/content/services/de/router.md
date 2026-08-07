---
title: "router"
description: "Absichtsmodell zur Auswahl von Aufgaben- und Modell-Tiers."
lead: "Ein intelligentes, kostensparendes lokales Modell-Gateway. Es sitzt transparent hinter deinen Agenten. Du behältst deine Pro-Abonnements; der Router entscheidet pro Turn, welches Modell am effizientesten und schnellsten ist."
icon: "router"
highlights:
  - title: "Kostenersparnis"
    description: "Durch Auslagerung großer Kontextaufgaben spart der Router bis zu 80% der API-Kosten bei voller Denkqualität."
  - title: "Null Latenz"
    description: "Der ~26M Parameter starke lokale Klassifizierer führt Entscheidungen in unter 50ms auf deinem Gerät aus."
  - title: "Selbstverbessernd"
    description: "Jede Korrektur kann in learn eingespeist werden, um den Router auf deinen persönlichen Workflow zu trainieren."
---

## Intent-basierte Steuerung

Anstatt alle Prompts gleich zu behandeln, sagt **hirn_router** die Absicht des Nutzers anhand von Merkmalen und Kontext voraus. Das Ergebnis ist eine strukturierte Routing-Entscheidung, die bestimmt, welches Modell als Nächstes läuft.

Hier leben Kosten- und Qualitätspolicies. Der Router bewertet, ob eine Anfrage routinemäßig, mittelschwer oder denkintensiv ist, und ordnet sie dem passenden Leistungstier zu.

## Das Problem: Undurchsichtige Preise & Token-Limits

Spitzenmodelle sind teuer und Planlimits schnell erreicht. Große Codebase-Prompts verbrennen Guthaben rasch – selbst wenn die Aufgabe nur moderate Fähigkeiten erfordert. Zum Beispiel kostet Claude Fable 5 $10/1M Input und $50/1M Output.

Der Router verhindert Überausgaben, indem er die Absicht zuerst bestimmt und eine deterministische Auswahlpolitik anwendet.

## Router-Entscheidungspipeline

Statt alles an ein Spitzenmodell zu senden, nutzt **hirn_router** eine mehrstufige Pipeline zur Evaluierung und Auswahl des optimalen Modells pro Turn.

1. **Intent-Klassifizierung:** Sagt die Nutzerabsicht aus Prompt-Inhalt, Sitzungskontext und Signalen voraus.
2. **Tier- und Pfadauswahl:** Ordnet die Absicht einem Pfad und Modell-Tier zu und bewertet Kandidaten nach Latenz, Qualität, Kosten und Limitdruck.
3. **Ausführungsübergabe:** Übergibt die Entscheidung an Ausführungskomponenten (insbesondere **hirn_server**).
4. **Feedback-Erfassung für learn:** Erfasst Ergebnisse und Korrekturen für das lokale Training.

## Die Mathematik: Gewichtigte Normalisierte Summe

Für Modellkandidaten im vorhergesagten Tier wendet der Router einen gewichteten normalisierten Score an:

```text
score = (w_q * Q) - (w_l * L) - (w_c * C) - (w_limit * U)
```

- **Q:** Normalisiertes Qualitätssignal für die Zielklasse.
- **L:** Normalisierte Latenzerwartung.
- **C:** Erwartete Grenzkosten der Anfrage.
- **U:** Malus für die Nähe zu Quoten- und Tokenlimits.

Der höchste Score gewinnt deterministisch.

## Trainiert von learn

Der Router verbessert sich kontinuierlich über **hirn_learn**, das Datensatz-Kuratierung und lokales Finetuning auf Basis deiner echten Agenten-Sitzungen bietet.
