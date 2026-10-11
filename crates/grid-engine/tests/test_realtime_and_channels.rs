use grid_core::compiler::ExecutionPlan;
use grid_core::message::AudioCommand;
use grid_engine::realtime::RealtimeAudioProcessor;
use grid_engine::ringbuffer::{
    create_channels, COMMAND_QUEUE_CAPACITY, TELEMETRY_QUEUE_CAPACITY,
};

#[test]
fn test_spsc_command_queue_transfer() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    // Host pousse des commandes
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

    // Buffer audio stéréo de 128 échantillons (64 frames)
    let mut buffer = [1.0f32; 128];
    processor.process_interleaved_buffer(&mut buffer);

    assert!(processor.is_playing);
    assert_eq!(processor.master_gain, 0.5);
    assert!(processor.current_plan.is_some());
    assert_eq!(
        processor
            .current_plan
            .as_ref()
            .unwrap()
            .required_scratch_buffers,
        4
    );

    // Test clamp du gain (ex: valeur négative ou > 2.0)
    host.command_producer
        .push(AudioCommand::SetMasterGain(5.0))
        .expect("Push should succeed");
    let mut buffer2 = [0.0f32; 4];
    processor.process_interleaved_buffer(&mut buffer2);
    assert_eq!(processor.master_gain, 2.0);
}

#[test]
fn test_command_queue_saturation_does_not_panic() {
    let (mut host, _engine) = create_channels();

    // Remplir complètement la queue
    for _ in 0..COMMAND_QUEUE_CAPACITY {
        assert!(host
            .command_producer
            .push(AudioCommand::SetMasterGain(0.1))
            .is_ok());
    }

    // Le push suivant doit échouer proprement sans paniquer
    let overflow = host.command_producer.push(AudioCommand::SetPlaying(true));
    assert!(overflow.is_err());
}

#[test]
fn test_realtime_processor_silence_when_stopped() {
    let (_host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    let mut buffer = [0.99f32; 256];
    processor.process_interleaved_buffer(&mut buffer);

    // Quand non-playing, le buffer doit être forcé à zéro
    for sample in buffer.iter() {
        assert_eq!(*sample, 0.0);
    }
}

#[test]
fn test_realtime_processor_running_playback() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    host.command_producer
        .push(AudioCommand::SetPlaying(true))
        .expect("Push should succeed");
    host.command_producer
        .push(AudioCommand::SetMasterGain(1.0))
        .expect("Push should succeed");

    let mut buffer = [0.5f32; 64];
    processor.process_interleaved_buffer(&mut buffer);

    // Le buffer muet sous lecture est mis à 0.0 (en attente des générateurs DSP)
    for sample in buffer.iter() {
        assert_eq!(*sample, 0.0);
    }
}

#[test]
fn test_telemetry_decimation_and_drop_on_full() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    // Régler l'intervalle de télémétrie très bas pour le test (toutes les 10 frames)
    processor.set_telemetry_interval(10);

    // Rendu d'un bloc de 40 frames stéréo (80 floats)
    let mut buffer = [0.0f32; 80];
    processor.process_interleaved_buffer(&mut buffer);

    // On s'attend à recevoir 4 trames de télémétrie (40 / 10 = 4)
    let mut frames_received = 0;
    while let Ok(frame) = host.telemetry_consumer.pop() {
        frames_received += 1;
        assert_eq!(frame.frame_index, frames_received as u64);
        assert_eq!(frame.peak_left, 0.0);
        assert_eq!(frame.peak_right, 0.0);
    }
    assert_eq!(frames_received, 4);

    // Test de saturation de la télémétrie :
    // On génère plus de frames que la capacité du buffer sans que l'hôte ne dépile
    for _ in 0..(TELEMETRY_QUEUE_CAPACITY * 2) {
        processor.process_interleaved_buffer(&mut buffer);
    }

    // Le processeur audio n'a pas paniqué et l'hôte peut dépiler jusqu'à saturation
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
