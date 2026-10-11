use crate::model::{ModuleId, Patch, PortDirection, SignalType};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
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

/// Graphe de flux audio validé et ordonné topologiquement
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AudioGraph {
    /// Liste ordonnée des modules selon leurs dépendances d'exécution
    pub execution_order: Vec<ModuleId>,
    /// Cartographie des connexions directes par câble validées
    pub dependencies: HashMap<ModuleId, Vec<ModuleId>>,
}

impl AudioGraph {
    /// Construit et valide le graphe d'exécution à partir d'un Patch déclaratif
    pub fn from_patch(patch: &Patch) -> Result<Self, GraphError> {
        let mut in_degree: HashMap<ModuleId, usize> = HashMap::new();
        let mut adjacency: HashMap<ModuleId, Vec<ModuleId>> = HashMap::new();

        // 1. Initialiser tous les nœuds
        for module in &patch.modules {
            in_degree.insert(module.id, 0);
            adjacency.insert(module.id, Vec::new());
        }

        // 2. Valider chaque câble et construire les arêtes dirigées
        for cable in &patch.cables {
            let from_mod = patch
                .find_module(cable.from_module)
                .ok_or(GraphError::ModuleNotFound(cable.from_module))?;
            let to_mod = patch
                .find_module(cable.to_module)
                .ok_or(GraphError::ModuleNotFound(cable.to_module))?;

            let from_port = from_mod
                .ports
                .iter()
                .find(|p| p.id == cable.from_port)
                .ok_or(GraphError::PortNotFound {
                    module: cable.from_module,
                    port: cable.from_port.0,
                })?;

            let to_port = to_mod
                .ports
                .iter()
                .find(|p| p.id == cable.to_port)
                .ok_or(GraphError::PortNotFound {
                    module: cable.to_module,
                    port: cable.to_port.0,
                })?;

            if from_port.direction != PortDirection::Output {
                return Err(GraphError::InvalidSourcePort {
                    module: cable.from_module,
                    port: cable.from_port.0,
                });
            }

            if to_port.direction != PortDirection::Input {
                return Err(GraphError::InvalidTargetPort {
                    module: cable.to_module,
                    port: cable.to_port.0,
                });
            }

            // Contrôle de compatibilité des types de signaux
            if from_port.signal_type != to_port.signal_type {
                // Règle de tolérance : CV vers Audio ou Audio vers CV est autorisé (modulation audio rate)
                let is_cv_audio_cross = matches!(
                    (from_port.signal_type, to_port.signal_type),
                    (SignalType::Audio, SignalType::ControlVoltage)
                        | (SignalType::ControlVoltage, SignalType::Audio)
                );
                if !is_cv_audio_cross {
                    return Err(GraphError::IncompatibleSignalType {
                        src: from_port.signal_type,
                        dst: to_port.signal_type,
                    });
                }
            }

            // Enregistrer la dépendance directe : from_module -> to_module
            adjacency
                .get_mut(&cable.from_module)
                .unwrap()
                .push(cable.to_module);

            *in_degree.get_mut(&cable.to_module).unwrap() += 1;
        }

        // 3. Algorithme de tri topologique de Kahn
        let mut queue: VecDeque<ModuleId> = VecDeque::new();
        for (&mod_id, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(mod_id);
            }
        }

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

        // 4. Détection de cycle : si tous les nœuds ne sont pas visités, il y a un cycle
        if execution_order.len() != patch.modules.len() {
            return Err(GraphError::CycleDetected);
        }

        Ok(Self {
            execution_order,
            dependencies: adjacency,
        })
    }
}
