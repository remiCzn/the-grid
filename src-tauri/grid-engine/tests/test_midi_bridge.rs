use grid_core::midi::MidiEvent;
use grid_engine::midi_bridge::{create_midi_bridge, MIDI_QUEUE_CAPACITY};
use grid_engine::realtime::RealtimeAudioProcessor;
use grid_engine::ringbuffer::create_channels;

#[test]
fn test_midi_producer_overflow_handling() {
    let (mut producer, mut consumer) = create_midi_bridge(MIDI_QUEUE_CAPACITY);

    assert_eq!(producer.slots(), MIDI_QUEUE_CAPACITY);
    assert!(!producer.is_full());

    // Remplir completement le ring buffer
    for i in 0..MIDI_QUEUE_CAPACITY {
        let event = MidiEvent::note_on(0, (i % 128) as u8, 100);
        let res = producer.try_push(event);
        assert!(res.is_ok(), "Pushing event {} should succeed", i);
    }

    assert_eq!(producer.slots(), 0);
    assert!(producer.is_full());

    // Tenter d'inserer un evenement supplementaire : doit echouer sans paniquer ni bloquer
    let overflow_event = MidiEvent::note_on(0, 60, 100);
    let overflow_result = producer.try_push(overflow_event);
    assert!(
        overflow_result.is_err(),
        "Pushing to a full queue must return Err"
    );

    // Consommer un element libere un slot
    let popped = consumer.pop().expect("Consumer should pop one element");
    assert_eq!(popped, MidiEvent::note_on(0, 0, 100));
    assert_eq!(producer.slots(), 1);
    assert!(!producer.is_full());

    // On peut a nouveau pusher
    assert!(producer.try_push(overflow_event).is_ok());
}

#[test]
fn test_midi_bridge_fifo_order_500_events() {
    let (mut producer, mut consumer) = create_midi_bridge(MIDI_QUEUE_CAPACITY);

    let sent_events: Vec<MidiEvent> = (0..500)
        .map(|i| {
            if i % 2 == 0 {
                MidiEvent::note_on((i % 16) as u8, (i % 128) as u8, ((i * 3) % 127 + 1) as u8)
            } else {
                MidiEvent::pitch_bend((i % 16) as u8, ((i * 30) % 8192) as i16)
            }

        })
        .collect();

    // Injection des 500 evenements
    for event in &sent_events {
        producer.try_push(*event).expect("Push should succeed");
    }

    // Verification de l'ordre FIFO strict a la reception
    let mut received_events = Vec::with_capacity(500);
    while let Ok(event) = consumer.pop() {
        received_events.push(event);
    }

    assert_eq!(received_events.len(), 500);
    assert_eq!(received_events, sent_events);
}

#[test]
fn test_realtime_processor_drains_midi_events() {
    let (mut host, engine) = create_channels();
    let mut processor = RealtimeAudioProcessor::new(engine, 48000.0);

    // Injecter 500 evenements MIDI via le producer hôte
    for i in 0..500 {
        let event = MidiEvent::note_on(0, (i % 128) as u8, 90);
        host.midi_producer
            .try_push(event)
            .expect("Push should succeed");
    }

    // Avant le cycle audio, les slots disponibles dans host sont reduits de 500
    assert_eq!(host.midi_producer.slots(), MIDI_QUEUE_CAPACITY - 500);

    // Executer un cycle audio
    let mut buffer = [0.0f32; 128];
    processor.process_interleaved_buffer(&mut buffer);

    // Le processeur a vide la file MIDI pendant le cycle audio
    assert_eq!(host.midi_producer.slots(), MIDI_QUEUE_CAPACITY);
}
