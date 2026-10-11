# 🏛️ Jalons 1 à 7 : Fondations & Moteur Modulaire v0.1.0

> **Statut :** Complété ✅ (v0.1.0 - Alpha Fondatrice)  
> **Scope :** Architecture multi-crates, moteur DSP temps réel, compilateur DAG, patcher Canvas 2D, IPC & télémétrie, persistance JSON.

---

## 1. Objectifs & Vision Initiale

Établir une base logicielle ultra-performante et résiliente pour **The Grid**, un environnement modulaire inspiré de *Bitwig The Grid* et *VCV Rack*, écrit en Rust et piloté par une interface utilisateur réactive (Tauri v2 + Canvas 2D).

### Contraintes Temps Réel Non-Négociables (Audio Thread)
- **Zero-Allocation** : Aucune allocation mémoire (`malloc`, `Box`, `Vec::push`, `String`) dans la boucle de rendu audio.
- **Zero-Lock** : Aucun mutex, rwlock ou blocage de synchronisation sur le thread audio.
- **Zero-I/O** : Aucun appel système, écriture disque ou socket réseau dans la boucle DSP.
- **Bufferisation statique** : Blocs DSP fixes de 64 samples (`BLOCK_SIZE = 64`), float 32-bit (`f32`).

---

## 2. Spécifications Détaillées des Composants Livrés

### 2.1 Workspace Cargo Multi-Crates
L'architecture sépare strictement les responsabilités :
- **`crates/grid-core`** :
  - Définition du graphe de patch : `Patch`, `NodeId`, `PortId`, `Connection`, `ModuleKind`.
  - Compilateur DAG : Tri topologique de Kahn, détection de cycles, détection de ports orphelins.
  - Plan d'exécution linéaire : `CompiledSchedule` ordonnançant l'évaluation des blocs.
- **`crates/grid-engine`** :
  - Moteur d'exécution DSP temps réel : `RealtimeAudioProcessor`.
  - Modules DSP élémentaires :
    - `Oscillator` : Formes d'ondes Sine, Triangle, Saw, Square avec algorithme anti-aliasing PolyBLEP.
    - `StateVariableFilter` : Filtre SVF TPT (Topology-Preserving Transform) 12 dB/octave (Lowpass, Highpass, Bandpass, Notch).
    - `AdsrEnvelope` : Enveloppe ADSR analog-like avec phases Attack, Decay, Sustain, Release.
    - `VcaGain` : Amplificateur commandé en tension / gain linéaire et exponentiel.
    - `Constant` : Source de tension continue / offset calibré.
    - `AudioOut` : Module terminal d'écoute stéréo.
  - Pilote CPAL stéréo : Gestionnaire de flux audio faible latence.
  - Communication lock-free : SPSC ring buffers (`rtrb`) pour les commandes UI -> Audio et la télémétrie Audio -> UI.
- **`crates/grid-app`** :
  - Application hôte Tauri v2.
  - Commandes IPC exposées au frontend (`start_audio`, `stop_audio`, `set_master_gain`, `set_parameter`, `apply_patch`, `get_current_patch`, `export_patch`, `import_patch`).
  - Thread de télémétrie décimée à 60 Hz envoyant l'état des vu-mètres stéréo (Peak & RMS en dBFS).
- **`frontend/`** :
  - Patcher modulaire Canvas 2D avec grille matricielle.
  - Gestion interactive des câbles (courbes de Bézier cubiques avec code couleur par type de signal).
  - Contrôle interactif des potentiomètres rotatifs et curseurs.
  - Télémétrie visuelle en temps réel (vu-mètre stéréo gradué).
  - Sauvegarde et chargement de patchs `.grid.json`.

---

## 3. Livrables Validés

1. **Workspace & Toolchain** :
   - Structure de crates Rust compilant sans avertissement sous `cargo check` et `cargo clippy`.
   - Frontend web bâti avec Vite et Bun.
2. **Suite de Tests Unitaires & Intégration** :
   - 20 tests automatisés validant la génération d'ondes PolyBLEP, la stabilité numérique du filtre SVF, l'anti-aliasing, la détection des cycles dans le compilateur DAG, et l'intégrité de la persistance JSON.
   - Couverture de code > 90 % sur les modules DSP et le compilateur topologique.
3. **Application Déployable & Fonctionnelle** :
   - Exécutable Tauri capable d'émettre un son modulaire pur en temps réel sans glitch, avec réactivité instantanée aux modifications de paramètres et de câblage.
