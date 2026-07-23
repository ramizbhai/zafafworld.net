import { z } from "zod";

export const vendorListingSchema = z.object({
    titleAr: z.string().min(1, "Arabic title is required"),
    titleEn: z.string().min(1, "English title is required"),
    descriptionAr: z.string().default(""),
    descriptionEn: z.string().default(""),
    priceOnInquiry: z.boolean().default(false),
    basePriceSar: z.string().optional(),
    // genderSection is OPTIONAL here: categories with usesCulturalSettings = false
    // skip Step 4 entirely, leaving this field as "". The DB column is nullable,
    // the backend validates only when the value is present, and Step 4's own
    // submit handler enforces non-empty when the step IS visible.
    genderSection: z.string().default(""),
    selectedCityId: z.string().min(1, "City is required"),
    coverItem: z.object({
        status: z.literal("completed")
    }).nullable().refine((val) => val !== null, "Cover photo is required"),
    coordinatorNameAr: z.string().min(1, "Coordinator Arabic name is required"),
    coordinatorNameEn: z.string().min(1, "Coordinator English name is required"),
    coordinatorPhone: z.string().min(1, "Coordinator phone is required"),
    coordinatorWhatsapp: z.string().min(1, "Coordinator WhatsApp is required"),
    coordinatorEmail: z.string().email("Valid coordinator email is required")
}).superRefine((data, ctx) => {
    if (!data.priceOnInquiry) {
        if (!data.basePriceSar || parseFloat(data.basePriceSar) <= 0 || isNaN(parseFloat(data.basePriceSar))) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Price must be greater than 0 if not on inquiry",
                path: ["basePriceSar"]
            });
        }
    }
    const lenAr = data.descriptionAr.trim().length;
    const lenEn = data.descriptionEn.trim().length;
    if (lenAr < 50 && lenEn < 50) {
        ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: "At least one description (English or Arabic) must be 50 characters or longer",
            path: ["descriptionEn"]
        });
    }
});

export type VendorListingData = z.infer<typeof vendorListingSchema>;
