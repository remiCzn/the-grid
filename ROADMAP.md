# 🗺️ Roadmap : The Grid

Ce document détaille la trajectoire de développement de **The Grid**. Les priorités sont articulées pour préserver l'intransigeance temps réel du moteur, améliorer l'ergonomie visuelle du patcher et étendre progressivement le potentiel sonore du synthétiseur.

---

## 📌 État Actuel : Jalons 1 à 7 (v0.1.0 - Alpha Fondatrice) ✅

- [x] **Workspace Cargo multi-crates** : Séparation stricte `grid-core`, `grid-engine`, `grid-app`.
- [x] **Thread audio temps réel** : Pilote CPAL, SPSC Ring Buffers (`rtrb`), zéro allocation, zéro mutex.
- [x] **Catalogue DSP initial** : VCO (PolyBLEP anti-aliasing), VCF (SVF TPT 12dB), VCA, ADSR, Constant, AudioOut.
- [x] **Compilateur DAG** : Tri topologique de Kahn avec génération de plan d'exécution linéaire.
- [x] **Patcher Canvas 2D** : Câblage Bézier interactif, drag-and-drop des modules, réglage des potentiomètres.
- [x] **Télémétrie 60 Hz** : Vu-mètre stéréo calibré en dBFS sans surcharge IPC.
- [x] **Persistance** : Sérialisation et désérialisation de patches en format JSON (`.grid.json`).
- [x] **Qualité** : Couverture de tests > 90 % sur le core DSP et la validation topologique.

---

## 🚀 Jalon 8 : Jouabilité Immédiate & MIDI (v0.2.0)
*Objectif : Rendre le patcher jouable en direct au clavier matériel et virtuel.*

- [ ] **Intégration MIDI matérielle (`midir`)** :
  - Détection automatique et connexion à chaud des périphériques USB-MIDI.
  - Ring buffer lock-free dédié pour injecter les événements MIDI dans le thread audio.
- [ ] **Gestionnaire de voix monophonique** :
  - Modes de priorité de note configurables : *Last Note*, *Low Note*, *High Note*.
  - Prise en charge du *Legato* (enchaînement de notes sans réamorcer l'enveloppe) et du *Retrigger*.
- [ ] **Module `MidiIn` étendu** :
  - Sorties CV normalisées : Pitch ($1\,\text{V/oct}$), Gate, Velocity, ModWheel (CC1), Pitch Bend.
- [ ] **Clavier virtuel à l'écran** :
  - Barre de clavier rétractable en bas de l'interface pour jouer à la souris ou au clavier AZERTY/QWERTY.

---

## 🔄 Jalon 9 : Résilience du Graphe & Boucles de Feedback (v0.3.0)
*Objectif : Autoriser les interconnexions cycliques indispensables au son modulaire.*

- [ ] **Détection des arcs de rétroaction (*Back-Edges*)** :
  - Parcours DFS à 3 couleurs pour isoler les connexions circulaires avant le tri de Kahn.
- [ ] **Insertion automatique de délai unitaire ($z^{-1}$)** :
  - Routage des cycles vers un jeu de buffers d'historique (retard d'un bloc de 64 samples).
  - Levée des blocages de compilation lors des auto-modulations (cross-modulation FM, feedback de filtre).
- [ ] **Atténuation par câble (*Cable Amount*)** :
  - Dosage de l'amplitude $[-1.0, 1.0]$ directement sur chaque câble sans nécessiter de module atténuateur dédié.

---

## 🖥️ Jalon 10 : Ergonomie Spatiale & Workflow Canvas (v0.4.0)
*Objectif : Fluidifier la manipulation sur des patchs denses et complexes.*

- [ ] **Navigation infinie (Pan & Zoom)** :
  - Translation par clic molette / espace + glisser.
  - Zoom fluide centré sur le curseur avec grille matricielle adaptative (LOD).
- [ ] **Navigateur de modules contextuel (*Module Browser*)** :
  - Menu contextuel au clic-droit avec filtre de recherche instantané au clavier.
  - Catégorisation des modules (Générateurs, Filtres, Enveloppes, Maths, Routage).
- [ ] **Productivité d'édition** :
  - Boîte de sélection multiple, déplacement de groupes, duplication (`Ctrl+D`), suppression (`Suppr`).
- [ ] **Visualisation de signal sur les câbles** :
  - Rendu visuel de l'amplitude ou des impulsions par pulsation lumineuse sur les courbes de Bézier.

---

## 🎛️ Jalon 11 : Expansion du Catalogue Modulaire (v0.5.0)
*Objectif : Débloquer la création de textures sonores avancées typiques de Bitwig.*

- [ ] **Modulateurs & Contrôle** :
  - **`LFO`** : Multi-formes d'ondes, synchronisation tempo (divisions/multiplications) ou mode continu en Hz.
  - **`Noise`** : Générateur de bruit blanc et bruit rose.
  - **`Sample & Hold`** : Échantillonneur-bloqueur avec entrée horloge externe.
- [ ] **Effets DSP** :
  - **`Stereo Delay`** : Ligne à retard synchronisée avec filtrage coupe-haut/coupe-bas dans la boucle.
  - **`Drive / Saturation`** : Algorithmes de waveshaping non-linéaires (soft clipping, saturation asymétrique).
- [ ] **Maths & Logique** :
  - **`Math`** : Modules d'addition, soustraction, multiplication et inversion bipolaire.
  - **`Min/Max` & `Comparator`** : Traitement de signaux de contrôle complexes.

---

## ⏱️ Jalon 12 : Transport & Séquençage (v0.6.0)
*Objectif : Permettre la création de motifs autonomes et génératifs sans DAW externe.*

- [ ] **Moteur de Transport Interne** :
  - Gestionnaire de tempo BPM maître, lecture/arrêt, signatures rythmiques.
- [ ] **Step Sequencer (8 / 16 pas)** :
  - Éditeur visuel de pas avec curseurs de pitch, probabilité et durée de gate par pas.
- [ ] **Diviseur d'horloge (*Clock Divider*)** :
  - Génération de sous-rythmes ($/2, /4, /8, /16$) pour piloter des enveloppes secondaires.

---

## 🎹 Jalon 13 : Moteur Polyphonique (v0.7.0)
*Objectif : Passer du synthétiseur monophonique à un synthé polyphonique complet.*

- [ ] **Pool de voix DSP vectorisé** :
  - Allocation dynamique de 4 à 8 voix avec algorithme de vol de voix (*voice stealing*).
- [ ] **Isolation des sections mono vs poly** :
  - Détection automatique des nœuds polyphoniques (VCO, VCF, ADSR) et sommation vers les effets master stéréo.

---

## 📦 Jalon 14 : Distribution & Intégrations Externes (v1.0.0)
*Objectif : Mettre l'outil entre les mains des producteurs sur tous les OS.*

- [ ] **Panneau de configuration audio** :
  - Choix de la carte son, sélection du sample rate (44.1, 48, 96 kHz) et réglage de la taille du buffer (64 à 512).
- [ ] **Installeurs natifs multiplateformes** :
  - Pipelines CI/CD pour paquets `.deb`, `AppImage`, `.dmg` (notarisé macOS) et `.msi` (Windows).
- [ ] **Hébergement sous forme de Plugin (CLAP / VST3)** :
  - Wrapper via `nih-plug` ou `clack` permettant de charger *The Grid* directement dans Bitwig, Reaper ou Ableton Live.
