<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';

    interface Props {
        status: string;
    }

    let { status }: Props = $props();
    const i18n = getI18n();

    let config = $derived.by(() => {
        const t = i18n.t.couples;
        const statusMap: Record<string, { label: string; cls: string }> = {
            new:      { label: t.statusNew,     cls: 'badge-new' },
            done:     { label: t.statusDone,    cls: 'badge-done' },
            expired:  { label: t.statusExpired, cls: 'badge-expired' },
            rejected: { label: t.statusRejected,cls: 'badge-rejected' },
            negot:    { label: t.statusNegot,   cls: 'badge-negot' },
            unreach:  { label: t.statusUnreach, cls: 'badge-unreach' },
            paid:     { label: t.statusPaid,    cls: 'badge-paid' },
        };
        return statusMap[status] ?? { label: status, cls: 'badge-unreach' };
    });
</script>

<span class="badge {config.cls}">{config.label}</span>
