//! Gestionnaire de voix monophonique pour la synthese temps reel.
//!
//! Ce module fournit les structures necessaires a la conversion d'un flux d'evenements
//! MIDI en signaux de controle continus (hauteur CV, gate, velocite, pitch bend, molette de modulation),
//! sans aucune allocation dynamique de memoire.

/// Capacite statique maximale de la pile de notes maintenues (touches simultanees).
pub const NOTE_STACK_CAPACITY: usize = 16;

/// Entree representant une note enfoncee avec sa velocite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteEntry {
    /// Numero de note MIDI (0..127).
    pub note: u8,
    /// Velocite a l'enfoncement (1..127).
    pub velocity: u8,
}

/// Pile statique circulaire a capacite fixe stockant les notes actuellement maintenues.
///
/// Elle garantit zero allocation dynamique et recycle la note la plus ancienne
/// en cas de debordement au-dela de `NOTE_STACK_CAPACITY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteStack {
    entries: [Option<NoteEntry>; NOTE_STACK_CAPACITY],
    len: usize,
}

impl Default for NoteStack {
    fn default() -> Self {
        Self::new()
    }
}

impl NoteStack {
    /// Cree une nouvelle pile de notes vide.
    pub const fn new() -> Self {
        Self {
            entries: [None; NOTE_STACK_CAPACITY],
            len: 0,
        }
    }

    /// Renvoie le nombre de notes actuellement maintenues.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Indique si la pile est vide.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Vide integralement la pile de notes.
    pub fn clear(&mut self) {
        self.entries.fill(None);
        self.len = 0;
    }


    /// Ajoute ou remonte une note dans la pile.
    ///
    /// Si la note est deja presente, sa velocite est mise a jour et elle est deplacee
    /// au sommet de la pile (derniere note jouee).
    /// Si la pile est pleine, la note la plus ancienne est evictee au profit de la nouvelle.
    pub fn push(&mut self, note: u8, velocity: u8) {
        // Verifier si la note existe deja
        if let Some(pos) = self.find_index(note) {
            // Decaler les elements suivants vers la gauche
            for i in pos..(self.len - 1) {
                self.entries[i] = self.entries[i + 1];
            }
            self.entries[self.len - 1] = Some(NoteEntry { note, velocity });
            return;
        }

        if self.len < NOTE_STACK_CAPACITY {
            self.entries[self.len] = Some(NoteEntry { note, velocity });
            self.len += 1;
        } else {
            // Pile pleine : evincer la plus ancienne note a l'index 0
            for i in 0..(NOTE_STACK_CAPACITY - 1) {
                self.entries[i] = self.entries[i + 1];
            }
            self.entries[NOTE_STACK_CAPACITY - 1] = Some(NoteEntry { note, velocity });
        }
    }

    /// Retire une note de la pile si elle est presente.
    ///
    /// Renvoie `true` si la note a ete trouvee et retiree, `false` sinon.
    pub fn remove(&mut self, note: u8) -> bool {
        if let Some(pos) = self.find_index(note) {
            for i in pos..(self.len - 1) {
                self.entries[i] = self.entries[i + 1];
            }
            self.entries[self.len - 1] = None;
            self.len -= 1;
            true
        } else {
            false
        }
    }

    /// Renvoie la derniere note enfoncee (sommet de la pile).
    pub const fn get_last(&self) -> Option<NoteEntry> {
        if self.len == 0 {
            None
        } else {
            self.entries[self.len - 1]
        }
    }

    fn get_by_extreme<F>(&self, compare: F) -> Option<NoteEntry>
    where
        F: Fn(u8, u8) -> bool,
    {
        let mut extreme: Option<NoteEntry> = None;
        for i in 0..self.len {
            if let Some(entry) = self.entries[i] {
                match extreme {
                    None => extreme = Some(entry),
                    Some(cur) if compare(entry.note, cur.note) => extreme = Some(entry),
                    _ => {}
                }
            }
        }
        extreme
    }

    /// Renvoie la note la plus grave parmi les notes actives.
    pub fn get_lowest(&self) -> Option<NoteEntry> {
        self.get_by_extreme(|candidate, current| candidate < current)
    }

    /// Renvoie la note la plus aigue parmi les notes actives.
    pub fn get_highest(&self) -> Option<NoteEntry> {
        self.get_by_extreme(|candidate, current| candidate > current)
    }


    fn find_index(&self, note: u8) -> Option<usize> {
        for i in 0..self.len {
            if let Some(entry) = self.entries[i] {
                if entry.note == note {
                    return Some(i);
                }
            }
        }
        None
    }
}

