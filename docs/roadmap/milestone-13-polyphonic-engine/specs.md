# 🎹 Jalon 13 : Moteur Polyphonique (v0.7.0)

> **Statut :** Spécifications initiales (Découpage en tâches à venir) ⏳  
> **Version cible :** `v0.7.0`  
> **Objectif :** Faire évoluer l'architecture pour supporter la polyphonie complète (4 à 16 voix) tout en conservant une exécution SIMD/vectorisée ultra-rapide et sans allocation.

---

## 1. Description & Objectifs

Jusqu'au Jalon 12, le synthétiseur opère en monophonie. Le Jalon 13 transforme le moteur audio pour traiter plusieurs voix simultanées. Ce passage est une refonte structurante du compilateur et du moteur DSP : séparation claire des sections polyphoniques (par voix : VCO, VCF, ADSR) et des sections monophoniques (effets master, delay, sommation finale).

---

## 2. Spécifications Fonctionnelles & Techniques

### 2.1 Pool de Voix DSP Vectorisé
- Gestionnaire de voix polyphonique capable de piloter de 4 à 16 voix.
- Algorithme de vol de voix (*Voice Stealing*) déterministe :
  - Priorité aux voix libres (phase Idle).
  - À défaut, vol de la note la plus ancienne ou de la voix la plus avancée dans sa phase de Release.
- Traitement optimisé par blocs pour maximiser l'autovectorisation SIMD (AVX2 / NEON).

### 2.2 Séparation Polyphonique vs Monophonique dans le Graphe
- Marquage automatique ou explicite des nœuds :
  - *Poly Nodes* : dupliqués pour chaque voix active.
  - *Mono Nodes* : exécutés une seule fois par bloc de 64 samples.
- Module de sommation / mélangeur de voix (*Voice Summing Node*) agrégeant les voix individuelles en signal stéréo master.

---

## 3. Critères d'Acceptation Globaux

- Jeu fluide d'accords de 4 à 8 notes sans artéfacts de saturation ou de dropouts de buffer.
- Empreinte CPU maîtrisée ($\le 15\%$ CPU pour un synthé soustractif 8 voix sur matériel standard).
- Transition inaudible lors du vol de voix grâce à un fondu enchaîné court (*quick release*).
