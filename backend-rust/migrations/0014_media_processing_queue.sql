-- Migration: Create public.media_processing_jobs table
CREATE TABLE IF NOT EXISTS public.media_processing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uploaded_file_id UUID NOT NULL REFERENCES public.uploaded_files(id) ON DELETE CASCADE,
    job_type VARCHAR(20) NOT NULL, -- 'image' or 'video'
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed'
    priority INTEGER DEFAULT 0 NOT NULL,
    attempt_count INTEGER DEFAULT 0 NOT NULL,
    max_attempts INTEGER DEFAULT 5 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    worker_id UUID,
    processing_time_ms BIGINT,
    CONSTRAINT check_job_status CHECK (status IN ('pending', 'processing', 'completed', 'failed'))
);

-- Indexing for fast lookups by worker
CREATE INDEX IF NOT EXISTS idx_media_processing_jobs_status_priority 
ON public.media_processing_jobs (status, priority, created_at) 
WHERE status IN ('pending', 'failed');

-- Ownership and Grants
ALTER TABLE public.media_processing_jobs OWNER TO zafaf_schema_owner;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.media_processing_jobs TO app_client_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.media_processing_jobs TO app_vendor_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.media_processing_jobs TO app_admin_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.media_processing_jobs TO zafaf_app_user;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.media_processing_jobs TO zafaf_db_admin;
