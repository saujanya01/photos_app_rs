import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export type Density = "compact" | "default" | "roomy";
export type ViewerDepth = "black" | "dim" | "jet";

export interface Preferences {
  /** Appearance */
  density: Density;
  accentHue: number;
  viewerDepth: ViewerDepth;

  /** Library */
  thumbnailQuality: "low" | "medium" | "high";
  cacheBudgetMb: number;

  /** Backup */
  defaultBackupTarget: string | null;
  defaultConflictPolicy: "skip" | "overwrite" | "rename";
  sidecarDefaultOn: boolean;

  /** Misc */
  schemaVersion: 1;
}

const DEFAULTS: Preferences = {
  density: "default",
  accentHue: 60,
  viewerDepth: "dim",
  thumbnailQuality: "medium",
  cacheBudgetMb: 2048,
  defaultBackupTarget: null,
  defaultConflictPolicy: "skip",
  sidecarDefaultOn: true,
  schemaVersion: 1,
};

interface PrefsState {
  prefs: Preferences;
  loaded: boolean;
  setPref: <K extends keyof Preferences>(key: K, value: Preferences[K]) => void;
  load: () => Promise<void>;
}

export const usePrefs = create<PrefsState>((set, get) => ({
  prefs: DEFAULTS,
  loaded: false,
  setPref: (key, value) => {
    const next = { ...get().prefs, [key]: value };
    set({ prefs: next });
    // Fire-and-forget save; preference UI is forgiving.
    invoke("write_preferences", { json: JSON.stringify(next) }).catch((e) =>
      console.error("write_preferences failed:", e)
    );
    applyAppearance(next);
  },
  load: async () => {
    try {
      const raw = await invoke<string>("read_preferences");
      const parsed = JSON.parse(raw || "{}") as Partial<Preferences>;
      const merged: Preferences = { ...DEFAULTS, ...parsed, schemaVersion: 1 };
      set({ prefs: merged, loaded: true });
      applyAppearance(merged);
    } catch (e) {
      console.error("read_preferences failed, using defaults:", e);
      set({ loaded: true });
      applyAppearance(DEFAULTS);
    }
  },
}));

/** Apply appearance prefs to CSS vars on :root. */
export function applyAppearance(p: Preferences) {
  const root = document.documentElement;
  root.style.setProperty("--accent-h", String(p.accentHue));
}