/// Priorite de note determinant quelle note de la pile est affectee a la voix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotePriority {
    /// La derniere touche enfoncee a la priorite (par defaut).
    #[default]
    LastNote,
    /// La note la plus grave parmi les touches actives est jouee.
    LowNote,
    /// La note la plus aigue parmi les touches actives est jouee.
    HighNote,
}

/// Alias pour `NotePriority` conformement a la specification.
pub type Priority = NotePriority;


/// Mode de jeu determinant le comportement de la Gate et de l'enveloppe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayMode {
    /// Mode Legato : la Gate reste a 1.0 lors des liaisons et seul le Pitch change (par defaut).
    #[default]
    Legato,
    /// Mode Retrigger : chaque nouvelle note active force un re-trigger de l'enveloppe.
    Retrigger,
}

/// Configuration des modes de jeu et plages de modulation de l'allocateur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoiceModeConfig {
    /// Règle de priorite des notes.
    pub priority: NotePriority,
    /// Mode d'articulation (Legato ou Retrigger).
    pub play_mode: PlayMode,
    /// Plage maximale de Pitch Bend en demi-tons (par defaut 2.0 demi-tons).
    pub pitch_bend_range_semitones: f32,
}

impl Default for VoiceModeConfig {
    fn default() -> Self {
        Self {
            priority: NotePriority::LastNote,
            play_mode: PlayMode::Legato,
            pitch_bend_range_semitones: 2.0,
        }
    }
}

/// Etat de sortie instantane de l'allocateur de voix monophonique.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VoiceState {
    /// Tension de hauteur de note (1 V/octave, C4 = note 60 = 0.0V).
    pub pitch_cv: f32,
    /// Signal logique de Gate (1.0 = actif, 0.0 = relache).
    pub gate: f32,
    /// Velocite normalisee de la note active (0.0 a 1.0).
    pub velocity: f32,
    /// Valeur de la molette de modulation CC 1 (0.0 a 1.0).
    pub mod_wheel: f32,
    /// Valeur normalisee du pitch bend (-1.0 a +1.0).
    pub pitch_bend: f32,
    /// Indicateur d'impulsion de re-trigger d'enveloppe.
    pub is_gate_triggered: bool,
}

impl VoiceState {
    /// Calcule la tension de pitch totale incluant le pitch bend applique en 1 V/octave.
    #[inline]
    pub fn total_pitch_cv(&self, pitch_bend_range_semitones: f32) -> f32 {
        self.pitch_cv + (self.pitch_bend * pitch_bend_range_semitones / 12.0)
    }
}

/// Gestionnaire de voix monophonique sans allocation dynamique.
#[derive(Debug, Clone)]
pub struct MonophonicVoiceAllocator {
    config: VoiceModeConfig,
    stack: NoteStack,
    state: VoiceState,
    active_note: Option<u8>,
}

impl Default for MonophonicVoiceAllocator {
    fn default() -> Self {
        Self::new(VoiceModeConfig::default())
    }
}

impl MonophonicVoiceAllocator {
    /// Cree un nouvel allocateur avec la configuration specifiee.
    pub fn new(config: VoiceModeConfig) -> Self {
        Self {
            config,
            stack: NoteStack::new(),
            state: VoiceState::default(),
            active_note: None,
        }
    }

    /// Renvoie la configuration courante.
    pub fn config(&self) -> &VoiceModeConfig {
        &self.config
    }

    /// Modifie la configuration courante.
    pub fn set_config(&mut self, config: VoiceModeConfig) {
        self.config = config;
        self.update_voice_state();
    }

    /// Renvoie l'etat courant de sortie de la voix.
    pub fn state(&self) -> &VoiceState {
        &self.state
    }

    /// Calcule la tension de pitch totale actuelle incluant l'offset de pitch bend selon la plage configuree.
    #[inline]
    pub fn total_pitch_cv(&self) -> f32 {
        self.state.total_pitch_cv(self.config.pitch_bend_range_semitones)
    }

    /// Renvoie une reference a la pile de notes active.
    pub fn stack(&self) -> &NoteStack {
        &self.stack
    }

    /// Reinitialise completement l'allocateur (vide la pile, eteint la gate et reinitialise les controleurs).
    pub fn reset(&mut self) {
        self.all_notes_off();
        self.state.pitch_bend = 0.0;
        self.state.mod_wheel = 0.0;
    }

    /// Coupe toutes les notes actives et remet la Gate a 0.0 sans reinitialiser les controleurs (pitch bend / mod wheel).
    pub fn all_notes_off(&mut self) {
        self.stack.clear();
        self.active_note = None;
        self.state.gate = 0.0;
        self.state.velocity = 0.0;
        self.state.is_gate_triggered = false;
    }

