import { PatchSession } from "../session";
import { ModuleInstance, SignalType } from "../types";
import {
  ActiveWire,
  getModuleHeight,
  getPortCoordinates,
  MODULE_WIDTH,
  PORT_RADIUS,
} from "./renderer";

export class GridInteractionHandler {
  private canvas: HTMLCanvasElement;
  private session: PatchSession;

  public activeWire: ActiveWire | null = null;
  public hoveredPort: { moduleId: number; portId: number } | null = null;
  public selectedModuleId: number | null = null;

  private isDraggingModule = false;
  private dragOffset = { x: 0, y: 0 };
  private isDraggingParam = false;
  private activeParamModule: ModuleInstance | null = null;

  constructor(canvas: HTMLCanvasElement, session: PatchSession) {
    this.canvas = canvas;
    this.session = session;

    this.bindEvents();
  }

  private getCanvasPos(e: MouseEvent): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    return {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
  }

  private findPortAt(
    x: number,
    y: number,
  ): {
    mod: ModuleInstance;
    portId: number;
    coords: {
      x: number;
      y: number;
      type: SignalType;
      direction: "Input" | "Output";
    };
  } | null {
    const patch = this.session.getPatch();
    for (const mod of patch.modules) {
      for (const port of mod.ports) {
        const coords = getPortCoordinates(mod, port.id);
        if (!coords) continue;
        const dist = Math.hypot(coords.x - x, coords.y - y);
        if (dist <= PORT_RADIUS + 6) {
          return { mod, portId: port.id, coords };
        }
      }
    }
    return null;
  }

  private findModuleAt(x: number, y: number): ModuleInstance | null {
    const patch = this.session.getPatch();
    for (let i = patch.modules.length - 1; i >= 0; i--) {
      const mod = patch.modules[i];
      const modHeight = getModuleHeight(mod);
      if (
        x >= mod.position.x &&
        x <= mod.position.x + MODULE_WIDTH &&
        y >= mod.position.y &&
        y <= mod.position.y + modHeight
      ) {
        return mod;
      }
    }
    return null;
  }

  private bindEvents(): void {
    this.canvas.addEventListener("mousedown", this.onMouseDown.bind(this));
    window.addEventListener("mousemove", this.onMouseMove.bind(this));
    window.addEventListener("mouseup", this.onMouseUp.bind(this));
    this.canvas.addEventListener("contextmenu", this.onContextMenu.bind(this));
  }

  private onMouseDown(e: MouseEvent): void {
    if (e.button !== 0) return;
    const { x, y } = this.getCanvasPos(e);

    // 1. Clic sur un port
    const portHit = this.findPortAt(x, y);
    if (portHit) {
      if (portHit.coords.direction === "Output") {
        this.activeWire = {
          fromModuleId: portHit.mod.id,
          fromPortId: portHit.portId,
          startX: portHit.coords.x,
          startY: portHit.coords.y,
          currentX: x,
          currentY: y,
          signalType: portHit.coords.type,
        };
        return;
      } else {
        // Clic sur une entrée déjà connectée : débrancher le câble
        this.session.disconnectCableAt(portHit.mod.id, portHit.portId);
        return;
      }
    }

    // 2. Clic sur un module
    const modHit = this.findModuleAt(x, y);
    if (modHit) {
      this.selectedModuleId = modHit.id;

      // Clic sur le curseur de paramètre
      if (y >= modHit.position.y + 115 && y <= modHit.position.y + 142) {
        this.isDraggingParam = true;
        this.activeParamModule = modHit;
        this.adjustParameter(modHit, x);
        return;
      }

      // Clic sur l'en-tête (déplacement)
      if (y <= modHit.position.y + 36) {
        this.isDraggingModule = true;
        this.dragOffset = {
          x: x - modHit.position.x,
          y: y - modHit.position.y,
        };
      }
    } else {
      this.selectedModuleId = null;
    }
  }

  private onMouseMove(e: MouseEvent): void {
    const { x, y } = this.getCanvasPos(e);

    // Détection du survol de port
    const portHit = this.findPortAt(x, y);
    this.hoveredPort = portHit
      ? { moduleId: portHit.mod.id, portId: portHit.portId }
      : null;

    // Déplacement du câble en cours
    if (this.activeWire) {
      this.activeWire.currentX = x;
      this.activeWire.currentY = y;
      return;
    }

    // Déplacement de curseur de paramètre
    if (this.isDraggingParam && this.activeParamModule) {
      this.adjustParameter(this.activeParamModule, x);
      return;
    }

    // Déplacement de module
    if (this.isDraggingModule && this.selectedModuleId !== null) {
      this.session.moveModule(
        this.selectedModuleId,
        x - this.dragOffset.x,
        y - this.dragOffset.y,
      );
    }
  }

  private onMouseUp(e: MouseEvent): void {
    const { x, y } = this.getCanvasPos(e);

    // Fin de tirage de câble
    if (this.activeWire) {
      const targetPort = this.findPortAt(x, y);
      if (targetPort && targetPort.coords.direction === "Input") {
        this.session.connectCable(
          this.activeWire.fromModuleId,
          this.activeWire.fromPortId,
          targetPort.mod.id,
          targetPort.portId,
        );
      }
      this.activeWire = null;
    }

    this.isDraggingModule = false;
    this.isDraggingParam = false;
    this.activeParamModule = null;
  }

  private onContextMenu(e: MouseEvent): void {
    e.preventDefault();
    const { x, y } = this.getCanvasPos(e);

    // Clic droit sur un port : déconnecter le câble
    const portHit = this.findPortAt(x, y);
    if (portHit) {
      this.session.disconnectCableAt(portHit.mod.id, portHit.portId);
    }
  }

  private adjustParameter(mod: ModuleInstance, mouseX: number): void {
    if (mod.parameters.length === 0) return;
    const param = mod.parameters[0];
    const trackLeft = mod.position.x + 16;
    const trackWidth = MODULE_WIDTH - 32;

    const ratio = Math.max(0, Math.min(1, (mouseX - trackLeft) / trackWidth));
    const newVal = param.min + ratio * (param.max - param.min);
    this.session.setParameter(mod.id, param.id, newVal);
  }
}
