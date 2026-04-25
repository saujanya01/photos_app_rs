import { useState, useEffect } from "react";
import { Search, SlidersHorizontal, Camera, Filter } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { IconBtn, Kbd } from "./ui";

export default function SearchBar() {
  const { filter, setFilter, stats } = useAppStore();
  const [query, setQuery] = useState(filter.query || "");

  // 120ms debounce — local FTS5 is fast (claude_design_doc.md §10).
  useEffect(() => {
    const timer = setTimeout(() => {
      setFilter({ query: query || undefined });
    }, 120);
    return () => clearTimeout(timer);
  }, [query, setFilter]);

  return (
    <div className="h-11 flex items-center gap-2 px-3 vibrancy border-b hair">
      <div className="flex-1 flex items-center gap-2 max-w-[640px] mx-auto w-full">
        <div
          className="flex-1 h-7 rounded-[6px] flex items-center gap-2 px-2.5 transition-shadow focus-within:shadow-[0_0_0_1.5px_var(--accent)]"
          style={{ background: "var(--ink-3)", border: "1px solid var(--hair)" }}
        >
          <Search size={13} strokeWidth={1.5} className="fg-3" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search by camera, lens, filename, or date…"
            className="flex-1 bg-transparent outline-none text-[12.5px] fg-0 placeholder:text-[var(--fg-4)]"
          />
          <Kbd>⌘K</Kbd>
        </div>
      </div>

      {/* Native selects retained until Phase 4 lands the popover primitive.
         Styled to match the chrome rather than left as OS dropdowns. */}
      <div className="flex items-center gap-1">
        {stats && stats.cameras.length > 0 && (
          <select
            value={filter.camera_model || ""}
            onChange={(e) =>
              setFilter({ camera_model: e.target.value || undefined })
            }
            className="h-7 px-2 rounded-[6px] text-[12px] tight fg-2 surface-3 border border-[var(--hair)] focus-ring"
          >
            <option value="">All cameras</option>
            {stats.cameras.map((cam) => (
              <option key={cam} value={cam}>
                {cam}
              </option>
            ))}
          </select>
        )}
        <select
          value={filter.extension || ""}
          onChange={(e) =>
            setFilter({ extension: e.target.value || undefined })
          }
          className="h-7 px-2 rounded-[6px] text-[12px] tight fg-2 surface-3 border border-[var(--hair)] focus-ring"
        >
          <option value="">All types</option>
          <option value="jpg">JPG</option>
          <option value="jpeg">JPEG</option>
          <option value="png">PNG</option>
          <option value="arw">ARW</option>
          <option value="mp4">MP4</option>
          <option value="mov">MOV</option>
        </select>

        <div
          className="w-px h-5 mx-1"
          style={{ background: "var(--hair-2)" }}
        />

        {/* placeholder icon-only toggles — wired up in later phases */}
        <IconBtn aria-label="Cameras" disabled>
          <Camera size={14} strokeWidth={1.5} />
        </IconBtn>
        <IconBtn aria-label="Filters" disabled>
          <Filter size={14} strokeWidth={1.5} />
        </IconBtn>
        <IconBtn aria-label="Settings" disabled>
          <SlidersHorizontal size={14} strokeWidth={1.5} />
        </IconBtn>
      </div>
    </div>
  );
}
