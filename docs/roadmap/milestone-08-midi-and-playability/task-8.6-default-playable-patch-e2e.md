# 📋 Tâche 8.6 : Patch Monophonique Jouable par Défaut & Validation E2E

> **Jalon :** 8 (Jouabilité Immédiate & MIDI)  
> **Composants cibles :** `crates/grid-core`, `crates/grid-engine`, `crates/grid-app`, `frontend`  
> **Priorité :** P3 (Tâche de clôture du Jalon 8)

---

## 1. Description du Travail

Rassembler l'ensemble des modules créés dans le Jalon 8 pour fournir une expérience prête à jouer dès l'ouverture de l'application :
1. **Création d'un patch d'usine par défaut (*Default Monosynth*)** :
   - `MidiIn.pitch` -> `VCO.pitch_cv`
   - `VCO.audio_out` -> `VCF.audio_in`
   - `VCF.audio_out` -> `VCA.audio_in`
   - `MidiIn.gate` -> `ADSR.gate`
   - `ADSR.env_out` -> `VCA.gain_cv`
   - `VCA.audio_out` -> `AudioOut.in_left` & `AudioOut.in_right`
   - `MidiIn.mod_wheel` -> `VCF.cutoff_cv`
2. **Chargement automatique au premier lancement** si aucun patch n'est présent.
3. **Validation de bout en bout (E2E)** :
   - Tests de jeu en direct au clavier virtuel et matériel.
   - Vérification de l'absence de coupure de son (underrun), de clic audio à l'attaque/relâchement, et de latence résiduelle.

---

## 2. Livrable

- Définition du patch par défaut dans `crates/grid-core/src/presets.rs`.
- Initialisation automatique de ce patch dans `grid-app` lors de l'appel `get_current_patch` / `start_audio`.
- Test d'intégration de bout en bout simulant le rendu audio complet d'une mélodie MIDI avec ADSR, VCO et VCF.
- Démonstration fonctionnelle validant la fermeture du Jalon 8 (v0.2.0).

---

## 3. Définition de Terminé (DoD)

- [ ] L'ouverture de l'application affiche immédiatement un synthétiseur modulaire câblé et prêt à l'emploi.
- [ ] L'activation du moteur audio (`Start Engine`) et l'appui sur une touche du clavier virtuel produit immédiatement un son de synthé chaud et articulé par l'enveloppe.
- [ ] La molette ModWheel ou son slider module en direct la brillance du filtre.
- [ ] Le pitch bend fait varier la note continûment d'un ton sans glitch.
- [ ] Le vu-mètre stéréo s'anime en temps réel à chaque note jouée et retombe à $-100\,\text{dBFS}$ après la phase de release.
- [ ] La couverture globale de tests du workspace dépasse 90%.

---

## 4. Choses Testables & Critères d'Acceptation

### Test d'Intégration Moteur (End-to-End)
- **Scénario de test automatisé :**
  1. Instancier le patch usine dans `RealtimeAudioProcessor`.
  2. Vérifier que sans événement MIDI, le buffer de sortie reste silencieux (énergie RMS $\le -90\,\text{dBFS}$).
  3. Injecter `MidiEvent::NoteOn { note: 69, velocity: 100 }` (Note A4, 440 Hz).
  4. Calculer 10 blocs de 64 samples :
     - Vérifier que le niveau sonore monte progressivement (phase Attack de l'ADSR).
     - Vérifier que la fréquence fondamentale du signal de sortie est centrée sur $440\,\text{Hz} \pm 2\%$.
  5. Injecter `MidiEvent::NoteOff { note: 69, velocity: 0 }`.
  6. Calculer 20 blocs de 64 samples :
     - Vérifier la décroissance exponentielle du niveau sonore (phase Release).
     - Vérifier le retour au silence complet.
- **Scénario Legato :**
  - Enchaîner Note 60 puis Note 62 sans NoteOff intermédiaire : l'onde passe continûment de 261 Hz à 293 Hz sans discontinuité de phase ni ré-attaque à zéro.
