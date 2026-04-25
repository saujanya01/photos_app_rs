import { useState, useEffect } from "react";
import { useAppStore } from "../stores/appStore";

export default function SearchBar() {
  const { filter, setFilter, stats } = useAppStore();
  const [query, setQuery] = useState(filter.query || "");

  useEffect(() => {
    const timer = setTimeout(() => {
      setFilter({ query: query || undefined });
    }, 300);
    return () => clearTimeout(timer);
  }, [query, setFilter]);

  return (
    <div className="flex items-center gap-4 p-4 border-b border-neutral-700 bg-neutral-800">
      <input
        type="text"
        placeholder="Search by camera, extension..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        className="flex-1 bg-neutral-700 border border-neutral-600 rounded px-4 py-2 text-white placeholder-neutral-400 focus:outline-none focus:border-blue-500"
      />

      {stats && stats.cameras.length > 0 && (
        <select
          value={filter.camera_model || ""}
          onChange={(e) =>
            setFilter({ camera_model: e.target.value || undefined })
          }
          className="bg-neutral-700 border border-neutral-600 rounded px-3 py-2 text-white focus:outline-none focus:border-blue-500"
        >
          <option value="">All Cameras</option>
          {stats.cameras.map((cam) => (
            <option key={cam} value={cam}>
              {cam}
            </option>
          ))}
        </select>
      )}

      <select
        value={filter.extension || ""}
        onChange={(e) => setFilter({ extension: e.target.value || undefined })}
        className="bg-neutral-700 border border-neutral-600 rounded px-3 py-2 text-white focus:outline-none focus:border-blue-500"
      >
        <option value="">All Types</option>
        <option value="jpg">JPG</option>
        <option value="jpeg">JPEG</option>
        <option value="png">PNG</option>
        <option value="arw">ARW</option>
        <option value="mp4">MP4</option>
        <option value="mov">MOV</option>
      </select>
    </div>
  );
}
