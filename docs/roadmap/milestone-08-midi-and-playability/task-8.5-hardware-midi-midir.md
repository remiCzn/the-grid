# 📋 Tâche 8.5 : Intégration Périphériques Physiques USB-MIDI (`midir`)

> **Jalon :** 8 (Jouabilité Immédiate & MIDI)  
> **Crates cibles :** `crates/grid-app`  
> **Priorité :** P2 (Dépend des tâches 8.1 et 8.2)

---

## 1. Description du Travail

Intégrer la bibliothèque cross-platform `midir` dans l'application hôte Tauri pour permettre la détection, la sélection et l'écoute à chaud de claviers maîtres et contrôleurs USB-MIDI physiques.

### Fonctionnalités Clés
1. **Gestionnaire de connexion MIDI (`MidiDeviceManager`) :**
   - Énumération des ports MIDI d'entrée disponibles sur le système (ALSA sous Linux, CoreMIDI sous macOS, WinMM/UWP sous Windows).
   - Connexion automatique au premier périphérique MIDI détecté ou reconnexion automatique en cas de déconnexion/reconnexion (hot-plug).
2. **Callback d'écoute faible latence :**
   - Le callback natif de `midir` reçoit les paquets d'octets bruts sur un thread dédié.
   - Les octets sont convertis via `parse_midi_message` (défini dans la tâche 8.1).
   - L'événement résultant est immédiatement injecté dans le `MidiProducer` SPSC vers le thread audio.
3. **Contrôle depuis l'Interface Utilisateur :**
   - Commande Tauri `list_midi_inputs() -> Vec<MidiPortInfo>` listant les contrôleurs connectés.
   - Commande Tauri `select_midi_input(port_index: usize)` pour basculer d'un contrôleur à un autre.
   - Menu déroulant discret dans la barre supérieure de l'UI pour choisir le périphérique MIDI.

---

## 2. Livrable

- Ajout de la dépendance `midir = "0.10"` dans `crates/grid-app/Cargo.toml`.
- Structure `MidiDeviceManager` dans `crates/grid-app/src/midi_manager.rs`.
- Commandes Tauri `list_midi_inputs` et `select_midi_input` exposées à l'UI.
- Sélecteur de périphérique MIDI dans la barre d'outils du frontend.
- Gestion robuste des erreurs (absence de périphérique, déconnexion inopinée).

---

## 3. Définition de Terminé (DoD)

- [ ] L'application démarre normalement même si aucun périphérique MIDI physique n'est branché.
- [ ] Le branchement d'un contrôleur USB-MIDI est détecté et visible dans la liste des périphériques.
- [ ] Les messages NoteOn, NoteOff, ModWheel, PitchBend reçus du périphérique physique sont routés dans le thread audio sans craquement sonore.
- [ ] La déconnexion brutale d'un câble USB n'entraîne aucun crash ni freeze de l'application hôte ou du thread audio.
- [ ] Zéro warning `clippy` et documentation complète sur le cycle de vie de la connexion MIDI.

---

## 4. Choses Testables & Critères d'Acceptation

### Tests Unitaires & Intégration
- **Test d'émulation de port virtuel :**
  - Sur Linux avec ALSA virtual rawmidi ou port virtuel de test : émission de messages MIDI simulés et réception vérifiée dans le ring buffer audio.
- **Test de gestion d'erreur :**
  - Sélection d'un index de port invalide retournant une erreur structurée sans paniquer.
- **Test de reconnexion :**
  - Fermeture et réouverture d'un flux d'entrée MIDI sans fuite de thread ni double écoute.
