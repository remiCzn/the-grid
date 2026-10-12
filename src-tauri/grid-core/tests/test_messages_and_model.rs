use grid_core::compiler::{compile_patch, ExecutionPlan};
use grid_core::graph::{AudioGraph, GraphError};
use grid_core::message::{AudioCommand, TelemetryFrame};
use grid_core::model::{
    Cable, ModuleId, ModuleInstance, ModuleKind, Patch, PortId, Position, SignalType,
};

#[test]
fn test_audio_command_serialization() {
    let cmd = AudioCommand::SetMasterGain(0.75);
    let serialized = serde_json::to_string(&cmd).expect("Serialization failed");
    let deserialized: AudioCommand =
        serde_json::from_str(&serialized).expect("Deserialization failed");

    assert_eq!(cmd, deserialized);

    let param_cmd = AudioCommand::SetParameter {
        module_id: 1,
        param_id: 2,
        value: 440.0,
    };
    let json_param = serde_json::to_string(&param_cmd).unwrap();
    let des_param: AudioCommand = serde_json::from_str(&json_param).unwrap();
    assert_eq!(param_cmd, des_param);

    let plan_cmd = AudioCommand::LoadExecutionPlan(ExecutionPlan::default());
    let json_plan = serde_json::to_string(&plan_cmd).unwrap();
    let des_plan: AudioCommand = serde_json::from_str(&json_plan).unwrap();
    assert_eq!(plan_cmd, des_plan);
}

#[test]
fn test_telemetry_frame_defaults_and_serialization() {
    let def_frame = TelemetryFrame::default();
    assert_eq!(def_frame.frame_index, 0);
    assert_eq!(def_frame.peak_left, 0.0);
    assert_eq!(def_frame.peak_right, 0.0);
    assert_eq!(def_frame.rms_left, 0.0);
    assert_eq!(def_frame.rms_right, 0.0);

    let frame = TelemetryFrame {
        frame_index: 42,
        peak_left: 0.85,
        peak_right: 0.90,
        rms_left: 0.45,
        rms_right: 0.50,
    };
    let json = serde_json::to_string(&frame).expect("Serialization failed");
    let parsed: TelemetryFrame = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(frame, parsed);
}

#[test]
fn test_module_factories_creation() {
    let osc = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::Oscillator,
        "VCO",
        Position { x: 0.0, y: 0.0 },
    );
    assert_eq!(osc.kind, ModuleKind::Oscillator);
    assert_eq!(osc.ports.len(), 2);
    assert_eq!(osc.parameters.len(), 1);

    let gain = ModuleInstance::new(
        ModuleId(2),
        ModuleKind::Gain,
        "VCA",
        Position { x: 100.0, y: 0.0 },
    );
    assert_eq!(gain.kind, ModuleKind::Gain);
    assert_eq!(gain.ports.len(), 3);

    let filter = ModuleInstance::new(
        ModuleId(3),
        ModuleKind::Filter,
        "VCF",
        Position { x: 200.0, y: 0.0 },
    );
    assert_eq!(filter.kind, ModuleKind::Filter);
    assert_eq!(filter.ports.len(), 3);
    assert_eq!(filter.parameters.len(), 2);

    let env = ModuleInstance::new(
        ModuleId(4),
        ModuleKind::Envelope,
        "ADSR",
        Position { x: 0.0, y: 100.0 },
    );
    assert_eq!(env.kind, ModuleKind::Envelope);
    assert_eq!(env.ports.len(), 2);
    assert_eq!(env.parameters.len(), 4);

    let cst = ModuleInstance::new(
        ModuleId(5),
        ModuleKind::Constant,
        "Bias",
        Position { x: 50.0, y: 50.0 },
    );
    assert_eq!(cst.kind, ModuleKind::Constant);
    assert_eq!(cst.ports.len(), 1);

    let out = ModuleInstance::new(
        ModuleId(6),
        ModuleKind::AudioOut,
        "Master",
        Position { x: 300.0, y: 0.0 },
    );
    assert_eq!(out.kind, ModuleKind::AudioOut);
    assert_eq!(out.ports.len(), 2);
}

#[test]
fn test_patch_topological_compilation() {
    let mut patch = Patch::new("Test Patch");
    let osc = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::Oscillator,
        "VCO",
        Position { x: 0.0, y: 0.0 },
    );
    let gain = ModuleInstance::new(
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

    patch.modules.extend(vec![osc, gain, out]);

    patch.cables.push(Cable {
        id: 1,
        from_module: ModuleId(1),
        from_port: PortId(1),
        to_module: ModuleId(2),
        to_port: PortId(0),
    });

    patch.cables.push(Cable {
        id: 2,
        from_module: ModuleId(2),
        from_port: PortId(2),
        to_module: ModuleId(3),
        to_port: PortId(0),
    });

    let graph = AudioGraph::from_patch(&patch).expect("Graph must be valid");

    assert_eq!(
        graph.execution_order,
        vec![ModuleId(1), ModuleId(2), ModuleId(3)]
    );

    let plan = compile_patch(&graph, &patch).expect("Compilation must succeed");
    assert_eq!(plan.step_count(), 3);
    assert_eq!(plan.required_scratch_buffers, 2);
}

