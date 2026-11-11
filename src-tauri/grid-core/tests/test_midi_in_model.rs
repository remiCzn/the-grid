use grid_core::compiler::PatchCompiler;
use grid_core::model::{
    Cable, ModuleId, ModuleInstance, ModuleKind, Patch, PortDirection, PortId, Position, SignalType,
};

#[test]
fn test_midi_in_module_factory_ports() {
    let midi_mod = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::MidiIn,
        "MIDI In",
        Position { x: 0.0, y: 0.0 },
    );

    assert_eq!(midi_mod.kind, ModuleKind::MidiIn);
    assert_eq!(midi_mod.ports.len(), 5);

    // Port 0: Pitch (CV, 1 V/oct)
    assert_eq!(midi_mod.ports[0].id, PortId(0));
    assert_eq!(midi_mod.ports[0].name, "Pitch");
    assert_eq!(midi_mod.ports[0].signal_type, SignalType::ControlVoltage);
    assert_eq!(midi_mod.ports[0].direction, PortDirection::Output);

    // Port 1: Gate (Trigger, 0.0/1.0)
    assert_eq!(midi_mod.ports[1].id, PortId(1));
    assert_eq!(midi_mod.ports[1].name, "Gate");
    assert_eq!(midi_mod.ports[1].signal_type, SignalType::Trigger);
    assert_eq!(midi_mod.ports[1].direction, PortDirection::Output);

    // Port 2: Velocity (CV, 0.0..1.0)
    assert_eq!(midi_mod.ports[2].id, PortId(2));
    assert_eq!(midi_mod.ports[2].name, "Velocity");
    assert_eq!(midi_mod.ports[2].signal_type, SignalType::ControlVoltage);
    assert_eq!(midi_mod.ports[2].direction, PortDirection::Output);

    // Port 3: Mod Wheel (CV, 0.0..1.0)
    assert_eq!(midi_mod.ports[3].id, PortId(3));
    assert_eq!(midi_mod.ports[3].name, "Mod Wheel");
    assert_eq!(midi_mod.ports[3].signal_type, SignalType::ControlVoltage);
    assert_eq!(midi_mod.ports[3].direction, PortDirection::Output);

    // Port 4: Pitch Bend (CV, -1.0..+1.0)
    assert_eq!(midi_mod.ports[4].id, PortId(4));
    assert_eq!(midi_mod.ports[4].name, "Pitch Bend");
    assert_eq!(midi_mod.ports[4].signal_type, SignalType::ControlVoltage);
    assert_eq!(midi_mod.ports[4].direction, PortDirection::Output);
}

#[test]
fn test_midi_in_topological_compilation() {
    let mut patch = Patch::new("Midi Synth Patch");

    let midi_in = ModuleInstance::new(
        ModuleId(10),
        ModuleKind::MidiIn,
        "MIDI In",
        Position { x: 0.0, y: 0.0 },
    );
    let osc = ModuleInstance::new(
        ModuleId(20),
        ModuleKind::Oscillator,
        "VCO",
        Position { x: 100.0, y: 0.0 },
    );
    let audio_out = ModuleInstance::new(
        ModuleId(30),
        ModuleKind::AudioOut,
        "Master Out",
        Position { x: 200.0, y: 0.0 },
    );

    patch.modules.extend(vec![midi_in, osc, audio_out]);

    // Cable 1: MidiIn Pitch (out port 0) -> Oscillator Freq CV (in port 0)
    patch.cables.push(Cable {
        id: 1,
        from_module: ModuleId(10),
        from_port: PortId(0),
        to_module: ModuleId(20),
        to_port: PortId(0),
    });

    // Cable 2: Oscillator Out (out port 1) -> AudioOut Left (in port 0)
    patch.cables.push(Cable {
        id: 2,
        from_module: ModuleId(20),
        from_port: PortId(1),
        to_module: ModuleId(30),
        to_port: PortId(0),
    });

    let plan = PatchCompiler::compile(&patch).expect("Compilation must succeed");
    assert_eq!(plan.step_count(), 3);

    // MidiIn must be scheduled first
    assert_eq!(plan.steps[0].module_id, ModuleId(10));
    assert_eq!(plan.steps[0].kind, ModuleKind::MidiIn);
    assert_eq!(plan.steps[1].module_id, ModuleId(20));
    assert_eq!(plan.steps[1].kind, ModuleKind::Oscillator);
    assert_eq!(plan.steps[2].module_id, ModuleId(30));
    assert_eq!(plan.steps[2].kind, ModuleKind::AudioOut);
}

#[test]
fn test_midi_in_patch_json_serialization_roundtrip() {
    let mut patch = Patch::new("Midi Serialization Patch");

    let midi_in = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::MidiIn,
        "MIDI In",
        Position { x: 10.0, y: 20.0 },
    );
    let env = ModuleInstance::new(
        ModuleId(2),
        ModuleKind::Envelope,
        "ADSR",
        Position { x: 120.0, y: 20.0 },
    );

    patch.modules.extend(vec![midi_in, env]);

    // Cable: MidiIn Gate (port 1) -> Envelope Gate (port 0)
    patch.cables.push(Cable {
        id: 1,
        from_module: ModuleId(1),
        from_port: PortId(1),
        to_module: ModuleId(2),
        to_port: PortId(0),
    });

    let serialized = serde_json::to_string(&patch).expect("Serialization failed");
    let deserialized: Patch = serde_json::from_str(&serialized).expect("Deserialization failed");

    assert_eq!(patch, deserialized);
    assert_eq!(deserialized.modules[0].kind, ModuleKind::MidiIn);
}
