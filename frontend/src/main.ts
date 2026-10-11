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
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Fond sombre
    ctx.fillStyle = "#121316";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Grille de points
    ctx.fillStyle = "#222530";
    const step = 28;
    for (let x = step; x < canvas.width; x += step) {
      for (let y = step; y < canvas.height; y += step) {
        ctx.beginPath();
        ctx.arc(x, y, 1.2, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    const modWidth = 170;
    const modHeight = 150;

    // Helper pour trouver les coordonnées d'un port
    function getPortCoords(modId: number, portId: number): { x: number; y: number; type: string } | null {
      const mod = defaultPatch.modules.find((m) => m.id === modId);
      if (!mod) return null;
      const portIndex = mod.ports.findIndex((p) => p.id === portId);
      if (portIndex === -1) return null;
      const port = mod.ports[portIndex];
      const isInput = port.direction === "Input";

      const idxWithinGroup = isInput
        ? mod.ports.filter((p) => p.direction === "Input").indexOf(port)
        : mod.ports.filter((p) => p.direction === "Output").indexOf(port);

      const spacing = 28;
      const startY = mod.position.y + 60;
      const py = startY + idxWithinGroup * spacing;
      const px = isInput ? mod.position.x : mod.position.x + modWidth;

      return { x: px, y: py, type: port.signal_type };
    }

    // 1. Dessin des câbles (Courbes de Bézier cubiques)
    for (const cable of defaultPatch.cables) {
      const pFrom = getPortCoords(cable.from_module, cable.from_port);
      const pTo = getPortCoords(cable.to_module, cable.to_port);
      if (!pFrom || !pTo) continue;

      ctx.beginPath();
      ctx.moveTo(pFrom.x, pFrom.y);

      const dx = Math.abs(pTo.x - pFrom.x) * 0.5 + 40;
      ctx.bezierCurveTo(pFrom.x + dx, pFrom.y, pTo.x - dx, pTo.y, pTo.x, pTo.y);

      ctx.lineWidth = 3.5;
      ctx.strokeStyle = pFrom.type === "Audio" ? "#ff5200" : "#00a8ff";
      ctx.lineCap = "round";
      ctx.stroke();

      // Halo lumineux subtil
      ctx.lineWidth = 7;
      ctx.strokeStyle = pFrom.type === "Audio" ? "rgba(255, 82, 0, 0.15)" : "rgba(0, 168, 255, 0.15)";
      ctx.stroke();
    }

    // 2. Dessin des modules
    for (const mod of defaultPatch.modules) {
      const mx = mod.position.x;
      const my = mod.position.y;

      // Ombre du module
      ctx.shadowColor = "rgba(0, 0, 0, 0.4)";
      ctx.shadowBlur = 12;
      ctx.shadowOffsetX = 0;
      ctx.shadowOffsetY = 6;

      // Carte du module
      ctx.fillStyle = "#1e2029";
      ctx.beginPath();
      ctx.roundRect(mx, my, modWidth, modHeight, 8);
      ctx.fill();

      // Bordure
      ctx.shadowColor = "transparent";
      ctx.lineWidth = 1;
      ctx.strokeStyle = "#303444";
      ctx.stroke();

      // En-tête du module
      ctx.fillStyle = "#272a36";
      ctx.beginPath();
      ctx.roundRect(mx, my, modWidth, 32, [8, 8, 0, 0]);
      ctx.fill();

      // Badge type
      ctx.fillStyle = "#ff5200";
      ctx.fillRect(mx + 10, my + 11, 4, 11);

      // Titre
      ctx.fillStyle = "#f0f2f8";
      ctx.font = "bold 12px Inter, sans-serif";
      ctx.fillText(mod.name, mx + 20, my + 20);

      // Paramètre (potentiomètre / affichage slider)
      if (mod.parameters.length > 0) {
        const param = mod.parameters[0];
        ctx.fillStyle = "#8a8f9f";
        ctx.font = "11px Inter, sans-serif";
        ctx.fillText(param.name, mx + 16, my + 120);

        ctx.fillStyle = "#4a4f62";
        ctx.fillRect(mx + 16, my + 128, modWidth - 32, 6);
        ctx.fillStyle = "#ff5200";
        const fillW = ((param.current_value - param.min) / (param.max - param.min)) * (modWidth - 32);
        ctx.fillRect(mx + 16, my + 128, Math.max(4, fillW), 6);
      }

      // Dessin des ports
      for (const port of mod.ports) {
        const coords = getPortCoords(mod.id, port.id);
        if (!coords) continue;

        // Anneau extérieur du port
        ctx.beginPath();
        ctx.arc(coords.x, coords.y, 6, 0, Math.PI * 2);
        ctx.fillStyle = "#161820";
        ctx.fill();
        ctx.lineWidth = 2;
        ctx.strokeStyle = port.signal_type === "Audio" ? "#ff5200" : port.signal_type === "ControlVoltage" ? "#00a8ff" : "#ffd32a";
        ctx.stroke();

        // Broche intérieure
        ctx.beginPath();
        ctx.arc(coords.x, coords.y, 2.5, 0, Math.PI * 2);
        ctx.fillStyle = ctx.strokeStyle;
        ctx.fill();

        // Label du port
        ctx.fillStyle = "#73788b";
        ctx.font = "10px Inter, sans-serif";
        if (port.direction === "Input") {
          ctx.fillText(port.name, coords.x + 10, coords.y + 3);
        } else {
          const w = ctx.measureText(port.name).width;
          ctx.fillText(port.name, coords.x - w - 10, coords.y + 3);
        }
      }
    }
  }

  window.addEventListener("resize", resize);
  resize();
}
