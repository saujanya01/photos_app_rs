import { useEffect, useState, useCallback } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  ArrowLeft,
  ArrowRight,
  Info,
  Star,
  X,
} from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { getFullPath, getMediaDetail, MediaFile } from "../hooks/useTauri";
import { IconBtn } from "./ui";

/**
 * Closes only on explicit ESC / close button. Backdrop click is NOT a
 * close trigger — touchpads misfire constantly (claude_design_doc.md §9).
 * Keys: Esc close · I info · ←/→ navigate · J/K navigate.
 */
export default function MediaViewer() {
  const { viewingMedia, setViewingMedia } = useAppStore();
  const [mediaFile, setMediaFile] = useState<MediaFile | null>(null);
  const [fileSrc, setFileSrc] = useState<string>("");
  const [showInfo, setShowInfo] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (viewingMedia === null) return;

    setLoading(true);
    setError(null);

    Promise.all([getMediaDetail(viewingMedia), getFullPath(viewingMedia)])
      .then(([detail, path]) => {
        setMediaFile(detail);
        setFileSrc(convertFileSrc(path));
      })
      .catch((e) => {
        console.error("Failed to load media:", e);
        setError(String(e));
      })
      .finally(() => setLoading(false));
  }, [viewingMedia]);

  const close = useCallback(() => setViewingMedia(null), [setViewingMedia]);
  const toggleInfo = useCallback(() => setShowInfo((s) => !s), []);
  // Phase 10 will wire navigation to flattened TimelineGroup[]; for now
  // the keys are bound but no-op so the viewer's keyboard model is stable.
  const navigate = useCallback((_dir: -1 | 1) => {
    /* TODO: phase 10 — flatten groups, advance viewingMedia */
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
      else if (e.key === "i" || e.key === "I") toggleInfo();
      else if (e.key === "ArrowLeft" || e.key === "j" || e.key === "J")
        navigate(-1);
      else if (e.key === "ArrowRight" || e.key === "k" || e.key === "K")
        navigate(1);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [close, toggleInfo, navigate]);

  if (viewingMedia === null) return null;

  const filename = mediaFile?.path.split("/").pop() ?? "";

  return (
    <div className="fixed inset-0 z-30 flex flex-col rise surface-0">
      {/* top chrome */}
      <div
        className="h-11 flex items-center px-3 gap-3 border-b hair"
        style={{ background: "oklch(0 0 0 / 0.35)", backdropFilter: "blur(10px)" }}
      >
        <IconBtn aria-label="Close" onClick={close}>
          <X size={14} strokeWidth={1.5} />
        </IconBtn>
        <div className="flex-1 flex items-center justify-center gap-3 min-w-0">
          {mediaFile && (
            <>
              <span className="mono text-[11px] fg-3 uppercase">
                {mediaFile.extension}
              </span>
              <span className="w-1 h-1 rounded-full bg-[var(--fg-4)]" />
              <span className="mono text-[11px] fg-2 truncate max-w-[60%]">
                {filename}
              </span>
              <span className="w-1 h-1 rounded-full bg-[var(--fg-4)]" />
              <span className="mono text-[11px] fg-3">
                {fmtBytes(mediaFile.file_size_bytes)}
              </span>
            </>
          )}
        </div>
        <div className="flex items-center gap-1">
          <IconBtn aria-label="Star" disabled>
            <Star size={14} strokeWidth={1.5} />
          </IconBtn>
          <IconBtn aria-label="Info" active={showInfo} onClick={toggleInfo}>
            <Info size={14} strokeWidth={1.5} />
          </IconBtn>
        </div>
      </div>

      {/* image area + info */}
      <div className="flex-1 flex min-h-0">
        <div className="flex-1 relative flex items-center justify-center p-6">
          {loading && <div className="fg-3 text-[13px]">Loading…</div>}
          {error && (
            <div className="fg-1 p-4 text-center max-w-[480px]">
              <p className="text-[14px]">Failed to load file</p>
              <p className="text-[12px] fg-3 mt-2 break-all">{error}</p>
              <p className="text-[11px] fg-4 mt-2 break-all">{fileSrc}</p>
            </div>
          )}
          {!loading && !error && mediaFile?.media_type === "video" && (
            <video
              src={fileSrc}
              controls
              className="max-h-full max-w-full"
              onError={() => setError("Failed to load video")}
            />
          )}
          {!loading &&
            !error &&
            mediaFile?.media_type !== "video" &&
            fileSrc && (
              <img
                src={fileSrc}
                alt=""
                className="max-h-full max-w-full object-contain"
                style={{
                  boxShadow:
                    "0 30px 80px -20px oklch(0 0 0 / 0.7), 0 0 0 1px oklch(1 0 0 / 0.05)",
                }}
                onError={() => setError(`Failed to load image from: ${fileSrc}`)}
              />
            )}

          {/* arrow buttons (no-op until phase 10) */}
          <button
            onClick={() => navigate(-1)}
            disabled
            aria-label="Previous"
            className="absolute left-4 w-9 h-9 rounded-full flex items-center justify-center fg-3 hover:fg-0 disabled:opacity-30 disabled:cursor-not-allowed focus-ring"
            style={{ background: "oklch(1 0 0 / 0.06)" }}
          >
            <ArrowLeft size={16} strokeWidth={1.5} />
          </button>
          <button
            onClick={() => navigate(1)}
            disabled
            aria-label="Next"
            className="absolute right-4 w-9 h-9 rounded-full flex items-center justify-center fg-3 hover:fg-0 disabled:opacity-30 disabled:cursor-not-allowed focus-ring"
            style={{ background: "oklch(1 0 0 / 0.06)" }}
          >
            <ArrowRight size={16} strokeWidth={1.5} />
          </button>
        </div>

        {showInfo && mediaFile && <InfoPanel item={mediaFile} />}
      </div>

      <div className="absolute top-3 right-4 mono text-[10.5px] fg-4 pointer-events-none">
        ESC close · I info · ←/→ navigate
      </div>
    </div>
  );
}

