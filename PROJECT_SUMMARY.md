# 🎛️ The Grid : Synthétiseur Modulaire Visuel Open Source
> **Architecture, État d'avancement et Perspectives d'évolution**  
> *Dépôt GitHub :* [github.com/remiCzn/the-grid](https://github.com/remiCzn/the-grid)

---

## 1. Vision du Projet

**The Grid** est un clone open source inspiré de l'environnement modulaire de Bitwig Studio.  
Le projet combine la fluidité d'un **patcher visuel sur Canvas 2D/WebGL** avec l'intransigeance d'un **moteur audio temps réel bas niveau en Rust**, conçu selon les standards les plus stricts de l'industrie audio professionnelle.

### Objectifs Clés
- **Zéro latence perçue & Stabilité absolue :** Aucun blocage de thread, aucun décrochage audio (*audio glitch/underrun*).
- **Interface réactive et légère :** Frontend moderne en TypeScript/Canvas propulsé par **Tauri v2** (pas d'encombrement Chromium/Electron lourd).
- **Architecture modulaire extensible :** Découplage strict entre interface déclarative, validation topologique du graphe et exécution DSP.

---

## 2. Architecture et Sécurité Temps Réel

Le projet repose sur un découpage en **Cargo Workspace multi-crates** garantissant l'isolation des responsabilités :

```text
┌────────────────────────────────────────────────────────┐
│                   Frontend (Tauri v2)                  │
│       Vite + Bun + Canvas 2D Réactif (60/120 FPS)      │
│  Grille de points, Modules, Câbles Bézier, Vu-mètres   │
└──────────────────────────┬─────────────────────────────┘
                           │ IPC Tauri (invoke / events)
┌──────────────────────────▼─────────────────────────────┐
│                 Host Backend (crates/grid-app)         │
│         - Persistance JSON & Validation sémantique     │
│         - Relais asynchrone de télémétrie (60 Hz)      │
└──────────────────────────┬─────────────────────────────┘
                           │ Ring Buffers SPSC Lock-Free (rtrb)
┌──────────────────────────▼─────────────────────────────┐
│             Logique Core (crates/grid-core)            │
│         - Modèle déclaratif (Patch, Modules, Ports)    │
│         - Validation DAG & Tri topologique (Kahn)      │
│         - Compilateur en plan linéaire d'instructions  │
└──────────────────────────┬─────────────────────────────┘
                           │ Scratch Buffers Préalloués
┌──────────────────────────▼─────────────────────────────┐
│          Moteur Audio Temps Réel (crates/grid-engine)  │
│   - Thread audio isolé haute priorité (pilote CPAL)    │
│   - Zéro allocation dynamique (no malloc)              │
│   - Zéro verrouillage bloquant (no mutex)              │
│   - Zéro appel système ou I/O direct                   │
└────────────────────────────────────────────────────────┘
```

### Les Règles d'Or du Thread Audio
1. **Zéro allocation dynamique (`no malloc`) :** Tous les buffers temporaires (*scratch buffers*) et processeurs DSP sont préalloués au démarrage.
2. **Zéro verrouillage bloquant (`no mutex`) :** Communication bidirectionnelle exclusive via des files circulaires mono-producteur / mono-consommateur lock-free (`rtrb`).
3. **Zéro appel système / I/O :** Aucun accès fichier, réseau ou affichage dans la boucle de rendu audio.
4. **Télémétrie décimée & Drop-on-Full :** Les métriques d'affichage (crêtes, RMS) sont transmises à 60 Hz vers l'UI ; en cas de saturation, la trame est ignorée pour ne jamais ralentir le moteur.

---

## 3. Ce qui est Fonctionnel Aujourd'hui (Jalons 1 à 7)

Le projet dispose d'une base technique complète et opérationnelle :

- ✅ **Moteur Audio CPAL Réel :** Rendu sonore stéréo entrelacé, gestion du gain maître et arrêt silencieux.
- ✅ **Catalogue de 6 Modules DSP Préalloués :**
  - **`Oscillator` (VCO) :** Formes d'ondes Sine, Triangle, Sawtooth, Square avec algorithme anti-aliasing **PolyBLEP** et modulation de fréquence $1\,\text{V/oct}$.
  - **`Gain` (VCA) :** Amplificateur contrôlé en tension avec lissage passe-bas anti-zipper noise.
  - **`Filter` (VCF) :** Filtre à état variable (SVF TPT) 2 pôles résonant (LowPass, HighPass, BandPass, Notch) avec stabilité inconditionnelle.
  - **`Envelope` (ADSR) :** Générateur d'enveloppe 4 étages déclenché par signaux Gate/Trigger.
  - **`Constant` :** Source de tension CV statique.
  - **`AudioOut` :** Sortie master stéréo connectée au driver de la carte son.
- ✅ **Compilateur de Graphe Topologique :** Algorithme de Kahn vérifiant l'acyclicité (rejet des cycles non retardés) et générant un plan d'exécution séquentiel optimisé.
- ✅ **Interface Graphique Canvas 2D Interactif :**
  - Grille sombre haute performance (60/120 FPS).
  - Déplacement des modules par glisser-déposer (*drag & drop*).
  - Câblage interactif au clic/glisser avec courbes de Bézier cubiques stylisées et halo lumineux (code couleur : Audio orange, CV bleu, Trigger jaune).
  - Déconnexion rapide au clic droit.
  - Contrôle continu des paramètres par glisser horizontal.
- ✅ **Télémétrie & Vu-Mètre Stéréo :** Barres de niveau L/R et affichage numérique calibré en dBFS alimenté à 60 Hz sans surcharge IPC.
- ✅ **Persistance & Échange :** Import et export complet de patchs en JSON (`.grid.json`).
- ✅ **Qualité & Couverture :** 20 tests unitaires et d'intégration validés sans avertissement, **> 90% de couverture** sur la logique métier et DSP.

---

## 4. Feuille de Route : Jalons 8 à 14 (v0.2.0 à v1.0.0)

Pour faire évoluer ce moteur vers un instrument de production complet, la trajectoire officielle est structurée ainsi :

- 🚀 **Jalon 8 : Jouabilité Immédiate & MIDI (v0.2.0)**  
  Entrée MIDI USB (`midir`), gestionnaire de voix monophonique (*Last/Low/High Note*, Legato), module `MidiIn` (Pitch $1\,\text{V/oct}$, Gate, Velocity, CC1) et clavier virtuel rétractable à l'écran.
- 🔄 **Jalon 9 : Résilience du Graphe & Boucles de Feedback (v0.3.0)**  
  Détection des *Back-Edges* par DFS 3 couleurs, insertion automatique de délai unitaire ($z^{-1}$) pour les modulations cycliques (FM cross-mod, feedback de filtre) et atténuation par câble (*Cable Amount*).
- 🖥️ **Jalon 10 : Ergonomie Spatiale & Workflow Canvas (v0.4.0)**  
  Navigation infinie Pan & Zoom, navigateur de modules contextuel (*Module Browser* au clic-droit avec recherche), sélection multiple et visualisation de signal sur les câbles.
- 🎛️ **Jalon 11 : Expansion du Catalogue Modulaire (v0.5.0)**  
  LFO multi-formes synchronisable, générateur de bruit (Noise), Sample & Hold, Stereo Delay synchronisé, Drive / Saturation non-linéaire, et modules Math/Logique.
- ⏱️ **Jalon 12 : Transport & Séquençage (v0.6.0)**  
  Moteur de transport interne BPM maître, Step Sequencer 8/16 pas, et diviseurs d'horloge (*Clock Divider*).
- 🎹 **Jalon 13 : Moteur Polyphonique (v0.7.0)**  
  Pool de voix DSP vectorisé (4 à 8 voix avec *voice stealing*) et sommation dynamique des voies polyphoniques vers le bus master.
- 📦 **Jalon 14 : Distribution & Intégrations Externes (v1.0.0)**  
  Panneau de réglage audio (carte son, buffer size 64-512), packaging natif (.deb, AppImage, .dmg, .msi) et portage en plugin audio (CLAP / VST3).

---

## 5. Démarrage Rapide

### Prérequis
- **Rust** (toolchain stable `cargo`)
- **Bun** (gestionnaire de paquets frontend ultra-rapide)
- Dépendances audio : `libasound2-dev` (Linux ALSA)

### Commandes de base
```bash
# Cloner le projet
git clone https://github.com/remiCzn/the-grid.git
cd the-grid

# Lancer la suite de tests et vérifier la couverture
cargo test --workspace

# Installer les dépendances du frontend et compiler
bun install --cwd frontend
bun run --cwd frontend build

# Lancer l'application en mode développement
# (depuis crates/grid-app via tauri-cli ou bun dev)
```

---

## 6. Références du Projet
- 📜 **[Directives Architecturales (`GUIDELINES.md`)](https://github.com/remiCzn/the-grid/blob/main/GUIDELINES.md)** : Registre de couverture et règles d'or temps réel.
- 📐 **[Spécifications Techniques (`SPECIFICATIONS.md`)](https://github.com/remiCzn/the-grid/blob/main/SPECIFICATIONS.md)** : Modèle de données JSON, protocoles IPC et formats de signaux.
- 🗺️ **[Feuille de Route Détaillée (`ROADMAP.md`)](https://github.com/remiCzn/the-grid/blob/main/ROADMAP.md)** : Historique des jalons et tâches futures.
- 🖼️ **[Prévisualisation du Rendu Visuel (`FRONTEND_PREVIEW.md`)](https://github.com/remiCzn/the-grid/blob/main/FRONTEND_PREVIEW.md)** : Capture d'écran native de l'interface graphique.
