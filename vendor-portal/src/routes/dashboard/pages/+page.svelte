<script lang="ts">
    import { enhance } from '$app/forms';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import type { ActionData, PageData } from './$types';
    
    import { ProfileState } from '$lib/features/vendor/profileState.svelte.js';
    import ProfileHeader from '$lib/components/profile/ProfileHeader.svelte';
    import LanguageAssetsConfig from '$lib/components/profile/LanguageAssetsConfig.svelte';
    import OperationalDetails from '$lib/components/profile/OperationalDetails.svelte';
    import VenueFeatures from '$lib/components/profile/VenueFeatures.svelte';
    import ProfileSubmitBar from '$lib/components/profile/ProfileSubmitBar.svelte';

    let { data, form } = $props<{ data: PageData; form: ActionData }>();
    const i18n = getI18n();
    
    const state = new ProfileState();

    $effect(() => {
        state.initialize(data);
    });
</script>

<svelte:head>
    <title>{i18n.t.pagesConfig.corporateProfile} | {i18n.t.common.appName}</title>
</svelte:head>

<div class="settings-page" dir={i18n.locale === 'ar' ? 'rtl' : 'ltr'}>
    <ProfileHeader clientError={state.clientError} {form} />

    <div class="settings-container">
        <div class="card-glow"></div>
        
        <form method="POST" action="?/updateProfile" use:enhance={state.handleEnhance(i18n)} class="settings-form">
            <input type="hidden" name="version" value={data.vendor?.version ?? 1} />
            
            <LanguageAssetsConfig {state} />
            
            <OperationalDetails {state} {data} />
            
            <VenueFeatures {state} />
            
            <ProfileSubmitBar {state} />
        </form>
    </div>
</div>

<style>
    .settings-page {
        display: flex;
        flex-direction: column;
        gap: 2rem;
        max-width: 1300px;
        margin: 0 auto;
        animation: fade-in 0.4s ease-out;
    }

    @keyframes fade-in {
        from { opacity: 0; transform: translateY(8px); }
        to { opacity: 1; transform: translateY(0); }
    }

    .settings-container {
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: 16px;
        box-shadow: var(--shadow-md);
        padding: 2.5rem;
        position: relative;
        overflow: hidden;
    }

    .settings-form {
        position: relative;
        z-index: 2;
    }
</style>
