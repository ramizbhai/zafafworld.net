import { get } from 'svelte/store';
import { listingStore } from '$lib/stores/listingStore';
import { getApiUrl } from '$lib/utils/api';
import { wizardFetch } from '$lib/utils/wizardFetch';
import { goto } from '$app/navigation';
import { checkSubscriptionQuota } from '$lib/utils/subscriptionGuard';
import { triggerUpgrade } from '$lib/stores/upgradeStore';

export async function saveWizardStep({
    stepId,
    data,
    wizard,
    pageUrl,
    telemetry,
    i18n,
    buildPayload,
    onSuccess
}: {
    stepId: number;
    data: { sessionToken: string };
    wizard: any;
    pageUrl: URL;
    telemetry?: any;
    i18n?: any;
    buildPayload: () => Record<string, any> | null | Promise<Record<string, any> | null>;
    onSuccess?: (responseData: any) => Promise<void> | void;
}) {
    const isDirty = listingStore.isStepDirty(stepId, get(listingStore));
    const store = get(listingStore);
    
    // Bypass if not dirty and product exists
    if (!isDirty && store.productId) {
        listingStore.setHighestStep(stepId);
        if (onSuccess) {
            await onSuccess(null);
        } else {
            await goto(`${pageUrl.pathname.split("/step-")[0]}/step-${stepId + 1}`);
        }
        return true;
    }

    wizard.setSubmitting(true);
    listingStore.setError("");

    try {
        const payload = await buildPayload();
        if (!payload) {
            wizard.setSubmitting(false);
            return false;
        }

        const url = store.productId
            ? getApiUrl(`/api/v1/vendor/products/${store.productId}`)
            : getApiUrl(`/api/v1/vendor/products`);
        
        const method = store.productId ? "PUT" : "POST";

        let fetchResult;

        // Step 1 check subscription quota
        if (stepId === 1 && !store.productId && telemetry && i18n) {
            const { blocked, response } = await checkSubscriptionQuota(async () => {
                return await wizardFetch(url, {
                    method,
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${data.sessionToken}`,
                        "X-Trace-ID": listingStore.getTraceId(),
                    },
                    body: JSON.stringify(payload),
                });
            });

            if (blocked) {
                triggerUpgrade(
                    'products', 
                    telemetry?.vendor?.tier_id ?? '', 
                    i18n.locale === 'ar' ? 'لقد وصلت إلى الحد الأقصى لعدد الإعلانات.' : 'Subscription quota limit reached.'
                );
                listingStore.setError("Subscription quota limit reached.");
                return false;
            }
            fetchResult = response;
        } else {
            fetchResult = await wizardFetch(url, {
                method,
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${data.sessionToken}`,
                    "X-Trace-ID": listingStore.getTraceId(),
                },
                body: JSON.stringify(payload),
            });
        }

        if (!fetchResult.ok) {
            const err = await fetchResult.json().catch(() => ({}));
            throw new Error(err.message || err.error || `Server error ${fetchResult.status}`);
        }

        const responseData = await fetchResult.json();

        if (stepId === 1 && !store.productId && responseData.id) {
            listingStore.setProductId(responseData.id);
            if (typeof window !== 'undefined') {
                sessionStorage.setItem('zafaf_wiz_new_active', 'true');
            }
        }

        if (responseData.product?.version) {
            listingStore.setVersion(responseData.product.version);
        }

        listingStore.commitStepSave(stepId);
        listingStore.setHighestStep(stepId);

        if (stepId === 1 && store.productId && isDirty) {
            listingStore.clearCategoryDependentData();
        }

        if (onSuccess) {
            await onSuccess(responseData);
        } else {
            // Default navigation logic
            const targetId = responseData.id || get(listingStore).productId;
            if (stepId === 1 && pageUrl.pathname.includes('/new') && targetId) {
                await goto(`/dashboard/products/${targetId}/edit/step-2`);
            } else {
                await goto(`${pageUrl.pathname.split("/step-")[0]}/step-${stepId + 1}`);
            }
        }

        return true;
    } catch (err: any) {
        listingStore.setError(err.message || "Failed to save step.");
        return false;
    } finally {
        wizard.setSubmitting(false);
    }
}
