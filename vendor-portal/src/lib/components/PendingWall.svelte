<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Clock } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    
    import BaseWall from './wall/BaseWall.svelte';
    import StatusTimeline from './wall/StatusTimeline.svelte';
    import ActionRequiredList from './wall/ActionRequiredList.svelte';
    import { WallState } from '../features/vendor/wallState.svelte.js';

    const i18n = getI18n();

    interface Props {
        sessionToken: string;
        vendorName?: string;
    }

    let { sessionToken, vendorName = 'your business' }: Props = $props();
    
    const wallState = new WallState();

    $effect(() => {
        wallState.setSessionToken(sessionToken);
        wallState.startPolling();
        return () => {
            wallState.stopPolling();
        };
    });
</script>

<BaseWall {wallState}>
    <div class="status-header">
        <div class="spinner-circle">
            <Clock class="pulse-icon" size={24} />
        </div>
        <h2>{i18n.t.pendingWall.pendingReview}</h2>
        <p>{@html i18n.t.pendingWall.welcomeText.replace('{name}', `<strong>${vendorName}</strong>`)}</p>
    </div>

    <StatusTimeline />

    <ActionRequiredList 
        title={i18n.t.pendingWall.immediateAssistance} 
        description={i18n.t.pendingWall.assistanceDesc} 
    />
</BaseWall>

<style>
    .status-header {
        text-align: center;
    }

    .spinner-circle {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 4rem;
        height: 4rem;
        border-radius: 50%;
        background-color: rgba(91, 33, 182, 0.06);
        border: 2px dashed rgba(91, 33, 182, 0.25);
        color: var(--color-primary, #5b21b6);
        margin: 0 auto 1.5rem auto;
    }

    :global(.pulse-icon) {
        animation: rotate 10s linear infinite;
    }

    @keyframes rotate {
        100% { transform: rotate(360deg); }
    }

    .status-header h2 {
        font-size: 1.45rem;
        font-weight: 800;
        color: var(--color-text-dark, #1e293b);
        margin: 0 0 0.5rem 0;
        font-family: var(--font-heading, inherit);
    }

    .status-header p {
        color: #475569;
        font-size: 0.95rem;
        line-height: 1.5;
        margin: 0;
    }
</style>
