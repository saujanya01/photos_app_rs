import { useState, useEffect, useCallback } from "react";
import { Image as ImageIcon, Video } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { getTimeline, TimelineGroup, MediaItem } from "../hooks/useTauri";
import MediaGrid from "./MediaGrid";
import EmptyState from "./EmptyState";

/**
 * Earlier this file passed `groupCounts = items per group` and rendered
 * one `<MediaGrid items=[single]>` per Virtuoso item — which collapsed
 * the CSS grid to a vertical stack (claude_design_doc.md §10).
 *
 * For now we render plain sections + grids: simpler, correct, and fast
 * enough for libraries up to ~10k items. Phase 10 reintroduces
 * virtualization via `VirtuosoGrid` per group.
 */
export default function Timeline() {
  const { filter } = useAppStore();
  const [groups, setGroups] = useState<TimelineGroup[]>([]);
  const [loading, setLoading] = useState(false);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getTimeline(0, 200, filter);
      setGroups(data);
    } catch (e) {
      console.error("Failed to load timeline:", e);
    } finally {
      setLoading(false);
    }
  }, [filter]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  if (loading && groups.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center fg-3 text-[13px]">
        Loading…
      </div>
    );
  }

  if (groups.length === 0) {
    return <EmptyState />;
  }

  return (
    <div className="flex-1 overflow-auto surface-1">
      {groups.map((g) => {
        const photos = g.media.filter((m) => m.media_type !== "video");
        const videos = g.media.filter((m) => m.media_type === "video");
        const dominantCamera = mostCommonCamera(g.media);

        return (
          <section key={g.date} className="rise">
            <header className="date-stick px-5 py-3 flex items-baseline gap-4">
              <h2 className="tighter text-[17px] fg-0 font-semibold">
                {formatDate(g.date)}
              </h2>
              <div className="mono text-[11px] fg-3 tabular-nums">
                {photos.length} photos
                {videos.length > 0 && (
                  <>
                    {" · "}
                    <span>{videos.length} videos</span>
                  </>
                )}
              </div>
              {dominantCamera && (
                <div className="ml-auto mono text-[11px] fg-4 truncate">
                  {dominantCamera}
                </div>
              )}
            </header>

            {videos.length > 0 && (
              <div className="px-5 pb-2">
                <div className="wide-kicker mb-2 flex items-center gap-2">
                  <Video size={11} strokeWidth={1.5} /> Video
                </div>
                <MediaGrid items={videos} tileMin={240} gap={4} />
              </div>
            )}

            {photos.length > 0 && (
              <div className="px-5 pb-6">
                {videos.length > 0 && (
                  <div className="wide-kicker mb-2 mt-1 flex items-center gap-2">
                    <ImageIcon size={11} strokeWidth={1.5} /> Photos
                  </div>
                )}
                <MediaGrid items={photos} tileMin={140} gap={4} />
              </div>
            )}
          </section>
        );
      })}
    </div>
  );
}

function mostCommonCamera(items: MediaItem[]): string | null {
  const counts = new Map<string, number>();
  for (const it of items) {
    if (it.camera_model) counts.set(it.camera_model, (counts.get(it.camera_model) ?? 0) + 1);
  }
  let top: string | null = null;
  let topN = 0;
  for (const [k, v] of counts) {
    if (v > topN) {
      topN = v;
      top = k;
    }
  }
  return top;
}

function formatDate(dateStr: string): string {
  if (dateStr === "Unknown") return "Date unknown";
  try {
    const date = new Date(dateStr);
    if (Number.isNaN(date.getTime())) return dateStr;
    return date.toLocaleDateString("en-US", {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
    });
  } catch {
    return dateStr;
  }
}
