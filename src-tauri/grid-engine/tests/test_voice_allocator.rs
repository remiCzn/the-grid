use grid_engine::voice::{NoteEntry, NoteStack, NOTE_STACK_CAPACITY};

#[test]
fn test_note_stack_push_remove_and_capacity() {
    let mut stack = NoteStack::new();
    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);

    // Push notes
    stack.push(60, 100);
    stack.push(64, 90);
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.get_last(), Some(NoteEntry { note: 64, velocity: 90 }));

    // Push existing note: updates velocity and moves to top
    stack.push(60, 110);
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.get_last(), Some(NoteEntry { note: 60, velocity: 110 }));

    // Remove middle note
    assert!(stack.remove(64));
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.get_last(), Some(NoteEntry { note: 60, velocity: 110 }));

    // Remove non-existent note
    assert!(!stack.remove(70));
    assert_eq!(stack.len(), 1);

    // Clear
    stack.clear();
    assert!(stack.is_empty());
    assert_eq!(stack.get_last(), None);
}

#[test]
fn test_note_stack_overflow_recycles_oldest() {
    let mut stack = NoteStack::new();

    // Push 20 notes into capacity of 16
    for i in 0..20 {
        stack.push(40 + i, 80);
    }

    assert_eq!(stack.len(), NOTE_STACK_CAPACITY);
    // The oldest 4 notes (40, 41, 42, 43) should have been evicted.
    // The oldest remaining should be note 44, and the latest note 59.
    assert_eq!(stack.get_last(), Some(NoteEntry { note: 59, velocity: 80 }));
    assert!(!stack.remove(40)); // Already evicted
    assert!(!stack.remove(43)); // Already evicted
    assert!(stack.remove(44));  // Still present
}

#[test]
fn test_voice_allocator_last_note_priority() {
    use grid_core::midi::MidiEvent;
    use grid_engine::voice::{MonophonicVoiceAllocator, NotePriority, PlayMode, VoiceModeConfig};

    let mut allocator = MonophonicVoiceAllocator::new(VoiceModeConfig {
        priority: NotePriority::LastNote,
        play_mode: PlayMode::Legato,
        pitch_bend_range_semitones: 2.0,
    });

    // 1. Appui Note 60
    allocator.process_event(&MidiEvent::note_on(0, 60, 100));
    assert_eq!(allocator.state().gate, 1.0);
    assert!((allocator.state().pitch_cv - 0.0).abs() < 1e-5); // C4 = 0.0V
    assert!(allocator.state().is_gate_triggered);

    // 2. Appui Note 64 (Legato : Gate reste a 1.0, Pitch passe a E4, trigger false)
    allocator.process_event(&MidiEvent::note_on(0, 64, 90));
    assert_eq!(allocator.state().gate, 1.0);
    assert!((allocator.state().pitch_cv - (4.0 / 12.0)).abs() < 1e-5);
    assert!(!allocator.state().is_gate_triggered); // Legato: pas de re-trigger

    // 3. Relache Note 64 -> retourne a Note 60, Gate reste a 1.0
    allocator.process_event(&MidiEvent::note_off(0, 64, 0));
    assert_eq!(allocator.state().gate, 1.0);
    assert!((allocator.state().pitch_cv - 0.0).abs() < 1e-5);
    assert!(!allocator.state().is_gate_triggered);

    // 4. Relache Note 60 -> Gate passe a 0.0
    allocator.process_event(&MidiEvent::note_off(0, 60, 0));
    assert_eq!(allocator.state().gate, 0.0);
}

