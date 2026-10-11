import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Interfaces typées du patch synchronisées avec le backend Rust
export interface Position {
  x: number;
  y: number;
}

export interface PortDefinition {
  id: number;
  name: string;
  signal_type: "Audio" | "ControlVoltage" | "Trigger";
  direction: "Input" | "Output";
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
  kind: "Oscillator" | "Gain" | "Constant" | "AudioOut";
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

let isPlaying = false;
const btnPower = document.getElementById("btn-power") as HTMLButtonElement | null;
const masterGain = document.getElementById("master-gain") as HTMLInputElement | null;
const gainLabel = document.getElementById("gain-label") as HTMLSpanElement | null;
const canvas = document.getElementById("grid-canvas") as HTMLCanvasElement | null;

// Éléments du vu-mètre
const meterBarL = document.getElementById("meter-bar-l") as HTMLDivElement | null;
const meterBarR = document.getElementById("meter-bar-r") as HTMLDivElement | null;
const meterNumeric = document.getElementById("meter-numeric") as HTMLDivElement | null;

// Écoute des événements de télémétrie décimée émis par le Host Rust (~60 Hz)
listen<TelemetryFrame>("telemetry_update", (event) => {
  const frame = event.payload;

  // Calcul du pourcentage de crête pour les barres (0.0..=1.0)
  const pctL = Math.min(100, Math.max(0, frame.peak_left * 100));
  const pctR = Math.min(100, Math.max(0, frame.peak_right * 100));

  if (meterBarL) meterBarL.style.width = `${pctL.toFixed(1)}%`;
  if (meterBarR) meterBarR.style.width = `${pctR.toFixed(1)}%`;

  if (meterNumeric) {
    const maxPeak = Math.max(frame.peak_left, frame.peak_right);
    if (maxPeak <= 0.0001) {
      meterNumeric.textContent = "-inf dB";
    } else {
      const db = 20 * Math.log10(maxPeak);
      meterNumeric.textContent = `${db.toFixed(1)} dB`;
    }
  }
});

// Patch par défaut
const defaultPatch: Patch = {
  version: "1.0.0",
  name: "Init Patch",
  modules: [
    {
      id: 1,
      kind: "Oscillator",
      name: "VCO 1",
      position: { x: 100, y: 150 },
      ports: [
        { id: 0, name: "Freq CV", signal_type: "ControlVoltage", direction: "Input" },
        { id: 1, name: "Out", signal_type: "Audio", direction: "Output" },
      ],
      parameters: [
        { id: 0, name: "Frequency", min: 20, max: 20000, default_value: 440, current_value: 440 },
      ],
    },
    {
      id: 2,
      kind: "Gain",
      name: "VCA 1",
      position: { x: 300, y: 150 },
      ports: [
        { id: 0, name: "In", signal_type: "Audio", direction: "Input" },
        { id: 1, name: "Gain CV", signal_type: "ControlVoltage", direction: "Input" },
        { id: 2, name: "Out", signal_type: "Audio", direction: "Output" },
      ],
      parameters: [
        { id: 0, name: "Gain", min: 0, max: 2, default_value: 1, current_value: 1 },
      ],
    },
    {
      id: 3,
      kind: "AudioOut",
      name: "Master Out",
      position: { x: 500, y: 150 },
      ports: [
        { id: 0, name: "Left", signal_type: "Audio", direction: "Input" },
        { id: 1, name: "Right", signal_type: "Audio", direction: "Input" },
      ],
      parameters: [
        { id: 0, name: "Master Volume", min: 0, max: 1, default_value: 0.8, current_value: 0.8 },
      ],
    },
  ],
  cables: [
    { id: 1, from_module: 1, from_port: 1, to_module: 2, to_port: 0 },
    { id: 2, from_module: 2, from_port: 2, to_module: 3, to_port: 0 },
  ],
};

// Initialisation des bindings IPC
if (btnPower) {
  btnPower.addEventListener("click", async () => {
    isPlaying = !isPlaying;
    try {
      if (isPlaying) {
        await invoke("apply_patch", { patch: defaultPatch });
        await invoke("start_audio");
        btnPower.classList.add("active");
        btnPower.textContent = "Audio ON";
      } else {
        await invoke("stop_audio");
        btnPower.classList.remove("active");
        btnPower.textContent = "Audio OFF";
        if (meterBarL) meterBarL.style.width = "0%";
        if (meterBarR) meterBarR.style.width = "0%";
        if (meterNumeric) meterNumeric.textContent = "-inf dB";
      }
    } catch (err) {
      console.error("Erreur lors de l'appel IPC audio:", err);
    }
  });
}

if (masterGain) {
  masterGain.addEventListener("input", async (e) => {
    const val = parseFloat((e.target as HTMLInputElement).value);
    if (gainLabel) gainLabel.textContent = `${Math.round(val * 100)}%`;
    try {
      await invoke("set_master_gain", { gain: val });
    } catch (err) {
      console.error("Erreur gain:", err);
    }
  });
}

// Canvas minimal pour le rendu de la grille
if (canvas) {
  const ctx = canvas.getContext("2d");
  function resize() {
    if (!canvas) return;
    canvas.width = canvas.parentElement?.clientWidth || 800;
    canvas.height = canvas.parentElement?.clientHeight || 600;
    draw();
  }

  function draw() {
    if (!ctx || !canvas) return;
    ctx.fillStyle = "#121316";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Dessin de la grille de points (Grid dots)
    ctx.fillStyle = "#262933";
    const step = 32;
    for (let x = step; x < canvas.width; x += step) {
      for (let y = step; y < canvas.height; y += step) {
        ctx.fillRect(x, y, 2, 2);
      }
    }
  }

  window.addEventListener("resize", resize);
  resize();
}
