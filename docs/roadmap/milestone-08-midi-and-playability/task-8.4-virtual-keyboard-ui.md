# 📋 Tâche 8.4 : Clavier Virtuel à l'Écran (Frontend & IPC)

> **Jalon :** 8 (Jouabilité Immédiate & MIDI)  
> **Composants cibles :** `frontend/src/`, `crates/grid-app/src/`  
> **Priorité :** P2 (Dépend des tâches 8.1 et 8.3)

---

## 1. Description du Travail

Développer un clavier virtuel à l'écran, fluide et rétractable, permettant à l'utilisateur de jouer immédiatement du synthétiseur à la souris, au pavé tactile, ou au clavier d'ordinateur (AZERTY et QWERTY) sans nécessiter de matériel externe.

### Fonctionnalités Clés
1. **Composant UI Clavier Virtuel :**
   - Dockable en bas de la fenêtre avec un bouton rétractable (toggle fold/unfold) pour ne pas encombrer l'espace du canvas.
   - 2 à 3 octaves affichées avec touches blanches et noires stylisées dans le thème sombre / néon de *The Grid*.
   - Sélecteur d'octave transposée (`Octave -` / `Octave +`, plage d'octaves 1 à 7).
2. **Contrôle Souris & Tactile :**
   - Détection de l'enfoncement (`mousedown`/`touchstart`) -> émission `NoteOn`.
   - Détection du relâchement (`mouseup`/`mouseleave`/`touchend`) -> émission `NoteOff`.
   - Glissement d'une touche à l'autre tout en maintenant le clic (glissando monophonique).
   - Calcul de la vélocité selon la hauteur verticale du clic sur la touche (haut de la touche = vélocité douce $\approx 40$, bas de la touche = vélocité forte $\approx 127$).
3. **Contrôle au Clavier d'Ordinateur (AZERTY & QWERTY) :**
   - Rangée basse (blanches : `W, X, C, V, B, N...` ou `W, X, C, V...` ; noires : `S, D, G, H, J...`).
   - Rangée haute (octave supérieure : `A, Z, E, R, T, Y...`).
   - Raccourcis pour transposer l'octave (`1` / `2` ou `Z`/`X` selon layout).
   - Filtre anti-répétition de frappe native du système (`event.repeat === false`).
4. **Passerelle IPC Tauri :**
   - Commande Tauri `send_midi_event(event: MidiEventPayload)` dans `grid-app` recevant l'événement et l'injectant instantanément dans le `MidiProducer` du thread audio.

---

## 2. Livrable

- Composant `VirtualKeyboard` dans le frontend (`frontend/src/components/VirtualKeyboard.ts` ou `.vue` / Web Component selon stack).
- Style CSS néon sombre responsive et bouton de toggle.
- Module de gestion des touches du clavier d'ordinateur avec détection de répétition (`frontend/src/keyboard_input.ts`).
- Commande Tauri `send_midi_event` dans `crates/grid-app/src/main.rs` ou `crates/grid-app/src/commands.rs`.
- Rendu visuel dans le Canvas du module `MidiIn` avec ses 5 fiches de sortie.

---

## 3. Définition de Terminé (DoD)

- [ ] Le clavier virtuel s'affiche en bas de l'écran avec une transition d'ouverture/fermeture fluide.
- [ ] Le clic de la souris ou l'appui d'une touche AZERTY/QWERTY illumine la touche en temps réel.
- [ ] Un clic déclenche un appel IPC envoyant `NoteOn`, et le relâchement envoie `NoteOff`.
- [ ] Le glissé de souris de touche en touche enchaîne correctement les `NoteOff` et `NoteOn` sans laisser de note suspendue (*stuck note*).
- [ ] L'UI du module `MidiIn` est dessinée sur le Canvas avec ses 5 ports : `pitch`, `gate`, `velocity`, `mod_wheel`, `pitch_bend`.
- [ ] Aucun lag perceptible sur l'interface lors d'un jeu rapide.

---

## 4. Choses Testables & Critères d'Acceptation

### Tests Visuels & Interactifs
- **Écoute des événements :**
  - Appui sur la touche 'A' du clavier : log ou événement IPC vérifié avec `note: 60`, `velocity: 100`.
  - Relâchement de la touche 'A' : événement `note: 60`, `velocity: 0`.
- **Test anti-répétition :**
  - Maintenir enfoncée la touche 'A' pendant 3 secondes : vérifier qu'un seul `NoteOn` a été émis (pas d'avalanche de répétitions système).
- **Test Glissando Souris :**
  - Clic maintenu sur C4, glissement vers D4 : émission immédiate de `NoteOff(60)` puis `NoteOn(62)`.
- **Test Stuck Note Recovery :**
  - Sortie du curseur hors de la zone du clavier lors d'un clic maintenu : la note active est proprement relâchée (`NoteOff`).
- **Test Bouton Transposition :**
  - Clic sur `Octave +` : la même touche physique joue désormais la note 72 (+12 demi-tons).
