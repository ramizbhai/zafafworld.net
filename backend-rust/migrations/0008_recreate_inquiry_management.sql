--
-- Migration: 0008_recreate_inquiry_management.sql
-- Description: Re-create vendor_inquiry_management and vendor_inquiry_admin_notes referencing lead_inquiries
--

CREATE TABLE IF NOT EXISTS public.vendor_inquiry_management (
    inquiry_id UUID PRIMARY KEY REFERENCES public.lead_inquiries(id) ON DELETE CASCADE,
    assigned_admin_id UUID REFERENCES public.global_users(id) ON DELETE SET NULL,
    escalation_status VARCHAR(50) NOT NULL DEFAULT 'none',
    resolution_status VARCHAR(50) NOT NULL DEFAULT 'unresolved',
    priority VARCHAR(50) NOT NULL DEFAULT 'medium',
    assigned_at TIMESTAMPTZ,
    escalated_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.vendor_inquiry_admin_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inquiry_id UUID NOT NULL REFERENCES public.lead_inquiries(id) ON DELETE CASCADE,
    admin_id UUID REFERENCES public.global_users(id) ON DELETE SET NULL,
    note TEXT NOT NULL,
    note_type VARCHAR(50) NOT NULL DEFAULT 'general',
    is_internal BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
