use grid_core::midi::{parse_midi_message, MidiEvent};
use std::mem::size_of;

#[test]
fn test_midi_event_memory_size_and_traits() {
    assert!(
        size_of::<MidiEvent>() <= 8,
        "MidiEvent footprint must be <= 8 bytes, got {}",
        size_of::<MidiEvent>()
    );

    let event = MidiEvent::NoteOn {
        channel: 0,
        note: 60,
        velocity: 100,
    };
    let copied = event;
    assert_eq!(event, copied);
}

#[test]
fn test_parse_note_on_and_note_off() {
    // NoteOn standard
    let raw_on = [0x90, 60, 100];
    let parsed_on = parse_midi_message(&raw_on);
    assert_eq!(
        parsed_on,
        Some(MidiEvent::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100,
        })
    );

    // NoteOn with velocity 0 must be parsed as NoteOff
    let raw_zero_vel = [0x90, 60, 0];
    let parsed_zero_vel = parse_midi_message(&raw_zero_vel);
    assert_eq!(
        parsed_zero_vel,
        Some(MidiEvent::NoteOff {
            channel: 0,
            note: 60,
            velocity: 0,
        })
    );

    // NoteOff standard
    let raw_off = [0x80, 60, 64];
    let parsed_off = parse_midi_message(&raw_off);
    assert_eq!(
        parsed_off,
        Some(MidiEvent::NoteOff {
            channel: 0,
            note: 60,
            velocity: 64,
        })
    );

    // Channel 5 NoteOn
    let raw_ch5 = [0x95, 72, 80];
    let parsed_ch5 = parse_midi_message(&raw_ch5);
    assert_eq!(
        parsed_ch5,
        Some(MidiEvent::NoteOn {
            channel: 5,
            note: 72,
            velocity: 80,
        })
    );
}

#[test]
fn test_parse_control_change_and_all_notes_off() {
    // ModWheel (CC 1) sur canal 0
    let raw_mod = [0xB0, 1, 127];
    assert_eq!(
        parse_midi_message(&raw_mod),
        Some(MidiEvent::ControlChange {
            channel: 0,
            controller: 1,
            value: 127,
        })
    );

    // CC standard sur canal 9
    let raw_cc = [0xB9, 7, 90];
    assert_eq!(
        parse_midi_message(&raw_cc),
        Some(MidiEvent::ControlChange {
            channel: 9,
            controller: 7,
            value: 90,
        })
    );

    // CC 120 (All Sound Off) converti en AllNotesOff
    let raw_cc120 = [0xB2, 120, 0];
    assert_eq!(
        parse_midi_message(&raw_cc120),
        Some(MidiEvent::AllNotesOff { channel: 2 })
    );

    // CC 123 (All Notes Off)
    let raw_cc123 = [0xBF, 123, 0];
    assert_eq!(
        parse_midi_message(&raw_cc123),
        Some(MidiEvent::AllNotesOff { channel: 15 })
    );
}

#[test]
fn test_parse_pitch_bend_14bit() {
    // Centre neutre : LSB=0x00, MSB=0x40 (0x2000 = 8192 -> 0)
    let raw_center = [0xE0, 0x00, 0x40];
    assert_eq!(
        parse_midi_message(&raw_center),
        Some(MidiEvent::PitchBend {
            channel: 0,
            value: 0,
        })
    );

    // Minimum : LSB=0x00, MSB=0x00 (0 -> -8192)
    let raw_min = [0xE3, 0x00, 0x00];
    assert_eq!(
        parse_midi_message(&raw_min),
        Some(MidiEvent::PitchBend {
            channel: 3,
            value: -8192,
        })
    );

    // Maximum : LSB=0x7F, MSB=0x7F (16383 -> 8191)
    let raw_max = [0xEA, 0x7F, 0x7F];
    assert_eq!(
        parse_midi_message(&raw_max),
        Some(MidiEvent::PitchBend {
            channel: 10,
            value: 8191,
        })
    );
}

#[test]
fn test_parse_invalid_or_incomplete_bytes() {
    assert_eq!(parse_midi_message(&[]), None);
    assert_eq!(parse_midi_message(&[0x90]), None);
    assert_eq!(parse_midi_message(&[0x90, 60]), None);
    assert_eq!(parse_midi_message(&[0x70, 60, 100]), None); // Pas un status byte (< 0x80)
    assert_eq!(parse_midi_message(&[0xFF, 0, 0]), None); // System Realtime
    assert_eq!(parse_midi_message(&[0xF0, 0, 0]), None); // SysEx
    assert_eq!(parse_midi_message(&[0x90, 0x80, 60]), None); // data1 non-standard (bit 7 = 1)
    assert_eq!(parse_midi_message(&[0x90, 60, 0x80]), None); // data2 non-standard (bit 7 = 1)
    assert_eq!(parse_midi_message(&[0x90, 60, 100, 0]), None); // Tranche trop longue avec octets superflus
    assert_eq!(parse_midi_message(&[0xC0, 10, 0]), None); // Program Change (non gere actuellement)
}


#[test]
fn test_midi_event_constructors_and_helpers() {
    let on = MidiEvent::note_on(1, 64, 120);
    assert_eq!(on.channel(), 1);
    assert_eq!(
        on,
        MidiEvent::NoteOn {
            channel: 1,
            note: 64,
            velocity: 120
        }
    );

    let off = MidiEvent::note_off(2, 64, 0);
    assert_eq!(off.channel(), 2);
    assert_eq!(
        off,
        MidiEvent::NoteOff {
            channel: 2,
            note: 64,
            velocity: 0
        }
    );

    let cc = MidiEvent::control_change(3, 1, 64);
    assert_eq!(cc.channel(), 3);
    assert_eq!(
        cc,
        MidiEvent::ControlChange {
            channel: 3,
            controller: 1,
            value: 64
        }
    );

    let pb = MidiEvent::pitch_bend(4, -500);
    assert_eq!(pb.channel(), 4);
    assert_eq!(pb, MidiEvent::PitchBend { channel: 4, value: -500 });

    let ano = MidiEvent::all_notes_off(5);
    assert_eq!(ano.channel(), 5);
    assert_eq!(ano, MidiEvent::AllNotesOff { channel: 5 });
}

#[test]
fn test_midi_event_serde_roundtrip() {
    let events = vec![
        MidiEvent::note_on(0, 60, 100),
        MidiEvent::note_off(0, 60, 0),
        MidiEvent::control_change(1, 1, 127),
        MidiEvent::pitch_bend(2, 2048),
        MidiEvent::all_notes_off(3),
    ];

    for event in events {
        let serialized = serde_json::to_string(&event).expect("Serialization failed");
        let deserialized: MidiEvent =
            serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(event, deserialized);
    }
}

