use grid_engine::dsp::MidiInNode;
use grid_engine::voice::VoiceState;

#[test]
fn test_midi_in_node_default_state() {
    let mut node = MidiInNode::new();
    assert_eq!(node, MidiInNode::default());
    let mut out_pitch = [999.0f32; 64];
    let mut out_gate = [999.0f32; 64];
    let mut out_vel = [999.0f32; 64];
    let mut out_mod = [999.0f32; 64];
    let mut out_bend = [999.0f32; 64];

    let mut outputs: [&mut [f32]; 5] = [
        &mut out_pitch,
        &mut out_gate,
        &mut out_vel,
        &mut out_mod,
        &mut out_bend,
    ];

    node.process_block(&mut outputs);

    assert!(out_pitch.iter().all(|&v| v == 0.0));
    assert!(out_gate.iter().all(|&v| v == 0.0));
    assert!(out_vel.iter().all(|&v| v == 0.0));
    assert!(out_mod.iter().all(|&v| v == 0.0));
    assert!(out_bend.iter().all(|&v| v == 0.0));
}

#[test]
fn test_midi_in_node_process_block_from_voice_state() {
    let mut node = MidiInNode::default();

    // Simuler l'etat de voix pour note C5 (72), Gate ON, vel 0.75, mod 0.5, bend -0.2
    let state = VoiceState {
        pitch_cv: 1.0, // (72 - 60) / 12 = 1.0 V
        gate: 1.0,
        velocity: 0.75,
        mod_wheel: 0.5,
        pitch_bend: -0.2,
        is_gate_triggered: true,
    };

    node.update_from_voice_state(&state);

    let mut out_pitch = [0.0f32; 64];
    let mut out_gate = [0.0f32; 64];
    let mut out_vel = [0.0f32; 64];
    let mut out_mod = [0.0f32; 64];
    let mut out_bend = [0.0f32; 64];

    let mut outputs: [&mut [f32]; 5] = [
        &mut out_pitch,
        &mut out_gate,
        &mut out_vel,
        &mut out_mod,
        &mut out_bend,
    ];

    node.process_block(&mut outputs);

    assert!(out_pitch.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    assert!(out_gate.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    assert!(out_vel.iter().all(|&v| (v - 0.75).abs() < 1e-6));
    assert!(out_mod.iter().all(|&v| (v - 0.5).abs() < 1e-6));
    assert!(out_bend.iter().all(|&v| (v - (-0.2)).abs() < 1e-6));
}

#[test]
fn test_midi_in_node_partial_outputs() {
    let mut node = MidiInNode::default();
    let state = VoiceState {
        pitch_cv: -1.0, // C3 (48) -> -1.0 V
        gate: 1.0,
        velocity: 0.9,
        mod_wheel: 0.0,
        pitch_bend: 0.0,
        is_gate_triggered: false,
    };
    node.update_from_voice_state(&state);

    // Seulement 2 sorties fournies (Pitch et Gate)
    let mut out_pitch = [0.0f32; 64];
    let mut out_gate = [0.0f32; 64];
    let mut outputs: [&mut [f32]; 2] = [&mut out_pitch, &mut out_gate];

    node.process_block(&mut outputs);

    assert!(out_pitch.iter().all(|&v| (v - (-1.0)).abs() < 1e-6));
    assert!(out_gate.iter().all(|&v| (v - 1.0).abs() < 1e-6));
}

#[test]
fn test_midi_in_node_reset() {
    let mut node = MidiInNode::default();
    node.update_from_voice_state(&VoiceState {
        pitch_cv: 1.5,
        gate: 1.0,
        velocity: 0.8,
        mod_wheel: 0.7,
        pitch_bend: 0.5,
        is_gate_triggered: false,
    });

    node.reset();

    let mut out_pitch = [1.0f32; 32];
    let mut out_gate = [1.0f32; 32];
    let mut outputs: [&mut [f32]; 2] = [&mut out_pitch, &mut out_gate];
    node.process_block(&mut outputs);

    assert!(out_pitch.iter().all(|&v| v == 0.0));
    assert!(out_gate.iter().all(|&v| v == 0.0));
}
