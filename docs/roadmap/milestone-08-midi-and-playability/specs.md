# 🎹 Jalon 8 : Jouabilité Immédiate & MIDI (v0.2.0)

> **Statut :** Planifié / En cours de spécification 🚀  
> **Version cible :** `v0.2.0`  
> **Objectif :** Transformer le synthétiseur modulaire en un instrument jouable instantanément au clavier matériel USB-MIDI et au clavier virtuel interactif.

---

## 1. Vue d'Ensemble & Périmètre

Le jalon 8 introduit la chaîne complète de contrôle instrumental pour **The Grid** :
1. **Couche d'entrée matérielle** : Prise en charge des claviers et contrôleurs USB-MIDI via `midir`.
2. **Couche de transport lock-free** : Acheminement sans latence ni allocation des messages MIDI depuis le thread d'entrée (USB ou UI) vers le thread audio CPAL.
3. **Moteur d'allocation de voix monophonique** : Gestion rigoureuse de la priorité des notes, du jeu legato et du retrigger d'enveloppe.
4. **Module DSP `MidiIn`** : Génération des signaux CV normalisés (Pitch $1\,\text{V/oct}$, Gate, Velocity, ModWheel, Pitch Bend).
5. **Clavier virtuel à l'écran** : Interface rétractable jouable à la souris et au clavier AZERTY/QWERTY.

---

## 2. Architecture Technique

```
┌─────────────────────────┐       ┌─────────────────────────┐
│ Clavier USB MIDI        │       │ Clavier Virtuel UI      │
│ (midir input thread)    │       │ (Souris / AZERTY / Web) │
└────────────┬────────────┘       └────────────┬────────────┘
             │                                 │
             │ Raw MIDI Bytes                  │ Tauri IPC command
             ▼                                 ▼
┌───────────────────────────────────────────────────────────┐
│ Host Controller (`grid-app`)                              │
│ Normalisation vers `MidiEvent`                            │
└────────────────────────────┬──────────────────────────────┘
                             │
                             ▼ SPSC Ring Buffer (lock-free)
┌───────────────────────────────────────────────────────────┐
│ Audio Thread (`grid-engine`)                              │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Monophonic Voice Allocator                          │  │
│  │ (Note Stack, Last/Low/High Note, Legato/Retrigger)  │  │
│  └──────────────────────────┬──────────────────────────┘  │
│                             │ CV Signals                  │
│                             ▼                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ `MidiIn` DSP Module                                 │  │
│  │  - Pitch ($1\,\text{V/oct}$)                        │  │
│  │  - Gate ($0.0$ / $1.0$)                             │  │
│  │  - Velocity ($0.0 \dots 1.0$)                       │  │
│  │  - ModWheel ($0.0 \dots 1.0$)                       │  │
│  │  - Pitch Bend ($-1.0 \dots +1.0$)                   │  │
│  └─────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────┘
```

---

## 3. Exigences Temps Réel & Performance

- **Latence de réponse MIDI** : Traitement immédiat des événements au bloc DSP suivant ($\le 1.45\,\text{ms}$ à $44.1\,\text{kHz}$ pour $64$ samples).
- **Zéro allocation mémoire** : La pile de notes et le ring buffer ont des capacités statiques préallouées.
- **Zéro blocage (Wait-Free)** : Les producteurs (threads MIDI et IPC) n'interrompent jamais le thread audio, même en cas de saturation de buffer.

---

## 4. Découpage en Tâches Atomiques

Le Jalon 8 est divisé en 6 tâches de développement atomiques et ordonnancées :

1. [Tâche 8.1 : Protocole MIDI & Bridge SPSC Lock-Free](./task-8.1-midi-protocol-lockfree-bridge.md)
2. [Tâche 8.2 : Gestionnaire de Voix Monophonique DSP](./task-8.2-monophonic-voice-allocator.md)
3. [Tâche 8.3 : Module DSP `MidiIn` dans le Graphe de Patch](./task-8.3-midi-in-dsp-module.md)
4. [Tâche 8.4 : Clavier Virtuel à l'Écran (Frontend & IPC)](./task-8.4-virtual-keyboard-ui.md)
5. [Tâche 8.5 : Intégration Périphériques Physiques USB-MIDI (`midir`)](./task-8.5-hardware-midi-midir.md)
6. [Tâche 8.6 : Patch Monophonique Jouable par Défaut & Validation E2E](./task-8.6-default-playable-patch-e2e.md)
