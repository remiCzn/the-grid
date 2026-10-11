# 📦 Jalon 14 : Distribution & Intégrations Externes (v1.0.0)

> **Statut :** Spécifications initiales (Découpage en tâches à venir) ⏳  
> **Version cible :** `v1.0.0` (Sortie Majeure)  
> **Objectif :** Mettre The Grid entre les mains des musiciens et producteurs sur tous les systèmes d'exploitation majeurs en application autonome et en format plugin (CLAP / VST3).

---

## 1. Description & Objectifs

La version 1.0.0 concrétise le projet en tant que produit professionnel : installeurs natifs pour Linux, macOS et Windows, panneau de préférences audio complet, et compatibilité avec les DAWs professionnelles (Bitwig, Reaper, Ableton Live) via les standards de plugins modernes.

---

## 2. Spécifications Fonctionnelles & Techniques

### 2.1 Panneau de Configuration Audio
- Sélection de l'interface audio (cartes son USB, interfaces ASIO, CoreAudio, ALSA/JACK/PipeWire).
- Sélection de la fréquence d'échantillonnage ($44.1\,\text{kHz}$, $48\,\text{kHz}$, $96\,\text{kHz}$).
- Réglage de la taille du buffer audio ($64$, $128$, $256$, $512$ samples) avec affichage de la latence théorique estimée en millisecondes.

### 2.2 Chaîne de Packaging & Déploiement Multiplateforme
- Pipeline CI/CD GitHub Actions automatisé :
  - **Linux** : Paquet `.deb`, `AppImage`, archive tarball.
  - **macOS** : Image disque `.dmg` signée et notarisée Apple (Apple Silicon & Intel).
  - **Windows** : Fichier installeur `.msi` / `.exe` signé.

### 2.3 Hébergement sous Forme de Plugin (CLAP / VST3)
- Création d'une crate wrapper `crates/grid-plugin` s'appuyant sur le framework `nih-plug` ou `clack`.
- Exportation sous le standard **CLAP** (standard ouvert supporté nativement par Bitwig et Reaper avec support de la modulation polyphonique par note) et **VST3**.
- Intégration de l'interface web Canvas dans la fenêtre hôte du plugin.

---

## 3. Critères d'Acceptation Globaux

- L'installeur fonctionne sans configuration préalable sur une machine vierge.
- Le plugin CLAP se charge dans Bitwig Studio et Reaper, répond aux événements MIDI de la piste hôte et synchronise son transport sur le tempo de la DAW.
- Couverture de tests automatisés $\ge 90\%$ sur l'ensemble de la base de code du workspace.
