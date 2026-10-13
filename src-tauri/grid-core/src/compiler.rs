use crate::model::{ModuleId, ModuleKind, Patch, PortDirection, SignalType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

/// Erreur survenue lors de la validation topologique ou de la compilation du Patch
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompileError {
    #[error("Cycle non retardé détecté dans le graphe")]
    CycleDetected,
    #[error("Module introuvable: {0:?}")]
    ModuleNotFound(ModuleId),
    #[error("Port introuvable sur le module {module:?}: {port:?}")]
    PortNotFound { module: ModuleId, port: u32 },
    #[error("Le port source {port:?} du module {module:?} n'est pas une sortie")]
    InvalidSourcePort { module: ModuleId, port: u32 },
    #[error("Le port cible {port:?} du module {module:?} n'est pas une entrée")]
    InvalidTargetPort { module: ModuleId, port: u32 },
    #[error("Incompatibilité de signaux entre source ({src:?}) et cible ({dst:?})")]
    IncompatibleSignalType { src: SignalType, dst: SignalType },
}

pub type GraphError = CompileError;

/// Indice d'un scratch buffer préalloué dans le runtime audio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferIndex(pub u16);

/// Étape élémentaire d'exécution linéaire d'un module
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub module_id: ModuleId,
    pub kind: ModuleKind,
    /// Indices des scratch buffers d'entrée connectés par port d'entrée
    pub input_buffers: Vec<Option<BufferIndex>>,
    /// Indices des scratch buffers de sortie produits par port de sortie
    pub output_buffers: Vec<Option<BufferIndex>>,
    /// Paramètres initiaux
    pub initial_parameters: Vec<f32>,
}

/// Plan linéaire compilé, prêt à être exécuté par le moteur sans aucune allocation
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    /// Nombre total de buffers temporaires requis par le plan
    pub required_scratch_buffers: usize,
}

impl ExecutionPlan {
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

/// Compilateur de Patch transformant un graphe modulaire en un plan d'exécution linéaire
pub struct PatchCompiler;

impl PatchCompiler {
    /// Valide la topologie, vérifie les invariants de câblage et compile le Patch en ExecutionPlan
    pub fn compile(patch: &Patch) -> Result<ExecutionPlan, CompileError> {
        let execution_order = Self::sort_topologically(patch)?;

        let mut steps = Vec::with_capacity(execution_order.len());
        let mut buffer_counter: u16 = 0;

        let mut output_routes: HashMap<(ModuleId, u32), BufferIndex> = HashMap::new();

        // 1. Allouer un buffer pour chaque sortie qui alimente un câble
        for cable in &patch.cables {
            let route_key = (cable.from_module, cable.from_port.0);
            output_routes.entry(route_key).or_insert_with(|| {
                let idx = BufferIndex(buffer_counter);
                buffer_counter += 1;
                idx
            });
        }

        // 2. Ordonner séquentiellement les étapes selon execution_order
        for &module_id in &execution_order {
            let module = patch
                .find_module(module_id)
                .ok_or(CompileError::ModuleNotFound(module_id))?;

            let mut input_buffers = Vec::new();
            for port in &module.ports {
                if port.direction == PortDirection::Input {
                    let buf_opt = patch
                        .cables
                        .iter()
                        .find(|c| c.to_module == module_id && c.to_port == port.id)
                        .and_then(|cable| {
                            output_routes
                                .get(&(cable.from_module, cable.from_port.0))
                                .copied()
                        });
                    input_buffers.push(buf_opt);
                }
            }

            let mut output_buffers = Vec::new();
            for port in &module.ports {
                if port.direction == PortDirection::Output {
                    let buf_opt = output_routes.get(&(module_id, port.id.0)).copied();
                    output_buffers.push(buf_opt);
                }
            }

            let initial_parameters = module.parameters.iter().map(|p| p.current_value).collect();

            steps.push(ExecutionStep {
                module_id,
                kind: module.kind,
                input_buffers,
                output_buffers,
                initial_parameters,
            });
        }

        Ok(ExecutionPlan {
            steps,
            required_scratch_buffers: buffer_counter as usize,
        })
    }

    /// Trie les modules topologiquement via l'algorithme de Kahn tout en validant les câbles et cycles
    fn sort_topologically(patch: &Patch) -> Result<Vec<ModuleId>, CompileError> {
        let mut in_degree: HashMap<ModuleId, usize> = HashMap::new();
        let mut adjacency: HashMap<ModuleId, Vec<ModuleId>> = HashMap::new();

        for module in &patch.modules {
            in_degree.insert(module.id, 0);
            adjacency.insert(module.id, Vec::new());
        }

        for cable in &patch.cables {
            let from_mod = patch
                .find_module(cable.from_module)
                .ok_or(CompileError::ModuleNotFound(cable.from_module))?;
            let to_mod = patch
                .find_module(cable.to_module)
                .ok_or(CompileError::ModuleNotFound(cable.to_module))?;

            let from_port = from_mod
                .ports
                .iter()
                .find(|p| p.id == cable.from_port)
                .ok_or(CompileError::PortNotFound {
                    module: cable.from_module,
                    port: cable.from_port.0,
                })?;

            let to_port = to_mod.ports.iter().find(|p| p.id == cable.to_port).ok_or(
                CompileError::PortNotFound {
                    module: cable.to_module,
                    port: cable.to_port.0,
                },
            )?;

            if from_port.direction != PortDirection::Output {
                return Err(CompileError::InvalidSourcePort {
                    module: cable.from_module,
                    port: cable.from_port.0,
                });
            }

            if to_port.direction != PortDirection::Input {
                return Err(CompileError::InvalidTargetPort {
                    module: cable.to_module,
                    port: cable.to_port.0,
                });
            }

            if from_port.signal_type != to_port.signal_type {
                let is_cv_audio_cross = matches!(
                    (from_port.signal_type, to_port.signal_type),
                    (SignalType::Audio, SignalType::ControlVoltage)
                        | (SignalType::ControlVoltage, SignalType::Audio)
                );
                if !is_cv_audio_cross {
                    return Err(CompileError::IncompatibleSignalType {
                        src: from_port.signal_type,
                        dst: to_port.signal_type,
                    });
                }
            }

            adjacency
                .get_mut(&cable.from_module)
                .unwrap()
                .push(cable.to_module);

            *in_degree.get_mut(&cable.to_module).unwrap() += 1;
        }

        let mut root_nodes: Vec<ModuleId> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&mod_id, _)| mod_id)
            .collect();

        // Placer MidiIn en tete des noeuds sources, puis trier par ModuleId pour un ordre deterministe
        root_nodes.sort_by_key(|&mod_id| {
            let is_midi_in = patch
                .find_module(mod_id)
                .map(|m| m.kind == ModuleKind::MidiIn)
                .unwrap_or(false);
            (!is_midi_in, mod_id.0)
        });

        let mut queue: VecDeque<ModuleId> = VecDeque::from(root_nodes);

        let mut execution_order = Vec::with_capacity(patch.modules.len());

        while let Some(current) = queue.pop_front() {
            execution_order.push(current);

            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    let entry = in_degree.get_mut(&neighbor).unwrap();
                    *entry -= 1;
                    if *entry == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if execution_order.len() != patch.modules.len() {
            return Err(CompileError::CycleDetected);
        }

        Ok(execution_order)
    }
}

/// Raccourci pour compiler un Patch
pub fn compile_patch(patch: &Patch) -> Result<ExecutionPlan, CompileError> {
    PatchCompiler::compile(patch)
}
