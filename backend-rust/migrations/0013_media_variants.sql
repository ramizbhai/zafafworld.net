-- Migration: Create public.uploaded_file_variants table
CREATE TABLE IF NOT EXISTS public.uploaded_file_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uploaded_file_id UUID NOT NULL REFERENCES public.uploaded_files(id) ON DELETE CASCADE,
    format VARCHAR(10) NOT NULL,
    variant VARCHAR(20) NOT NULL,
    width INTEGER,
    height INTEGER,
    size_bytes BIGINT NOT NULL,
    object_key VARCHAR(1024) UNIQUE NOT NULL,
    checksum VARCHAR(64),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Indexing for fast parent-child lookups
CREATE INDEX IF NOT EXISTS idx_uploaded_file_variants_file_id ON public.uploaded_file_variants (uploaded_file_id);

-- Ownership and Grants
ALTER TABLE public.uploaded_file_variants OWNER TO zafaf_schema_owner;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_file_variants TO app_client_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_file_variants TO app_vendor_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_file_variants TO app_admin_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_file_variants TO zafaf_app_user;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_file_variants TO zafaf_db_admin;

-- Migrate existing variant records from public.uploaded_files to public.uploaded_file_variants
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_schema = 'public' AND table_name = 'uploaded_files' AND column_name = 'parent_id'
    ) THEN
        INSERT INTO public.uploaded_file_variants (uploaded_file_id, format, variant, size_bytes, object_key, created_at)
        SELECT 
            parent_id, 
            CASE WHEN mime_type = 'image/avif' THEN 'avif' ELSE 'webp' END,
            CASE 
                WHEN object_key LIKE '%_thumb.%' THEN 'thumb'
                WHEN object_key LIKE '%_card.%' THEN 'card'
                WHEN object_key LIKE '%_medium.%' THEN 'medium'
                WHEN object_key LIKE '%_large.%' THEN 'large'
                ELSE 'original'
            END,
            file_size,
            object_key,
            created_at
        FROM public.uploaded_files
        WHERE parent_id IS NOT NULL;

        -- Remove old variant rows from public.uploaded_files
        DELETE FROM public.uploaded_files WHERE parent_id IS NOT NULL;

        -- Drop parent_id column and index
        ALTER TABLE public.uploaded_files DROP CONSTRAINT IF EXISTS uploaded_files_parent_id_fkey;
        DROP INDEX IF EXISTS public.idx_uploaded_files_parent;
        ALTER TABLE public.uploaded_files DROP COLUMN IF EXISTS parent_id;
    END IF;
END $$;
