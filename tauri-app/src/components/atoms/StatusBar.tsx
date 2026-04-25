import { useEffect, useState } from "react";
import { exists } from "@tauri-apps/plugin-fs";

interface Props {
  itemsIndexed: number;
  totalSizeBytes: number;
  watchPath?: string | null;
  filterLabel?: string;
}

function formatGB(bytes: number): string {
  if (bytes < 1e9) return (bytes / 1e6).toFixed(0) + " MB";
  return (bytes / 1e9).toFixed(2) + " GB";
}

/**
 * 24px status bar: online dot · items indexed · library size · filter ·
 * source state · sidecar sync. Polls watchPath for connectivity.
 */
export default function StatusBar({
  itemsIndexed,
  totalSizeBytes,
  watchPath,
  filterLabel,
}: Props) {
  const [online, setOnline] = useState<boolean | null>(null);

  useEffect(() => {
    if (!watchPath) {
      setOnline(null);
      return;
    }
    let cancelled = false;
    const check = async () => {
      try {
        const ok = await exists(watchPath);
        if (!cancelled) setOnline(ok);
      } catch {
        if (!cancelled) setOnline(false);
      }
    };
    check();
    const id = setInterval(check, 5000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [watchPath]);

  return (
    <div
      className="h-6 flex items-center px-3 gap-3 border-t hair mono text-[10.5px] fg-4"
      style={{ background: "var(--ink-2)" }}
    >
      {online === false ? (
        <>
          <span style={{ color: "var(--warn)" }}>● offline</span>
          <span>· source disconnected</span>
        </>
      ) : (
        <>
          <div className="flex items-center gap-1.5">
            <div
              className="w-1.5 h-1.5 rounded-full"
              style={{ background: "var(--ok)" }}
            />
            online
          </div>
          <span>· {itemsIndexed.toLocaleString()} items indexed</span>
        </>
      )}
      <span>·</span>
      <span>{formatGB(totalSizeBytes)}</span>
      {filterLabel && (
        <>
          <span>·</span>
          <span>{filterLabel}</span>
        </>
      )}
      <div className="ml-auto flex items-center gap-3">
        <span>source · read-only</span>
      </div>
    </div>
  );
}
