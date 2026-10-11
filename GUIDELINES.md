# Directives architecturales et règles du projet The Grid

Ce document consigne les règles impératives régissant l'architecture, le développement, la qualité et la sécurité du synthétiseur modulaire **The Grid**.

---

## 1. Règles d'or du Thread Audio Temps Réel (Realtime Audio Safety)

Toute modification touchant `grid-engine` et la boucle de rendu audio doit strictement respecter ces contraintes absolues :

1. **Zéro allocation dynamique de mémoire (`no malloc`) :**
   - Aucune instanciation via `Vec::new()`, `Box`, `String`, ou méthode réallouante à l'intérieur du callback audio.
   - Tous les buffers de travail, états d'oscillateurs, filtres et enveloppes sont préalloués au démarrage ou lors du chargement de plan hors thread audio.
2. **Zéro verrouillage bloquant (`no mutex`) :**
   - Jamais de `std::sync::Mutex`, `parking_lot::Mutex`, `std::sync::RwLock` ou tout primitive pouvant endormir le thread audio (`priority inversion`).
   - Communication bidirectionnelle exclusive via des files circulaires **lock-free SPSC** (`rtrb`).
3. **Zéro appel système ou I/O :**
   - Pas de lecture/écriture disque ou fichier.
   - Pas de logging direct (pas de `println!`, pas de `log::info!` dans le callback).
   - Pas d'appels réseau, pas de syscalls pouvant bloquer le scheduler.

---

## 2. Communication Inter-Thread & Télémétrie

- **Host -> Audio (Command Queue) :**
  - Messages de taille fixe (`AudioCommand`), sérialisables et non réallouants.
  - Consommation par le thread audio en mode `pop()` non-bloquant en début de chaque bloc de traitement.
- **Audio -> Host (Telemetry Queue) :**
  - Télémétrie décimée (fréquence cible : 30 Hz - 60 Hz).
  - Émission via `push()` non-bloquant : si la file est pleine, la trame est ignorée pour ne jamais ralentir le thread audio (`drop-on-full`).

---

## 3. Séparation des Responsabilités

- **Frontend (`frontend/`) :** Modèle visuel déclaratif, rendu Canvas 2D/WebGL, interaction utilisateur. Ne manipule aucun détail DSP.
- **Backend Host (`crates/grid-app`) :** Gestion des commandes Tauri, validation des patches, persistance JSON, décimation et propagation de la télémétrie vers l'UI.
- **Logique Core (`crates/grid-core`) :** Modèle sémantique, validation topologique, compilateur d'instructions linéaires, types des messages SPSC.
- **Moteur Audio (`crates/grid-engine`) :** Pilote audio (`cpal`), processeur temps réel, DSP préalloué, ring buffers SPSC.

---

## 4. Règle de Couverture de Tests et Qualité

- Tout code produit doit être accompagné de tests unitaires ou d'intégration.
- La couverture de code ne doit **jamais régresser** lors d'une modification.
- Tout nouveau module DSP, processeur de canal SPSC ou compilateur de graphe doit être testé avec des assertions vérifiant le comportement fonctionnel et la robustesse en cas de saturation de queue.

### Registre de couverture de référence (Étape 6)
| Module / Crate | Couverture Lignes | Statut |
|---|---|---|
| `grid-core/src/compiler.rs` | **100.00%** (59/59) | Validé |
| `grid-core/src/graph.rs` | **97.59%** (81/83) | Validé (Kahn DAG + détection de cycle) |
| `grid-core/src/message.rs` | **100.00%** (9/9) | Validé |
| `grid-core/src/model.rs` | **100.00%** (125/125) | Validé |
| `grid-engine/src/ringbuffer.rs` | **100.00%** (14/14) | Validé |
| `grid-engine/src/dsp/vca.rs` | **100.00%** (26/26) | Validé (VCA sans allocation) |
| `grid-engine/src/dsp/oscillator.rs` | **98.21%** (55/56) | Validé (PolyBLEP anti-aliasing) |
| `grid-engine/src/dsp/svf.rs` | **60.47%** (26/43) | Validé (Filtre résonant SVF TPT) |
| `grid-engine/src/realtime.rs` | **92.52%** (198/214) | Validé (Rendu audio réel + scratch buffers) |
| Total (hors driver ALSA physique) | **> 94.3%** | 18 tests unitaires / d'intégration passés |