    /// Consomme l'indicateur d'impulsion de trigger et le remet a `false`.
    ///
    /// Utilise par le moteur audio a chaque cycle pour detecter un nouveau front montant.
    #[inline]
    pub fn consume_gate_trigger(&mut self) -> bool {
        let triggered = self.state.is_gate_triggered;
        self.state.is_gate_triggered = false;
        triggered
    }

    /// Traite un evenement MIDI et met a jour instantanement l'etat de la voix.
    pub fn process_event(&mut self, event: &grid_core::midi::MidiEvent) {
        use grid_core::midi::MidiEvent;
        match event {
            MidiEvent::NoteOn { note, velocity, .. } => {
                if *velocity == 0 {
                    self.handle_note_off(*note);
                } else {
                    self.handle_note_on(*note, *velocity);
                }
            }
            MidiEvent::NoteOff { note, .. } => {
                self.handle_note_off(*note);
            }
            MidiEvent::ControlChange { controller, value, .. } => {
                if *controller == 1 {
                    // Molette de modulation CC 1 : normalisee dans [0.0, 1.0]
                    self.state.mod_wheel = (*value as f32) / 127.0;
                }
            }
            MidiEvent::PitchBend { value, .. } => {
                // Pitch bend MIDI 14-bit centre sur 0 dans [-8192, 8191] -> normalise dans [-1.0, 1.0]
                self.state.pitch_bend = (*value as f32) / 8192.0;
            }
            MidiEvent::AllNotesOff { .. } => {
                self.all_notes_off();
            }
        }
    }

    fn select_target_note(&self) -> Option<NoteEntry> {
        match self.config.priority {
            NotePriority::LastNote => self.stack.get_last(),
            NotePriority::LowNote => self.stack.get_lowest(),
            NotePriority::HighNote => self.stack.get_highest(),
        }
    }

    fn apply_target_note(&mut self, target: NoteEntry) {
        self.active_note = Some(target.note);
        self.state.pitch_cv = (target.note as f32 - 60.0) / 12.0;
        self.state.gate = 1.0;
        self.state.velocity = (target.velocity as f32) / 127.0;
    }

    fn handle_note_on(&mut self, note: u8, velocity: u8) {
        let had_active = self.active_note.is_some();
        self.stack.push(note, velocity);

        if let Some(target) = self.select_target_note() {
            self.apply_target_note(target);

            if !had_active {
                // Front montant initial depuis le silence
                self.state.is_gate_triggered = true;
            } else {
                match self.config.play_mode {
                    PlayMode::Retrigger => {
                        // En mode retrigger, chaque note pressee declenche une impulsion
                        self.state.is_gate_triggered = true;
                    }
                    PlayMode::Legato => {
                        // En mode legato, la gate reste continue et aucun retrigger n'est emis
                        self.state.is_gate_triggered = false;
                    }
                }
            }
        }
    }

    fn handle_note_off(&mut self, note: u8) {
        self.stack.remove(note);

        if let Some(target) = self.select_target_note() {
            self.apply_target_note(target);
            // Le relachement d'une touche ne declenche jamais de retrigger d'attaque
            self.state.is_gate_triggered = false;
        } else {
            // Aucune note restante dans la pile
            self.active_note = None;
            self.state.gate = 0.0;
            self.state.is_gate_triggered = false;
            // self.state.pitch_cv est conserve pour permettre a la queue de release de sonner juste
        }
    }

    fn update_voice_state(&mut self) {
        if let Some(target) = self.select_target_note() {
            self.apply_target_note(target);
        } else {
            self.active_note = None;
            self.state.gate = 0.0;
            self.state.is_gate_triggered = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grid_core::midi::MidiEvent;

    #[test]
    fn test_embedded_voice_allocator_basic_flow() {
        let mut allocator = MonophonicVoiceAllocator::default();
        allocator.process_event(&MidiEvent::note_on(0, 60, 100));
        assert_eq!(allocator.state().gate, 1.0);
        assert_eq!(allocator.state().pitch_cv, 0.0);
        assert_eq!(allocator.total_pitch_cv(), 0.0);

        // AllNotesOff preserves mod wheel
        allocator.process_event(&MidiEvent::control_change(0, 1, 100));
        allocator.process_event(&MidiEvent::all_notes_off(0));
        assert_eq!(allocator.state().gate, 0.0);
        assert!((allocator.state().mod_wheel - (100.0 / 127.0)).abs() < 1e-5);
    }
}



