# 🎛️ Jalon 11 : Expansion du Catalogue Modulaire (v0.5.0)

> **Statut :** Spécifications initiales (Découpage en tâches à venir) ⏳  
> **Version cible :** `v0.5.0`  
> **Objectif :** Débloquer la création de textures sonores avancées, d'effets d'espace et de traitements logiques/mathématiques typiques de Bitwig et Eurorack.

---

## 1. Description & Objectifs

Après avoir stabilisé l'infrastructure audio, le graphe et l'ergonomie, le Jalon 11 enrichit considérablement la palette sonore en implémentant les modules indispensables à la synthèse complexe : modulation cyclique et stochastique, traitement audio créatif (delay, distorsion) et calcul analogique/logique.

---

## 2. Spécifications Fonctionnelles & Techniques

### 2.1 Modulateurs & Contrôle
- **`LFO` (Low-Frequency Oscillator)** :
  - Formes d'ondes : Sine, Triangle, Saw Up/Down, Square, Random (Sampled Noise).
  - Mode libre continu ($0.01\,\text{Hz} \dots 50\,\text{Hz}$) ou synchronisé au tempo (divisions $1/4$, $1/8$, $1/16$, triolets).
  - Réinitialisation de phase par entrée `Reset / Sync`.
- **`Noise` (Générateur de bruit)** :
  - Génération de bruit blanc (distribution uniforme) et bruit rose (filtre de Voss-McCartney).
- **`Sample & Hold`** :
  - Échantillonnage de l'entrée signal à chaque front montant de l'entrée d'horloge.

### 2.2 Effets DSP
- **`Stereo Delay`** :
  - Ligne à retard circulaire interpolée (interpolation cubique Hermite).
  - Contrôle du temps de retard indépendant gauche/droite, feedback avec filtre coupe-bas et coupe-haut intégré.
- **`Drive / Saturation`** :
  - Algorithmes non-linéaires de mise en forme d'ondes (*waveshaping*) :
    - Soft clipping hyperbolique ($\tanh$).
    - Saturation asymétrique typée lampe à vide.
  - Suréchantillonnage interne 2x ou compensation DC offset.

### 2.3 Maths & Logique
- **`Math`** :
  - Opérations élémentaires sur deux signaux : addition ($A + B$), soustraction ($A - B$), multiplication ($A \cdot B$), inversion ($-A$).
- **`Min/Max` & `Comparator`** :
  - Sélection du signal minimal/maximal.
  - Comparateur produisant une impulsion Gate ($1.0$ ou $0.0$) selon un seuil configurable.

---

## 3. Critères d'Acceptation Globaux

- Tous les nouveaux modules sont intégrés dans `grid-core` (définition de nœud) et `grid-engine` (calcul DSP).
- Zéro allocation dans la boucle de rendu audio pour tous les nouveaux algorithmes DSP.
- Tests unitaires et d'intégration couvrant la fidélité spectrale, la linéarité et la stabilité numérique de chaque module.
