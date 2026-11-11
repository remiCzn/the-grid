import { getNoteHint, KeyboardLayout } from "./keyboard_input";
import { MidiEvent } from "./types";

export function calculateKeyVelocity(y: number, height: number): number {
  if (height <= 0) return 100;
  const ratio = Math.max(0, Math.min(1, y / height));
  const vel = Math.round(40 + ratio * 87);
  return Math.max(1, Math.min(127, vel));
}

export interface VirtualKeyboardOptions {
  octave?: number;
  layout?: KeyboardLayout;
  onMidiEvent?: (event: MidiEvent) => void;
  onOctaveChange?: (octave: number) => void;
  onLayoutChange?: (layout: KeyboardLayout) => void;
}

const OCTAVE_PATTERN: { semitone: number; name: string; isBlack: boolean }[] = [
  { semitone: 0, name: "C", isBlack: false },
  { semitone: 1, name: "C#", isBlack: true },
  { semitone: 2, name: "D", isBlack: false },
  { semitone: 3, name: "D#", isBlack: true },
  { semitone: 4, name: "E", isBlack: false },
  { semitone: 5, name: "F", isBlack: false },
  { semitone: 6, name: "F#", isBlack: true },
  { semitone: 7, name: "G", isBlack: false },
  { semitone: 8, name: "G#", isBlack: true },
  { semitone: 9, name: "A", isBlack: false },
  { semitone: 10, name: "A#", isBlack: true },
  { semitone: 11, name: "B", isBlack: false },
];

export class VirtualKeyboard {
  private container: HTMLElement;
  private octave: number;
  private layout: KeyboardLayout;
  private folded = false;
  private isPointerDown = false;
  private activeNote: number | null = null;
  private keyElements = new Map<number, HTMLElement>();
  private onMidiEvent?: (event: MidiEvent) => void;
  private onOctaveChange?: (octave: number) => void;
  private onLayoutChange?: (layout: KeyboardLayout) => void;

  private rootEl: HTMLElement | null = null;
  private keysTrackEl: HTMLElement | null = null;
  private toggleBtnEl: HTMLButtonElement | null = null;
  private octDisplayEl: HTMLElement | null = null;
  private layoutBtnEl: HTMLButtonElement | null = null;

  private boundWindowMouseUp = () => {
    if (this.isPointerDown || this.activeNote !== null) {
      this.isPointerDown = false;
      this.releaseActive();
    }
  };

  private boundWindowBlur = () => {
    this.isPointerDown = false;
    this.releaseActive();
  };

  private boundTouchMove = (e: TouchEvent) => {
    if (!this.isPointerDown || e.touches.length === 0) return;
    const touch = e.touches[0];
    const target = document.elementFromPoint(touch.clientX, touch.clientY);
    if (!target) return;
    const keyEl = (target as HTMLElement).closest(".vk-key") as HTMLElement | null;
    if (keyEl && keyEl.dataset.note) {
      const note = parseInt(keyEl.dataset.note, 10);
      if (this.activeNote !== note) {
        this.releaseActive();
        this.activeNote = note;
        this.setNoteActive(note, true);
        const rect = keyEl.getBoundingClientRect();
        const y = touch.clientY - rect.top;
        const velocity = calculateKeyVelocity(y, rect.height);
        const event: MidiEvent = {
          NoteOn: { channel: 0, note, velocity },
        };
        if (this.onMidiEvent) {
          this.onMidiEvent(event);
        }
      }
    }
  };

  constructor(container: HTMLElement, options: VirtualKeyboardOptions = {}) {
    this.container = container;
    this.octave = Math.max(1, Math.min(7, options.octave ?? 4));
    this.layout = options.layout ?? "AZERTY";
    this.onMidiEvent = options.onMidiEvent;
    this.onOctaveChange = options.onOctaveChange;
    this.onLayoutChange = options.onLayoutChange;

    this.mount();
  }

  public getOctave(): number {
    return this.octave;
  }

  public setOctave(octave: number): void {
    const clamped = Math.max(1, Math.min(7, octave));
    if (this.octave !== clamped) {
      this.releaseActive();
      this.octave = clamped;
      if (this.octDisplayEl) {
        this.octDisplayEl.textContent = `Octave ${this.octave}`;
      }
      this.renderKeys();
      if (this.onOctaveChange) {
        this.onOctaveChange(this.octave);
      }
    }
  }

  public octaveUp(): void {
    if (this.octave < 7) {
      this.setOctave(this.octave + 1);
    }
  }

  public octaveDown(): void {
    if (this.octave > 1) {
      this.setOctave(this.octave - 1);
    }
  }

  public getLayout(): KeyboardLayout {
    return this.layout;
  }

  public setLayout(layout: KeyboardLayout): void {
    if (this.layout !== layout) {
      this.layout = layout;
      if (this.layoutBtnEl) {
        this.layoutBtnEl.textContent = `Layout: ${this.layout}`;
      }
      this.renderKeys();
      if (this.onLayoutChange) {
        this.onLayoutChange(this.layout);
      }
    }
  }

