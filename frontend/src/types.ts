// Types partagés pour le frontend The Grid

export interface Position {
  x: number;
  y: number;
}

export type SignalType = "Audio" | "ControlVoltage" | "Trigger";
export type PortDirection = "Input" | "Output";
export type ModuleKind =
  | "Oscillator"
  | "Gain"
  | "Constant"
  | "Filter"
  | "Envelope"
  | "AudioOut";

export interface PortDefinition {
  id: number;
  name: string;
  signal_type: SignalType;
  direction: PortDirection;
}

export interface ParameterDefinition {
  id: number;
  name: string;
  min: number;
  max: number;
  default_value: number;
  current_value: number;
}

export interface ModuleInstance {
  id: number;
  kind: ModuleKind;
  name: string;
  position: Position;
  ports: PortDefinition[];
  parameters: ParameterDefinition[];
}

export interface Cable {
  id: number;
  from_module: number;
  from_port: number;
  to_module: number;
  to_port: number;
}

export interface Patch {
  version: string;
  name: string;
  modules: ModuleInstance[];
  cables: Cable[];
}

export interface TelemetryFrame {
  frame_index: number;
  peak_left: number;
  peak_right: number;
  rms_left: number;
  rms_right: number;
}
