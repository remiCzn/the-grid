use grid_core::compiler::{ExecutionPlan, PatchCompiler};
use grid_core::message::AudioCommand;
use grid_core::model::{Cable, ModuleId, ModuleInstance, ModuleKind, Patch, PortId, Position};
use grid_engine::realtime::RealtimeAudioProcessor;
use grid_engine::ringbuffer::{
    create_channels, COMMAND_QUEUE_CAPACITY, TELEMETRY_QUEUE_CAPACITY,
};

#[test]
fn test_spsc_command_queue_transfer() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    host.command_producer
        .push(AudioCommand::SetMasterGain(0.5))
        .expect("Push should succeed");
    host.command_producer
        .push(AudioCommand::SetParameter {
            module_id: 10,
            param_id: 2,
            value: 0.75,
        })
        .expect("Push should succeed");
    host.command_producer
        .push(AudioCommand::LoadExecutionPlan(ExecutionPlan {
            steps: vec![],
            required_scratch_buffers: 4,
        }))
        .expect("Push should succeed");
    host.command_producer
        .push(AudioCommand::SetPlaying(true))
        .expect("Push should succeed");

    let mut buffer = [1.0f32; 128];
    processor.process_interleaved_buffer(&mut buffer);

    assert!(processor.is_playing);
    assert_eq!(processor.master_gain, 0.5);
    assert!(processor.current_plan.is_some());

    // Test clamp du gain
    host.command_producer
        .push(AudioCommand::SetMasterGain(5.0))
        .expect("Push should succeed");
    let mut buffer2 = [0.0f32; 4];
    processor.process_interleaved_buffer(&mut buffer2);
    assert_eq!(processor.master_gain, 2.0);
}

#[test]
fn test_realtime_processor_active_sound_generation_and_parameter_update() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    // Créer un patch avec VCO 1 -> VCA 1 -> AudioOut
    let mut patch = Patch::new("Sound Patch");
    let osc = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::Oscillator,
        "VCO",
        Position { x: 0.0, y: 0.0 },
    );
    let vca = ModuleInstance::new(
        ModuleId(2),
        ModuleKind::Gain,
        "VCA",
        Position { x: 100.0, y: 0.0 },
    );
    let out = ModuleInstance::new(
        ModuleId(3),
        ModuleKind::AudioOut,
        "Master",
        Position { x: 200.0, y: 0.0 },
    );

    patch.modules.extend(vec![osc, vca, out]);

    // Câble 1: Osc Out (port 1) -> VCA In (port 0)
    patch.cables.push(Cable {
        id: 1,
        from_module: ModuleId(1),
        from_port: PortId(1),
        to_module: ModuleId(2),
        to_port: PortId(0),
    });

    // Câble 2: VCA Out (port 2) -> Master Left (port 0)
    patch.cables.push(Cable {
        id: 2,
        from_module: ModuleId(2),
        from_port: PortId(2),
        to_module: ModuleId(3),
        to_port: PortId(0),
    });

    let plan = PatchCompiler::compile(&patch).expect("Plan compilation");

    // Envoyer le plan et activer la lecture
    host.command_producer
        .push(AudioCommand::LoadExecutionPlan(plan))
        .expect("Load plan");
    host.command_producer
        .push(AudioCommand::SetPlaying(true))
        .expect("Set playing");
    host.command_producer
        .push(AudioCommand::SetMasterGain(1.0))
        .expect("Set gain");

    let mut buffer = [0.0f32; 128]; // 64 frames stéréo
    processor.process_interleaved_buffer(&mut buffer);

    let max_amp = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    assert!(
        max_amp > 0.1,
        "Le moteur audio doit générer un signal réel (max: {})",
        max_amp
    );

    // Mettre à jour la fréquence du VCO et le gain du VCA en direct via SetParameter
    host.command_producer
        .push(AudioCommand::SetParameter {
            module_id: 1,
            param_id: 0,
            value: 880.0,
        })
        .expect("Set freq");
    host.command_producer
        .push(AudioCommand::SetParameter {
            module_id: 2,
            param_id: 0,
            value: 0.5,
        })
        .expect("Set gain");
    host.command_producer
        .push(AudioCommand::SetParameter {
            module_id: 3,
            param_id: 0,
            value: 0.9,
        })
        .expect("Set master volume");

    let mut buffer2 = [0.0f32; 128];
    processor.process_interleaved_buffer(&mut buffer2);
    let max_amp2 = buffer2.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    assert!(max_amp2 > 0.05);
}

#[test]
fn test_command_queue_saturation_does_not_panic() {
    let (mut host, _engine) = create_channels();

    for _ in 0..COMMAND_QUEUE_CAPACITY {
        assert!(host
            .command_producer
            .push(AudioCommand::SetMasterGain(0.1))
            .is_ok());
    }

    let overflow = host.command_producer.push(AudioCommand::SetPlaying(true));
    assert!(overflow.is_err());
}

#[test]
fn test_realtime_processor_silence_when_stopped() {
    let (_host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    let mut buffer = [0.99f32; 256];
    processor.process_interleaved_buffer(&mut buffer);

    for sample in buffer.iter() {
        assert_eq!(*sample, 0.0);
    }
}

#[test]
fn test_telemetry_decimation_and_drop_on_full() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    processor.set_telemetry_interval(10);

    let mut buffer = [0.0f32; 80];
    processor.process_interleaved_buffer(&mut buffer);

    let mut frames_received = 0;
    while let Ok(frame) = host.telemetry_consumer.pop() {
        frames_received += 1;
        assert_eq!(frame.frame_index, frames_received as u64);
        assert_eq!(frame.peak_left, 0.0);
        assert_eq!(frame.peak_right, 0.0);
    }
    assert_eq!(frames_received, 4);

    for _ in 0..(TELEMETRY_QUEUE_CAPACITY * 2) {
        processor.process_interleaved_buffer(&mut buffer);
    }

    let mut total_drained = 0;
    while host.telemetry_consumer.pop().is_ok() {
        total_drained += 1;
    }
    assert_eq!(total_drained, TELEMETRY_QUEUE_CAPACITY);
}

#[test]
fn test_reset_command_clears_metrics() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    host.command_producer
        .push(AudioCommand::Reset)
        .expect("Push reset failed");

    let mut buffer = [0.0f32; 32];
    processor.process_interleaved_buffer(&mut buffer);

    assert_eq!(processor.master_gain, 0.8);
}
