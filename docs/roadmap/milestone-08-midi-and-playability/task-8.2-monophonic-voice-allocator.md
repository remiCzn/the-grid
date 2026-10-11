# 📋 Tâche 8.2 : Gestionnaire de Voix Monophonique DSP

> **Jalon :** 8 (Jouabilité Immédiate & MIDI)  
> **Crates cibles :** `crates/grid-core`, `crates/grid-engine`  
> **Priorité :** P1 (Dépend de la tâche 8.1)

---

## 1. Description du Travail

Implémenter un gestionnaire de voix monophonique sans allocation dynamique (`MonophonicVoiceAllocator`), responsable de transformer un flux de `MidiEvent` en signaux de contrôle continus pour le synthétiseur.

### Fonctionnalités Clés
1. **Pile de notes active (Note Stack)** :
   - Capacité fixe (ex. 16 notes simultanées en mémoire statique) pour retenir les touches actuellement maintenues enfoncées.
   - Gestion stricte des ajouts et retraits de touches sans allocation.
2. **Priorités de notes sélectionnables** :
   - `Priority::LastNote` (par défaut) : la dernière note enfoncée prévaut ; si elle est relâchée, retour à la note précédente encore enfoncée.
   - `Priority::LowNote` : la note la plus grave parmi les touches actives est jouée (style Minimoog).
   - `Priority::HighNote` : la note la plus aiguë parmi les touches actives est jouée.
3. **Modes de jeu (Legato vs Retrigger)** :
   - `PlayMode::Legato` : si une note est déjà active et qu'une autre est pressée, la Gate reste à `1.0` (pas de réamorçage de l'enveloppe) et seul le Pitch change.
   - `PlayMode::Retrigger` : chaque nouvelle note pressée force une transition de Gate à `0.0` pendant 1 bloc ou une impulsion de re-trigger pour réamorcer l'attaque de l'enveloppe ADSR.
4. **Calculs & Normalisation CV** :
   - Formule $1\,\text{V/oct}$ : $\text{pitch\_cv} = \frac{\text{note} - 69}{12.0}$ ou $\frac{\text{note} - 60}{12.0}$ (C4 = 0.0V).
   - Pitch Bend : ajout d'un offset continu en volts (par défaut $\pm 2$ demi-tons = $\pm \frac{2}{12}$ V).
   - Gate : `1.0` si une note est active, `0.0` si aucune note n'est pressée.
   - Velocity : normalisée de $0.0$ à $1.0$ ($\frac{\text{velocity}}{127.0}$).
   - ModWheel : normalisée de $0.0$ à $1.0$ ($\frac{\text{cc1}}{127.0}$).

---

## 2. Livrable

- Structure `MonophonicVoiceAllocator` dans `crates/grid-engine/src/voice.rs` (ou `grid-core::voice`).
- Configuration de l'allocateur (`VoiceModeConfig` : priorité, retrigger, pitch bend range).
- État de sortie instantané (`VoiceState { pitch_cv: f32, gate: f32, velocity: f32, mod_wheel: f32, pitch_bend: f32, is_gate_triggered: bool }`).
- Suite de tests unitaires couvrant les 3 modes de priorité, le legato et le relâchement en cascade.

---

## 3. Définition de Terminé (DoD)

- [ ] L'allocateur de voix est entièrement `no_std`-compatible ou n'alloue aucune mémoire dynamique au cours de son cycle de vie.
- [ ] Les 3 modes de priorité (`LastNote`, `LowNote`, `HighNote`) sont implémentés et validés par des tests unitaires dédiés.
- [ ] Le mode Legato maintient le signal Gate haut lors des transitions, tandis que le mode Retrigger signale un front descendant/montant.
- [ ] Le relâchement successif des touches ramène correctement la hauteur de note à la note sous-jacente restante.
- [ ] L'envoi de `AllNotesOff` vide immédiatement la pile et remet la Gate à 0.0.
- [ ] Couverture de tests unitaires $\ge 95\%$ avec zéro warning clippy.

---

## 4. Choses Testables & Critères d'Acceptation

### Tests Unitaires
- **Test Last Note Priority :**
  - Appuyer Note 60 (Gate = 1.0, Pitch = C4).
  - Appuyer Note 64 (Gate = 1.0, Pitch = E4).
  - Relâcher Note 64 -> le Pitch doit instantanément revenir à C4 (Note 60), Gate toujours à 1.0.
  - Relâcher Note 60 -> Gate passe à 0.0.
- **Test Low Note Priority :**
  - Appuyer Note 60, puis Note 67 -> Le Pitch reste sur 60 (la note la plus basse).
  - Appuyer Note 48 -> Le Pitch bascule sur 48.
  - Relâcher Note 48 -> Le Pitch remonte à 60.
- **Test High Note Priority :**
  - Appuyer Note 60, puis Note 48 -> Le Pitch reste à 60.
  - Appuyer Note 72 -> Le Pitch passe à 72.
- **Test Débordement de Pile :**
  - Appuyer 20 notes successives sur une pile de capacité 16 -> La plus ancienne note est recyclée sans panique mémoire.
- **Test Retrigger vs Legato :**
  - En mode Retrigger, l'appui sur une seconde note génère un trigger de réarmement.
  - En mode Legato, `is_gate_triggered` ne se réarme pas tant que la Gate ne repasse pas par 0.
- **Test Pitch Bend & ModWheel :**
  - Vérifier la variation linéaire du pitch bend selon la plage configurée.
