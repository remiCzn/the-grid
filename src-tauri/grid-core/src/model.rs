use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParamId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    Audio,
    ControlVoltage,
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortDefinition {
    pub id: PortId,
    pub name: String,
    pub signal_type: SignalType,
    pub direction: PortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParameterDefinition {
    pub id: ParamId,
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub default_value: f32,
    pub current_value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleKind {
    Oscillator,
    Gain,
    Constant,
    Filter,
    Envelope,
    AudioOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleInstance {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub name: String,
    pub position: Position,
    pub ports: Vec<PortDefinition>,
    pub parameters: Vec<ParameterDefinition>,
}

impl ModuleInstance {
    /// Crée un module standard prêt à l'emploi selon son type (factory)
    pub fn new(id: ModuleId, kind: ModuleKind, name: &str, position: Position) -> Self {
        match kind {
            ModuleKind::Oscillator => Self {
                id,
                kind,
                name: name.to_string(),
                position,
                ports: vec![
                    PortDefinition {
                        id: PortId(0),
                        name: "Freq CV".to_string(),
                        signal_type: SignalType::ControlVoltage,
                        direction: PortDirection::Input,
                    },
                    PortDefinition {
                        id: PortId(1),
                        name: "Out".to_string(),
                        signal_type: SignalType::Audio,
                        direction: PortDirection::Output,
                    },
                ],
                parameters: vec![ParameterDefinition {
                    id: ParamId(0),
                    name: "Frequency".to_string(),
                    min: 20.0,
                    max: 20000.0,
                    default_value: 440.0,
                    current_value: 440.0,
                }],
            },
            ModuleKind::Gain => Self {
                id,
                kind,
                name: name.to_string(),
                position,
                ports: vec![
                    PortDefinition {
                        id: PortId(0),
                        name: "In".to_string(),
                        signal_type: SignalType::Audio,
                        direction: PortDirection::Input,
                    },
                    PortDefinition {
                        id: PortId(1),
                        name: "Gain CV".to_string(),
                        signal_type: SignalType::ControlVoltage,
                        direction: PortDirection::Input,
                    },
                    PortDefinition {
                        id: PortId(2),
                        name: "Out".to_string(),
                        signal_type: SignalType::Audio,
                        direction: PortDirection::Output,
                    },
                ],
                parameters: vec![ParameterDefinition {
                    id: ParamId(0),
                    name: "Gain".to_string(),
                    min: 0.0,
                    max: 2.0,
                    default_value: 1.0,
                    current_value: 1.0,
                }],
            },
            ModuleKind::Constant => Self {
                id,
                kind,
                name: name.to_string(),
                position,
                ports: vec![PortDefinition {
                    id: PortId(0),
                    name: "Out".to_string(),
                    signal_type: SignalType::ControlVoltage,
                    direction: PortDirection::Output,
                }],
                parameters: vec![ParameterDefinition {
                    id: ParamId(0),
                    name: "Value".to_string(),
                    min: -1.0,
                    max: 1.0,
                    default_value: 0.0,
                    current_value: 0.0,
                }],
            },
            ModuleKind::Filter => Self {
                id,
                kind,
                name: name.to_string(),
                position,
                ports: vec![
                    PortDefinition {
                        id: PortId(0),
                        name: "In".to_string(),
                        signal_type: SignalType::Audio,
                        direction: PortDirection::Input,
                    },
                    PortDefinition {
                        id: PortId(1),
                        name: "Cutoff CV".to_string(),
                        signal_type: SignalType::ControlVoltage,
                        direction: PortDirection::Input,
                    },
                    PortDefinition {
                        id: PortId(2),
                        name: "Out".to_string(),
                        signal_type: SignalType::Audio,
                        direction: PortDirection::Output,
                    },
                ],
                parameters: vec![
                    ParameterDefinition {
                        id: ParamId(0),
                        name: "Cutoff".to_string(),
                        min: 20.0,
                        max: 20000.0,
                        default_value: 1000.0,
                        current_value: 1000.0,
                    },
                    ParameterDefinition {
                        id: ParamId(1),
                        name: "Resonance".to_string(),
                        min: 0.1,
                        max: 10.0,
                        default_value: 0.707,
                        current_value: 0.707,
                    },
                ],
            },
            ModuleKind::Envelope => Self {
                id,
                kind,
                name: name.to_string(),
                position,
                ports: vec![
                    PortDefinition {
                        id: PortId(0),
                        name: "Gate".to_string(),
                        signal_type: SignalType::Trigger,
                        direction: PortDirection::Input,
                    },
                    PortDefinition {
                        id: PortId(1),
                        name: "Env Out".to_string(),
                        signal_type: SignalType::ControlVoltage,
                        direction: PortDirection::Output,
                    },
                ],
                parameters: vec![
                    ParameterDefinition {
                        id: ParamId(0),
                        name: "Attack".to_string(),
                        min: 1.0,
                        max: 2000.0,
                        default_value: 10.0,
                        current_value: 10.0,
                    },
                    ParameterDefinition {
                        id: ParamId(1),
                        name: "Decay".to_string(),
                        min: 1.0,
                        max: 2000.0,
                        default_value: 100.0,
                        current_value: 100.0,
                    },
                    ParameterDefinition {
                        id: ParamId(2),
                        name: "Sustain".to_string(),
                        min: 0.0,
                        max: 1.0,
                        default_value: 0.7,
                        current_value: 0.7,
                    },
                    ParameterDefinition {
                        id: ParamId(3),
                        name: "Release".to_string(),
                        min: 1.0,
                        max: 5000.0,
                        default_value: 200.0,
                        current_value: 200.0,
                    },
                ],
            },
            ModuleKind::AudioOut => Self {
                id,
                kind,
                name: name.to_string(),
                position,
                ports: vec![
                    PortDefinition {
                        id: PortId(0),
                        name: "Left".to_string(),
                        signal_type: SignalType::Audio,
                        direction: PortDirection::Input,
                    },
                    PortDefinition {
                        id: PortId(1),
                        name: "Right".to_string(),
                        signal_type: SignalType::Audio,
                        direction: PortDirection::Input,
                    },
                ],
                parameters: vec![ParameterDefinition {
                    id: ParamId(0),
                    name: "Master Volume".to_string(),
                    min: 0.0,
                    max: 1.0,
                    default_value: 0.8,
                    current_value: 0.8,
                }],
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cable {
    pub id: u32,
    pub from_module: ModuleId,
    pub from_port: PortId,
    pub to_module: ModuleId,
    pub to_port: PortId,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Patch {
    pub version: String,
    pub name: String,
    pub modules: Vec<ModuleInstance>,
    pub cables: Vec<Cable>,
}

impl Patch {
    pub fn new(name: &str) -> Self {
        Self {
            version: "1.0.0".to_string(),
            name: name.to_string(),
            modules: Vec::new(),
            cables: Vec::new(),
        }
    }

    pub fn find_module(&self, id: ModuleId) -> Option<&ModuleInstance> {
        self.modules.iter().find(|m| m.id == id)
    }
}