function fmtBytes(b: number): string {
  if (!b) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(b) / Math.log(k));
  return `${parseFloat((b / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

function InfoPanel({ item }: { item: MediaFile }) {
  return (
    <aside className="w-[320px] border-l hair flex flex-col surface-1 overflow-y-auto">
      <Block kicker="Capture">
        <div className="text-[15px] tight fg-0">{item.camera_model ?? "—"}</div>
        <div className="mono text-[11px] fg-3 mt-0.5">
          {item.lens_model ?? "—"}
        </div>
      </Block>

      <Block kicker="Exposure">
        <div className="grid grid-cols-2 gap-y-3 gap-x-2">
          <Stat label="ISO" value={item.iso} />
          <Stat label="Aperture" value={item.aperture && `ƒ/${item.aperture}`} />
          <Stat label="Shutter" value={item.shutter_speed} />
          <Stat label="Focal" value={item.focal_length} />
        </div>
      </Block>

      <Block kicker="File">
        <Row k="Format" v={item.extension.toUpperCase()} />
        <Row k="Size" v={fmtBytes(item.file_size_bytes)} />
        {item.resolution_width && item.resolution_height && (
          <Row
            k="Dimensions"
            v={`${item.resolution_width} × ${item.resolution_height}`}
          />
        )}
        {item.duration_seconds != null && (
          <Row k="Duration" v={`${item.duration_seconds.toFixed(2)} s`} />
        )}
        <Row k="Path" v={item.path} mono />
      </Block>

      {(item.gps_latitude != null || item.gps_longitude != null) && (
        <Block kicker="Location">
          <Row
            k="Lat"
            v={item.gps_latitude != null ? item.gps_latitude.toFixed(6) : "—"}
          />
          <Row
            k="Lng"
            v={item.gps_longitude != null ? item.gps_longitude.toFixed(6) : "—"}
          />
        </Block>
      )}

      <div className="mt-auto p-3 border-t hair flex items-center justify-between">
        <div className="mono text-[10.5px] fg-4">
          non-destructive · source untouched
        </div>
        <div
          className="w-1.5 h-1.5 rounded-full"
          style={{ background: "var(--ok)" }}
        />
      </div>
    </aside>
  );
}

function Block({
  kicker,
  children,
}: {
  kicker: string;
  children: React.ReactNode;
}) {
  return (
    <div className="p-5 border-b hair">
      <div className="wide-kicker mb-3">{kicker}</div>
      {children}
    </div>
  );
}

function Stat({
  label,
  value,
}: {
  label: string;
  value: string | null | undefined;
}) {
  return (
    <div>
      <div className="mono text-[10.5px] fg-4 uppercase tracking-wider">
        {label}
      </div>
      <div className="mono text-[15px] fg-0 tabular-nums">{value ?? "—"}</div>
    </div>
  );
}

function Row({
  k,
  v,
  mono,
}: {
  k: string;
  v: string;
  mono?: boolean;
}) {
  return (
    <div className="flex items-baseline justify-between gap-4 py-1">
      <div className="mono text-[10.5px] fg-4 uppercase tracking-wider">
        {k}
      </div>
      <div
        className={`${mono ? "mono" : "tight"} text-[11.5px] fg-1 truncate max-w-[200px] text-right`}
      >
        {v}
      </div>
    </div>
  );
}
