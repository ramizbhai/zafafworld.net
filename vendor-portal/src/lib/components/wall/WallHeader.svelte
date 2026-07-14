<script lang="ts">
    import { Building2, LogOut } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';

    const i18n = getI18n();
    
    let { title = '' } = $props<{ title?: string }>();

    function handleLogout() {
        window.location.href = '/logout';
    }
</script>

<header class="topbar">
    <div class="logo">
        <div class="logo-wrap">
            <Building2 size={16} />
        </div>
        <span>
            {#if title}
                {title}
            {:else}
                ZafafWorld <span class="badge">{i18n.t.pendingWall.step2Title}</span>
            {/if}
        </span>
    </div>
    <button class="btn btn-outline" onclick={handleLogout}>
        <LogOut size={14} />
        <span>{i18n.t.pendingWall.logOut}</span>
    </button>
</header>

<style>
    .topbar {
        height: 70px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 2rem;
        background-color: #ffffff;
        border-bottom: 1px solid rgba(0, 0, 0, 0.06);
        margin-bottom: 2rem;
    }

    .logo {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 1.15rem;
        font-weight: 800;
        color: var(--color-text-dark, #1e293b);
        font-family: var(--font-heading, inherit);
    }

    .logo-wrap {
        width: 32px;
        height: 32px;
        border-radius: 8px;
        background-color: rgba(91, 33, 182, 0.08);
        color: var(--color-primary, #5b21b6);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .badge {
        font-size: 0.65rem;
        background-color: rgba(91, 33, 182, 0.08);
        color: var(--color-primary, #5b21b6);
        padding: 0.15rem 0.45rem;
        border-radius: 6px;
        border: 1px solid rgba(91, 33, 182, 0.15);
        font-weight: 700;
        margin-inline-start: 0.45rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        font-size: 0.85rem;
        font-weight: 700;
        border-radius: 0.5rem;
        border: none;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-outline {
        background-color: transparent;
        border: 1px solid rgba(0, 0, 0, 0.12);
        color: #475569;
        padding: 0.5rem 1rem;
    }

    .btn-outline:hover {
        background-color: rgba(0, 0, 0, 0.02);
        color: var(--color-text-dark, #1e293b);
        border-color: rgba(0, 0, 0, 0.25);
    }
</style>
