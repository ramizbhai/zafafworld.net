import { listingStore } from "$lib/stores/listingStore";
import { vendorListingSchema } from "$lib/shared/builder/builder.schema";
import { getApiUrl } from "$lib/utils/api";
import { checkSubscriptionQuota } from "$lib/utils/subscriptionGuard";
import { triggerUpgrade } from "$lib/stores/upgradeStore";
import { wizardFetch } from "$lib/utils/wizardFetch";
import { goto } from "$app/navigation";

export class WizardFinalState {
    private sessionToken: string = "";

    isSubmitting = $state(false);
    agreedToTerms = $state(false);
    validationError = $state<string | null>(null);

    setSessionToken(token: string) {
        this.sessionToken = token;
    }

    validate(): boolean {
        this.validationError = null;
        let fd = null;
        listingStore.subscribe(state => fd = state.formData)();
        
        if (!fd) {
            this.validationError = "Form data is missing.";
            return false;
        }

        const result = vendorListingSchema.safeParse(fd);
        if (!result.success) {
            this.validationError = result.error.issues.map((e: any) => e.message).join(", ");
            return false;
        }

        if (!this.agreedToTerms) {
            this.validationError = "You must agree to the Terms & Conditions.";
            return false;
        }

        return true;
    }

    async submitListing(locale: string) {
        if (this.isSubmitting) return;
        if (!this.validate()) return;

        this.isSubmitting = true;
        listingStore.setError("");

        try {
            let productId: string | null = null;
            listingStore.subscribe(state => {
                productId = state.productId;
            })();
            const traceId = listingStore.getTraceId();

            const url = getApiUrl(`/api/v1/vendor/products/${productId}/status`);
            const payload = { status: "pending_approval" };

            const { blocked, response: res } = await checkSubscriptionQuota(async () => {
                return await wizardFetch(url, {
                    method: "PATCH",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${this.sessionToken}`,
                        "X-Trace-ID": traceId || "",
                    },
                    body: JSON.stringify(payload),
                });
            });

            if (blocked) {
                triggerUpgrade('products', '', locale === "ar" ? "تم الوصول إلى الحد الأقصى للاشتراك." : 'Subscription quota limit reached.');
                listingStore.setError(locale === "ar" ? "تم الوصول إلى الحد الأقصى للاشتراك." : "Subscription quota limit reached.");
                this.isSubmitting = false;
                return;
            }

            if (!res.ok) {
                const err = await res.json().catch(() => ({}));
                throw new Error(err.message || err.error || `Server error ${res.status}`);
            }
            
            // Wait for navigation to complete FIRST, so the wizard layout unmounts
            await goto('/dashboard/products');
            
            // Now safe to clear store without triggering layout's $effect guard
            listingStore.reset();
        } catch (err: any) {
            listingStore.setError(err.message || "Failed to submit listing.");
        } finally {
            this.isSubmitting = false;
        }
    }
}
