import { z } from 'zod';

export const blockSchema = z.object({
  type: z.string(),
  contentAr: z.string().optional(),
  contentEn: z.string().optional(),
  url: z.string().optional(),
  layout: z.enum(['left', 'right']).optional(),
});

export const baseInfoSchema = z.object({
  titleAr: z.string().min(3, "Arabic title must be at least 3 characters").max(100),
  titleEn: z.string().min(3, "English title must be at least 3 characters").max(100),
  metaTitleAr: z.string().optional(),
  metaTitleEn: z.string().optional(),
  metaDescriptionAr: z.string().optional(),
  metaDescriptionEn: z.string().optional(),
});

export type Block = z.infer<typeof blockSchema>;
export type BaseInfo = z.infer<typeof baseInfoSchema>;
