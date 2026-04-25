import { open } from "@tauri-apps/plugin-dialog";
import {
  Camera,
  HardDrive,
  Image as ImageIcon,
  Layers,
  ScanLine,
  Star,
  Video,
} from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { scanDirectory, getStats } from "../hooks/useTauri";
import { Btn, Kicker } from "./ui";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

interface NavItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  type?: string;
  count?: number;
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

  const navItems: NavItem[] = [
    {
      id: "library",
      label: "Library",
      icon: <Layers size={14} strokeWidth={1.5} />,
      type: undefined,
      count: stats?.total_media,
    },
    {
      id: "photos",
      label: "Photos",
      icon: <ImageIcon size={14} strokeWidth={1.5} />,
      type: "image",
      count: stats?.total_images,
    },
    {
      id: "videos",
      label: "Videos",
      icon: <Video size={14} strokeWidth={1.5} />,
      type: "video",
      count: stats?.total_videos,
    },
  ];

  const activeId = !filter.media_type
    ? "library"
    : filter.media_type === "image"
    ? "photos"
    : filter.media_type === "video"
    ? "videos"
    : null;

  return (
    <aside
      className="h-full flex flex-col vibrancy border-r hair"
      style={{ width: 240 }}
    >
      {/* App mark */}
      <div className="h-11 px-4 flex items-center gap-2 border-b hair">
        <div
          className="w-5 h-5 rounded-[5px] flex items-center justify-center"
          style={{ background: "var(--accent-soft)" }}
        >
          <div
            className="w-1.5 h-1.5 rounded-full"
            style={{ background: "var(--accent)" }}
          />
        </div>
        <div className="text-[13px] tight fg-0 font-medium">photo_app_rs</div>
        <div className="ml-auto mono text-[10px] fg-4">0.6.0</div>
      </div>

      <div className="flex-1 overflow-auto px-2 py-3">
        {/* Nav */}
        <nav className="space-y-[1px]">
          {navItems.map((it) => {
            const active = activeId === it.id;
            return (
              <button
                key={it.id}
                onClick={() => filterByType(it.type)}
                className={`w-full flex items-center gap-2.5 h-7 rounded-[5px] px-2 text-[12.5px] transition-colors focus-ring ${
                  active ? "fg-0 surface-3" : "fg-2 hover:fg-1"
                }`}
              >
                <span className={active ? "accent" : "fg-3"}>{it.icon}</span>
                <span className="tight">{it.label}</span>
                {it.count !== undefined && (
                  <span className="ml-auto mono text-[10.5px] fg-4 tabular-nums">
                    {it.count.toLocaleString()}
                  </span>
                )}
              </button>
            );
          })}
        </nav>

        {/* Cameras */}
        {stats && stats.cameras.length > 0 && (
          <div className="mt-5">
            <div className="mb-1.5 px-2">
              <Kicker>Cameras</Kicker>
            </div>
            <div className="space-y-[1px]">
              {stats.cameras.map((c) => (
                <button
                  key={c}
                  onClick={() => setFilter({ camera_model: c })}
                  className="w-full flex items-center gap-2.5 h-7 rounded-[5px] px-2 hover:bg-[var(--ink-3)] fg-2 hover:fg-0 focus-ring"
                >
                  <Camera size={13} strokeWidth={1.5} className="fg-4" />
                  <span className="text-[12.5px] tight truncate">{c}</span>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Drives — placeholder until mount-watcher lands in Phase 8 */}
        <div className="mt-5">
          <div className="mb-1.5 px-2">
            <Kicker>Drives</Kicker>
          </div>
          <div className="px-2 py-2 text-[11.5px] fg-4 tight">
            <HardDrive
              size={13}
              strokeWidth={1.5}
              className="inline mr-1.5 mb-0.5"
            />
            Drive watching arrives in Phase 8.
          </div>
        </div>
      </div>

      {/* Footer — library meter + scan */}
      <div className="p-2 border-t hair space-y-2">
        {stats && stats.total_media > 0 && (
          <div className="px-2 pt-1 pb-2">
            <div className="flex items-baseline justify-between">
              <Kicker>Library</Kicker>
              <span className="mono text-[10.5px] fg-3">
                {formatBytes(stats.total_size_bytes)}
              </span>
            </div>
            <div className="mt-1.5 mono text-[10.5px] fg-4 flex justify-between">
              <span>{stats.total_media.toLocaleString()} items</span>
              <span>indexed</span>
            </div>
          </div>
        )}
        <Btn
          onClick={handleScan}
          disabled={scanning}
          icon={<ScanLine size={13} strokeWidth={1.5} />}
          kbd="⌘O"
          className="w-full"
        >
          {scanning ? "Scanning…" : "Scan a drive"}
        </Btn>
        {!stats?.cameras.length && !scanning && (
          <div className="px-2 pt-1 pb-1 flex items-center gap-1.5 mono text-[10.5px] fg-4">
            <Star size={11} strokeWidth={1.5} />
            point at a folder to begin
          </div>
        )}
      </div>
    </aside>
  );
}
