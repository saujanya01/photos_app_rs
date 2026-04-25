import { useState, useEffect, useCallback } from "react";
import { GroupedVirtuoso } from "react-virtuoso";
import { useAppStore } from "../stores/appStore";
import { getTimeline, TimelineGroup } from "../hooks/useTauri";
import MediaGrid from "./MediaGrid";

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
      <div className="flex-1 flex items-center justify-center text-neutral-400">
        Loading...
      </div>
    );
  }

  if (groups.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-neutral-400">
        No media found. Click "Scan Directory" to index a folder.
      </div>
    );
  }

  const groupCounts = groups.map((g) => g.media.length);

  return (
    <div className="flex-1 overflow-hidden">
      <GroupedVirtuoso
        groupCounts={groupCounts}
        groupContent={(index) => (
          <div className="sticky top-0 bg-neutral-900 px-4 py-2 text-neutral-300 font-medium border-b border-neutral-700">
            {formatDate(groups[index].date)}
          </div>
        )}
        itemContent={(index) => {
          let itemIndex = index;
          let groupIndex = 0;
          for (let i = 0; i < groups.length; i++) {
            if (itemIndex < groups[i].media.length) {
              groupIndex = i;
              break;
            }
            itemIndex -= groups[i].media.length;
          }
          return (
            <MediaGrid
              items={[groups[groupIndex].media[itemIndex]]}
              key={groups[groupIndex].media[itemIndex].id}
            />
          );
        }}
      />
    </div>
  );
}

function formatDate(dateStr: string): string {
  if (dateStr === "Unknown") return dateStr;
  try {
    const date = new Date(dateStr);
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
