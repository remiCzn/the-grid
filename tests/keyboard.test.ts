import { describe, expect, it } from "bun:test";
import { ComputerKeyboard } from "../src/keyboard_input";
import { MidiEvent } from "../src/types";
import { calculateKeyVelocity, VirtualKeyboard } from "../src/virtual_keyboard";

// Mock DOM helper for headless test execution
class MockClassList {
  private classes = new Set<string>();
  add(cls: string) {
    this.classes.add(cls);
  }
  remove(cls: string) {
    this.classes.delete(cls);
  }
  contains(cls: string) {
    return this.classes.has(cls);
  }
}

class MockElement {
  public tagName: string;
  public className = "";
  public classList = new MockClassList();
  public dataset: Record<string, string> = {};
  public textContent = "";
  public children: MockElement[] = [];
  public parentElement: MockElement | null = null;
  public innerHTML = "";
  public listeners = new Map<string, ((event: any) => void)[]>();
  public rect = { top: 0, left: 0, width: 40, height: 120 };
  public style: Record<string, string> = {};

  constructor(tagName: string) {
    this.tagName = tagName;
  }

  appendChild(child: MockElement) {
    child.parentElement = this;
    this.children.push(child);
    return child;
  }

  removeChild(child: MockElement) {
    const idx = this.children.indexOf(child);
    if (idx !== -1) {
      this.children.splice(idx, 1);
      child.parentElement = null;
    }
    return child;
  }

  addEventListener(event: string, handler: (e: any) => void) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(handler);
  }

  removeEventListener(event: string, handler: (e: any) => void) {
    const list = this.listeners.get(event);
    if (list) {
      const idx = list.indexOf(handler);
      if (idx !== -1) list.splice(idx, 1);
    }
  }

  dispatchEvent(event: string, payload: any = {}) {
    const list = this.listeners.get(event);
    if (list) {
      const eventObj = {
        ...payload,
        stopPropagation: () => {},
        preventDefault: () => {},
      };
      for (const h of list) {
        h(eventObj);
      }
    }
  }

  getBoundingClientRect() {
    return {
      top: this.rect.top,
      left: this.rect.left,
      width: this.rect.width,
      height: this.rect.height,
      bottom: this.rect.top + this.rect.height,
      right: this.rect.left + this.rect.width,
    };
  }

  closest(selector: string): MockElement | null {
    if (selector.startsWith(".") && this.classList.contains(selector.slice(1))) {
      return this;
    }
    if (this.className.includes(selector.replace(".", ""))) {
      return this;
    }
    if (this.parentElement) {
      return this.parentElement.closest(selector);
    }
    return null;
  }
}

function setupMockDom() {
  const windowListeners = new Map<string, ((e: any) => void)[]>();
  let hitElement: MockElement | null = null;

  const mockWindow = {
    addEventListener: (event: string, handler: (e: any) => void) => {
      if (!windowListeners.has(event)) windowListeners.set(event, []);
      windowListeners.get(event)!.push(handler);
    },
    removeEventListener: (event: string, handler: (e: any) => void) => {
      const list = windowListeners.get(event);
      if (list) {
        const idx = list.indexOf(handler);
        if (idx !== -1) list.splice(idx, 1);
      }
    },
    dispatchEvent: (event: string, payload: any = {}) => {
      const list = windowListeners.get(event);
      if (list) {
        for (const h of list) h(payload);
      }
    },
    getListenerCount: (event: string) => {
      return windowListeners.get(event)?.length ?? 0;
    },
  };

  const mockDocument = {
    createElement: (tag: string) => new MockElement(tag),
    setHitElement: (el: MockElement | null) => {
      hitElement = el;
    },
    elementFromPoint: (_x: number, _y: number) => hitElement,
  };

  (globalThis as any).document = mockDocument;
  (globalThis as any).window = mockWindow;

  return { mockWindow, mockDocument };
}

