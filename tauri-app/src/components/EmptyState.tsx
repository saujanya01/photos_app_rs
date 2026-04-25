import { open } from "@tauri-apps/plugin-dialog";
import { ScanLine } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { scanDirectory, getStats } from "../hooks/useTauri";
import { Btn } from "./ui";

export default function EmptyState() {
  const { setStats, setScanning, scanning, setLastScanRoot } = useAppStore();

  const handleScan = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      setScanning(true);
      setLastScanRoot(selected);
      try {
        await scanDirectory(selected);
        const s = await getStats();
        setStats(s);
      } catch (e) {
        console.error("Scan failed:", e);
      } finally {
        setScanning(false);
      }
    }
  };

  return (
    <div className="flex-1 flex flex-col items-center justify-center px-6 gap-5 surface-1">
      <div className="text-[28px] tighter fg-0 font-semibold">
        No library yet.
      </div>
      <div className="text-[13.5px] fg-2 max-w-[420px] text-center leading-relaxed">
        Point photo_app_rs at a folder or drive. Files stay where they are —
        only thumbnails are cached locally.
      </div>
      <Btn
        onClick={handleScan}
        disabled={scanning}
        icon={<ScanLine size={14} strokeWidth={1.5} />}
        kbd="⌘O"
      >
        {scanning ? "Scanning…" : "Scan a drive"}
      </Btn>
      <div className="mono text-[11px] fg-4 mt-4">
        non-destructive · source untouched
      </div>
    </div>
  );
}
