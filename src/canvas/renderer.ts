import { ModuleInstance, Patch, SignalType } from "../types";

export interface ActiveWire {
  fromModuleId: number;
  fromPortId: number;
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
  signalType: SignalType;
}

export const MODULE_WIDTH = 175;
export const MODULE_HEIGHT = 150;
export const PORT_RADIUS = 6;

export function getModuleHeight(mod: ModuleInstance): number {
  const inputCount = mod.ports.filter((p) => p.direction === "Input").length;
  const outputCount = mod.ports.filter((p) => p.direction === "Output").length;
  const maxPorts = Math.max(inputCount, outputCount);
  const neededHeight = 60 + maxPorts * 28 + 12;
  return Math.max(MODULE_HEIGHT, neededHeight);
}

export function getSignalColor(signalType: SignalType): string {
  switch (signalType) {
    case "Audio":
      return "#ff5200";
    case "ControlVoltage":
      return "#00a8ff";
    case "Trigger":
      return "#ffd32a";
    default:
      return "#ffffff";
  }
}

export function getPortCoordinates(
  mod: ModuleInstance,
  portId: number,
): {
  x: number;
  y: number;
  type: SignalType;
  direction: "Input" | "Output";
} | null {
  const portIndex = mod.ports.findIndex((p) => p.id === portId);
  if (portIndex === -1) return null;
  const port = mod.ports[portIndex];

  const isInput = port.direction === "Input";
  const group = mod.ports.filter((p) => p.direction === port.direction);
  const idxWithinGroup = group.indexOf(port);

  const spacing = 28;
  const startY = mod.position.y + 60;
  const py = startY + idxWithinGroup * spacing;
  const px = isInput ? mod.position.x : mod.position.x + MODULE_WIDTH;

  return { x: px, y: py, type: port.signal_type, direction: port.direction };
}

