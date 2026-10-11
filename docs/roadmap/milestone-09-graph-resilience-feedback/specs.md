# 🔄 Jalon 9 : Résilience du Graphe & Boucles de Feedback (v0.3.0)

> **Statut :** Spécifications initiales (Découpage en tâches à venir) ⏳  
> **Version cible :** `v0.3.0`  
> **Objectif :** Autoriser les interconnexions cycliques indispensables au son modulaire (FM croisée, feedback de filtre, auto-oscillations chaotiques) et introduire l'atténuation sur chaque câble.

---

## 1. Description & Objectifs

Actuellement, le compilateur DAG applique un tri topologique de Kahn strict qui rejette tout graphe comportant des dépendances circulaires. Or, dans la synthèse modulaire (Eurorack, Bitwig The Grid), le feedback est au cœur de la création sonore (auto-modulation FM, injection de la sortie d'un filtre dans son entrée CV, delays réinjectés).

Le Jalon 9 a pour mission de lever cette limitation en intégrant une détection des cycles et l'insertion automatique d'un délai unitaire d'un bloc DSP ($z^{-1} = 64$ samples) pour briser les cycles topologiques sans jamais bloquer le compilateur.

---

## 2. Spécifications Fonctionnelles & Techniques

### 2.1 Détection des Arcs de Rétroaction (*Back-Edges*)
- Algorithme de parcours en profondeur (DFS) avec marquage tricolore (*White / Gray / Black*) sur le graphe de modules.
- Identification précise des connexions fermant une boucle dirigée.
- Conservation d'un sous-graphe acyclique pour l'ordonnancement séquentiel via Kahn.

### 2.2 Insertion Automatique de Délai Unitaire ($z^{-1}$)
- Pour chaque arc de feedback identifié, allocation d'un slot dans un buffer d'historique circulaire.
- Lors de l'évaluation du bloc $N$, la connexion de feedback lit les 64 samples générés au bloc précédent $N-1$.
- Zéro allocation dynamique : les buffers de retard d'un bloc sont préalloués dans le processeur audio.
- Comportement déterministe et stable à faible latence (retard de $1.45\,\text{ms}$ à $44.1\,\text{kHz}$).

### 2.3 Atténuation par Câble (*Cable Amount*)
- Ajout d'une propriété d'atténuation bipolaire $A \in [-1.0, 1.0]$ sur chaque connexion (`Connection`).
- Rendu visuel d'un petit badge ou potentiomètre sur la courbe de Bézier au clic sur le câble.
- Calcul direct dans le processeur DSP : multiplication de l'échantillon par $A$ sans nécessiter l'ajout d'un module atténuateur dédié.

---

## 3. Critères d'Acceptation Globaux

- Tout patch circulaire (ex. VCO 1 modulant VCO 2 et VCO 2 modulant VCO 1) compile sans erreur et produit un son continu riche en harmoniques FM.
- L'inversion de phase ($A = -1.0$) sur un câble annule parfaitement un signal en opposition de phase.
- Zéro dégradation des performances du thread audio.
