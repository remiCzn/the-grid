import { describe, expect, it } from "bun:test";
import { BackendTransport, PatchSession } from "../src/session";
import { Patch } from "../src/types";

const mockPatch: Patch = {
  version: "1.0.0",
  name: "Test Patch",
  modules: [
    {
      id: 1,
      kind: "Oscillator",
      name: "VCO 1",
      position: { x: 50, y: 50 },
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
      position: { x: 250, y: 50 },
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
      kind: "Envelope",
      name: "ADSR 1",
      position: { x: 50, y: 250 },
      ports: [
        { id: 0, name: "Gate", signal_type: "Trigger", direction: "Input" },
        { id: 1, name: "Out", signal_type: "ControlVoltage", direction: "Output" },
      ],
      parameters: [],
    },
  ],
  cables: [],
};

class MockTransport implements BackendTransport {
  public appliedPatches: Patch[] = [];
  public paramUpdates: { moduleId: number; paramId: number; value: number }[] = [];
  public midiEvents: import("../src/types").MidiEvent[] = [];

  async applyPatch(patch: Patch): Promise<void> {
    this.appliedPatches.push(JSON.parse(JSON.stringify(patch)));
  }

  async setParameter(moduleId: number, paramId: number, value: number): Promise<void> {
    this.paramUpdates.push({ moduleId, paramId, value });
  }

  async sendMidiEvent(event: import("../src/types").MidiEvent): Promise<void> {
    this.midiEvents.push(event);
  }
}

describe("PatchSession", () => {
  it("initializes with deep copy and allows reading patch", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);
    const patch = session.getPatch();

    expect(patch.name).toBe("Test Patch");
    expect(patch.modules.length).toBe(3);
    expect(patch.cables.length).toBe(0);
  });

  it("enforces signal compatibility rules", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    expect(session.isSignalCompatible("Audio", "Audio")).toBe(true);
    expect(session.isSignalCompatible("Audio", "ControlVoltage")).toBe(true);
    expect(session.isSignalCompatible("ControlVoltage", "Audio")).toBe(true);
    expect(session.isSignalCompatible("Trigger", "Trigger")).toBe(true);
    expect(session.isSignalCompatible("Trigger", "Audio")).toBe(false);
    expect(session.isSignalCompatible("Trigger", "ControlVoltage")).toBe(false);
  });

  it("connects valid cables and triggers transport sync", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    // VCO Out (Audio) -> VCA In (Audio)
    const success = session.connectCable(1, 1, 2, 0);
    expect(success).toBe(true);
    expect(session.getPatch().cables.length).toBe(1);
    expect(session.getPatch().cables[0]).toEqual({
      id: 1,
      from_module: 1,
      from_port: 1,
      to_module: 2,
      to_port: 0,
    });
    expect(transport.appliedPatches.length).toBe(1);
  });

  it("rejects invalid cable connections", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    // Same module
    expect(session.connectCable(1, 1, 1, 0)).toBe(false);

    // Non-existent module
    expect(session.connectCable(99, 0, 1, 0)).toBe(false);

    // Incompatible signals: ADSR Out (CV) to ADSR Gate (Trigger)
    expect(session.connectCable(3, 1, 3, 0)).toBe(false);

    // Wrong direction: Input to Output
    expect(session.connectCable(2, 0, 1, 1)).toBe(false);

    expect(session.getPatch().cables.length).toBe(0);
  });

  it("replaces existing cable on the target input port", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    session.connectCable(1, 1, 2, 0);
    expect(session.getPatch().cables.length).toBe(1);

    // Connect another cable to the same input port (VCA In)
    session.connectCable(3, 1, 2, 0);
    expect(session.getPatch().cables.length).toBe(1);
    expect(session.getPatch().cables[0].from_module).toBe(3);
  });

  it("disconnects cables at a specified port", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    session.connectCable(1, 1, 2, 0);
    expect(session.getPatch().cables.length).toBe(1);

    const disconnected = session.disconnectCableAt(2, 0);
    expect(disconnected).toBe(true);
    expect(session.getPatch().cables.length).toBe(0);

    // Disconnecting empty port returns false
    expect(session.disconnectCableAt(2, 0)).toBe(false);
  });

  it("moves module and clamps minimum coordinates", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    session.moveModule(1, 300, 400);
    expect(session.findModule(1)?.position).toEqual({ x: 300, y: 400 });

    session.moveModule(1, -50, -20);
    expect(session.findModule(1)?.position).toEqual({ x: 10, y: 10 });
  });

  it("updates parameters with clamping and notifies transport", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    session.setParameter(1, 0, 880);
    expect(session.findModule(1)?.parameters[0].current_value).toBe(880);
    expect(transport.paramUpdates).toEqual([{ moduleId: 1, paramId: 0, value: 880 }]);

    // Clamping to max (20000)
    session.setParameter(1, 0, 50000);
    expect(session.findModule(1)?.parameters[0].current_value).toBe(20000);

    // Clamping to min (20)
    session.setParameter(1, 0, 5);
    expect(session.findModule(1)?.parameters[0].current_value).toBe(20);
  });

  it("notifies subscribers on mutation", () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    let callCount = 0;
    const unsubscribe = session.subscribe(() => {
      callCount++;
    });

    session.connectCable(1, 1, 2, 0);
    session.moveModule(1, 100, 100);
    session.setParameter(1, 0, 500);

    expect(callCount).toBe(3);
    unsubscribe();

    session.disconnectCableAt(2, 0);
    expect(callCount).toBe(3);
  });

  it("sends MIDI events to the transport", async () => {
    const transport = new MockTransport();
    const session = new PatchSession(mockPatch, transport);

    const event: import("../src/types").MidiEvent = {
      NoteOn: { channel: 0, note: 60, velocity: 100 },
    };
    await session.sendMidiEvent(event);

    expect(transport.midiEvents.length).toBe(1);
    expect(transport.midiEvents[0]).toEqual(event);
  });
});
