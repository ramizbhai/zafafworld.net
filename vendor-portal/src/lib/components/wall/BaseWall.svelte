<script lang="ts">
    import type { Snippet } from 'svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import WallHeader from './WallHeader.svelte';
    import SupportContactWidget from './SupportContactWidget.svelte';
    import type { WallState } from '../../features/vendor/wallState.svelte.js';

    let { wallState, topbarTitle, chatTitle, chatDescription, chatEmptyMessage, children }: { 
        wallState: WallState; 
        topbarTitle?: string;
        chatTitle?: string;
        chatDescription?: string;
        chatEmptyMessage?: string;
        children: Snippet;
    } = $props();

    const i18n = getI18n();
</script>

<div class="base-wall-container" dir={i18n.locale === 'ar' ? 'rtl' : 'ltr'}>
    <!-- Topbar navigation header -->
    <WallHeader title={topbarTitle} />

    <div class="wall-grid">
        <!-- Brand Info Card (Slot for Pending or Expired content) -->
        <div class="info-card">
            {@render children()}
        </div>

        <!-- Chat Card -->
        <SupportContactWidget 
            {wallState} 
            title={chatTitle} 
            description={chatDescription} 
            emptyMessage={chatEmptyMessage} 
        />
    </div>
</div>

<style>
    :root {
        --color-bg-warm: #faf8f5;
        --color-primary: #5b21b6;
        --color-text-dark: #1e293b;
        --font-heading: "Outfit", "Cairo", sans-serif;
    }

    .base-wall-container {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        width: 100%;
        background-color: var(--color-bg-warm, #faf8f5);
        color: var(--color-text-dark, #1e293b);
        font-family: 'Cairo', system-ui, sans-serif;
        padding-bottom: 3rem;
    }

    .wall-grid {
        display: grid;
        grid-template-columns: 1fr;
        gap: 2rem;
        padding: 0 2rem;
        max-width: 1200px;
        width: 100%;
        margin: 0 auto;
        flex: 1;
    }

    @media (min-width: 1024px) {
        .wall-grid {
            grid-template-columns: 1.25fr 1fr;
        }
    }

    /* Info Card Styles */
    .info-card {
        background: #ffffff;
        border: 1px solid rgba(0, 0, 0, 0.06);
        border-radius: 1.25rem;
        padding: 2.5rem;
        display: flex;
        flex-direction: column;
        gap: 2.25rem;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.03);
    }
</style>
