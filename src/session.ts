import { invoke } from "@tauri-apps/api/core";
import { Cable, ModuleInstance, Patch, PortDefinition, SignalType } from "./types";

export interface BackendTransport {
  applyPatch(patch: Patch): Promise<void>;
  setParameter(moduleId: number, paramId: number, value: number): Promise<void>;
}

export class TauriBackendTransport implements BackendTransport {
  async applyPatch(patch: Patch): Promise<void> {
    try {
      await invoke("apply_patch", { patch });
    } catch (err) {
      console.warn("Tauri apply_patch warning:", err);
    }
  }

  async setParameter(moduleId: number, paramId: number, value: number): Promise<void> {
    try {
      await invoke("set_parameter", { moduleId, paramId, value });
    } catch (err) {
      console.warn("Tauri set_parameter warning:", err);
    }
  }
}

export class PatchSession {
  private patch: Patch;
  private transport: BackendTransport;
  private nextCableId = 1;
  private listeners: Set<(patch: Patch) => void> = new Set();

  constructor(initialPatch: Patch, transport?: BackendTransport) {
    this.patch = JSON.parse(JSON.stringify(initialPatch));
    this.transport = transport || new TauriBackendTransport();

    for (const cable of this.patch.cables) {
      if (cable.id >= this.nextCableId) {
        this.nextCableId = cable.id + 1;
      }
    }
  }

  public getPatch(): Patch {
    return this.patch;
  }

  public findModule(id: number): ModuleInstance | undefined {
    return this.patch.modules.find((m) => m.id === id);
  }

  public findPort(moduleId: number, portId: number): PortDefinition | undefined {
    const mod = this.findModule(moduleId);
    if (!mod) return undefined;
    return mod.ports.find((p) => p.id === portId);
  }

  public isSignalCompatible(src: SignalType, dst: SignalType): boolean {
    if (src === dst) return true;
    return (
      (src === "Audio" && dst === "ControlVoltage") ||
      (src === "ControlVoltage" && dst === "Audio")
    );
  }

  public connectCable(
    fromModuleId: number,
    fromPortId: number,
    toModuleId: number,
    toPortId: number,
  ): boolean {
    if (fromModuleId === toModuleId) {
      return false;
    }

    const fromPort = this.findPort(fromModuleId, fromPortId);
    const toPort = this.findPort(toModuleId, toPortId);

    if (!fromPort || !toPort) {
      return false;
    }

    if (fromPort.direction !== "Output" || toPort.direction !== "Input") {
      return false;
    }

    if (!this.isSignalCompatible(fromPort.signal_type, toPort.signal_type)) {
      return false;
    }

    // Supprimer un câble existant connecté à cette entrée
    this.patch.cables = this.patch.cables.filter(
      (c) => !(c.to_module === toModuleId && c.to_port === toPortId),
    );

    const newCable: Cable = {
      id: this.nextCableId++,
      from_module: fromModuleId,
      from_port: fromPortId,
      to_module: toModuleId,
      to_port: toPortId,
    };

    this.patch.cables.push(newCable);
    this.notifyAndSync();
    return true;
  }

  public disconnectCableAt(moduleId: number, portId: number): boolean {
    const initialLen = this.patch.cables.length;
    this.patch.cables = this.patch.cables.filter(
      (c) =>
        !(
          (c.to_module === moduleId && c.to_port === portId) ||
          (c.from_module === moduleId && c.from_port === portId)
        ),
    );

    if (this.patch.cables.length !== initialLen) {
      this.notifyAndSync();
      return true;
    }
    return false;
  }

  public moveModule(moduleId: number, x: number, y: number): void {
    const mod = this.findModule(moduleId);
    if (!mod) return;

    mod.position.x = Math.max(10, x);
    mod.position.y = Math.max(10, y);
    this.notifyListeners();
  }

  public setParameter(moduleId: number, paramId: number, value: number): void {
    const mod = this.findModule(moduleId);
    if (!mod) return;

    const param = mod.parameters.find((p) => p.id === paramId);
    if (!param) return;

    const clamped = Math.max(param.min, Math.min(param.max, value));
    param.current_value = clamped;

    this.transport.setParameter(moduleId, paramId, clamped);
    this.notifyListeners();
  }

  public async loadPatch(newPatch: Patch): Promise<void> {
    this.patch = JSON.parse(JSON.stringify(newPatch));
    for (const cable of this.patch.cables) {
      if (cable.id >= this.nextCableId) {
        this.nextCableId = cable.id + 1;
      }
    }
    await this.notifyAndSync();
  }

  public async sync(): Promise<void> {
    await this.transport.applyPatch(this.patch);
  }

  public subscribe(listener: (patch: Patch) => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notifyListeners(): void {
    for (const listener of this.listeners) {
      listener(this.patch);
    }
  }

  private async notifyAndSync(): Promise<void> {
    this.notifyListeners();
    await this.sync();
  }
}
