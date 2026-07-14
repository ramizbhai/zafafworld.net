<script lang="ts">
    import { CheckCircle2, AlertCircle } from 'lucide-svelte';
    import { getApprovedRequest } from "../../services/subscription.service";

    let { i18n, subscriptionState } = $props<{ i18n: any, subscriptionState: any }>();
</script>

{#if subscriptionState.requestResult.success}
    <div class="success-banner glass-effect">
        <CheckCircle2 size={32} color="#10b981" />
        <div class="success-text">
            <h3>{i18n.locale === 'ar' ? 'تم إرسال الطلب بنجاح!' : 'Request Sent Successfully!'}</h3>
            <p>{i18n.locale === 'ar' ? 'سيقوم فريقنا بمراجعة طلبك والتواصل معك قريباً.' : 'Our team will review your request and contact you shortly.'}</p>
        </div>
    </div>
{/if}

{#if subscriptionState.requestResult.error}
    <div class="error-banner">
        <AlertCircle size={20} />
        {subscriptionState.requestResult.error}
    </div>
{/if}

{#if getApprovedRequest(subscriptionState.requests) && subscriptionState.activeTier}
    <div class="success-banner glass-effect mb-6" style="background: rgba(16, 185, 129, 0.1); border-color: rgba(16, 185, 129, 0.3);">
        <CheckCircle2 size={32} color="#047857" />
        <div class="success-text">
            <h3 style="color: #047857;">{i18n.locale === 'ar' ? 'تمت الموافقة على اشتراكك!' : 'Your Subscription is Approved!'}</h3>
            <p>{i18n.locale === 'ar' ? `لقد تمت الموافقة على اشتراكك في باقة ${subscriptionState.activeTier.name} بنجاح.` : `Your subscription to the ${subscriptionState.activeTier.name} plan has been approved.`}</p>
        </div>
    </div>
{/if}