  public toggleLayout(): void {
    this.setLayout(this.layout === "AZERTY" ? "QWERTY" : "AZERTY");
  }

  public isFolded(): boolean {
    return this.folded;
  }

  public setFolded(folded: boolean): void {
    this.folded = folded;
    if (this.rootEl) {
      if (this.folded) {
        this.rootEl.classList.add("folded");
      } else {
        this.rootEl.classList.remove("folded");
      }
    }
    if (this.toggleBtnEl) {
      this.toggleBtnEl.textContent = this.folded ? "⌨ Clavier ▲" : "⌨ Clavier ▼";
    }
  }

  public toggleFold(): void {
    this.setFolded(!this.folded);
  }

  public setNoteActive(note: number, active: boolean): void {
    const el = this.keyElements.get(note);
    if (el) {
      if (active) {
        el.classList.add("active");
      } else {
        el.classList.remove("active");
      }
    }
  }

  public releaseActive(): void {
    if (this.activeNote !== null) {
      const note = this.activeNote;
      this.setNoteActive(note, false);
      this.activeNote = null;
      const event: MidiEvent = {
        NoteOff: {
          channel: 0,
          note,
          velocity: 0,
        },
      };
      if (this.onMidiEvent) {
        this.onMidiEvent(event);
      }
    }
  }

  public mount(): void {
    this.unmount();

    const root = document.createElement("div");
    root.className = "virtual-keyboard-container";
    this.rootEl = root;

    // Toggle bar on top
    const toggleBar = document.createElement("div");
    toggleBar.className = "vk-toggle-bar";

    const toggleBtn = document.createElement("button");
    toggleBtn.className = "vk-toggle-btn";
    toggleBtn.textContent = "⌨ Clavier ▼";
    toggleBtn.type = "button";
    toggleBtn.addEventListener("click", () => this.toggleFold());
    this.toggleBtnEl = toggleBtn;
    toggleBar.appendChild(toggleBtn);

    root.appendChild(toggleBar);

    // Main dock panel
    const dock = document.createElement("div");
    dock.className = "vk-dock";

    // Controls header
    const header = document.createElement("div");
    header.className = "vk-header";

    const titleGroup = document.createElement("div");
    titleGroup.className = "vk-title-group";
    const title = document.createElement("span");
    title.className = "vk-title";
    title.textContent = "CLAVIER VIRTUEL";
    titleGroup.appendChild(title);
    header.appendChild(titleGroup);

    const controlsGroup = document.createElement("div");
    controlsGroup.className = "vk-controls-group";

    // Octave controls
    const octDownBtn = document.createElement("button");
    octDownBtn.className = "vk-btn";
    octDownBtn.textContent = "Oct -";
    octDownBtn.type = "button";
    octDownBtn.addEventListener("click", () => this.octaveDown());
    controlsGroup.appendChild(octDownBtn);

    const octDisplay = document.createElement("span");
    octDisplay.className = "vk-oct-display";
    octDisplay.textContent = `Octave ${this.octave}`;
    this.octDisplayEl = octDisplay;
    controlsGroup.appendChild(octDisplay);

    const octUpBtn = document.createElement("button");
    octUpBtn.className = "vk-btn";
    octUpBtn.textContent = "Oct +";
    octUpBtn.type = "button";
    octUpBtn.addEventListener("click", () => this.octaveUp());
    controlsGroup.appendChild(octUpBtn);

    // Layout toggle
    const layoutBtn = document.createElement("button");
    layoutBtn.className = "vk-btn vk-btn-layout";
    layoutBtn.textContent = `Layout: ${this.layout}`;
    layoutBtn.type = "button";
    layoutBtn.addEventListener("click", () => this.toggleLayout());
    this.layoutBtnEl = layoutBtn;
    controlsGroup.appendChild(layoutBtn);

    header.appendChild(controlsGroup);
    dock.appendChild(header);

    // Keys track
    const keysTrack = document.createElement("div");
    keysTrack.className = "vk-keys-track";
    this.keysTrackEl = keysTrack;

    // Track pointer interactions for glissando
    keysTrack.addEventListener("mouseleave", () => {
      this.isPointerDown = false;
      this.releaseActive();
    });

    keysTrack.addEventListener("touchmove", this.boundTouchMove as EventListener);

    if (typeof window !== "undefined") {
      window.addEventListener("mouseup", this.boundWindowMouseUp);
      window.addEventListener("blur", this.boundWindowBlur);
    }

    dock.appendChild(keysTrack);
    root.appendChild(dock);
    this.container.appendChild(root);

    this.renderKeys();
  }