#[test]
fn test_patch_serialization_and_deserialization() {
    let mut patch = Patch::new("Full Modular Patch");
    let osc = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::Oscillator,
        "VCO 1",
        Position { x: 50.0, y: 100.0 },
    );
    let filter = ModuleInstance::new(
        ModuleId(2),
        ModuleKind::Filter,
        "SVF 1",
        Position { x: 250.0, y: 100.0 },
    );
    let env = ModuleInstance::new(
        ModuleId(3),
        ModuleKind::Envelope,
        "ADSR 1",
        Position { x: 50.0, y: 300.0 },
    );
    let out = ModuleInstance::new(
        ModuleId(4),
        ModuleKind::AudioOut,
        "Out",
        Position { x: 450.0, y: 100.0 },
    );

    patch.modules.extend(vec![osc, filter, env, out]);
    patch.cables.push(Cable {
        id: 1,
        from_module: ModuleId(1),
        from_port: PortId(1),
        to_module: ModuleId(2),
        to_port: PortId(0),
    });

    let json = serde_json::to_string_pretty(&patch).expect("Export failed");
    assert!(json.contains("Full Modular Patch"));
    assert!(json.contains("VCO 1"));
    assert!(json.contains("SVF 1"));

    let imported: Patch = serde_json::from_str(&json).expect("Import failed");
    assert_eq!(imported.name, patch.name);
    assert_eq!(imported.modules.len(), 4);
    assert_eq!(imported.cables.len(), 1);
}

#[test]
fn test_graph_cycle_detection() {
    let mut patch = Patch::new("Cycle Patch");
    let mod_a = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::Gain,
        "Gain A",
        Position { x: 0.0, y: 0.0 },
    );
    let mod_b = ModuleInstance::new(
        ModuleId(2),
        ModuleKind::Gain,
        "Gain B",
        Position { x: 100.0, y: 0.0 },
    );

    patch.modules.extend(vec![mod_a, mod_b]);

    patch.cables.push(Cable {
        id: 1,
        from_module: ModuleId(1),
        from_port: PortId(2),
        to_module: ModuleId(2),
        to_port: PortId(0),
    });

    patch.cables.push(Cable {
        id: 2,
        from_module: ModuleId(2),
        from_port: PortId(2),
        to_module: ModuleId(1),
        to_port: PortId(0),
    });

    let res = AudioGraph::from_patch(&patch);
    assert_eq!(res.unwrap_err(), GraphError::CycleDetected);
}

#[test]
fn test_graph_validation_errors() {
    let mut patch = Patch::new("Invalid Patch");
    let osc = ModuleInstance::new(
        ModuleId(1),
        ModuleKind::Oscillator,
        "VCO",
        Position { x: 0.0, y: 0.0 },
    );
    patch.modules.push(osc);

    patch.cables.push(Cable {
        id: 1,
        from_module: ModuleId(1),
        from_port: PortId(1),
        to_module: ModuleId(99),
        to_port: PortId(0),
    });
    assert!(matches!(
        AudioGraph::from_patch(&patch).unwrap_err(),
        GraphError::ModuleNotFound(ModuleId(99))
    ));
    patch.cables.clear();

    patch.cables.push(Cable {
        id: 2,
        from_module: ModuleId(1),
        from_port: PortId(999),
        to_module: ModuleId(1),
        to_port: PortId(0),
    });
    assert!(matches!(
        AudioGraph::from_patch(&patch).unwrap_err(),
        GraphError::PortNotFound { .. }
    ));
}

#[test]
fn test_graph_error_display() {
    let err = GraphError::CycleDetected;
    assert_eq!(format!("{err}"), "Cycle non retardé détecté dans le graphe");

    let err_mod = GraphError::ModuleNotFound(ModuleId(42));
    assert_eq!(format!("{err_mod}"), "Module introuvable: ModuleId(42)");

    let err_port = GraphError::PortNotFound {
        module: ModuleId(1),
        port: 7,
    };
    assert_eq!(
        format!("{err_port}"),
        "Port introuvable sur le module ModuleId(1): 7"
    );

    let err_src = GraphError::InvalidSourcePort {
        module: ModuleId(1),
        port: 0,
    };
    assert_eq!(
        format!("{err_src}"),
        "Le port source 0 du module ModuleId(1) n'est pas une sortie"
    );

    let err_dst = GraphError::InvalidTargetPort {
        module: ModuleId(2),
        port: 1,
    };
    assert_eq!(
        format!("{err_dst}"),
        "Le port cible 1 du module ModuleId(2) n'est pas une entrée"
    );

    let err_sig = GraphError::IncompatibleSignalType {
        src: SignalType::Trigger,
        dst: SignalType::Audio,
    };
    assert_eq!(
        format!("{err_sig}"),
        "Incompatibilité de signaux entre source (Trigger) et cible (Audio)"
    );
}
