# 🖥️ Jalon 10 : Ergonomie Spatiale & Workflow Canvas (v0.4.0)

> **Statut :** Spécifications initiales (Découpage en tâches à venir) ⏳  
> **Version cible :** `v0.4.0`  
> **Objectif :** Fluidifier la manipulation sur des patchs denses et complexes grâce à un canvas infini navigable et des outils d'édition rapides.

---

## 1. Description & Objectifs

À mesure que les patchs grandissent en complexité, l'espace d'affichage fixe devient une contrainte majeure. Le Jalon 10 modernise l'espace de travail Canvas 2D pour lui apporter la souplesse spatiale d'outils modernes comme Blender, Figma ou Bitwig : navigation panoramique, zoom fluide centré sur le curseur, navigateur de modules avec recherche prédictive, et feedback visuel temps réel sur les câbles.

---

## 2. Spécifications Fonctionnelles & Techniques

### 2.1 Navigation Infinie (Pan & Zoom)
- Translation du plan de travail : clic molette enfoncé + déplacement ou barre d'espace + glisser.
- Zoom continu à la molette centré sur la position de la souris avec matrices de transformation 2D.
- Grille matricielle avec niveaux de détail dynamiques (LOD) s'adaptant à l'échelle de zoom.

### 2.2 Navigateur de Modules Contextuel (*Module Browser*)
- Ouverture au clic droit sur le canvas ou via la touche `Espace` / `Tab`.
- Champ de recherche textuel instantané avec filtre au clavier.
- Catégories thématiques : *Générateurs*, *Filtres*, *Enveloppes*, *Modulateurs*, *Routage/Master*.
- Positionnement automatique du module inséré exactement à l'emplacement du curseur.

### 2.3 Productivité d'Édition
- Boîte de sélection multiple (lasso rectangulaire par glisser-déposer sur le fond).
- Déplacement groupé de plusieurs modules connectés.
- Raccourcis clavier universels : `Ctrl+D` (duplication des modules et réglages), `Suppr` / `Backspace` (suppression), `Ctrl+Z` / `Ctrl+Y` (historique d'annulation).

### 2.4 Visualisation de Signal sur les Câbles
- Effet visuel dynamique sur les courbes de Bézier reflétant le flux de signal en temps réel.
- Modulation de l'intensité lumineuse néon ou train d'ondes lumineuses pulsant selon l'amplitude RMS ou les impulsions de Gate.

---

## 3. Critères d'Acceptation Globaux

- Navigation à 60 FPS constants même sur des patchs comportant plus de 50 modules et 100 câbles.
- Insertion d'un module par recherche clavier en moins d'une seconde sans quitter les mains du clavier.
- Aucune régression sur le drag & drop des fiches de connexion et potentiomètres.
