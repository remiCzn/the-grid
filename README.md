# The Grid - Modular Synthesizer (Tauri v2 + Realtime Audio Rust Engine)

Un clone modulaire type Bitwig « The Grid » associant un frontend Tauri v2 (Canvas 2D réactif) et un moteur audio temps réel haute fidélité (Rust, zero-alloc, zero-lock, SPSC lock-free).

---

## Documentation de Référence

- 📄 **[Résumé Exécutif du Projet (`PROJECT_SUMMARY.md`)](PROJECT_SUMMARY.md)** : Synthèse globale d'architecture, état d'avancement et perspectives d'évolution (idéal pour partage).
- 📜 **[Règles et Directives Architecturales](GUIDELINES.md)** : Règles d'or temps réel (`no malloc`, `no mutex`, `no I/O`), protocole de communication SPSC, et registre de couverture des tests.
- 📐 **[Spécifications Techniques et Fonctionnelles](SPECIFICATIONS.md)** : Modèle de patch JSON, typologie des signaux (Audio, CV, Trigger), validation DAG du graphe, commandes IPC Tauri et dimensionnement des files.
- 🗺️ **[Feuille de Route (Roadmap)](ROADMAP.md)** : Étapes d'implémentation, livrables terminés et jalons futurs.
- 🖼️ **[Prévisualisation du Rendu Visuel (`FRONTEND_PREVIEW.md`)](FRONTEND_PREVIEW.md)** : Capture d'écran native du frontend.

---

## Architecture des Crates

- **`crates/grid-core`** : Modèle déclaratif (modules, câbles, ports), compilation du graphe d'exécution et définition des payloads SPSC.
- **`crates/grid-engine`** : Runtime audio temps réel isolé (cpal, zero-alloc, zero-lock, zero-IO) avec ring buffers SPSC (`rtrb`).
- **`crates/grid-app`** : Application Tauri v2 hôte (commands IPC, persistance, orchestration, télémétrie décimée).
- **`frontend/`** : Interface utilisateur (Vite + Bun + Canvas 2D, gestion du patch, télémétrie temps réel).
