--
-- Migration: 0017_align_product_category_check
-- Description: Align the chk_product_category check constraint on vendor_products table to support all 31 categories, preventing validation crashes.
--

-- Drop the old constraint if it exists
ALTER TABLE public.vendor_products DROP CONSTRAINT IF EXISTS chk_product_category;

-- Add the corrected check constraint containing all 31 categories
ALTER TABLE public.vendor_products ADD CONSTRAINT chk_product_category CHECK (
    product_category::text = ANY (ARRAY[
        'wedding-palace', 'hotel-venue', 'villa-resort', 'restaurant-event', 'outdoor-garden', 'rooftop-venue', 'private-beach', 'chalet',
        'wedding-gown', 'haute-couture', 'abaya-jalabiya', 'groom-attire',
        'hair-makeup', 'beauty-skincare', 'henna-art',
        'photography-video', 'photo-studio',
        'catering', 'wedding-cake', 'wedding-sweets',
        'entertainment-dj', 'zaffa', 'nasheed-band',
        'wedding-jewelry', 'wedding-gifts',
        'wedding-planner', 'khosha-decor', 'flowers-floral', 'wedding-invitation', 'lighting-av',
        'wedding-car'
    ]::text[])
);
