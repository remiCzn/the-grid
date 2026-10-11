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

## 4. Feuille de Route : Les Prochaines Étapes

Pour faire évoluer ce moteur vers un instrument de production complet, quatre axes prioritaires sont planifiés :

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        AXE 1 : MIDI & JOUABILITÉ                       │
│  - Entrée MIDI USB temps réel (crate midir)                            │
│  - Allocation de voix polyphoniques (4 à 8 voix, modes Mono / Legato)   │
│  - Clavier de piano virtuel interactif à l'écran                       │
└────────────────────────────────────────────────────────────────────────┘
                                    │
┌────────────────────────────────────────────────────────────────────────┐
│                   AXE 2 : CATALOGUE MODULAIRE ÉTENDU                   │
│  - LFO multi-formes synchronisable au tempo                            │
│  - Séquenceur pas à pas (Step Sequencer 8/16 pas)                      │
│  - Effets DSP intégrés : Stereo Delay / Echo et Saturation analogique  │
│  - Générateur de bruit (Noise) & Sample & Hold                         │
└────────────────────────────────────────────────────────────────────────┘
                                    │
┌────────────────────────────────────────────────────────────────────────┐
│                 AXE 3 : EXPÉRIENCE UTILISATEUR & CANVAS                │
│  - Menu contextuel d'ajout de modules au clic-droit (Browser modulaire)│
│  - Pan & Zoom fluide sur la grille                                     │
│  - Mini-oscilloscope temps réel sur les câbles (visualisation du son)  │
└────────────────────────────────────────────────────────────────────────┘
                                    │
┌────────────────────────────────────────────────────────────────────────┐
│                    AXE 4 : FORMATS & DISTRIBUTIONS                     │
│  - Sélecteur de carte son & taille de buffer (64 à 512 samples)        │
│  - Packages natifs (.deb, AppImage, .dmg, .msi)                        │
│  - Export sous forme de plugin audio hôte (CLAP / VST3 via nih-plug)   │
└────────────────────────────────────────────────────────────────────────┘
```

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
