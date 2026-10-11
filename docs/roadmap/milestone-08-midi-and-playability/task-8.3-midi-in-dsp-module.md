# 📋 Tâche 8.3 : Module DSP `MidiIn` dans le Graphe de Patch

> **Jalon :** 8 (Jouabilité Immédiate & MIDI)  
> **Crates cibles :** `crates/grid-core`, `crates/grid-engine`  
> **Priorité :** P1 (Dépend des tâches 8.1 et 8.2)

---

## 1. Description du Travail

Intégrer le module `MidiIn` comme citoyen de premier ordre dans le graphe modulaire de **The Grid** :
1. Déclarer la variante de module `ModuleKind::MidiIn` dans `grid-core`.
2. Définir les ports de sortie du module :
   - `pitch` : Sortie CV de hauteur de note ($1\,\text{V/oct}$, connectable à l'entrée pitch d'un VCO ou cutoff d'un VCF).
   - `gate` : Signal logique binaire ($0.0$ / $1.0$, connectable au trigger/gate d'un ADSR).
   - `velocity` : Signal CV de dynamique de frappe ($0.0 \dots 1.0$).
   - `mod_wheel` : Signal CV de la molette de modulation ($0.0 \dots 1.0$).
   - `pitch_bend` : Signal bipolaire de pitch bend ($-1.0 \dots +1.0$).
3. Mettre à jour le compilateur DAG (`crates/grid-core/src/compiler.rs`) pour ordonnancer `MidiIn` comme nœud source sans entrées requises.
4. Dans `grid-engine`, coupler le module DSP avec l'instance de `MonophonicVoiceAllocator` pour remplir les buffers de 64 samples lors de `process_block`.
5. Mettre à jour l'oscillateur (`Oscillator`) pour accepter une modulation de fréquence externe en $1\,\text{V/oct}$ ($f = f_0 \cdot 2^{V}$).

---

## 2. Livrable

- Mise à jour de `ModuleKind` et des descripteurs de ports dans `crates/grid-core/src/patch.rs`.
- Mise à jour du compilateur de graphe dans `crates/grid-core/src/compiler.rs`.
- Implémentation DSP du module `MidiIn` dans `crates/grid-engine/src/dsp/midi_in.rs`.
- Adaptation du module `Oscillator` pour prendre en compte le port `pitch_cv` ($f = 440 \cdot 2^{V}$).
- Tests d'intégration démontrant qu'un graphe `MidiIn` -> `Oscillator` -> `AudioOut` compile et génère les fréquences attendues.

---

## 3. Définition de Terminé (DoD)

- [ ] Le module `MidiIn` possède ses 5 ports de sortie déclarés et typés dans le modèle de patch.
- [ ] Le compilateur DAG accepte le module `MidiIn` sans erreur de validation et le place en tête du plan d'exécution.
- [ ] Le bloc DSP de `MidiIn` écrit les 64 valeurs de chaque sortie avec exactitude selon l'état courant de l'allocateur de voix.
- [ ] L'oscillateur réagit à la formule $1\,\text{V/oct}$ : $V = 0.0 \implies 261.63\,\text{Hz}$ (C4) ou $440\,\text{Hz}$ (A4), chaque variation de $+1.0\,\text{V}$ doublant la fréquence.
- [ ] Sérialisation et désérialisation JSON (`.grid.json`) opérationnelle avec le nouveau module `MidiIn`.
- [ ] Couverture de tests unitaires et d'intégration $\ge 90\%$.

---

## 4. Choses Testables & Critères d'Acceptation

### Tests Unitaires & Intégration
- **Validation topologique :**
  - Un patch contenant `MidiIn` relié à `Oscillator` et `AudioOut` produit un `CompiledSchedule` valide de taille 3.
- **Vérification de la formule 1 V/oct :**
  - Si `MidiIn` émet $0.0\,\text{V}$, `Oscillator` oscille à $f_0$.
  - Si `MidiIn` émet $+1.0\,\text{V}$, `Oscillator` oscille à $2 \cdot f_0$ (octave supérieure).
  - Si `MidiIn` émet $-1.0\,\text{V}$, `Oscillator` oscille à $\frac{f_0}{2}$ (octave inférieure).
- **Vérification de la Gate :**
  - Relier `MidiIn.gate` à `AdsrEnvelope.gate` : l'enveloppe doit entrer en phase Attack quand la Gate passe à 1.0, puis Release quand la Gate retombe à 0.0.
- **Sérialisation / Désérialisation :**
  - Exporter un patch avec `MidiIn` en JSON, le réimporter et vérifier l'égalité structurelle du graphe.