  public unmount(): void {
    this.releaseActive();
    if (typeof window !== "undefined") {
      window.removeEventListener("mouseup", this.boundWindowMouseUp);
      window.removeEventListener("blur", this.boundWindowBlur);
    }
    if (this.keysTrackEl) {
      this.keysTrackEl.removeEventListener("touchmove", this.boundTouchMove as EventListener);
    }
    if (this.rootEl && this.rootEl.parentElement) {
      this.rootEl.parentElement.removeChild(this.rootEl);
    }
    this.rootEl = null;
    this.keysTrackEl = null;
    this.keyElements.clear();
  }

  private renderKeys(): void {
    if (!this.keysTrackEl) return;
    this.keysTrackEl.innerHTML = "";
    this.keyElements.clear();

    const baseNote = (this.octave + 1) * 12; // C4 = 60 when octave = 4

    // 2 full octaves: 14 white keys + 1 high C = 15 white keys
    // White keys rendered first, black keys rendered as absolute siblings
    const blackKeysToRender: {
      note: number;
      semitone: number;
      name: string;
      whiteIndex: number;
    }[] = [];

    let currentWhiteIndex = 0;
    for (let semitone = 0; semitone <= 24; semitone++) {
      const semitoneInOctave = semitone % 12;
      const pattern = OCTAVE_PATTERN[semitoneInOctave];

      if (!pattern.isBlack) {
        // White key
        const note = baseNote + semitone;
        const whiteKey = document.createElement("div");
        whiteKey.className = "vk-key vk-key-white";
        whiteKey.dataset.note = String(note);

        const label = document.createElement("div");
        label.className = "vk-key-label";
        const octNum = Math.floor(note / 12) - 1;
        label.textContent = `${pattern.name}${octNum}`;
        whiteKey.appendChild(label);

        const hint = getNoteHint(semitone, this.layout);
        if (hint) {
          const hintEl = document.createElement("div");
          hintEl.className = "vk-key-hint";
          hintEl.textContent = hint;
          whiteKey.appendChild(hintEl);
        }

        this.attachKeyListeners(whiteKey, note);
        this.keyElements.set(note, whiteKey);
        this.keysTrackEl.appendChild(whiteKey);

        // Check if there is an adjacent sharp (C#, D#, F#, G#, A#)
        if (semitone < 24) {
          const nextPattern = OCTAVE_PATTERN[(semitone + 1) % 12];
          if (nextPattern.isBlack) {
            blackKeysToRender.push({
              note: baseNote + semitone + 1,
              semitone: semitone + 1,
              name: nextPattern.name,
              whiteIndex: currentWhiteIndex,
            });
          }
        }

        currentWhiteIndex++;
      }
    }

    // Render black keys as sibling elements over the track
    for (const bKey of blackKeysToRender) {
      const blackKey = document.createElement("div");
      blackKey.className = "vk-key vk-key-black";
      blackKey.dataset.note = String(bKey.note);
      // Position center of black key exactly at the border between white keys
      const leftPercent = ((bKey.whiteIndex + 1) * (100 / 15)).toFixed(3);
      blackKey.style.left = `calc(${leftPercent}% - 13px)`;

      const blackLabel = document.createElement("div");
      blackLabel.className = "vk-key-label";
      const bOctNum = Math.floor(bKey.note / 12) - 1;
      blackLabel.textContent = `${bKey.name}${bOctNum}`;
      blackKey.appendChild(blackLabel);

      const blackHint = getNoteHint(bKey.semitone, this.layout);
      if (blackHint) {
        const bHintEl = document.createElement("div");
        bHintEl.className = "vk-key-hint";
        bHintEl.textContent = blackHint;
        blackKey.appendChild(bHintEl);
      }

      this.attachKeyListeners(blackKey, bKey.note);
      this.keyElements.set(bKey.note, blackKey);
      this.keysTrackEl.appendChild(blackKey);
    }
  }

  private attachKeyListeners(el: HTMLElement, note: number): void {
    const handlePress = (_e: MouseEvent | TouchEvent, clientY: number) => {
      const rect = el.getBoundingClientRect();
      const y = clientY - rect.top;
      const velocity = calculateKeyVelocity(y, rect.height);

      if (this.activeNote !== null && this.activeNote !== note) {
        this.releaseActive();
      }

      this.activeNote = note;
      this.isPointerDown = true;
      this.setNoteActive(note, true);

      const event: MidiEvent = {
        NoteOn: {
          channel: 0,
          note,
          velocity,
        },
      };
      if (this.onMidiEvent) {
        this.onMidiEvent(event);
      }
    };

    el.addEventListener("mousedown", (e) => {
      e.stopPropagation();
      handlePress(e, e.clientY);
    });

    el.addEventListener("mouseenter", (e) => {
      if (this.isPointerDown) {
        if (this.activeNote !== note) {
          handlePress(e, e.clientY);
        }
      }
    });

    el.addEventListener("touchstart", (e) => {
      e.stopPropagation();
      if (e.touches.length > 0) {
        handlePress(e, e.touches[0].clientY);
      }
    });

    el.addEventListener("touchend", (e) => {
      e.stopPropagation();
      this.isPointerDown = false;
      this.releaseActive();
    });
  }
}
