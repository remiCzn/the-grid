use crate::graph::{AudioGraph, GraphError};
use crate::model::{ModuleId, ModuleKind, Patch};
use serde::{Deserialize, Serialize};

/// Indice d'un scratch buffer préalloué dans le runtime audio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferIndex(pub u16);

/// Étape élémentaire d'exécution linéaire d'un module
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub module_id: ModuleId,
    pub kind: ModuleKind,
    /// Indices des scratch buffers d'entrée connectés
    pub input_buffers: Vec<BufferIndex>,
    /// Indices des scratch buffers de sortie produits
    pub output_buffers: Vec<BufferIndex>,
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

/// Compile un graphe audio validé en un plan d'exécution linéaire
pub fn compile_patch(graph: &AudioGraph, patch: &Patch) -> Result<ExecutionPlan, GraphError> {
    let mut steps = Vec::with_capacity(graph.execution_order.len());
    let mut buffer_counter: u16 = 0;

    // Table de correspondance (ModuleId, OutputPortId) -> BufferIndex
    let mut output_routes: std::collections::HashMap<(ModuleId, u32), BufferIndex> =
        std::collections::HashMap::new();

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
    for &module_id in &graph.execution_order {
        let module = patch
            .find_module(module_id)
            .ok_or(GraphError::ModuleNotFound(module_id))?;

        // Résolution des buffers d'entrée connectés par des câbles vers ce module
        let mut input_buffers = Vec::new();
        for port in &module.ports {
            if port.direction == crate::model::PortDirection::Input {
                if let Some(cable) = patch
                    .cables
                    .iter()
                    .find(|c| c.to_module == module_id && c.to_port == port.id)
                {
                    if let Some(&buf_idx) =
                        output_routes.get(&(cable.from_module, cable.from_port.0))
                    {
                        input_buffers.push(buf_idx);
                    }
                }
            }
        }

        // Résolution des buffers de sortie
        let mut output_buffers = Vec::new();
        for port in &module.ports {
            if port.direction == crate::model::PortDirection::Output {
                if let Some(&buf_idx) = output_routes.get(&(module_id, port.id.0)) {
                    output_buffers.push(buf_idx);
                }
            }
        }

        let initial_parameters = module
            .parameters
            .iter()
            .map(|p| p.current_value)
            .collect();

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
