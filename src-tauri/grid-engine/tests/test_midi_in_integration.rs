use grid_core::compiler::PatchCompiler;
use grid_core::message::AudioCommand;
use grid_core::midi::MidiEvent;
use grid_core::model::{Cable, ModuleId, ModuleInstance, ModuleKind, Patch, PortId, Position};
use grid_engine::dsp::{AdsrEnvelope, EnvelopeStage, Oscillator};
use grid_engine::realtime::RealtimeAudioProcessor;
use grid_engine::ringbuffer::create_channels;

#[test]
fn test_midi_in_oscillator_1v_per_octave_response() {
    let sample_rate = 48000.0;
    let base_freq = 440.0;
    let mut osc = Oscillator::new(sample_rate);
    osc.set_frequency(base_freq);

    // 1. CV = 0.0 V (C4 -> f0 = 440 Hz)
    let mut buf_0v = [0.0f32; 4800];
    let cv_0v = [0.0f32; 4800];
    osc.process_block(&mut buf_0v, Some(&cv_0v));

    // Compter les passages par zero pour estimer la frequence
    let zero_crossings =
        |buf: &[f32]| -> usize { buf.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count() };

    let cycles_0v = zero_crossings(&buf_0v);

    // 2. CV = +1.0 V (C5 -> 2 * f0 = 880 Hz, octave superieure)
    osc.phase = 0.0;
    let mut buf_p1v = [0.0f32; 4800];
    let cv_p1v = [1.0f32; 4800];
    osc.process_block(&mut buf_p1v, Some(&cv_p1v));
    let cycles_p1v = zero_crossings(&buf_p1v);

    // 3. CV = -1.0 V (C3 -> f0 / 2 = 220 Hz, octave inferieure)
    osc.phase = 0.0;
    let mut buf_m1v = [0.0f32; 4800];
    let cv_m1v = [-1.0f32; 4800];
    osc.process_block(&mut buf_m1v, Some(&cv_m1v));
    let cycles_m1v = zero_crossings(&buf_m1v);

    // Dans 4800 echantillons a 48000 Hz (0.1 sec) :
    // 440 Hz attend ~44 cycles
    // 880 Hz attend ~88 cycles
    // 220 Hz attend ~22 cycles
    assert!((cycles_0v as i32 - 44).abs() <= 1);
    assert!((cycles_p1v as i32 - 88).abs() <= 1);
    assert!((cycles_m1v as i32 - 22).abs() <= 1);
}

#[test]
fn test_midi_in_gate_to_adsr_envelope() {
    let sample_rate = 48000.0;
    let mut env = AdsrEnvelope::new(sample_rate);
    env.set_attack(10.0);
    env.set_decay(20.0);
    env.set_sustain(0.5);
    env.set_release(50.0);

    // Au repos, Gate = 0.0 -> Idle
    let out_idle = env.process_sample(0.0);
    assert_eq!(out_idle, 0.0);
    assert_eq!(env.stage, EnvelopeStage::Idle);

    // Front montant Gate = 1.0 -> Attack
    let _ = env.process_sample(1.0);
    assert_eq!(env.stage, EnvelopeStage::Attack);

    // Laisser progresser l'attaque
    for _ in 0..1000 {
        env.process_sample(1.0);
    }
    assert!(env.stage == EnvelopeStage::Attack || env.stage == EnvelopeStage::Decay);

    // Front descendant Gate = 0.0 -> Release
    env.process_sample(0.0);
    assert_eq!(env.stage, EnvelopeStage::Release);
}

#[test]
fn test_realtime_processor_midi_in_patch_end_to_end() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    // Construire le patch MidiIn -> Oscillator -> AudioOut
    let mut patch = Patch::new("End To End Midi Synth");

    let midi_in = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::MidiIn,
        "MIDI In",
        Position { x: 0.0, y: 0.0 },
    );
    let osc = ModuleInstance::new(
        ModuleId(2),
        ModuleKind::Oscillator,
        "VCO",
        Position { x: 100.0, y: 0.0 },
    );
    let audio_out = ModuleInstance::new(
        ModuleId(3),
        ModuleKind::AudioOut,
        "Audio Out",
        Position { x: 200.0, y: 0.0 },
    );

    patch.modules.extend(vec![midi_in, osc, audio_out]);

    // Cable 1: MidiIn Pitch (port 0) -> Oscillator Freq CV (port 0)
    patch.cables.push(Cable {
        id: 1,
        from_module: ModuleId(1),
        from_port: PortId(0),
        to_module: ModuleId(2),
        to_port: PortId(0),
    });

    // Cable 2: Oscillator Out (port 1) -> AudioOut Left (port 0)
    patch.cables.push(Cable {
        id: 2,
        from_module: ModuleId(2),
        from_port: PortId(1),
        to_module: ModuleId(3),
        to_port: PortId(0),
    });

    // Cable 3: Oscillator Out (port 1) -> AudioOut Right (port 1)
    patch.cables.push(Cable {
        id: 3,
        from_module: ModuleId(2),
        from_port: PortId(1),
        to_module: ModuleId(3),
        to_port: PortId(1),
    });

    let plan = PatchCompiler::compile(&patch).expect("Compilation must succeed");
    assert_eq!(plan.step_count(), 3);

    host.command_producer
        .push(AudioCommand::LoadExecutionPlan(plan))
        .expect("Load plan must succeed");
    host.command_producer
        .push(AudioCommand::SetPlaying(true))
        .expect("Play command must succeed");
    host.command_producer
        .push(AudioCommand::SetMasterGain(1.0))
        .expect("Set gain must succeed");

    // 1. Jouer C4 (note 60) -> Pitch CV 0.0 V -> Fréquence 440 Hz
    host.midi_producer
        .try_push(MidiEvent::note_on(0, 60, 100))
        .expect("Push NoteOn C4 must succeed");

    let mut out_buffer_c4 = [0.0f32; 128]; // 64 frames stereo
    processor.process_interleaved_buffer(&mut out_buffer_c4);

    // Verifier que l'audio n'est pas silencieux
    let max_amp_c4 = out_buffer_c4
        .iter()
        .fold(0.0f32, |acc, &v| acc.max(v.abs()));
    assert!(max_amp_c4 > 0.1, "Oscillator must generate audio for C4");

    // 2. Jouer C5 (note 72) -> Pitch CV +1.0 V -> Frequence 880 Hz
    host.midi_producer
        .try_push(MidiEvent::note_on(0, 72, 110))
        .expect("Push NoteOn C5 must succeed");

    let mut out_buffer_c5 = [0.0f32; 128];
    processor.process_interleaved_buffer(&mut out_buffer_c5);

    let max_amp_c5 = out_buffer_c5
        .iter()
        .fold(0.0f32, |acc, &v| acc.max(v.abs()));
    assert!(max_amp_c5 > 0.1, "Oscillator must generate audio for C5");

    // 3. AllNotesOff -> Gate a 0.0
    host.midi_producer
        .try_push(MidiEvent::all_notes_off(0))
        .expect("Push AllNotesOff must succeed");

    processor.process_interleaved_buffer(&mut out_buffer_c5);
    assert_eq!(processor.voice_allocator.state().gate, 0.0);
}
