# ⏱️ Jalon 12 : Transport & Séquençage (v0.6.0)

> **Statut :** Spécifications initiales (Découpage en tâches à venir) ⏳  
> **Version cible :** `v0.6.0`  
> **Objectif :** Permettre la création de motifs mélodiques, rythmiques autonomes et génératifs sans nécessiter d'environnement DAW externe.

---

## 1. Description & Objectifs

Pour devenir un véritable instrument autonome, **The Grid** a besoin d'une horloge interne maîtresse et d'outils de séquençage pas à pas. Le Jalon 12 dote l'application d'un moteur de transport global et d'une suite de modules rythmiques et mélodiques.

---

## 2. Spécifications Fonctionnelles & Techniques

### 2.1 Moteur de Transport Interne
- Gestionnaire de tempo BPM maître ($20.0 \dots 300.0\,\text{BPM}$).
- États de lecture : *Play*, *Pause*, *Stop*, *Rewind*.
- Horloge d'échantillons échantillon-exacte (PPQN 96 ou 480 ticks par noire).
- Barre de transport dans l'interface utilisateur avec commandes Play/Stop, affichage BPM, mesure et temps.

### 2.2 Step Sequencer (8 / 16 pas)
- Éditeur visuel interactif sur le module :
  - Curseur de hauteur de note (Pitch par pas en demi-tons ou $1\,\text{V/oct}$).
  - Toggle Gate ON/OFF et potentiomètre de durée de gate (*Gate Length / Duty Cycle*).
  - Contrôle de probabilité d'événement par pas ($0\% \dots 100\%$).
- Sorties du module : `Pitch`, `Gate`, `End of Cycle Pulse`.

### 2.3 Diviseur d'Horloge (*Clock Divider*)
- Entrée horloge générale.
- Sorties sous-divisions multiples ($/1$, $/2$, $/4$, $/8$, $/16$, triolets) pour déclencher des enveloppes secondaires ou faire tourner des LFOs en polyrythmie.

---

## 3. Critères d'Acceptation Globaux

- Précision d'horloge absolue : aucune dérive temporelle mesurable après 30 minutes de lecture continue.
- Édition en direct des pas de séquençage dans l'UI sans coupure audio.
- Prise en charge des signatures temporelles 4/4 et 3/4.
