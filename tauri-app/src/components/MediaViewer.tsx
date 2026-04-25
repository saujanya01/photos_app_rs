import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";
import { getFullPath, getMediaDetail, MediaFile } from "../hooks/useTauri";

export default function MediaViewer() {
  const { viewingMedia, setViewingMedia } = useAppStore();
  const [mediaFile, setMediaFile] = useState<MediaFile | null>(null);
  const [fileSrc, setFileSrc] = useState<string>("");
  const [showInfo, setShowInfo] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (viewingMedia === null) return;

    setLoading(true);
    setError(null);

    Promise.all([getMediaDetail(viewingMedia), getFullPath(viewingMedia)])
      .then(([detail, path]) => {
        console.log("File path:", path);
        setMediaFile(detail);
        const src = convertFileSrc(path);
        console.log("Converted src:", src);
        setFileSrc(src);
      })
      .catch((e) => {
        console.error("Failed to load media:", e);
        setError(String(e));
      })
      .finally(() => setLoading(false));
  }, [viewingMedia]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") setViewingMedia(null);
      if (e.key === "i") setShowInfo((s) => !s);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setViewingMedia]);

  if (viewingMedia === null) return null;

  return (
    <div
      className="fixed inset-0 z-50 bg-black/95 flex"
      onClick={() => setViewingMedia(null)}
    >
      <div
        className="flex-1 flex items-center justify-center"
        onClick={(e) => e.stopPropagation()}
      >
        {loading && (
          <div className="text-neutral-400">Loading...</div>
        )}
        {error && (
          <div className="text-red-400 p-4 text-center">
            <p>Failed to load image</p>
            <p className="text-sm mt-2">{error}</p>
            <p className="text-sm mt-2 text-neutral-500">{fileSrc}</p>
          </div>
        )}
        {!loading && !error && mediaFile?.media_type === "video" && (
          <video
            src={fileSrc}
            controls
            className="max-h-full max-w-full"
            onError={(e) => {
              console.error("Video load error:", e);
              setError("Failed to load video");
            }}
          />
        )}
        {!loading && !error && mediaFile?.media_type !== "video" && fileSrc && (
          <img
            src={fileSrc}
            alt=""
            className="max-h-full max-w-full object-contain"
            onError={(e) => {
              console.error("Image load error:", e);
              setError(`Failed to load image from: ${fileSrc}`);
            }}
            onLoad={() => console.log("Image loaded successfully")}
          />
        )}
      </div>

      {showInfo && mediaFile && (
        <div
          className="w-80 bg-neutral-800 p-4 overflow-auto"
          onClick={(e) => e.stopPropagation()}
        >
          <h3 className="font-bold mb-4">Info</h3>
          <dl className="space-y-2 text-sm">
            <InfoRow label="Camera" value={mediaFile.camera_model} />
            <InfoRow label="Lens" value={mediaFile.lens_model} />
            <InfoRow label="Date" value={mediaFile.date_taken} />
            <InfoRow label="ISO" value={mediaFile.iso} />
            <InfoRow label="Aperture" value={mediaFile.aperture} />
            <InfoRow label="Shutter" value={mediaFile.shutter_speed} />
            <InfoRow label="Focal Length" value={mediaFile.focal_length} />
            <InfoRow label="Path" value={mediaFile.path} />
            <InfoRow label="Src URL" value={fileSrc} />
          </dl>
        </div>
      )}

      <div className="absolute top-4 right-4 text-neutral-400 text-sm">
        Press ESC to close, I for info
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string | null }) {
  if (!value) return null;
  return (
    <div>
      <dt className="text-neutral-400">{label}</dt>
      <dd className="text-white break-all">{value}</dd>
    </div>
  );
}