export class GridRenderer {
  private ctx: CanvasRenderingContext2D;
  private canvas: HTMLCanvasElement;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Impossible d'obtenir le contexte Canvas 2D");
    this.ctx = context;
  }

  public resize(): void {
    const parent = this.canvas.parentElement;
    if (!parent) return;
    const dpr = window.devicePixelRatio || 1;
    const width = parent.clientWidth;
    const height = parent.clientHeight;

    this.canvas.width = width * dpr;
    this.canvas.height = height * dpr;
    this.canvas.style.width = `${width}px`;
    this.canvas.style.height = `${height}px`;

    this.ctx.resetTransform?.();
    this.ctx.scale(dpr, dpr);
  }

  public render(
    patch: Patch,
    activeWire: ActiveWire | null,
    hoveredPort: { moduleId: number; portId: number } | null,
    selectedModuleId: number | null,
  ): void {
    const width = this.canvas.clientWidth;
    const height = this.canvas.clientHeight;
    const ctx = this.ctx;

    ctx.clearRect(0, 0, width, height);

    // 1. Fond sombre de la grille
    ctx.fillStyle = "#121316";
    ctx.fillRect(0, 0, width, height);

    // 2. Grille de points (Grid Dots)
    ctx.fillStyle = "#222530";
    const step = 28;
    for (let x = step; x < width; x += step) {
      for (let y = step; y < height; y += step) {
        ctx.beginPath();
        ctx.arc(x, y, 1.2, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    // 3. Dessin des câbles établis
    for (const cable of patch.cables) {
      const fromMod = patch.modules.find((m) => m.id === cable.from_module);
      const toMod = patch.modules.find((m) => m.id === cable.to_module);
      if (!fromMod || !toMod) continue;

      const pFrom = getPortCoordinates(fromMod, cable.from_port);
      const pTo = getPortCoordinates(toMod, cable.to_port);
      if (!pFrom || !pTo) continue;

      this.drawBezierCable(
        pFrom.x,
        pFrom.y,
        pTo.x,
        pTo.y,
        getSignalColor(pFrom.type),
      );
    }

    // 4. Dessin du câble en cours de glissement
    if (activeWire) {
      this.drawBezierCable(
        activeWire.startX,
        activeWire.startY,
        activeWire.currentX,
        activeWire.currentY,
        getSignalColor(activeWire.signalType),
        true,
      );
    }

    // 5. Dessin des modules
    for (const mod of patch.modules) {
      this.drawModule(mod, mod.id === selectedModuleId, hoveredPort);
    }
  }

  private drawBezierCable(
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    color: string,
    isDraft = false,
  ): void {
    const ctx = this.ctx;
    ctx.beginPath();
    ctx.moveTo(x1, y1);

    const dx = Math.abs(x2 - x1) * 0.5 + 40;
    ctx.bezierCurveTo(x1 + dx, y1, x2 - dx, y2, x2, y2);

    // Câble principal
    ctx.lineWidth = isDraft ? 2.5 : 3.5;
    ctx.strokeStyle = color;
    ctx.lineCap = "round";
    ctx.stroke();

    // Halo lumineux
    ctx.lineWidth = isDraft ? 5 : 7;
    ctx.strokeStyle =
      color === "#ff5200"
        ? "rgba(255, 82, 0, 0.18)"
        : "rgba(0, 168, 255, 0.18)";
    ctx.stroke();
  }

  private drawModule(
    mod: ModuleInstance,
    isSelected: boolean,
    hoveredPort: { moduleId: number; portId: number } | null,
  ): void {
    const ctx = this.ctx;
    const mx = mod.position.x;
    const my = mod.position.y;

    // Ombre du module
    ctx.shadowColor = isSelected
      ? "rgba(255, 82, 0, 0.35)"
      : "rgba(0, 0, 0, 0.45)";
    ctx.shadowBlur = isSelected ? 16 : 10;
    ctx.shadowOffsetX = 0;
    ctx.shadowOffsetY = 6;

    const modHeight = getModuleHeight(mod);

    // Fond de la carte modulaire
    ctx.fillStyle = "#1e2029";
    ctx.beginPath();
    ctx.roundRect(mx, my, MODULE_WIDTH, modHeight, 8);
    ctx.fill();

    // Bordure
    ctx.shadowColor = "transparent";
    ctx.lineWidth = isSelected ? 1.5 : 1;
    ctx.strokeStyle = isSelected ? "#ff5200" : "#303444";
    ctx.stroke();

    // En-tête du module (Drag bar)
    ctx.fillStyle = isSelected ? "#2a2d3b" : "#252834";
    ctx.beginPath();
    ctx.roundRect(mx, my, MODULE_WIDTH, 32, [8, 8, 0, 0]);
    ctx.fill();

    // Badge de type de module
    ctx.fillStyle =
      mod.kind === "Oscillator"
        ? "#ff5200"
        : mod.kind === "Gain"
          ? "#00a8ff"
          : mod.kind === "MidiIn"
            ? "#a55eea"
            : "#2ecc71";
    ctx.fillRect(mx + 10, my + 10, 4, 12);

    // Titre
    ctx.fillStyle = "#f0f2f8";
    ctx.font = "bold 12px Inter, system-ui, sans-serif";
    ctx.fillText(mod.name, mx + 20, my + 20);

    // Paramètre interactif (Slider horizontal)
    if (mod.parameters.length > 0) {
      const param = mod.parameters[0];
      ctx.fillStyle = "#8a8f9f";
      ctx.font = "11px Inter, sans-serif";
      const displayVal = param.name.includes("Freq")
        ? `${Math.round(param.current_value)} Hz`
        : param.current_value.toFixed(2);
      ctx.fillText(`${param.name} : ${displayVal}`, mx + 16, my + 118);

      // Fond de jauge
      ctx.fillStyle = "#2d313e";
      ctx.beginPath();
      ctx.roundRect(mx + 16, my + 126, MODULE_WIDTH - 32, 8, 4);
      ctx.fill();

      // Remplissage de la jauge
      const ratio = Math.max(
        0,
        Math.min(
          1,
          (param.current_value - param.min) / (param.max - param.min),
        ),
      );
      ctx.fillStyle = "#ff5200";
      ctx.beginPath();
      ctx.roundRect(
        mx + 16,
        my + 126,
        Math.max(6, ratio * (MODULE_WIDTH - 32)),
        8,
        4,
      );
      ctx.fill();
    }

    // Dessin des ports
    for (const port of mod.ports) {
      const coords = getPortCoordinates(mod, port.id);
      if (!coords) continue;

      const isHovered =
        hoveredPort?.moduleId === mod.id && hoveredPort?.portId === port.id;

      const color = getSignalColor(port.signal_type);

      // Cercle extérieur
      ctx.beginPath();
      ctx.arc(
        coords.x,
        coords.y,
        isHovered ? PORT_RADIUS + 2 : PORT_RADIUS,
        0,
        Math.PI * 2,
      );
      ctx.fillStyle = "#161820";
      ctx.fill();
      ctx.lineWidth = isHovered ? 2.5 : 2;
      ctx.strokeStyle = color;
      ctx.stroke();

      // Broche centrale
      ctx.beginPath();
      ctx.arc(coords.x, coords.y, 2.5, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();

      // Label du port
      ctx.fillStyle = isHovered ? "#ffffff" : "#7d8296";
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
