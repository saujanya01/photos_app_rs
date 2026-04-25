import { invoke } from "@tauri-apps/api/core";
import { SearchFilter, IndexStats } from "../stores/appStore";

export interface MediaItem {
  id: number;
  media_type: string;
  extension: string;
  date_taken: string | null;
  camera_model: string | null;
  file_path: string;
  thumb_small_b64: string | null;
}

export interface TimelineGroup {
  date: string;
  media: MediaItem[];
}

export interface MediaFile {
  id: number | null;
  hash: string;
  file_size_bytes: number;
  media_type: string;
  extension: string;
  camera_make: string | null;
  camera_model: string | null;
  lens_model: string | null;
  date_taken: string | null;
  iso: string | null;
  aperture: string | null;
  shutter_speed: string | null;
  focal_length: string | null;
  software: string | null;
  duration_seconds: number | null;
  resolution_width: number | null;
  resolution_height: number | null;
  path: string;
  date_added: number;
  date_modified: number;
}

export async function scanDirectory(path: string): Promise<number> {
  return invoke("scan_directory", { path });
}

export async function getTimeline(
  offset: number,
  limit: number,
  filter: SearchFilter
): Promise<TimelineGroup[]> {
  return invoke("get_timeline", { offset, limit, filter });
}

export async function getMediaDetail(id: number): Promise<MediaFile> {
  return invoke("get_media_detail", { id });
}

export async function getFullPath(id: number): Promise<string> {
  return invoke("get_full_path", { id });
}

export async function getStats(): Promise<IndexStats> {
  return invoke("get_stats");
}

export async function searchMedia(
  filter: SearchFilter
): Promise<TimelineGroup[]> {
  return invoke("search_media", { filter });
}
