<script lang="ts">
    import { enhance } from '$app/forms';
    import { invalidateAll } from '$app/navigation';
    import { getStatusBadgeClass, getStatusLabel } from "../../services/inquiries.service";

    let { i18n, inquiriesState } = $props<{ i18n: any, inquiriesState: any }>();

    function closeDrawer() {
        inquiriesState.isDrawerOpen = false;
        inquiriesState.selectedInquiry = null;
    }

    function handleStatusEnhance() {
        inquiriesState.isSubmitting = true;
        return async ({ action, result, update }: any) => {
            // Optimistic update logic inside the component using the store
            const id = action.searchParams.get('id') || inquiriesState.activeSelectedInquiry?.id;
            // The value is submitted via the button's name="status" value="..." which we can't easily extract here directly from `action` in all browsers without parsing FormData, 
            // but we can trust the server result for the final truth, or optimistically update if we parse FormData. 
            // Actually, Svelte Kit's enhance receives `formData` in the submit handler:
            // But we didn't add it in the submit handler, we are in the return function (result handler).
            // To do optimistic update before the server returns, we would do it in the first part of use:enhance.
            
            inquiriesState.isSubmitting = false;
            if (result.type === 'success') {
                await invalidateAll();
            }
            await update();
        };
    }
</script>

{#if inquiriesState.isDrawerOpen && inquiriesState.activeSelectedInquiry}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="drawer-overlay" onclick={closeDrawer} role="presentation">
        <div class="drawer-card" onclick={(e) => e.stopPropagation()} role="presentation">
            <div class="drawer-glow"></div>
            
            <div class="drawer-header">
                <div class="title-block">
                    <span class="header-pre">{i18n.locale === 'ar' ? 'بيانات الاستفسار' : 'Lead Inquiry Details'}</span>
                    <h2>
                        {inquiriesState.activeSelectedInquiry.client_first_name 
                            ? `${inquiriesState.activeSelectedInquiry.client_first_name} ${inquiriesState.activeSelectedInquiry.client_last_name || ''}` 
                            : (inquiriesState.activeSelectedInquiry.name || inquiriesState.activeSelectedInquiry.client_email || inquiriesState.activeSelectedInquiry.email || (i18n.locale === 'ar' ? 'عميل زائر' : 'Guest Client'))
                        }
                    </h2>
                    <span class="status-indicator {getStatusBadgeClass(inquiriesState.activeSelectedInquiry.status)}">
                        {getStatusLabel(inquiriesState.activeSelectedInquiry.status, i18n)}
                    </span>
                </div>
                <button onclick={closeDrawer} class="close-drawer-btn" aria-label={i18n.locale === 'ar' ? 'إغلاق' : 'Close'}>✕</button>
            </div>

            <div class="drawer-content">
                <div class="specs-grid">
                    <div class="spec-card">
                        <span class="label">{i18n.locale === 'ar' ? 'تاريخ الحفل المخطط' : 'Target Event Date'}</span>
                        <span class="value">📅 {inquiriesState.activeSelectedInquiry.event_date}</span>
                    </div>
                    <div class="spec-card">
                        <span class="label">{i18n.locale === 'ar' ? 'عدد الضيوف المتوقع' : 'Expected Guest Count'}</span>
                        <span class="value">👥 {inquiriesState.activeSelectedInquiry.guest_count} {i18n.locale === 'ar' ? 'ضيف' : 'Guests'}</span>
                    </div>
                </div>

                <div class="drawer-section">
                    <h3>{i18n.locale === 'ar' ? 'تفاصيل الاتصال بالعميل' : 'Customer Profile Context'}</h3>
                    <div class="parameters-grid">
                        {#if inquiriesState.activeSelectedInquiry.client_phone || inquiriesState.activeSelectedInquiry.phone}
                            <div class="param-card">
                                <span class="label">{i18n.locale === 'ar' ? 'رقم الهاتف المباشر' : 'Direct Mobile Phone'}</span>
                                <span class="value">{inquiriesState.activeSelectedInquiry.client_phone || inquiriesState.activeSelectedInquiry.phone}</span>
                                <a href="tel:{inquiriesState.activeSelectedInquiry.client_phone || inquiriesState.activeSelectedInquiry.phone}" class="tel-dial-btn">
                                    📞 {i18n.locale === 'ar' ? 'اتصال مباشر بالهاتف' : 'Dial Client Mobile'}
                                </a>
                            </div>
                        {/if}

                        {#if inquiriesState.activeSelectedInquiry.client_email || inquiriesState.activeSelectedInquiry.email}
                            <div class="param-card">
                                <span class="label">{i18n.locale === 'ar' ? 'البريد الإلكتروني للعميل' : 'Client Email'}</span>
                                <span class="value truncate">{inquiriesState.activeSelectedInquiry.client_email || inquiriesState.activeSelectedInquiry.email}</span>
                                <a href="mailto:{inquiriesState.activeSelectedInquiry.client_email || inquiriesState.activeSelectedInquiry.email}" class="email-dial-btn">
                                ✉️ {i18n.locale === 'ar' ? 'إرسال بريد إلكتروني' : 'Send Direct Email'}
                            </a>
                        </div>
                        {/if}
                    </div>
                </div>

                <div class="drawer-section message-section">
                    <h3>{i18n.locale === 'ar' ? 'رسالة العميل ووصف المناسبة' : 'Client Message details'}</h3>
                    <div class="inquiry-message-container">
                        <p class="full-message">{inquiriesState.activeSelectedInquiry.message}</p>
                    </div>
                </div>
            </div>

            <div class="drawer-footer">
                <button type="button" onclick={closeDrawer} class="back-btn">
                    {i18n.locale === 'ar' ? 'الرجوع للقائمة' : 'Back to Feed'}
                </button>

                {#if inquiriesState.activeSelectedInquiry.status === 'pending'}
                    <form method="POST" action="?/updateStatus" use:enhance={({ formData }) => {
                        // Optimistic UI Update
                        const targetStatus = formData.get('status') as string;
                        const id = formData.get('id') as string;
                        const previousStatus = inquiriesState.activeSelectedInquiry.status;
                        
                        inquiriesState.optimisticallyUpdateStatus(id, targetStatus);
                        inquiriesState.isSubmitting = true;
                        
                        return async ({ result, update }) => {
                            inquiriesState.isSubmitting = false;
                            if (result.type === 'success') {
                                await invalidateAll();
                            } else {
                                // Revert optimistic update on failure
                                inquiriesState.optimisticallyUpdateStatus(id, previousStatus);
                            }
                            await update();
                        };
                    }} class="flex gap-2">
                        <input type="hidden" name="id" value={inquiriesState.activeSelectedInquiry.id} />
                        
                        <button 
                            type="submit" 
                            name="status" 
                            value="declined" 
                            disabled={inquiriesState.isSubmitting} 
                            class="btn-decline"
                        >
                            ❌ {i18n.locale === 'ar' ? 'رفض الطلب' : 'Decline'}
                        </button>

                        <button 
                            type="submit" 
                            name="status" 
                            value="replied" 
                            disabled={inquiriesState.isSubmitting} 
                            class="btn-reply"
                        >
                            ✅ {i18n.locale === 'ar' ? 'تحديد كمجيب عليه' : 'Mark as Replied'}
                        </button>
                    </form>
                {:else}
                    <button type="button" disabled class="disabled-read-btn">
                        ✓ {i18n.locale === 'ar' ? 'تمت معالجة الاستفسار' : 'Lead Processed'}
                    </button>
                {/if}
            </div>
        </div>
    </div>
{/if}
