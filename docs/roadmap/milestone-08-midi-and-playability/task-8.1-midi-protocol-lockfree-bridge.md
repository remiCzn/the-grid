# 📋 Tâche 8.1 : Protocole MIDI & Bridge SPSC Lock-Free

> **Jalon :** 8 (Jouabilité Immédiate & MIDI)  
> **Crates cibles :** `crates/grid-core`, `crates/grid-engine`  
> **Priorité :** P0 (Bloquant pour toutes les tâches du Jalon 8)

---

## 1. Description du Travail

Définir les types fondamentaux de messages MIDI dans `grid-core` et mettre en place un canal de transmission lock-free (SPSC Ring Buffer avec `rtrb`) pour acheminer les événements MIDI depuis l'application hôte (ou les périphériques) vers le moteur audio en temps réel sans allocation dynamique ni blocage mutex.

### Composants à Développer
1. **Types de données MIDI dans `grid-core` :**
   - Enum / struct compacte `MidiEvent` (`Copy`, `Clone`, `Debug`, `PartialEq`, `Serialize`, `Deserialize`) :
     - `NoteOn { channel: u8, note: u8, velocity: u8 }`
     - `NoteOff { channel: u8, note: u8, velocity: u8 }`
     - `ControlChange { channel: u8, controller: u8, value: u8 }`
     - `PitchBend { channel: u8, value: i16 }` (centré à 0, plage -8192..8191)
     - `AllNotesOff { channel: u8 }`
   - Décodeur d'octets MIDI bruts (`fn parse_midi_message(bytes: &[u8]) -> Option<MidiEvent>`).
2. **Ring Buffer MIDI dans `grid-engine` :**
   - Paire producteur/consommateur `rtrb::RingBuffer<MidiEvent>` dimensionnée à 1024 événements.
   - Intégration du consommateur dans `RealtimeAudioProcessor` pour vider les événements reçus à chaque cycle audio (`process_block`).

---

## 2. Livrable

- Fichier `crates/grid-core/src/midi.rs` (avec export dans `lib.rs`) contenant :
  - `MidiEvent`
  - Les constructeurs et helpers de conversion
  - La fonction de décodage d'octets MIDI standard (MIDI 1.0)
- Fichier `crates/grid-engine/src/midi_bridge.rs` (ou enrichissement de `processor.rs` / `driver.rs`) intégrant :
  - `MidiProducer` pour l'envoi d'événements
  - Réception lock-free dans le thread audio
- Tests unitaires complets couvrant l'ensemble des cas nominaux et cas d'erreur.

---

## 3. Définition de Terminé (DoD)

- [ ] Les types `MidiEvent` sont `Copy`, `Send`, et ont une empreinte mémoire minimale ($\le 8$ octets par événement).
- [ ] Le parser gère NoteOn (avec vélocité 0 traité comme NoteOff), NoteOff, CC (dont ModWheel CC 1), Pitch Bend 14-bit, et AllNotesOff (CC 120 / 123).
- [ ] Le ring buffer MIDI est initialisé au démarrage du moteur audio sans allocation dynamique dans la boucle de rendu.
- [ ] La documentation Rustdoc est présente sur toutes les structures et fonctions publiques.
- [ ] Zéro warning au compilateur (`cargo check` et `cargo clippy --all-targets -- -D warnings`).
- [ ] Couverture de tests unitaires $\ge 95\%$ sur le module `midi.rs`.

---

## 4. Choses Testables & Critères d'Acceptation

### Tests Unitaires
- **Parsing d'octets bruts :**
  - `[0x90, 60, 100]` -> `MidiEvent::NoteOn { channel: 0, note: 60, velocity: 100 }`.
  - `[0x90, 60, 0]` -> `MidiEvent::NoteOff { channel: 0, note: 60, velocity: 0 }` (cas standard NoteOn avec vélocité zéro).
  - `[0x80, 60, 64]` -> `MidiEvent::NoteOff { channel: 0, note: 60, velocity: 64 }`.
  - `[0xB0, 1, 127]` -> `MidiEvent::ControlChange { channel: 0, controller: 1, value: 127 }`.
  - `[0xE0, 0x00, 0x40]` -> `MidiEvent::PitchBend { channel: 0, value: 0 }` (centre neutre).
  - Tranches d'octets invalides ou incomplètes (`&[0x90]`, `&[0xFF]`) retournant `None` sans panic.
- **Ring Buffer Lock-Free :**
  - Injection de 500 événements dans le `MidiProducer` et consommation complète dans le processeur audio avec vérification de l'ordre FIFO strict.
  - Test de débordement de buffer (*Buffer Overflow*) : si le buffer est plein, le producteur rejette l'événement proprement sans bloquer ni paniquer.
