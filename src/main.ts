import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { GridInteractionHandler } from "./canvas/interaction";
import { GridRenderer } from "./canvas/renderer";
import { ComputerKeyboard } from "./keyboard_input";
import { PatchSession } from "./session";
import { Patch, TelemetryFrame } from "./types";
import { VirtualKeyboard } from "./virtual_keyboard";

let isPlaying = false;
const btnPower = document.getElementById("btn-power") as HTMLButtonElement | null;
const masterGain = document.getElementById("master-gain") as HTMLInputElement | null;
const gainLabel = document.getElementById("gain-label") as HTMLSpanElement | null;
const canvas = document.getElementById("grid-canvas") as HTMLCanvasElement | null;

const meterBarL = document.getElementById("meter-bar-l") as HTMLDivElement | null;
const meterBarR = document.getElementById("meter-bar-r") as HTMLDivElement | null;
const meterNumeric = document.getElementById("meter-numeric") as HTMLDivElement | null;

// Modele de patch initial avec module MidiIn connecte
const initialPatch: Patch = {
  version: "1.0.0",
  name: "Interactive Patch",
  modules: [
    {
      id: 4,
      kind: "MidiIn",
      name: "MIDI In",
      position: { x: 40, y: 80 },
      ports: [
        {
          id: 0,
          name: "pitch",
          signal_type: "ControlVoltage",
          direction: "Output",
        },
        { id: 1, name: "gate", signal_type: "Trigger", direction: "Output" },
        {
          id: 2,
          name: "velocity",
          signal_type: "ControlVoltage",
          direction: "Output",
        },
        {
          id: 3,
          name: "mod_wheel",
          signal_type: "ControlVoltage",
          direction: "Output",
        },
        {
          id: 4,
          name: "pitch_bend",
          signal_type: "ControlVoltage",
          direction: "Output",
        },
      ],
      parameters: [],
    },
    {
      id: 1,
      kind: "Oscillator",
      name: "VCO 1",
      position: { x: 280, y: 80 },
      ports: [
        {
          id: 0,
          name: "Freq CV",
          signal_type: "ControlVoltage",
          direction: "Input",
        },
        { id: 1, name: "Out", signal_type: "Audio", direction: "Output" },
      ],
      parameters: [
        {
          id: 0,
          name: "Frequency",
          min: 20,
          max: 2000,
          default_value: 440,
          current_value: 440,
        },
      ],
    },
    {
      id: 2,
      kind: "Gain",
      name: "VCA 1",
      position: { x: 520, y: 80 },
      ports: [
        { id: 0, name: "In", signal_type: "Audio", direction: "Input" },
        {
          id: 1,
          name: "Gain CV",
          signal_type: "ControlVoltage",
          direction: "Input",
        },
        { id: 2, name: "Out", signal_type: "Audio", direction: "Output" },
      ],
      parameters: [
        {
          id: 0,
          name: "Gain",
          min: 0,
          max: 2,
          default_value: 1,
          current_value: 1,
        },
      ],
    },
    {
      id: 3,
      kind: "AudioOut",
      name: "Master Out",
      position: { x: 760, y: 80 },
      ports: [
        { id: 0, name: "Left", signal_type: "Audio", direction: "Input" },
        { id: 1, name: "Right", signal_type: "Audio", direction: "Input" },
      ],
      parameters: [
        {
          id: 0,
          name: "Master Volume",
          min: 0,
          max: 1,
          default_value: 0.8,
          current_value: 0.8,
        },
      ],
    },
  ],
  cables: [
    { id: 1, from_module: 4, from_port: 0, to_module: 1, to_port: 0 },
    { id: 2, from_module: 1, from_port: 1, to_module: 2, to_port: 0 },
    { id: 3, from_module: 2, from_port: 2, to_module: 3, to_port: 0 },
  ],
};

// Instance centrale de session de patch
const session = new PatchSession(initialPatch);

// Initialisation du Clavier Virtuel et Clavier Ordinateur
const appContainer = document.getElementById("app") || document.body;

const virtualKeyboard = new VirtualKeyboard(appContainer, {
  octave: 4,
  layout: "AZERTY",
  onMidiEvent: (event) => {
    session.sendMidiEvent(event);
  },
  onOctaveChange: (oct) => {
    computerKeyboard.setOctave(oct);
  },
  onLayoutChange: (layout) => {
    computerKeyboard.setLayout(layout);
  },
});

const computerKeyboard = new ComputerKeyboard({
  octave: 4,
  layout: "AZERTY",
  onMidiEvent: (event) => {
    session.sendMidiEvent(event);
  },
  onNoteChange: (note, active) => {
    virtualKeyboard.setNoteActive(note, active);
  },
});

computerKeyboard.attach(window);

// Initialisation du Canvas 2D interactif
if (canvas) {
  const renderer = new GridRenderer(canvas);
  const interaction = new GridInteractionHandler(canvas, session);

  window.addEventListener("resize", () => {
    renderer.resize();
  });
  renderer.resize();

  // Boucle d'animation fluide à 60/120 FPS
  function renderLoop() {
    renderer.render(
      session.getPatch(),
      interaction.activeWire,
      interaction.hoveredPort,
      interaction.selectedModuleId,
    );
    requestAnimationFrame(renderLoop);
  }
  requestAnimationFrame(renderLoop);
}

// Écoute de la télémétrie décimée (~60 Hz)
listen<TelemetryFrame>("telemetry_update", (event) => {
  const frame = event.payload;
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

// Contrôles Toolbar
if (btnPower) {
  btnPower.addEventListener("click", async () => {
    isPlaying = !isPlaying;
    try {
      if (isPlaying) {
        await session.sync();
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

// Export / Import de Patch (.json / .grid)
const btnSave = document.getElementById("btn-save") as HTMLButtonElement | null;
const btnLoad = document.getElementById("btn-load") as HTMLButtonElement | null;
const fileInput = document.getElementById("file-input") as HTMLInputElement | null;

if (btnSave) {
  btnSave.addEventListener("click", async () => {
    try {
      const jsonStr = (await invoke("export_patch")) as string;
      const blob = new Blob([jsonStr], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      const currentPatch = session.getPatch();
      a.download = `${currentPatch.name.toLowerCase().replace(/\s+/g, "-")}.grid.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error("Erreur export patch:", err);
    }
  });
}

if (btnLoad && fileInput) {
  btnLoad.addEventListener("click", () => {
    fileInput.click();
  });

  fileInput.addEventListener("change", async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      const loadedPatch = (await invoke("import_patch", {
        jsonStr: text,
      })) as Patch;
      await session.loadPatch(loadedPatch);
      console.log("Patch chargé avec succès:", loadedPatch.name);
    } catch (err) {
      console.error("Erreur import patch:", err);
      alert(`Erreur d'importation du patch: ${err}`);
    }
    fileInput.value = "";
  });
}
