--
-- Migration: 0007_enrich_lead_inquiries
-- Description: Add missing fields from vendor_inquiries to lead_inquiries
--

ALTER TABLE public.lead_inquiries
    ADD COLUMN email character varying(255),
    ADD COLUMN guest_count integer DEFAULT 0,
    ADD COLUMN updated_at timestamp with time zone DEFAULT now() NOT NULL,
    ADD COLUMN conversation_id uuid;

-- Update triggers for updated_at
CREATE TRIGGER touch_lead_inquiries_updated_at
BEFORE UPDATE ON public.lead_inquiries
FOR EACH ROW
EXECUTE FUNCTION public.touch_updated_at();