#[test]
fn test_voice_allocator_low_note_priority() {
    use grid_core::midi::MidiEvent;
    use grid_engine::voice::{MonophonicVoiceAllocator, NotePriority, VoiceModeConfig};

    let mut allocator = MonophonicVoiceAllocator::new(VoiceModeConfig {
        priority: NotePriority::LowNote,
        ..Default::default()
    });

    // Appuyer Note 60, puis Note 67 -> Le Pitch reste sur 60 (la plus basse)
    allocator.process_event(&MidiEvent::note_on(0, 60, 100));
    allocator.process_event(&MidiEvent::note_on(0, 67, 100));
    assert!((allocator.state().pitch_cv - 0.0).abs() < 1e-5);

    // Appuyer Note 48 -> Le Pitch bascule sur 48 (-1.0V)
    allocator.process_event(&MidiEvent::note_on(0, 48, 100));
    assert!((allocator.state().pitch_cv - (-1.0)).abs() < 1e-5);

    // Relacher Note 48 -> Le Pitch remonte a 60
    allocator.process_event(&MidiEvent::note_off(0, 48, 0));
    assert!((allocator.state().pitch_cv - 0.0).abs() < 1e-5);

    // Relacher Note 60 -> Le Pitch remonte a 67 (+7/12 V)
    allocator.process_event(&MidiEvent::note_off(0, 60, 0));
    assert!((allocator.state().pitch_cv - (7.0 / 12.0)).abs() < 1e-5);
    assert_eq!(allocator.state().gate, 1.0);

    // Relacher Note 67 -> Gate = 0.0
    allocator.process_event(&MidiEvent::note_off(0, 67, 0));
    assert_eq!(allocator.state().gate, 0.0);
}

#[test]
fn test_voice_allocator_high_note_priority() {
    use grid_core::midi::MidiEvent;
    use grid_engine::voice::{MonophonicVoiceAllocator, NotePriority, VoiceModeConfig};

    let mut allocator = MonophonicVoiceAllocator::new(VoiceModeConfig {
        priority: NotePriority::HighNote,
        ..Default::default()
    });

    // Appuyer Note 60, puis Note 48 -> Le Pitch reste sur 60 (la plus aigue)
    allocator.process_event(&MidiEvent::note_on(0, 60, 100));
    allocator.process_event(&MidiEvent::note_on(0, 48, 100));
    assert!((allocator.state().pitch_cv - 0.0).abs() < 1e-5);

    // Appuyer Note 72 -> Le Pitch passe a 72 (+1.0V)
    allocator.process_event(&MidiEvent::note_on(0, 72, 100));
    assert!((allocator.state().pitch_cv - 1.0).abs() < 1e-5);

    // Relacher Note 72 -> Le Pitch redescend a 60
    allocator.process_event(&MidiEvent::note_off(0, 72, 0));
    assert!((allocator.state().pitch_cv - 0.0).abs() < 1e-5);
}

#[test]
fn test_voice_allocator_retrigger_mode() {
    use grid_core::midi::MidiEvent;
    use grid_engine::voice::{MonophonicVoiceAllocator, PlayMode, VoiceModeConfig};

    let mut allocator = MonophonicVoiceAllocator::new(VoiceModeConfig {
        play_mode: PlayMode::Retrigger,
        ..Default::default()
    });

    // Premiere note
    allocator.process_event(&MidiEvent::note_on(0, 60, 100));
    assert!(allocator.state().is_gate_triggered);

    // Seconde note en mode Retrigger : is_gate_triggered doit etre true
    allocator.process_event(&MidiEvent::note_on(0, 64, 100));
    assert_eq!(allocator.state().gate, 1.0);
    assert!(allocator.state().is_gate_triggered);
}

#[test]
fn test_voice_allocator_all_notes_off() {
    use grid_core::midi::MidiEvent;
    use grid_engine::voice::MonophonicVoiceAllocator;

    let mut allocator = MonophonicVoiceAllocator::default();
    allocator.process_event(&MidiEvent::note_on(0, 60, 100));
    allocator.process_event(&MidiEvent::note_on(0, 64, 100));
    assert_eq!(allocator.state().gate, 1.0);
    assert_eq!(allocator.stack().len(), 2);

    // AllNotesOff
    allocator.process_event(&MidiEvent::all_notes_off(0));
    assert_eq!(allocator.state().gate, 0.0);
    assert!(allocator.stack().is_empty());
    assert!(!allocator.state().is_gate_triggered);
}

