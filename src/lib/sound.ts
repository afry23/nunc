/**
 * Synthetisierte Signaltöne (Web Audio), keine Audiodateien nötig.
 * Namen wie in den Einstellungen: `bell` (Glocke), `ding`, `digital`.
 */

export const SOUND_NAMES = ["bell", "ding", "digital"] as const;
export type SoundName = (typeof SOUND_NAMES)[number];

export const SOUND_LABELS: Record<SoundName, string> = {
  bell: "Glocke",
  ding: "Ding",
  digital: "Digital",
};

type Tone = {
  freq: number;
  gain: number;
  attack: number;
  decay: number;
  delay?: number;
  type?: OscillatorType;
};

/** Töne eines Signals (Beschreibung ohne Audio-API, damit testbar). */
export function toneRecipe(name: string): Tone[] {
  switch (name) {
    case "bell":
      return [
        { freq: 440, gain: 0.55, attack: 0.01, decay: 2.8 },
        { freq: 880, gain: 0.28, attack: 0.01, decay: 2.0 },
        { freq: 1320, gain: 0.14, attack: 0.01, decay: 1.3 },
        { freq: 1760, gain: 0.07, attack: 0.01, decay: 0.9 },
      ];
    case "digital":
      return [0, 0.18, 0.36].map((delay) => ({
        freq: 1568,
        gain: 0.35,
        attack: 0.005,
        decay: 0.12,
        delay,
        type: "square" as const,
      }));
    default:
      return [
        { freq: 880, gain: 0.5, attack: 0.005, decay: 1.4 },
        { freq: 1760, gain: 0.2, attack: 0.005, decay: 0.9 },
      ];
  }
}

let context: AudioContext | null = null;

/** Spielt ein Signal mit Lautstärke 0–100. Best effort: Fehler blockieren nie die Oberfläche. */
export function playSound(name: string, volume: number): void {
  try {
    context ??= new AudioContext();
    if (context.state === "suspended") void context.resume();
    const level = Math.max(0, Math.min(100, volume)) / 100;
    const now = context.currentTime;
    for (const tone of toneRecipe(name)) {
      const start = now + (tone.delay ?? 0);
      const osc = context.createOscillator();
      const envelope = context.createGain();
      osc.connect(envelope);
      envelope.connect(context.destination);
      osc.type = tone.type ?? "sine";
      osc.frequency.setValueAtTime(tone.freq, start);
      envelope.gain.setValueAtTime(0, start);
      envelope.gain.linearRampToValueAtTime(
        tone.gain * level,
        start + tone.attack,
      );
      envelope.gain.exponentialRampToValueAtTime(
        0.0001,
        start + tone.attack + tone.decay,
      );
      osc.start(start);
      osc.stop(start + tone.attack + tone.decay + 0.05);
    }
  } catch {
    // kein Audio verfügbar
  }
}
