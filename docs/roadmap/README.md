# 🗺️ Architecture de la Roadmap : The Grid

Ce répertoire contient l'ensemble de la planification technique de **The Grid**, découpée en jalons majeurs et en tâches de développement atomiques.

---

## 📂 Structure du Répertoire

```text
docs/roadmap/
├── README.md                                  # Ce guide d'indexation
│
├── milestone-01-to-07-foundation/             # Jalons 1 à 7 réunis (v0.1.0 - Alpha) ✅
│   └── specs.md                               # Spécifications & bilan des fondations
│
├── milestone-08-midi-and-playability/         # Jalon 8 (v0.2.0 - Jouabilité & MIDI) 🚀
│   ├── specs.md                               # Spécifications détaillées du jalon
│   ├── task-8.1-midi-protocol-lockfree-bridge.md
│   ├── task-8.2-monophonic-voice-allocator.md
│   ├── task-8.3-midi-in-dsp-module.md
│   ├── task-8.4-virtual-keyboard-ui.md
│   ├── task-8.5-hardware-midi-midir.md
│   └── task-8.6-default-playable-patch-e2e.md
│
├── milestone-09-graph-resilience-feedback/    # Jalon 9 (v0.3.0 - Feedback & Graphe) ⏳
│   └── specs.md                               # Spécifications initiales
│
├── milestone-10-spatial-ergonomics-canvas/    # Jalon 10 (v0.4.0 - Canvas & Workflow) ⏳
│   └── specs.md                               # Spécifications initiales
│
├── milestone-11-modular-catalog-expansion/    # Jalon 11 (v0.5.0 - Catalogue DSP étendu) ⏳
│   └── specs.md                               # Spécifications initiales
│
├── milestone-12-transport-sequencing/         # Jalon 12 (v0.6.0 - Transport & Séquenceurs) ⏳
│   └── specs.md                               # Spécifications initiales
│
├── milestone-13-polyphonic-engine/            # Jalon 13 (v0.7.0 - Polyphonie complète) ⏳
│   └── specs.md                               # Spécifications initiales
│
└── milestone-14-distribution-plugins/         # Jalon 14 (v1.0.0 - CLAP / VST3 & Packaging) ⏳
    └── specs.md                               # Spécifications initiales
```

---

## 📌 Règles de Gouvernance Technique
1. **Zéro-Allocation dans l'Audio Thread** : Aucune allocation dynamique n'est tolérée dans la boucle de rendu DSP.
2. **Atomicité des Tâches** : Chaque fichier de tâche décrit un livrable précis, une Definition of Done (DoD) non-ambiguë et une liste de vérifications testables.
3. **Divulgation Progressive** : Les jalons futurs ne sont découpés en tâches atomiques qu'à l'approche de leur démarrage effectif.
