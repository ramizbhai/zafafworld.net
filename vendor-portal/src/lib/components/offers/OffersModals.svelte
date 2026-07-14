<script lang="ts">
    import { enhance } from "$app/forms";
    import { invalidateAll } from "$app/navigation";
    import { focusTrap } from "../../services/offers.service";

    let { i18n, offersState } = $props<{
        i18n: any;
        offersState: any;
    }>();

    function handleDeleteEnhance() {
        return async ({ result, update }: any) => {
            offersState.isDeleteConfirmOpen = false;
            offersState.offerToDelete = null;
            if (result.type === "success") {
                await invalidateAll();
            }
            await update();
        };
    }

    function handleRenewEnhance({ formData }: { formData: FormData }) {
        const id = formData.get('id')?.toString() || '';
        offersState.renewingId = id;
        offersState.isRenewModalOpen = false;

        return async ({ result, update }: any) => {
            offersState.renewingId = null;
            if (result.type === "success") {
                await invalidateAll();
            }
            await update();
        };
    }
</script>

<svelte:window onkeydown={(e) => {
    if (e.key === "Escape") {
        offersState.isDeleteConfirmOpen = false;
        offersState.isRenewModalOpen = false;
    }
}} />

<!-- ─── DELETE CONFIRMATION MODAL ─────────────────────────────────────── -->
{#if offersState.isDeleteConfirmOpen && offersState.offerToDelete}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="modal-backdrop"
        onclick={() => (offersState.isDeleteConfirmOpen = false)}
        role="presentation"
    >
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div
            class="modal delete-confirm-modal"
            onclick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-labelledby="delete-modal-title"
            use:focusTrap
        >
            <form method="POST" action="?/delete" use:enhance={handleDeleteEnhance}>
                <div class="modal-header">
                    <h2 id="delete-modal-title" class="modal-title">
                        {i18n.locale === "ar" ? "إلغاء العرض الترويجي" : "Cancel Promotion"}
                    </h2>
                    <button type="button" onclick={() => (offersState.isDeleteConfirmOpen = false)} class="modal-close" aria-label={i18n.t.common.close}>✕</button>
                </div>
                <div class="modal-body">
                    <input type="hidden" name="id" value={offersState.offerToDelete.id} />
                    <p>
                        {i18n.locale === "ar"
                            ? `هل أنت متأكد من رغبتك في إلغاء العرض الترويجي "${offersState.offerToDelete.title_ar}"؟`
                            : `Are you sure you want to cancel the promotion "${offersState.offerToDelete.title_en}"?`}
                    </p>
                </div>
                <div class="modal-footer">
                    <button type="button" onclick={() => (offersState.isDeleteConfirmOpen = false)} class="btn btn-ghost">
                        {i18n.t.common.cancel}
                    </button>
                    <button type="submit" class="btn btn-primary btn-delete">
                        {i18n.locale === "ar" ? "تأكيد الإلغاء" : "Confirm Cancel"}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}

<!-- ─── QUICK RENEW MODAL ─────────────────────────────────────────── -->
{#if offersState.isRenewModalOpen && offersState.offerToRenew}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="modal-backdrop"
        onclick={() => (offersState.isRenewModalOpen = false)}
        role="presentation"
    >
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div
            class="modal renew-modal"
            onclick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-labelledby="renew-modal-title"
            use:focusTrap
        >
            <form method="POST" action="?/renew" use:enhance={handleRenewEnhance}>
                <div class="modal-header">
                    <h2 id="renew-modal-title" class="modal-title">
                        {i18n.locale === "ar" ? "تجديد العرض الترويجي" : "Quick Renew Promotion"}
                    </h2>
                    <button type="button" onclick={() => (offersState.isRenewModalOpen = false)} class="modal-close" aria-label={i18n.t.common.close}>✕</button>
                </div>
                <div class="modal-body">
                    <input type="hidden" name="id" value={offersState.offerToRenew.id} />
                    <p style="margin-bottom: 16px;">
                        {i18n.locale === "ar"
                            ? `أنت على وشك تجديد العرض "${offersState.offerToRenew.title_ar}". اختر مدة التمديد:`
                            : `You are renewing the promotion "${offersState.offerToRenew.title_en}". Select duration extension:`}
                    </p>
                    
                    <div class="renew-duration-select" style="display: flex; flex-direction: column; gap: 8px;">
                        <label for="renew_days" style="font-weight: 500; font-size: 0.9rem;">
                            {i18n.locale === "ar" ? "مدة التجديد (أيام):" : "Extension Duration (days):"}
                        </label>
                        <select id="renew_days" name="days" bind:value={offersState.renewDays} 
                            style="padding: 10px; border: 1px solid var(--border-color, #e2e8f0); border-radius: 6px; background-color: white; width: 100%; outline: none;"
                        >
                            <option value={14}>{i18n.locale === "ar" ? "14 يوماً" : "14 Days"}</option>
                            <option value={30}>{i18n.locale === "ar" ? "30 يوماً" : "30 Days"}</option>
                        </select>
                    </div>
                </div>
                <div class="modal-footer" style="margin-top: 20px;">
                    <button type="button" onclick={() => (offersState.isRenewModalOpen = false)} class="btn btn-ghost">
                        {i18n.t.common.cancel}
                    </button>
                    <button type="submit" class="btn btn-primary" style="background-color: var(--primary-color, #ff477e); color: white;">
                        {i18n.locale === "ar" ? "تجديد الآن" : "Renew Now"}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}
