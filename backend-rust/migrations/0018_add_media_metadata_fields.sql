--
-- Migration: 0018_add_media_metadata_fields
-- Description: Add metadata columns to public.uploaded_files to store dimensions, duration, codec, bitrate, orientation, and checksum.
--

ALTER TABLE public.uploaded_files ADD COLUMN IF NOT EXISTS width INTEGER;
ALTER TABLE public.uploaded_files ADD COLUMN IF NOT EXISTS height INTEGER;
ALTER TABLE public.uploaded_files ADD COLUMN IF NOT EXISTS duration_seconds INTEGER;
ALTER TABLE public.uploaded_files ADD COLUMN IF NOT EXISTS codec VARCHAR(50);
ALTER TABLE public.uploaded_files ADD COLUMN IF NOT EXISTS bitrate BIGINT;
ALTER TABLE public.uploaded_files ADD COLUMN IF NOT EXISTS orientation INTEGER;
ALTER TABLE public.uploaded_files ADD COLUMN IF NOT EXISTS checksum VARCHAR(64);
