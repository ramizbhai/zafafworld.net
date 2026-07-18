-- 0007_drop_vendor_inquiries.sql
-- Drops the superseded vendor_inquiries table and its types.
-- The application has fully migrated to lead_inquiries as of 2026-07-18.

DROP TABLE IF EXISTS public.vendor_inquiries CASCADE;
DROP TABLE IF EXISTS public.vendor_inquiry_admin_notes CASCADE;
DROP TABLE IF EXISTS public.vendor_inquiry_management CASCADE;
