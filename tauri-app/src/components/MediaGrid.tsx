import { Play } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { MediaItem } from "../hooks/useTauri";

interface Props {
  items: MediaItem[];
  tileMin?: number;
  gap?: number;
  selected?: number | null;
}

/**
 * Real EXIF aspect ratio drives tile shape — landscapes land landscape.
 * Videos get 8% letterbox bars top + bottom + a mono duration chip,
 * never a "Video" text pill (claude_design_doc.md §7).
 */
export default function MediaGrid({
  items,
  tileMin = 140,
  gap = 4,
  selected,
}: Props) {
  const { setViewingMedia } = useAppStore();

  return (
    <div
      className="grid"
      style={{
        gridTemplateColumns: `repeat(auto-fill, minmax(${tileMin}px, 1fr))`,
        gap: `${gap}px`,
      }}
    >
      {items.map((item) => (
        <Tile
          key={item.id}
          item={item}
          selected={selected === item.id}
          onClick={() => setViewingMedia(item.id)}
        />
      ))}
    </div>
  );
}

function tileAspect(item: MediaItem): string {
  const w = item.resolution_width;
  const h = item.resolution_height;
  if (!w || !h) return "1 / 1";
  const ratio = w / h;
  if (ratio > 1.2) return "3 / 2"; // landscape
  if (ratio < 0.83) return "2 / 3"; // portrait
  return "1 / 1"; // square-ish (incl. iPhone live photos that are nearly square)
}

function fmtDuration(sec: number): string {
  const total = Math.floor(sec);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${String(s).padStart(2, "0")}`;
}

interface TileProps {
  item: MediaItem;
  selected?: boolean;
  onClick?: () => void;
}

function Tile({ item, selected, onClick }: TileProps) {
  const isVideo = item.media_type === "video";
  return (
    <button
      onClick={onClick}
      className={`tile focus-ring ${selected ? "selected" : ""}`}
      style={{ aspectRatio: tileAspect(item) }}
      aria-label={`${item.media_type} ${item.camera_model ?? ""}`.trim()}
    >
      {item.thumb_small_b64 ? (
        <img
          src={`data:image/jpeg;base64,${item.thumb_small_b64}`}
          alt=""
          className="w-full h-full object-cover"
          loading="lazy"
        />
      ) : (
        <div className="w-full h-full flex items-center justify-center fg-4 text-[11px]">
          no thumb
        </div>
      )}

      {/* Video letterbox bars + duration chip + play glyph */}
      {isVideo && (
        <>
          <div
            className="absolute top-0 left-0 right-0"
            style={{ height: "8%", background: "oklch(0 0 0 / 0.85)" }}
          />
          <div
            className="absolute bottom-0 left-0 right-0"
            style={{ height: "8%", background: "oklch(0 0 0 / 0.85)" }}
          />
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
            <div
              className="rounded-full p-2"
              style={{
                background: "oklch(0 0 0 / 0.35)",
                backdropFilter: "blur(4px)",
              }}
            >
              <Play size={14} strokeWidth={1.5} className="fg-0" />
            </div>
          </div>
          {item.duration_seconds != null && (
            <div
              className="absolute bottom-1.5 right-1.5 mono text-[10px] fg-0 px-1.5 py-0.5 rounded"
              style={{
                background: "oklch(0 0 0 / 0.55)",
                backdropFilter: "blur(6px)",
              }}
            >
              {fmtDuration(item.duration_seconds)}
            </div>
          )}
        </>
      )}

      {/* RAW chip for ARW / NEF / CR2 / etc. */}
      {!isVideo && /^(arw|nef|cr2|cr3|raf|dng|orf)$/i.test(item.extension) && (
        <div
          className="absolute bottom-1.5 left-1.5 mono text-[9px] fg-0 px-1 rounded"
          style={{ background: "oklch(0 0 0 / 0.55)" }}
        >
          RAW
        </div>
      )}

      {/* Star dots top-left for rated items */}
      {item.rating > 0 && (
        <div className="absolute top-1.5 left-1.5 flex gap-[2px]">
          {Array.from({ length: item.rating }).map((_, i) => (
            <div
              key={i}
              className="w-[3px] h-[3px] rounded-full"
              style={{ background: "var(--accent)" }}
            />
          ))}
        </div>
      )}
    </button>
  );
}
