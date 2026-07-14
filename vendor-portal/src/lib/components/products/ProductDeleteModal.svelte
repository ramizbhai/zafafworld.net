<script lang="ts">
    import { Trash2 } from 'lucide-svelte';
    import { enhance } from '$app/forms';
    import { invalidate } from '$app/navigation';

    let { i18n, productsState } = $props<{ i18n: any, productsState: any }>();
</script>

{#if productsState.productToDelete}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-overlay" onclick={() => productsState.productToDelete = null}>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="modal-content" onclick={e => e.stopPropagation()}>
            <div class="modal-header">
                <h3>⚠️ {i18n.locale === 'ar' ? 'تأكيد الحذف' : 'Confirm Delete'}</h3>
            </div>
            <div class="modal-body">
                <p>{i18n.locale === 'ar' ? 'هل أنت متأكد؟ لا يمكن التراجع عن هذا الإجراء.' : 'Are you sure? This cannot be undone.'}</p>
                <p class="delete-target"><strong>{productsState.productToDelete.title || i18n.t.listings.untitled}</strong></p>
            </div>
            <div class="modal-footer">
                <button class="btn-secondary" onclick={() => productsState.productToDelete = null}>
                    {i18n.locale === 'ar' ? 'إلغاء' : 'Cancel'}
                </button>
                <form method="POST" action="?/deleteProduct" use:enhance={() => {
                    productsState.deletingId = productsState.productToDelete.id;
                    return async ({ result, update }) => {
                        productsState.deletingId = null;
                        productsState.productToDelete = null;
                        if (result.type === 'success') await invalidate('app:vendor-products');
                        await update();
                    };
                }}>
                    <input type="hidden" name="product_id" value={productsState.productToDelete.id} />
                    <button type="submit" class="btn-danger" disabled={productsState.deletingId === productsState.productToDelete.id}>
                        {#if productsState.deletingId === productsState.productToDelete.id}
                            ⏳
                        {:else}
                            <Trash2 size={16} /> {i18n.locale === 'ar' ? 'نعم، احذف' : 'Yes, Delete'}
                        {/if}
                    </button>
                </form>
            </div>
        </div>
    </div>
{/if}
