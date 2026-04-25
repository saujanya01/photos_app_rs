import { useAppStore } from "../stores/appStore";

function fmtBytes(b: number): string {
  if (!b) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(b) / Math.log(k));
  return `${parseFloat((b / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Quiet mono status bar — claude_design_doc.md §6 / §7.
 * Phase 6 wires `scan_progress` through; Phase 8 adds device + sidecar
 * subscriptions. Until then it shows the static facts we already have.
 */
export default function StatusBar() {
  const { stats, scanning, lastScanRoot } = useAppStore();

  return (
    <div className="h-6 flex items-center px-3 gap-3 border-t hair mono text-[10.5px] fg-4 surface-2">
      <div className="flex items-center gap-1.5">
        <div
          className="w-1.5 h-1.5 rounded-full"
          style={{ background: scanning ? "var(--warn)" : "var(--ok)" }}
        />
        {scanning ? "scanning" : "online"}
      </div>
      <span>·</span>
      <span>{stats ? `${stats.total_media.toLocaleString()} items indexed` : "no library"}</span>
      {stats && (
        <>
          <span>·</span>
          <span>{fmtBytes(stats.total_size_bytes)}</span>
        </>
      )}
      <div className="ml-auto flex items-center gap-3">
        {lastScanRoot && (
          <>
            <span className="truncate max-w-[260px]" title={lastScanRoot}>
              {lastScanRoot}
            </span>
            <span>·</span>
          </>
        )}
        <span>source · read-only</span>
      </div>
    </div>
  );
}
