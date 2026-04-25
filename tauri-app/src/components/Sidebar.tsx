import { open } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "../stores/appStore";
import { scanDirectory, getStats } from "../hooks/useTauri";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

export default function Sidebar() {
  const { filter, setFilter, stats, setStats, scanning, setScanning } =
    useAppStore();

  const handleScan = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      setScanning(true);
      try {
        const newFiles = await scanDirectory(selected);
        console.log(`Indexed ${newFiles} new files`);
        const updatedStats = await getStats();
        setStats(updatedStats);
      } catch (e) {
        console.error("Scan failed:", e);
      } finally {
        setScanning(false);
      }
    }
  };

  const filterByType = (type: string | undefined) => {
    setFilter({ media_type: type });
  };

  return (
    <div className="w-52 bg-neutral-800 p-4 flex flex-col border-r border-neutral-700">
      <h1 className="text-xl font-bold mb-6">photo_app_rs</h1>

      <nav className="flex-1 space-y-1">
        <button
          onClick={() => filterByType(undefined)}
          className={`w-full text-left px-3 py-2 rounded hover:bg-neutral-700 ${
            !filter.media_type ? "bg-neutral-700" : ""
          }`}
        >
          All Media
        </button>
        <button
          onClick={() => filterByType("image")}
          className={`w-full text-left px-3 py-2 rounded hover:bg-neutral-700 ${
            filter.media_type === "image" ? "bg-neutral-700" : ""
          }`}
        >
          Photos
        </button>
        <button
          onClick={() => filterByType("video")}
          className={`w-full text-left px-3 py-2 rounded hover:bg-neutral-700 ${
            filter.media_type === "video" ? "bg-neutral-700" : ""
          }`}
        >
          Videos
        </button>
      </nav>

      <div className="border-t border-neutral-700 pt-4 mt-4">
        <button
          onClick={handleScan}
          disabled={scanning}
          className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-neutral-600 px-4 py-2 rounded font-medium"
        >
          {scanning ? "Scanning..." : "Scan Directory"}
        </button>
      </div>

      {stats && (
        <div className="mt-4 text-sm text-neutral-400 space-y-1">
          <div>Total: {stats.total_media.toLocaleString()}</div>
          <div>Photos: {stats.total_images.toLocaleString()}</div>
          <div>Videos: {stats.total_videos.toLocaleString()}</div>
          <div>Size: {formatBytes(stats.total_size_bytes)}</div>
        </div>
      )}
    </div>
  );
}
