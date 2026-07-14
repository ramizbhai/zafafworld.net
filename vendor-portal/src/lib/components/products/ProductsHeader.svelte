<script lang="ts">
    import { Plus } from 'lucide-svelte';
    import { goto } from '$app/navigation';

    let { i18n, form } = $props<{ i18n: any, form: any }>();

    function handleAddNew() {
        goto('/dashboard/products/new');
    }
</script>

<div class="page-header">
    <div class="header-text">
        <span class="page-tag">{i18n.t.listings.inventoryManagement}</span>
        <h1>{i18n.t.listings.title}</h1>
        <p class="subtitle">{i18n.t.listings.subtitle}</p>
    </div>
    <button class="btn-add" onclick={handleAddNew}>
        <Plus size={16} />
        {i18n.t.listings.addNew}
    </button>
</div>

{#if form?.error}
    <div class="alert alert-error">⚠️ {form.error}</div>
{/if}
{#if form?.success && form?.message}
    <div class="alert alert-success">✅ {form.message}</div>
{/if}