#[test]
fn test_voice_allocator_pitch_bend_and_mod_wheel() {
    use grid_core::midi::MidiEvent;
    use grid_engine::voice::MonophonicVoiceAllocator;

    let mut allocator = MonophonicVoiceAllocator::default();
    allocator.process_event(&MidiEvent::note_on(0, 60, 100)); // C4 = 0.0V

    // ModWheel (CC 1)
    allocator.process_event(&MidiEvent::control_change(0, 1, 127));
    assert_eq!(allocator.state().mod_wheel, 1.0);

    allocator.process_event(&MidiEvent::control_change(0, 1, 0));
    assert_eq!(allocator.state().mod_wheel, 0.0);

    allocator.process_event(&MidiEvent::control_change(0, 1, 64));
    assert!((allocator.state().mod_wheel - (64.0 / 127.0)).abs() < 1e-5);

    // Un autre controleur (ex. volume CC 7) ne doit pas alterer la molette
    allocator.process_event(&MidiEvent::control_change(0, 7, 100));
    assert!((allocator.state().mod_wheel - (64.0 / 127.0)).abs() < 1e-5);

    // Pitch Bend neutre (0)
    allocator.process_event(&MidiEvent::pitch_bend(0, 0));
    assert_eq!(allocator.state().pitch_bend, 0.0);
    assert_eq!(allocator.state().total_pitch_cv(2.0), 0.0);

    // Pitch Bend maximum vers le haut (+8191)
    allocator.process_event(&MidiEvent::pitch_bend(0, 8192));
    assert_eq!(allocator.state().pitch_bend, 1.0);
    // Plage par defaut 2 demi-tons : 1.0 * 2 / 12 = 1/6 V
    assert!((allocator.state().total_pitch_cv(2.0) - (2.0 / 12.0)).abs() < 1e-5);
    // Plage 12 demi-tons (1 octave) : 1.0 * 12 / 12 = 1.0 V
    assert!((allocator.state().total_pitch_cv(12.0) - 1.0).abs() < 1e-5);

    // Pitch Bend minimum vers le bas (-8192)
    allocator.process_event(&MidiEvent::pitch_bend(0, -8192));
    assert_eq!(allocator.state().pitch_bend, -1.0);
    assert!((allocator.state().total_pitch_cv(2.0) - (-2.0 / 12.0)).abs() < 1e-5);
}

#[test]
fn test_voice_allocator_velocity_normalization_and_trigger_consumption() {
    use grid_core::midi::MidiEvent;
    use grid_engine::voice::MonophonicVoiceAllocator;

    let mut allocator = MonophonicVoiceAllocator::default();

    allocator.process_event(&MidiEvent::note_on(0, 69, 127)); // A4
    assert_eq!(allocator.state().velocity, 1.0);
    assert!((allocator.state().pitch_cv - (9.0 / 12.0)).abs() < 1e-5);

    // Consommation du trigger
    assert!(allocator.consume_gate_trigger());
    assert!(!allocator.consume_gate_trigger());
    assert!(!allocator.state().is_gate_triggered);
}

#[test]
fn test_voice_allocator_dynamic_config_change() {
    use grid_core::midi::MidiEvent;
    use grid_engine::voice::{MonophonicVoiceAllocator, NotePriority, VoiceModeConfig};

    let mut allocator = MonophonicVoiceAllocator::default(); // LastNote par defaut

    // Appuyer Note 60, puis Note 72
    allocator.process_event(&MidiEvent::note_on(0, 60, 100));
    allocator.process_event(&MidiEvent::note_on(0, 72, 100));
    assert!((allocator.state().pitch_cv - 1.0).abs() < 1e-5); // Note 72

    // Changer dynamiquement la priorite pour LowNote
    allocator.set_config(VoiceModeConfig {
        priority: NotePriority::LowNote,
        ..Default::default()
    });
    // L'allocateur doit basculer instantanement vers la note 60 (la plus basse)
    assert!((allocator.state().pitch_cv - 0.0).abs() < 1e-5);
}