describe("ComputerKeyboard Input", () => {
  it("emits NoteOn 60 with velocity 100 on key 'A' press in AZERTY", () => {
    const emitted: MidiEvent[] = [];
    const kb = new ComputerKeyboard({
      layout: "AZERTY",
      octave: 4,
      onMidiEvent: (evt) => emitted.push(evt),
    });

    const evt = kb.handleKeyDown({ key: "A" });
    expect(evt).toEqual({
      NoteOn: { channel: 0, note: 60, velocity: 100 },
    });
    expect(emitted.length).toBe(1);
    expect(emitted[0]).toEqual({
      NoteOn: { channel: 0, note: 60, velocity: 100 },
    });
    expect(kb.getActiveNotes()).toEqual([60]);
    expect(kb.isKeyPressed("a")).toBe(true);
  });

  it("emits NoteOff 60 with velocity 0 on key 'A' release", () => {
    const emitted: MidiEvent[] = [];
    const kb = new ComputerKeyboard({
      layout: "AZERTY",
      octave: 4,
      onMidiEvent: (evt) => emitted.push(evt),
    });

    kb.handleKeyDown({ key: "A" });
    const offEvt = kb.handleKeyUp({ key: "A" });

    expect(offEvt).toEqual({
      NoteOff: { channel: 0, note: 60, velocity: 0 },
    });
    expect(emitted.length).toBe(2);
    expect(emitted[1]).toEqual({
      NoteOff: { channel: 0, note: 60, velocity: 0 },
    });
    expect(kb.getActiveNotes()).toEqual([]);
    expect(kb.isKeyPressed("a")).toBe(false);
  });

  it("ignores repeat events and prevents key repeat avalanches", () => {
    const emitted: MidiEvent[] = [];
    const kb = new ComputerKeyboard({
      layout: "AZERTY",
      octave: 4,
      onMidiEvent: (evt) => emitted.push(evt),
    });

    // First keydown
    const first = kb.handleKeyDown({ key: "A", repeat: false });
    expect(first).not.toBeNull();

    // Repeated keydown events while holding key
    const repeated1 = kb.handleKeyDown({ key: "A", repeat: true });
    expect(repeated1).toBeNull();

    const repeated2 = kb.handleKeyDown({ key: "A", repeat: false });
    expect(repeated2).toBeNull();

    expect(emitted.length).toBe(1);
  });

  it("transposes note pitch by 12 semitones on octave up", () => {
    const kb = new ComputerKeyboard({
      layout: "AZERTY",
      octave: 4,
    });

    expect(kb.getOctave()).toBe(4);
    const evtOct4 = kb.handleKeyDown({ key: "A" });
    expect(evtOct4).toEqual({
      NoteOn: { channel: 0, note: 60, velocity: 100 },
    });
    kb.handleKeyUp({ key: "A" });

    // Transpose up to octave 5
    kb.octaveUp();
    expect(kb.getOctave()).toBe(5);

    const evtOct5 = kb.handleKeyDown({ key: "A" });
    expect(evtOct5).toEqual({
      NoteOn: { channel: 0, note: 72, velocity: 100 },
    });
  });

  it("supports keyboard shortcuts 1 and 2 for octave transposition in AZERTY", () => {
    let notifiedOctave: number | null = null;
    const kb = new ComputerKeyboard({
      layout: "AZERTY",
      octave: 4,
      onOctaveChange: (oct) => {
        notifiedOctave = oct;
      },
    });

    // Press '2' transposes octave up
    kb.handleKeyDown({ key: "2" });
    expect(kb.getOctave()).toBe(5);
    expect(notifiedOctave).toBe(5);

    // Press '1' transposes octave down
    kb.handleKeyDown({ key: "1" });
    expect(kb.getOctave()).toBe(4);
    expect(notifiedOctave).toBe(4);
  });

  it("clamps octave between 1 and 7", () => {
    const kb = new ComputerKeyboard({ octave: 1 });
    kb.octaveDown();
    expect(kb.getOctave()).toBe(1);

    kb.setOctave(10);
    expect(kb.getOctave()).toBe(7);

    kb.octaveUp();
    expect(kb.getOctave()).toBe(7);
  });

  it("maps QWERTY layout keys properly", () => {
    const kb = new ComputerKeyboard({
      layout: "QWERTY",
      octave: 4,
    });

    // Q is C4 (60)
    const evtQ = kb.handleKeyDown({ key: "q" });
    expect(evtQ).toEqual({
      NoteOn: { channel: 0, note: 60, velocity: 100 },
    });
    kb.handleKeyUp({ key: "q" });

    // W is D4 (62)
    const evtW = kb.handleKeyDown({ key: "w" });
    expect(evtW).toEqual({
      NoteOn: { channel: 0, note: 62, velocity: 100 },
    });
    kb.handleKeyUp({ key: "w" });

    // Lower row: Z is C3 (48)
    const evtZ = kb.handleKeyDown({ key: "z" });
    expect(evtZ).toEqual({
      NoteOn: { channel: 0, note: 48, velocity: 100 },
    });
    kb.handleKeyUp({ key: "z" });
  });

  it("releases all active notes cleanly on releaseAll()", () => {
    const emitted: MidiEvent[] = [];
    const kb = new ComputerKeyboard({
      layout: "AZERTY",
      octave: 4,
      onMidiEvent: (evt) => emitted.push(evt),
    });

    kb.handleKeyDown({ key: "a" });
    kb.handleKeyDown({ key: "z" });
    expect(kb.getActiveNotes().length).toBe(2);

    const released = kb.releaseAll();
    expect(released.length).toBe(2);
    expect(kb.getActiveNotes().length).toBe(0);
    expect(emitted.length).toBe(4); // 2 NoteOn + 2 NoteOff
  });
});

