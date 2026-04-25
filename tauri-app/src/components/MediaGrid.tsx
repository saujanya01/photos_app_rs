import { useAppStore } from "../stores/appStore";
import { MediaItem } from "../hooks/useTauri";

interface Props {
  items: MediaItem[];
}

export default function MediaGrid({ items }: Props) {
  const { setViewingMedia } = useAppStore();

  return (
    <div className="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-1 p-1">
      {items.map((item) => (
        <div
          key={item.id}
          onClick={() => setViewingMedia(item.id)}
          className="relative aspect-square cursor-pointer overflow-hidden bg-neutral-800 hover:opacity-80 transition-opacity"
        >
          {item.thumb_small_b64 ? (
            <img
              src={`data:image/jpeg;base64,${item.thumb_small_b64}`}
              alt=""
              className="w-full h-full object-cover"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-neutral-500">
              No Thumb
            </div>
          )}
          {item.media_type === "video" && (
            <div className="absolute bottom-1 right-1 bg-black/70 px-1.5 py-0.5 rounded text-xs">
              Video
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
