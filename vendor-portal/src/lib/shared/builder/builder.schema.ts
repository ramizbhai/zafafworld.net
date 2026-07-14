import { z } from "zod";

export const vendorListingSchema = z.object({
    titleAr: z.string().min(1, "Arabic title is required"),
    titleEn: z.string().min(1, "English title is required"),
    descriptionAr: z.string().min(2, "Arabic description must be at least 2 characters"),
    descriptionEn: z.string().min(2, "English description must be at least 2 characters"),
    priceOnInquiry: z.boolean().default(false),
    basePriceSar: z.string().optional(),
    genderSection: z.string().min(1, "Gender section is required"),
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
});

export type VendorListingData = z.infer<typeof vendorListingSchema>;
