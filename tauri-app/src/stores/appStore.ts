import { create } from "zustand";

export interface SearchFilter {
  date_from?: string;
  date_to?: string;
  media_type?: string;
  camera_model?: string;
  extension?: string;
  query?: string;
}

export interface IndexStats {
  total_media: number;
  total_images: number;
  total_videos: number;
  total_size_bytes: number;
  cameras: string[];
  date_range: [string | null, string | null];
}

interface AppState {
  filter: SearchFilter;
  setFilter: (f: Partial<SearchFilter>) => void;
  stats: IndexStats | null;
  setStats: (s: IndexStats) => void;
  viewingMedia: number | null;
  setViewingMedia: (id: number | null) => void;
  scanning: boolean;
  setScanning: (s: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  filter: {},
  setFilter: (f) => set((state) => ({ filter: { ...state.filter, ...f } })),
  stats: null,
  setStats: (s) => set({ stats: s }),
  viewingMedia: null,
  setViewingMedia: (id) => set({ viewingMedia: id }),
  scanning: false,
  setScanning: (s) => set({ scanning: s }),
}));
