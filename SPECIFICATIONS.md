# Spécifications Techniques et Fonctionnelles : The Grid

Ce document formalise les spécifications complètes du clone open source de « The Grid » (synthétiseur modulaire visuel type Bitwig).

---

## 1. Vue d'Ensemble et Objectifs

L'application est un environnement de synthèse sonore modulaire graphique combinant la souplesse d'un patcher visuel et la rigueur d'un moteur audio temps réel durci.

### 1.1 Composants du Système
```text
┌────────────────────────────────────────────────────────┐
│                   Frontend (Tauri v2)                  │
│       Canvas 2D / WebGL (Grille de points, Modules,    │
│              Ports, Câbles de Bézier, Vu-mètres)       │
└──────────────────────────┬─────────────────────────────┘
                           │ IPC Tauri (invoke / events)
┌──────────────────────────▼─────────────────────────────┐
│                    Host Backend (Rust)                 │
│         - Persistance (JSON) & Validation sémantique   │
│         - Compilation topologique du graphe            │
│         - Décimation de la télémétrie (30-60 Hz)       │
└──────────────────────────┬─────────────────────────────┘
                           │ Lock-Free SPSC Ring Buffers
┌──────────────────────────▼─────────────────────────────┐
│               Moteur Audio Temps Réel (CPAL)           │
│   - Thread isolé à haute priorité (Zero-Alloc, Zero-Lock)│
│   - Exécution linéaire d'un plan d'instructions DSP    │
│   - Ring buffers audio et calcul de crête/RMS          │
└────────────────────────────────────────────────────────┘
```

---

## 2. Spécification du Modèle de Patch Déclaratif

### 2.1 Format de Persistance (JSON)
Le patch est sérialisable sans perte. Le frontend manipule cette représentation pour la persistance et l'affichage.

```json
{
  "version": "1.0.0",
  "name": "Init Patch",
  "bpm": 120.0,
  "modules": [
    {
      "id": 1,
      "kind": "Oscillator",
      "name": "VCO 1",
      "position": { "x": 100, "y": 150 },
      "parameters": {
        "frequency": 440.0,
        "waveform": "sine"
      }
    },
    {
      "id": 2,
      "kind": "MasterOut",
      "name": "Audio Output",
      "position": { "x": 500, "y": 150 },
      "parameters": {
        "gain": 0.8
      }
    }
  ],
  "cables": [
    {
      "id": 1,
      "from_module": 1,
      "from_port": 0,
      "to_module": 2,
      "to_port": 0,
      "color": "#ff5200"
    }
  ]
}
```

### 2.2 Typologie des Signaux
Les ports et câbles véhiculent trois familles de signaux :
1. **Audio Rate (`Audio`) :**
   - Signal bipolaire normalisé standard : $[-1.0, 1.0]$ (tolérance transitoire au-delà).
   - Taux d'échantillonnage natif ($f_s = 44.1\,\text{kHz}, 48\,\text{kHz}, 96\,\text{kHz}$).
   - Représenté par la couleur orange (`#ff5200`).
2. **Control Voltage (`ControlVoltage` / CV) :**
   - Signal de modulation bipolaire $[-1.0, 1.0]$ ou unipolaire $[0.0, 1.0]$ (ex: $1\,\text{V/octave}$ pour le pitch).
   - Traité au taux d'échantillonnage audio pour éliminer tout zipper noise (pas de sous-échantillonnage destructif sur la modulation).
   - Représenté par la couleur bleue (`#00a8ff`).
3. **Trigger / Gate (`Trigger`) :**
   - Événements discrets et signaux logiques binaires ($0.0$ ou $1.0$).
   - Représenté par la couleur jaune (`#ffd32a`).

---

## 3. Spécification de la Compilation du Graphe Audio

### 3.1 Validation Topologique (DAG)
1. Le graphe de connexions forme un Graphe Orienté Acyclique (DAG) pour les connexions instantanées.
2. **Détection de Cycle :** Tout cycle sans module de retard unitaire (bloc délai $z^{-1}$) est rejeté à la compilation avec l'erreur `GraphError::CycleDetected`.
3. **Tri Topologique (Kahn / Tarjan) :** Les nœuds sont ordonnés pour garantir que toute entrée d'un module $M_i$ est calculée par ses sources avant l'exécution de $M_i$.

### 3.2 Plan Linéaire d'Exécution (`ExecutionPlan`)
Le compilateur génère une séquence linéaire d'opérations exécutables dans un tableau préalloué :
- Aucun graphe dynamique, pointeur d'arbre ou liaison indirecte n'est résolu pendant le rendu audio.
- Les signaux entre modules transitent par un pool fini de **buffers temporaires préalloués** (scratch buffers).

---

## 4. Spécification du Protocole IPC Tauri v2

### 4.1 Commandes Hôte (`invoke`)
| Commande Tauri | Arguments | Description |
|---|---|---|
| `start_audio` | `{}` | Démarre la lecture du moteur audio |
| `stop_audio` | `{}` | Coupe le moteur audio et applique le silence |
| `set_master_gain` | `gain: f32` (clampé $[0.0, 2.0]$) | Modifie le gain de sortie général |
| `set_parameter` | `module_id: u32, param_id: u32, value: f32` | Met à jour un paramètre en continu |
| `apply_patch` | `patch: Patch` | Valide, compile et swap le plan d'exécution |

### 4.2 Événements Asynchrones UI (`listen`)
| Nom de l'événement | Charge utile | Fréquence |
|---|---|---|
| `telemetry_update` | `TelemetryFrame { frame_index, peak_left, peak_right, rms_left, rms_right }` | 30 Hz - 60 Hz |

---

## 5. Spécification de l'Audio Temps Réel & Sécurité

### 5.1 Communication SPSC Lock-Free
- **File des Commandes (`CommandQueue`) :** Capacité statique de 256 éléments. Push non bloquant côté hôte. Pop non bloquant côté audio thread au début de chaque bloc.
- **File de Télémétrie (`TelemetryQueue`) :** Capacité statique de 64 éléments.
  - **Décimation :** Une trame toutes les $N = f_s / 60$ frames audio.
  - **Drop-on-Full :** Si la file de télémétrie est pleine, la trame est jetée sans délai ni blocage.

### 5.2 Contraintes Matérielles & OS
- Pilote : ALSA (Linux), WASAPI (Windows), CoreAudio (macOS) via `cpal`.
- Format d'échantillonnage : `f32` entrelacé stéréo.
- Latence cible : blocs de 64 à 256 échantillons ($1.33\,\text{ms}$ à $5.33\,\text{ms}$ à $48\,\text{kHz}$).
