# The Grid

Modular visual synthesizer combining a node graph editor with a real-time audio engine.

## Language

**Patch**:
A complete graph configuration containing modules, parameters, and interconnected cables.
_Avoid_: Graph, project, document, session

**DspNode**:
An executable audio processing unit with uniform block rendering, parameter handling, and state management.
_Avoid_: Audio unit, processor, plugin, voice

**DspSlot**:
A preallocated execution slot in the real-time engine hosting a configured DspNode.
_Avoid_: Slot, rack item, voice container

**PatchCompiler**:
A compiler that validates patch topology, checks port compatibility, detects cycles, and generates a linear ExecutionPlan.
_Avoid_: Graph validator, topological sorter, scheduler

**ExecutionPlan**:
A linear ordered sequence of audio execution steps and scratch buffer assignments compiled from a Patch.
_Avoid_: Bytecode, pipeline, schedule

**Cable**:
A directional connection carrying audio rate, control voltage, or trigger signals from an output port to an input port.
_Avoid_: Wire, patch cord, link, edge

**Port**:
A typed connection terminal on a module designating signal direction and rate.
_Avoid_: Jack, pin, connector, socket
