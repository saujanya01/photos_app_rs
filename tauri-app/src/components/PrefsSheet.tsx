import { ReactNode } from "react";
import { Sheet, Kicker } from "./ui";
import { Density, usePrefs, ViewerDepth } from "../stores/prefs";

interface Props {
  open: boolean;
  onClose: () => void;
}

export default function PrefsSheet({ open, onClose }: Props) {
  const { prefs, setPref } = usePrefs();

  return (
    <Sheet open={open} onClose={onClose} title="Preferences" width={560}>
      <div className="max-h-[70vh] overflow-y-auto">
        <Section kicker="Appearance">
          <Row label="Density">
            <SegGroup<Density>
              value={prefs.density}
              onChange={(v) => setPref("density", v)}
              options={[
                ["compact", "Compact"],
                ["default", "Default"],
                ["roomy", "Roomy"],
              ]}
            />
          </Row>
          <Row label="Accent hue">
            <div className="flex items-center gap-3 flex-1">
              <input
                type="range"
                min={0}
                max={360}
                value={prefs.accentHue}
                onChange={(e) =>
                  setPref("accentHue", parseInt(e.target.value, 10))
                }
                className="flex-1"
              />
              <div
                className="w-7 h-7 rounded-[6px] border border-[var(--hair-2)]"
                style={{
                  background: `oklch(0.78 0.14 ${prefs.accentHue})`,
                }}
              />
              <span className="mono text-[11px] fg-3 w-10 text-right">
                {prefs.accentHue}°
              </span>
            </div>
          </Row>
          <Row label="Viewer depth">
            <SegGroup<ViewerDepth>
              value={prefs.viewerDepth}
              onChange={(v) => setPref("viewerDepth", v)}
              options={[
                ["black", "Pure"],
                ["dim", "Dim"],
                ["jet", "Jet"],
              ]}
            />
          </Row>
        </Section>

        <Section kicker="Library">
          <Row label="Thumbnail quality">
            <SegGroup
              value={prefs.thumbnailQuality}
              onChange={(v) => setPref("thumbnailQuality", v)}
              options={[
                ["low", "Low"],
                ["medium", "Medium"],
                ["high", "High"],
              ]}
            />
          </Row>
          <Row label="Cache budget">
            <input
              type="number"
              min={256}
              max={32768}
              step={256}
              value={prefs.cacheBudgetMb}
              onChange={(e) =>
                setPref("cacheBudgetMb", parseInt(e.target.value, 10) || 0)
              }
              className="h-7 px-2 rounded-[6px] text-[12px] tight fg-0 surface-3 border border-[var(--hair-2)] focus-ring w-28 text-right tabular-nums"
            />
            <span className="mono text-[11px] fg-3 ml-2">MB</span>
          </Row>
        </Section>

        <Section kicker="Backup">
          <Row label="Default target">
            <input
              type="text"
              value={prefs.defaultBackupTarget ?? ""}
              onChange={(e) =>
                setPref(
                  "defaultBackupTarget",
                  e.target.value.trim() || null
                )
              }
              placeholder="/Volumes/Archive SSD"
              className="flex-1 h-7 px-2 rounded-[6px] text-[12px] tight fg-0 surface-3 border border-[var(--hair-2)] focus-ring placeholder:text-[var(--fg-4)]"
            />
          </Row>
          <Row label="Conflict policy">
            <SegGroup
              value={prefs.defaultConflictPolicy}
              onChange={(v) => setPref("defaultConflictPolicy", v)}
              options={[
                ["skip", "Skip"],
                ["overwrite", "Overwrite"],
                ["rename", "Rename"],
              ]}
            />
          </Row>
          <Row label="Portable sidecar DB">
            <SegGroup
              value={prefs.sidecarDefaultOn ? "on" : "off"}
              onChange={(v) => setPref("sidecarDefaultOn", v === "on")}
              options={[
                ["on", "On"],
                ["off", "Off"],
              ]}
            />
          </Row>
        </Section>

        <Section kicker="Keyboard">
          <ul className="px-5 pb-1 space-y-1.5">
            {[
              ["⌘O", "Scan a drive"],
              ["⌘K", "Command palette"],
              ["⌘,", "Open preferences"],
              ["⌘\\", "Toggle sidebar"],
              ["I", "Toggle info panel in viewer"],
              ["←/→ · J/K", "Navigate in viewer"],
              ["Esc", "Close viewer / dismiss"],
              ["X", "Star selected (multi-select)"],
              ["T", "Tag selected (multi-select)"],
            ].map(([k, label]) => (
              <li
                key={k}
                className="flex items-center justify-between text-[12px] fg-1"
              >
                <span className="kbd">{k}</span>
                <span className="fg-2">{label}</span>
              </li>
            ))}
          </ul>
          <div className="px-5 pb-3 text-[11px] fg-4 mono">
            keybinding edits arrive in phase 10
          </div>
        </Section>

        <Section kicker="About">
          <ul className="px-5 pb-3 space-y-1.5 mono text-[11px] fg-2">
            <li>photo_app_rs 0.6.0</li>
            <li>data dir: ~/.photo_app_rs/</li>
            <li>schema: migrations 001 → 005</li>
          </ul>
        </Section>
      </div>
    </Sheet>
  );
}

function Section({
  kicker,
  children,
}: {
  kicker: string;
  children: ReactNode;
}) {
  return (
    <div className="border-b hair last:border-b-0">
      <div className="px-5 pt-4 pb-2">
        <Kicker>{kicker}</Kicker>
      </div>
      <div className="pb-2">{children}</div>
    </div>
  );
}

function Row({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="px-5 py-2 flex items-center gap-4">
      <div className="text-[12.5px] tight fg-2 w-36 shrink-0">{label}</div>
      <div className="flex-1 flex items-center">{children}</div>
    </div>
  );
}

interface SegGroupProps<V extends string> {
  value: V;
  onChange: (v: V) => void;
  options: ReadonlyArray<readonly [V, string]>;
}

function SegGroup<V extends string>({ value, onChange, options }: SegGroupProps<V>) {
  return (
    <div
      className="flex items-center gap-1 p-0.5 rounded-[6px]"
      style={{ background: "var(--ink-3)" }}
    >
      {options.map(([v, label]) => {
        const active = v === value;
        return (
          <button
            key={v}
            onClick={() => onChange(v)}
            className={`h-6 px-2.5 rounded-[4px] text-[11.5px] tight focus-ring transition-colors ${
              active ? "fg-0" : "fg-3 hover:fg-1"
            }`}
            style={{
              background: active ? "var(--ink-1)" : "transparent",
            }}
          >
            {label}
          </button>
        );
      })}
    </div>
  );
}
