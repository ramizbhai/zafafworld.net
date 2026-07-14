<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { ChevronRight, ChevronLeft } from 'lucide-svelte';

    interface Props {
        current: number;
        total: number;
        totalRows: number;
        perPage?: number;
    }

    let { current = $bindable(1), total, totalRows, perPage = 10 }: Props = $props();
    const i18n = getI18n();

    let startRow = $derived((current - 1) * perPage + 1);
    let endRow   = $derived(Math.min(current * perPage, totalRows));

    function pages(): (number | '…')[] {
        if (total <= 7) return Array.from({ length: total }, (_, i) => i + 1);
        const arr: (number | '…')[] = [1];
        if (current > 3) arr.push('…');
        for (let i = Math.max(2, current - 1); i <= Math.min(total - 1, current + 1); i++) arr.push(i);
        if (current < total - 2) arr.push('…');
        arr.push(total);
        return arr;
    }
</script>

<div class="pagination">
    <span class="pagination-info">
        {i18n.locale === 'ar'
            ? `${startRow}–${endRow} من ${totalRows}`
            : `${startRow}–${endRow} of ${totalRows}`}
    </span>
    <div class="pagination-buttons">
        <button
            class="page-btn"
            onclick={() => current--}
            disabled={current === 1}
            aria-label={i18n.locale === 'ar' ? 'السابق' : 'Previous'}
        >
            {#if i18n.locale === 'ar'}
                <ChevronRight size={14} />
            {:else}
                <ChevronLeft size={14} />
            {/if}
        </button>

        {#each pages() as p}
            {#if p === '…'}
                <span class="page-ellipsis">…</span>
            {:else}
                <button
                    class="page-btn"
                    class:active={p === current}
                    onclick={() => current = p as number}
                >
                    {p}
                </button>
            {/if}
        {/each}

        <button
            class="page-btn"
            onclick={() => current++}
            disabled={current === total}
            aria-label={i18n.locale === 'ar' ? 'التالي' : 'Next'}
        >
            {#if i18n.locale === 'ar'}
                <ChevronLeft size={14} />
            {:else}
                <ChevronRight size={14} />
            {/if}
        </button>
    </div>
</div>

<style>
    .page-ellipsis {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 34px;
        height: 34px;
        font-size: 13px;
        color: var(--text-light);
        user-select: none;
    }
</style>