describe("VirtualKeyboard Velocity Formula", () => {
  it("calculates 40 at the top of the key and 127 at the bottom", () => {
    const height = 120;
    // Top (y = 0)
    expect(calculateKeyVelocity(0, height)).toBe(40);

    // Bottom (y = height)
    expect(calculateKeyVelocity(height, height)).toBe(127);

    // Middle (y = height / 2) -> 40 + 0.5 * 87 = 83.5 -> 84
    expect(calculateKeyVelocity(height / 2, height)).toBe(84);
  });

  it("clamps velocity to [1, 127] on out-of-bounds inputs", () => {
    expect(calculateKeyVelocity(-10, 100)).toBe(40);
    expect(calculateKeyVelocity(200, 100)).toBe(127);
    expect(calculateKeyVelocity(0, 0)).toBe(100);
  });
});

describe("VirtualKeyboard Component & Glissando", () => {
  it("renders 2 octaves with 15 white keys and 10 black sibling keys (25 keys total)", () => {
    setupMockDom();
    const container = new MockElement("div") as any;
    new VirtualKeyboard(container, { octave: 4 });

    const keysTrack = container.children[0].children[1].children[1];
    expect(keysTrack).toBeDefined();

    // 15 white keys + 10 black keys = 25 keys
    expect(keysTrack.children.length).toBe(25);

    // First white key is C4 (note 60)
    const firstWhite = keysTrack.children[0];
    expect(firstWhite.dataset.note).toBe("60");

    // Sibling black keys are present with correct data-note
    const blackKeys = keysTrack.children.filter((c: MockElement) =>
      c.className.includes("vk-key-black"),
    );
    expect(blackKeys.length).toBe(10);
    expect(blackKeys[0].dataset.note).toBe("61"); // C#4
  });

  it("supports monophonic glissando from C4 to D4 without stuck notes", () => {
    setupMockDom();
    const container = new MockElement("div") as any;
    const emitted: MidiEvent[] = [];
    new VirtualKeyboard(container, {
      octave: 4,
      onMidiEvent: (evt) => emitted.push(evt),
    });

    const keysTrack = container.children[0].children[1].children[1];
    const keyC4 = keysTrack.children[0]; // note 60
    const keyD4 = keysTrack.children[1]; // note 62

    // 1. Mousedown on C4 (at bottom of key: y = 120 -> velocity 127)
    keyC4.dispatchEvent("mousedown", { clientY: 120 });
    expect(emitted.length).toBe(1);
    expect(emitted[0]).toEqual({
      NoteOn: { channel: 0, note: 60, velocity: 127 },
    });
    expect(keyC4.classList.contains("active")).toBe(true);

    // 2. Glissando to D4: mouseenter on D4 while mouse is held
    keyD4.dispatchEvent("mouseenter", { clientY: 60 });
    expect(emitted.length).toBe(3);
    // NoteOff for C4
    expect(emitted[1]).toEqual({
      NoteOff: { channel: 0, note: 60, velocity: 0 },
    });
    // NoteOn for D4
    expect(emitted[2]).toEqual({
      NoteOn: { channel: 0, note: 62, velocity: 84 },
    });
    expect(keyC4.classList.contains("active")).toBe(false);
    expect(keyD4.classList.contains("active")).toBe(true);

    // 3. Release mouse on window
    const win = (globalThis as any).window;
    win.dispatchEvent("mouseup");
    expect(emitted.length).toBe(4);
    expect(emitted[3]).toEqual({
      NoteOff: { channel: 0, note: 62, velocity: 0 },
    });
    expect(keyD4.classList.contains("active")).toBe(false);
  });

  it("supports tactile glissando via touchmove across keys", () => {
    const { mockDocument } = setupMockDom();
    const container = new MockElement("div") as any;
    const emitted: MidiEvent[] = [];
    new VirtualKeyboard(container, {
      octave: 4,
      onMidiEvent: (evt) => emitted.push(evt),
    });

    const keysTrack = container.children[0].children[1].children[1];
    const keyC4 = keysTrack.children[0]; // note 60
    const keyD4 = keysTrack.children[1]; // note 62

    // Touch start on C4
    keyC4.dispatchEvent("touchstart", {
      touches: [{ clientY: 120, clientX: 20 }],
    });
    expect(emitted.length).toBe(1);
    expect(emitted[0]).toEqual({
      NoteOn: { channel: 0, note: 60, velocity: 127 },
    });

    // Touch move over D4
    mockDocument.setHitElement(keyD4);
    keysTrack.dispatchEvent("touchmove", {
      touches: [{ clientY: 60, clientX: 60 }],
    });

    expect(emitted.length).toBe(3);
    expect(emitted[1]).toEqual({
      NoteOff: { channel: 0, note: 60, velocity: 0 },
    });
    expect(emitted[2]).toEqual({
      NoteOn: { channel: 0, note: 62, velocity: 84 },
    });

    // Touch end
    keyD4.dispatchEvent("touchend");
    expect(emitted.length).toBe(4);
    expect(emitted[3]).toEqual({
      NoteOff: { channel: 0, note: 62, velocity: 0 },
    });
  });

  it("releases active note on mouseleave from keyboard track", () => {
    setupMockDom();
    const container = new MockElement("div") as any;
    const emitted: MidiEvent[] = [];
    new VirtualKeyboard(container, {
      octave: 4,
      onMidiEvent: (evt) => emitted.push(evt),
    });

    const keysTrack = container.children[0].children[1].children[1];
    const keyC4 = keysTrack.children[0];

    // Mousedown on C4
    keyC4.dispatchEvent("mousedown", { clientY: 120 });
    expect(emitted.length).toBe(1);

    // Cursor leaves track
    keysTrack.dispatchEvent("mouseleave");
    expect(emitted.length).toBe(2);
    expect(emitted[1]).toEqual({
      NoteOff: { channel: 0, note: 60, velocity: 0 },
    });
  });

  it("toggles fold/unfold state cleanly", () => {
    setupMockDom();
    const container = new MockElement("div") as any;
    const vk = new VirtualKeyboard(container, { octave: 4 });

    expect(vk.isFolded()).toBe(false);
    vk.toggleFold();
    expect(vk.isFolded()).toBe(true);
    expect(container.children[0].classList.contains("folded")).toBe(true);

    vk.toggleFold();
    expect(vk.isFolded()).toBe(false);
    expect(container.children[0].classList.contains("folded")).toBe(false);
  });

  it("lights up key when setNoteActive is called", () => {
    setupMockDom();
    const container = new MockElement("div") as any;
    const vk = new VirtualKeyboard(container, { octave: 4 });

    const keysTrack = container.children[0].children[1].children[1];
    const keyC4 = keysTrack.children[0];

    vk.setNoteActive(60, true);
    expect(keyC4.classList.contains("active")).toBe(true);

    vk.setNoteActive(60, false);
    expect(keyC4.classList.contains("active")).toBe(false);
  });

  it("unmount cleans up window event listeners without leaks", () => {
    const { mockWindow } = setupMockDom();
    const container = new MockElement("div") as any;
    const vk = new VirtualKeyboard(container, { octave: 4 });

    expect(mockWindow.getListenerCount("mouseup")).toBe(1);
    expect(mockWindow.getListenerCount("blur")).toBe(1);

    vk.unmount();

    expect(mockWindow.getListenerCount("mouseup")).toBe(0);
    expect(mockWindow.getListenerCount("blur")).toBe(0);
  });
});
