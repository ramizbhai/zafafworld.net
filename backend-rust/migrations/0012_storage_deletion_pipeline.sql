-- ─── STORAGE DELETION PIPELINE MIGRATION ──────────────────────────────────────────

-- 1. Add parent_id to public.uploaded_files
ALTER TABLE public.uploaded_files 
ADD COLUMN IF NOT EXISTS parent_id UUID REFERENCES public.uploaded_files(id) ON DELETE CASCADE;

-- Create index for parent_id lookups
CREATE INDEX IF NOT EXISTS idx_uploaded_files_parent 
ON public.uploaded_files (parent_id) 
WHERE parent_id IS NOT NULL;

-- 2. Add file_id to public.listing_promotions
ALTER TABLE public.listing_promotions 
ADD COLUMN IF NOT EXISTS file_id UUID REFERENCES public.uploaded_files(id) ON DELETE SET NULL;

-- Create index for promotions file_id
CREATE INDEX IF NOT EXISTS idx_listing_promotions_file_id 
ON public.listing_promotions (file_id) 
WHERE file_id IS NOT NULL;

-- 3. Create public.storage_deletion_queue
CREATE TABLE IF NOT EXISTS public.storage_deletion_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_ids UUID[] NOT NULL,
    object_keys VARCHAR(512)[] NOT NULL,
    local_paths VARCHAR(512)[] NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    next_retry_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index for background worker polling
CREATE INDEX IF NOT EXISTS idx_storage_deletion_queue_status 
ON public.storage_deletion_queue (status, next_retry_at) 
WHERE status IN ('pending', 'retrying');
