import { Cable, ModuleInstance, Patch, SignalType } from "../types";
import {
  ActiveWire,
  getPortCoordinates,
  MODULE_HEIGHT,
  MODULE_WIDTH,
  PORT_RADIUS,
} from "./renderer";

export interface InteractionCallbacks {
  onPatchModified: () => void;
  onParameterChange: (moduleId: number, paramId: number, value: f32) => void;
}

// alias f32 -> number
type f32 = number;

export class GridInteractionHandler {
  private canvas: HTMLCanvasElement;
  private patch: Patch;
  private callbacks: InteractionCallbacks;

  public activeWire: ActiveWire | null = null;
  public hoveredPort: { moduleId: number; portId: number } | null = null;
  public selectedModuleId: number | null = null;

  private isDraggingModule = false;
  private dragOffset = { x: 0, y: 0 };
  private isDraggingParam = false;
  private activeParamModule: ModuleInstance | null = null;

  constructor(
    canvas: HTMLCanvasElement,
    patch: Patch,
    callbacks: InteractionCallbacks
  ) {
    this.canvas = canvas;
    this.patch = patch;
    this.callbacks = callbacks;

    this.bindEvents();
  }

  public updatePatch(patch: Patch): void {
    this.patch = patch;
  }

  private getCanvasPos(e: MouseEvent): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    return {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
  }

  private findPortAt(x: number, y: number): {
    mod: ModuleInstance;
    portId: number;
    coords: { x: number; y: number; type: SignalType; direction: "Input" | "Output" };
  } | null {
    for (const mod of this.patch.modules) {
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
    // Parcourir à l'envers pour sélectionner le module au premier plan
    for (let i = this.patch.modules.length - 1; i >= 0; i--) {
      const mod = this.patch.modules[i];
      if (
        x >= mod.position.x &&
        x <= mod.position.x + MODULE_WIDTH &&
        y >= mod.position.y &&
        y <= mod.position.y + MODULE_HEIGHT
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
    if (e.button !== 0) return; // Clic gauche uniquement
    const { x, y } = this.getCanvasPos(e);

    // 1. Clic sur un port ?
    const portHit = this.findPortAt(x, y);
    if (portHit) {
      if (portHit.coords.direction === "Output") {
        // Démarrage du tirage de câble depuis une sortie
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
        // Clic sur une entrée déjà connectée : débrancher le câble existant
        const existingIdx = this.patch.cables.findIndex(
          (c) => c.to_module === portHit.mod.id && c.to_port === portHit.portId
        );
        if (existingIdx !== -1) {
          this.patch.cables.splice(existingIdx, 1);
          this.callbacks.onPatchModified();
          return;
        }
      }
    }

    // 2. Clic sur un module ?
    const modHit = this.findModuleAt(x, y);
    if (modHit) {
      this.selectedModuleId = modHit.id;

      // Clic sur la jauge de paramètre ? (zone y entre position.y + 115 et + 140)
      if (y >= modHit.position.y + 115 && y <= modHit.position.y + 142) {
        this.isDraggingParam = true;
        this.activeParamModule = modHit;
        this.adjustParameter(modHit, x);
        return;
      }

      // Clic sur l'en-tête (zone de drag)
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
    this.hoveredPort = portHit ? { moduleId: portHit.mod.id, portId: portHit.portId } : null;

    // Déplacement de câble en cours
    if (this.activeWire) {
      this.activeWire.currentX = x;
      this.activeWire.currentY = y;
      return;
    }

    // Déplacement de slider paramètre
    if (this.isDraggingParam && this.activeParamModule) {
      this.adjustParameter(this.activeParamModule, x);
      return;
    }

    // Déplacement de module
    if (this.isDraggingModule && this.selectedModuleId !== null) {
      const mod = this.patch.modules.find((m) => m.id === this.selectedModuleId);
      if (mod) {
        mod.position.x = Math.max(10, x - this.dragOffset.x);
        mod.position.y = Math.max(10, y - this.dragOffset.y);
      }
    }
  }

  private onMouseUp(e: MouseEvent): void {
    const { x, y } = this.getCanvasPos(e);

    // Fin de tirage de câble
    if (this.activeWire) {
      const targetPort = this.findPortAt(x, y);
      if (
        targetPort &&
        targetPort.coords.direction === "Input" &&
        targetPort.mod.id !== this.activeWire.fromModuleId
      ) {
        // Vérifier la compatibilité des signaux
        const srcType = this.activeWire.signalType;
        const dstType = targetPort.coords.type;
        const isCompatible =
          srcType === dstType ||
          (srcType === "Audio" && dstType === "ControlVoltage") ||
          (srcType === "ControlVoltage" && dstType === "Audio");

        if (isCompatible) {
          // Supprimer un éventuel câble existant sur cette entrée
          this.patch.cables = this.patch.cables.filter(
            (c) => !(c.to_module === targetPort.mod.id && c.to_port === targetPort.portId)
          );

          // Ajouter le nouveau câble
          const newCable: Cable = {
            id: Date.now() % 1000000,
            from_module: this.activeWire.fromModuleId,
            from_port: this.activeWire.fromPortId,
            to_module: targetPort.mod.id,
            to_port: targetPort.portId,
          };
          this.patch.cables.push(newCable);

          this.callbacks.onPatchModified();
        }
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

    // Clic droit sur un port d'entrée : déconnecter le câble
    const portHit = this.findPortAt(x, y);
    if (portHit) {
      const lenBefore = this.patch.cables.length;
      this.patch.cables = this.patch.cables.filter(
        (c) =>
          !(
            (c.to_module === portHit.mod.id && c.to_port === portHit.portId) ||
            (c.from_module === portHit.mod.id && c.from_port === portHit.portId)
          )
      );
      if (this.patch.cables.length !== lenBefore) {
        this.callbacks.onPatchModified();
      }
    }
  }

  private adjustParameter(mod: ModuleInstance, mouseX: number): void {
    if (mod.parameters.length === 0) return;
    const param = mod.parameters[0];
    const trackLeft = mod.position.x + 16;
    const trackWidth = MODULE_WIDTH - 32;

    const ratio = Math.max(0, Math.min(1, (mouseX - trackLeft) / trackWidth));
    const newVal = param.min + ratio * (param.max - param.min);
    param.current_value = newVal;

    this.callbacks.onParameterChange(mod.id, param.id, newVal);
  }
}
