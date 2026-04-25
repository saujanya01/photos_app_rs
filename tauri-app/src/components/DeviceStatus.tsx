import { useState, useEffect } from "react";
import { exists } from "@tauri-apps/plugin-fs";

interface Props {
  path: string;
}

export default function DeviceStatus({ path }: Props) {
  const [accessible, setAccessible] = useState<boolean | null>(null);

  useEffect(() => {
    const checkAccess = async () => {
      try {
        const result = await exists(path);
        setAccessible(result);
      } catch {
        setAccessible(false);
      }
    };

    checkAccess();
    const interval = setInterval(checkAccess, 5000);
    return () => clearInterval(interval);
  }, [path]);

  if (accessible === null) return null;

  return (
    <div className="flex items-center gap-2 text-sm">
      <div
        className={`w-2 h-2 rounded-full ${
          accessible ? "bg-green-500" : "bg-red-500"
        }`}
      />
      <span className="text-neutral-400">
        {accessible ? "Connected" : "Disconnected"}
      </span>
    </div>
  );
}
