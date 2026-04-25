import { MouseEvent } from "react";
import { Ic } from "./icons";

export type VideoTreatment = "letterbox" | "play" | "pill";

export interface TileItem {
  id: number;
  thumbB64?: string | null;
  mediaType: "image" | "video";
  extension?: string | null;
  durationSeconds?: number | null;
  rating?: number;
}

interface Props {
  item: TileItem;
  selected?: boolean;
  selecting?: boolean;
  offline?: boolean;
  videoTreatment?: VideoTreatment;
  /** width / height. Defaults to 1 (square). */
  aspect?: number;
  onClick?: (e: MouseEvent<HTMLButtonElement>) => void;
  onToggle?: () => void;
}

function fmtDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${String(s).padStart(2, "0")}`;
}

export default function Tile({
  item,
  selected = false,
  selecting = false,
  offline = false,
  videoTreatment = "letterbox",
  aspect = 1,
  onClick,
  onToggle,
}: Props) {
  const aspectRatio = aspect === 1 ? "1/1" : aspect > 1 ? "3/2" : "2/3";
  const isRaw =
    item.extension &&
    ["arw", "cr2", "nef", "dng"].includes(item.extension.toLowerCase());

  return (
    <button
      onClick={onClick}
      className={
        "tile focus-ring " +
        (selected ? "selected " : "") +
        (offline ? "offline" : "")
      }
      style={{ aspectRatio }}
    >
      {item.thumbB64 ? (
        <img
          src={`data:image/jpeg;base64,${item.thumbB64}`}
          alt=""
          className="absolute inset-0 w-full h-full object-cover"
          loading="lazy"
        />
      ) : (
        <div
          className="absolute inset-0 flex items-center justify-center fg-4"
          style={{ background: "var(--ink-3)" }}
        >
          <span className="mono text-[10px]">no thumb</span>
        </div>
      )}

      {/* Subtle vignette so chips are readable on bright photos */}
      <div
        className="absolute inset-0 pointer-events-none"
        style={{
          background:
            "radial-gradient(120% 80% at 30% 20%, oklch(1 0 0 / 0.10), transparent 50%), radial-gradient(120% 80% at 80% 90%, oklch(0 0 0 / 0.25), transparent 40%)",
        }}
      />

      {/* select checkbox — visible when selecting or already selected */}
      {(selecting || selected) && (
        <span
          onClick={(e) => {
            e.stopPropagation();
            onToggle?.();
          }}
          className="absolute top-1.5 left-1.5 z-10"
          role="button"
        >
          <span
            className={"check " + (selected ? "on" : "")}
            style={{
              color: selected ? "oklch(0.16 0.006 60)" : "transparent",
            }}
          >
            {Ic.check({ s: 11 })}
          </span>
        </span>
      )}

      {/* Video letterbox bars */}
      {item.mediaType === "video" && videoTreatment === "letterbox" && (
        <>
          <div
            className="absolute top-0 left-0 right-0 h-[8%]"
            style={{ background: "oklch(0 0 0 / 0.85)" }}
          />
          <div
            className="absolute bottom-0 left-0 right-0 h-[8%]"
            style={{ background: "oklch(0 0 0 / 0.85)" }}
          />
        </>
      )}

      {/* Duration pill on videos */}
      {item.mediaType === "video" &&
        item.durationSeconds != null &&
        item.durationSeconds > 0 && (
          <div
            className="absolute bottom-1.5 right-1.5 mono text-[10px] fg-0 px-1.5 py-0.5 rounded"
            style={{
              background: "oklch(0 0 0 / 0.55)",
              backdropFilter: "blur(6px)",
            }}
          >
            {fmtDuration(item.durationSeconds)}
          </div>
        )}

      {/* Play glyph treatment */}
      {item.mediaType === "video" && videoTreatment === "play" && (
        <div className="absolute inset-0 flex items-center justify-center fg-0">
          {Ic.play({ s: 28 })}
        </div>
      )}

      {/* RAW badge */}
      {isRaw && (
        <div
          className="absolute bottom-1.5 left-1.5 mono text-[9px] fg-1 px-1 rounded"
          style={{ background: "oklch(0 0 0 / 0.55)" }}
        >
          RAW
        </div>
      )}

      {/* Star dots */}
      {(item.rating ?? 0) > 0 && (
        <div className="absolute top-1.5 right-1.5 flex gap-[1.5px]">
          {Array.from({ length: item.rating ?? 0 }).map((_, i) => (
            <div
              key={i}
              className="w-[3px] h-[3px] rounded-full"
              style={{ background: "var(--accent)" }}
            />
          ))}
        </div>
      )}

      {/* Offline chip */}
      {offline && (
        <div
          className="absolute bottom-1.5 left-1.5 px-1.5 py-0.5 rounded mono text-[9px]"
          style={{ background: "oklch(0 0 0 / 0.6)", color: "var(--fg-3)" }}
        >
          offline
        </div>
      )}
    </button>
  );
}
