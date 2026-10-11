# Feuille de Route (Roadmap) : The Grid

Cette feuille de route formalise les jalons d'implémentation de **The Grid**, de l'architecture initiale jusqu'au moteur modulaire complet.

---

## Vue d'Ensemble des Jalons

```text
[x] Étape 1 : Arborescence Workspace, Toolchain & Scaffolding
[x] Étape 2 : Runtime Audio CPAL, Canaux SPSC Lock-Free & Télémétrie
[x] Étape 3 : Modèle de Patch, Algorithme Topologique & Commandes IPC
[x] Étape 4 : Télémétrie Décimée Host -> Frontend & Vu-Mètres UI
[x] Étape 5 : Rendu Canvas 2D Interactif (Grille & Câbles de Bézier)
[ ] Étape 6 : Primitives DSP (Oscillateur Sin/Saw/Square, Filtre SVF, Enveloppe ADSR)
[ ] Étape 7 : Rendu Audio Réel & Gestion Polyphonique / Modulation CV
```

---

## Détail des Jalons

### ✅ Étape 1 : Arborescence Workspace, Toolchain & Scaffolding
- **Statut :** Terminé.
- **Livrables :**
  - Cargo Workspace avec crates isolées : `grid-core`, `grid-engine`, `grid-app`.
  - Frontend initialisé avec Vite, TypeScript et Bun.
  - Configuration Tauri v2 (`tauri.conf.json`) et styles de base (dark theme Bitwig).

### ✅ Étape 2 : Runtime Audio CPAL, Canaux SPSC Lock-Free & Télémétrie
- **Statut :** Terminé (11 tests passés, > 98% couverture logique DSP).
- **Livrables :**
  - Driver CPAL stéréo [`AudioDriver`](file:///home/freebox/the-grid/crates/grid-engine/src/driver.rs).
  - Processeur temps réel [`RealtimeAudioProcessor`](file:///home/freebox/the-grid/crates/grid-engine/src/realtime.rs) garanti sans allocation (`no malloc`) et sans mutex (`no lock`).
  - Ring buffers SPSC lock-free [`HostChannels` / `EngineChannels`](file:///home/freebox/the-grid/crates/grid-engine/src/ringbuffer.rs) (`rtrb`).
  - Télémétrie décimée à 60 Hz avec politique `drop-on-full`.
  - Registre de référence dans [`GUIDELINES.md`](file:///home/freebox/the-grid/GUIDELINES.md).

---

### ⏳ Étape 3 : Modèle de Patch, Algorithme Topologique & Commandes IPC
- **Objectifs :**
  1. Enrichir les structures de données du patch dans `grid-core` :
     - Définition d'un catalogue de modules (`Oscillator`, `Gain`, `AudioOut`).
     - Gestion des ports d'entrée/sortie typés (`Audio`, `CV`, `Trigger`).
     - Gestion des câbles connectant deux ports.
  2. Implémenter le tri topologique (Kahn / DFS DAG) dans `grid-core/src/graph.rs` :
     - Détection stricte des cycles de feedback immédiats.
     - Ordonnancement séquentiel des nœuds.
  3. Implémenter le compilateur `grid-core/src/compiler.rs` produisant un `ExecutionPlan` linéaire avec indexation de scratch buffers.
  4. Ajouter la commande IPC Tauri `apply_patch` dans `grid-app` qui valide, compile hors thread audio, et transmet le plan au moteur.
- **Critères de validation :**
  - Tests unitaires complets sur le tri topologique, la détection de cycle et l'assignation de mémoire.
  - Maintien strict du taux de couverture.

---

### ⏳ Étape 4 : Télémétrie Décimée Host -> Frontend & Vu-Mètres UI
- **Objectifs :**
  1. Implémenter dans `grid-app` une tâche de fond asynchrone (thread ou Tokio task) dépilant `telemetry_consumer` à 30–60 Hz.
  2. Émettre l'événement Tauri `telemetry_update` (`app.emit`) vers le WebView.
  3. Côté frontend : écouter l'événement via `@tauri-apps/api/event` et afficher un vu-mètre stéréo dynamique réactif (LED/barre de niveau crête et RMS).
- **Critères de validation :**
  - Aucune charge excessive sur l'IPC Tauri (décimation stricte garantie).
  - Tests d'intégration validant le cycle de dépilage.

---

### ⏳ Étape 5 : Rendu Canvas 2D Interactif (Grille & Câbles de Bézier)
- **Objectifs :**
  1. Moteur de rendu Canvas 2D léger et réactif dans `frontend/src/canvas/` :
     - Grille modulaire à pas fixe (ex: 32px ou 48px).
     - Rendu des modules sous forme de cartes modulaires avec slots de ports.
     - Tracé des connexions par courbes de Bézier cubiques stylisées couleur par type de signal (Audio orange, CV bleu, Trigger jaune).
  2. Interactions utilisateur : drag & drop des modules, création de câbles au clic/glisser entre ports compatibles.
  3. Synchronisation automatique avec le backend via `apply_patch`.
- **Critères de validation :**
  - Fluidité 60/120 FPS sur Canvas.
  - Validation interactive des connexions (interdire le raccordement sortie-sortie ou boucles illégales).

---

### ⏳ Étape 6 : Primitives DSP (Modules Sonores)
- **Objectifs :**
  1. Implémentation des structures DSP préallouées dans `grid-engine/src/dsp/` :
     - `Oscillator` : Sine, Triangle, Sawtooth, Pulse (anti-aliasing de base / PolyBLEP).
     - `StateVariableFilter` (SVF) : Low-pass, High-pass, Band-pass résonant.
     - `Envelope` : ADSR linéaire / exponentiel déclenché par Trigger.
     - `VcaGain` : Atténuateur / amplificateur contrôlé en tension.
  2. Rendu audio audible via CPAL.
- **Critères de validation :**
  - Stabilité numérique des filtres, absence de division par zéro ou NaN.
  - Respect absolu de la règle Zero-Alloc pendant le rendu des échantillons.

---

### ⏳ Étape 7 : Rendu Audio Réel & Gestion Polyphonique / Modulation CV
- **Objectifs :**
  1. Support de modulations croisées en temps réel (FM linéaire, modulation de coupure de filtre).
  2. Entrée MIDI / Clavier virtuel.
  3. Export et import de fichiers `.grid` (JSON).
- **Critères de validation :**
  - Test de stress de charge CPU (< 5% pour 30 modules à 48 kHz).
