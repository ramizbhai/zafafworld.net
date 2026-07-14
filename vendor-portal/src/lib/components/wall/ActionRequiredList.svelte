<script lang="ts">
    import { ShieldAlert } from 'lucide-svelte';

    let { title, description, style = '' } = $props<{ title: string; description: string; style?: string }>();
</script>

<div class="support-notice" {style}>
    <ShieldAlert size={20} class="notice-icon" />
    <div>
        <h4>{title}</h4>
        <p>{description}</p>
    </div>
</div>

<style>
    .support-notice {
        display: flex;
        gap: 1rem;
        align-items: flex-start;
        background-color: rgba(91, 33, 182, 0.03);
        border: 1px solid rgba(91, 33, 182, 0.08);
        border-radius: 0.75rem;
        padding: 1.25rem;
    }

    :global(.notice-icon) {
        color: var(--color-primary, #5b21b6);
        flex-shrink: 0;
        margin-top: 0.1rem;
    }

    .support-notice h4 {
        margin: 0 0 0.25rem 0;
        font-size: 0.9rem;
        font-weight: 700;
        color: var(--color-primary, #5b21b6);
    }

    .support-notice p {
        margin: 0;
        font-size: 0.825rem;
        color: #475569;
        line-height: 1.45;
    }
</style>
