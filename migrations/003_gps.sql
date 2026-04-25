-- ============================================================
-- Migration 003: GPS coordinates on media_files
-- Source: photo_app_rs v0.6 design direction doc · §3 Map view
-- ============================================================

ALTER TABLE media_files ADD COLUMN gps_latitude REAL;
ALTER TABLE media_files ADD COLUMN gps_longitude REAL;

CREATE INDEX IF NOT EXISTS idx_media_files_gps
    ON media_files(gps_latitude, gps_longitude)
    WHERE gps_latitude IS NOT NULL;
