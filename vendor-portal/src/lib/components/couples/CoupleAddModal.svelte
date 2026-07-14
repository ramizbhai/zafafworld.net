<script lang="ts">
    import { enhance } from '$app/forms';

    let { i18n, couplesState } = $props<{ i18n: any, couplesState: any }>();

    function handleAddEnhance() {
        couplesState.isSubmitting = true;
        return async ({ result, update }: any) => {
            couplesState.isSubmitting = false;
            if (result.type === 'success') {
                couplesState.isAddModalOpen = false;
                couplesState.newName = '';
                couplesState.newPhone = '';
                couplesState.newDate = '';
                couplesState.newMessage = '';
            }
            await update();
        };
    }
</script>

{#if couplesState.isAddModalOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="drawer-overlay" onclick={() => couplesState.isAddModalOpen = false} role="presentation">
        <div class="drawer-card" onclick={(e) => e.stopPropagation()} role="presentation" style="max-width: 520px;">
            <div class="drawer-glow"></div>
            <div class="drawer-header">
                <div class="title-block">
                    <span class="header-pre" style="font-size: 11px; text-transform: uppercase; color: var(--teal); font-weight: 700;">
                        {i18n.locale === 'ar' ? 'إضافة عميل جديد' : 'Register New Couple'}
                    </span>
                    <h2 style="margin: 4px 0 0 0; font-size: 20px; font-weight: 800; color: var(--text);">
                        {i18n.locale === 'ar' ? 'إضافة استفسار جديد' : 'New Couple Inquiry'}
                    </h2>
                </div>
                <button onclick={() => couplesState.isAddModalOpen = false} class="close-drawer-btn">✕</button>
            </div>
            
            <form method="POST" action="?/addCouple" use:enhance={handleAddEnhance} class="drawer-content" style="gap: 16px;">
                <div class="drawer-section" style="display: flex; flex-direction: column; gap: 6px;">
                    <label for="new-customer-name" class="form-label" style="font-weight: 700; color: var(--text); font-size: 13px;">
                        {i18n.locale === 'ar' ? 'اسم العروسين' : 'Couple / Customer Name'} *
                    </label>
                    <input 
                        id="new-customer-name"
                        type="text" 
                        name="customer_name" 
                        required 
                        bind:value={couplesState.newName}
                        placeholder={i18n.locale === 'ar' ? 'مثال: أحمد وسارة' : 'e.g. Sarah & Ahmed'} 
                        style="width: 100%; padding: 10px 12px; border: 1.5px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text);"
                    />
                </div>

                <div class="drawer-section" style="display: flex; flex-direction: column; gap: 6px;">
                    <label for="new-customer-phone" class="form-label" style="font-weight: 700; color: var(--text); font-size: 13px;">
                        {i18n.locale === 'ar' ? 'رقم الهاتف' : 'Phone Number'} *
                    </label>
                    <input 
                        id="new-customer-phone"
                        type="text" 
                        name="phone" 
                        required 
                        bind:value={couplesState.newPhone}
                        placeholder="+9665xxxxxxxx" 
                        style="width: 100%; padding: 10px 12px; border: 1.5px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text);"
                    />
                </div>

                <div class="drawer-section" style="display: flex; flex-direction: column; gap: 6px;">
                    <label for="new-wedding-date" class="form-label" style="font-weight: 700; color: var(--text); font-size: 13px;">
                        {i18n.locale === 'ar' ? 'تاريخ الحفل' : 'Wedding Date'} *
                    </label>
                    <input 
                        id="new-wedding-date"
                        type="date" 
                        name="wedding_date" 
                        required 
                        bind:value={couplesState.newDate}
                        style="width: 100%; padding: 10px 12px; border: 1.5px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text);"
                    />
                </div>

                <div class="drawer-section" style="display: flex; flex-direction: column; gap: 6px;">
                    <label for="new-message" class="form-label" style="font-weight: 700; color: var(--text); font-size: 13px;">
                        {i18n.locale === 'ar' ? 'الرسالة أو الملاحظات' : 'Message / Details'}
                    </label>
                    <textarea 
                        id="new-message"
                        name="message" 
                        bind:value={couplesState.newMessage}
                        placeholder={i18n.locale === 'ar' ? 'تفاصيل الطلب أو الخدمات المطلوبة...' : 'Details about packages, capacity, pricing request...'} 
                        rows="4"
                        style="width: 100%; padding: 10px 12px; border: 1.5px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text); resize: vertical; font-family: var(--font);"
                    ></textarea>
                </div>

                <div class="drawer-footer" style="padding: 16px 0 0 0; margin-top: auto; border-top: 1px solid var(--border); display: flex; gap: 12px; justify-content: flex-end;">
                    <button type="button" onclick={() => couplesState.isAddModalOpen = false} class="back-btn">
                        {i18n.locale === 'ar' ? 'إلغاء' : 'Cancel'}
                    </button>
                    <button type="submit" disabled={couplesState.isSubmitting} class="action-btn">
                        {couplesState.isSubmitting ? i18n.t.common.loading : (i18n.locale === 'ar' ? 'حفظ العميل' : 'Save Couple')}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}
