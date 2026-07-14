<script lang="ts">
    import { Sparkles, ShieldAlert, CheckCircle2, X } from 'lucide-svelte';

    let { vendorsState } = $props<{ vendorsState: any }>();
</script>

<header class="curation-banner">
    <div class="banner-glow"></div>
    <div class="banner-content">
        <span class="badge-premium">
            <Sparkles size={12} class="sparkle-icon" /> Administrative Governance
        </span>
        <h1>Vendor Curation & Subscriptions</h1>
        <p>Approve onboarding accounts, regulate subscription states, and communicate directly via support chat.</p>
    </div>
</header>

{#if vendorsState.errorMessage}
    <div class="notice-banner error">
        <ShieldAlert size={18} class="notice-icon" />
        <div class="notice-text">{vendorsState.errorMessage}</div>
        <button class="notice-close" onclick={() => vendorsState.errorMessage = ''}><X size={14} /></button>
    </div>
{/if}

{#if vendorsState.successMessage}
    <div class="notice-banner success">
        <CheckCircle2 size={18} class="notice-icon" />
        <div class="notice-text">{vendorsState.successMessage}</div>
        <button class="notice-close" onclick={() => vendorsState.successMessage = ''}><X size={14} /></button>
    </div>
{/if}
