<script lang="ts">
    import { Edit3, Trash2, ToggleLeft, ToggleRight, Users, Shield } from 'lucide-svelte';
    import { enhance } from '$app/forms';
    import { invalidate } from '$app/navigation';
    import { resolveMediaUrl } from '$lib/shared/utils/media';
    import { getCapacityDisplay, formatPrice, statusColors, getStatusLabels, getGenderLabels, getCategoryLabels } from '../../services/products.service';

    let { product, i18n, productsState, data } = $props<{ product: any, i18n: any, productsState: any, data: any }>();

    let categoryLabels = $derived(getCategoryLabels(data.metadata));
    let statusLabels = $derived(getStatusLabels(i18n));

    let category = $derived(categoryLabels[product.productCategory] ?? { en: product.productCategory, ar: '' });
    let isAvailable = $derived(product.metadata?.isAvailable ?? false);
    let status = $derived(product.metadata?.status ?? 'draft');
</script>

<div class="product-card" class:unavailable={!isAvailable}>
    <!-- Card Header -->
    <div class="card-header">
        <div class="card-title-group">
            <span class="card-icon">🏛️</span>
            <div>
                <h3 class="card-title">{product.title || i18n.t.listings.untitled}</h3>
                <p class="card-title-ar">{product.description || ''}</p>
            </div>
        </div>
        <div class="header-badges">
            {#if data.promoMap && data.promoMap[product.id]}
                <span class="promo-indicator-badge {data.promoMap[product.id].status.toLowerCase()}">
                    🏷️ {data.promoMap[product.id].discount}% {i18n.locale === 'ar' ? 'خصم' : 'OFF'}
                </span>
            {/if}
            <span class="status-badge {statusColors[status] ?? 'status-draft'}">
                {statusLabels[status] ?? status.replace('_', ' ')}
            </span>
        </div>
    </div>

    <!-- Tags -->
    <div class="gender-badge" style="border-color: #64748b20; background: #64748b10;">
        {#if product.attributes?.has_separate_entrances}
            <span class="privacy-tag"><Shield size={10} /> {i18n.t.listings.separateEntrances}</span>
        {/if}
        {#if product.attributes?.outdoor_seating_available}
            <span class="privacy-tag">🌳 {i18n.t.listings.outdoorSpace}</span>
        {/if}
        {#if product.attributes?.female_only_staff}
            <span class="privacy-tag" style="color: #ec4899; border-color: #ec489950;">♀ {i18n.t.listings.femaleStaff}</span>
        {/if}
    </div>

    <!-- Stats Row -->
    <div class="stats-row">
        <div class="stat">
            <span class="stat-label">{i18n.t.listings.category}</span>
            <span class="stat-value">{i18n.locale === 'ar' ? (category.ar || category.en) : category.en}</span>
        </div>
        <div class="stat">
            <span class="stat-label">{i18n.t.listings.capacitySize}</span>
            <span class="stat-value capacity-val">
                <Users size={12} />
                {getCapacityDisplay(product)}
            </span>
        </div>
        <div class="stat">
            <span class="stat-label">{i18n.t.listings.basePrice}</span>
            <span class="stat-value price-val">{formatPrice(product.pricing?.basePriceSar, i18n.locale)}</span>
        </div>
    </div>

    <!-- Coordinator -->
    {#if product.coordinator?.name}
        <div class="coordinator-row">
            {#if product.coordinator.avatar}
                <img src={resolveMediaUrl(product.coordinator.avatar, 'thumb')} alt={product.coordinator.name} class="coord-avatar" loading="lazy" />
            {:else}
                <div class="coord-avatar-fallback">
                    {product.coordinator.name[0] ?? '?'}
                </div>
            {/if}
            <div class="coord-info">
                <span class="coord-name">{product.coordinator.name}</span>
                {#if product.coordinator.phone}
                    <span class="coord-phone">{product.coordinator.phone}</span>
                {/if}
            </div>
            <span class="coord-gender-badge" class:female={product.coordinator.gender === 'female'} class:male={product.coordinator.gender === 'male'}>
                {product.coordinator.gender === 'female' ? '♀ ' + i18n.t.listings.female : product.coordinator.gender === 'male' ? '♂ ' + i18n.t.listings.male : i18n.t.listings.any}
            </span>
        </div>
    {/if}

    <!-- Card Actions -->
    <div class="card-actions">
        <!-- Edit -->
        <a href="/dashboard/products/{product.id}/edit" class="action-btn edit-btn">
            <Edit3 size={13} /> {i18n.t.common.edit}
        </a>

        <!-- Promotion -->
        {#if data.promoMap && data.promoMap[product.id]}
            <a href="/dashboard/offers" class="action-btn promo-btn active-promo">
                🏷️ {data.promoMap[product.id].discount}% OFF
            </a>
        {:else}
            <a href="/dashboard/offers/new?listing={product.id}" class="action-btn promo-btn">
                🏷️ {i18n.locale === 'ar' ? 'ترويج' : 'Promote'}
            </a>
        {/if}

        <!-- Availability Toggle -->
        <form method="POST" action="?/toggleAvailability" use:enhance={({ formData }) => {
            productsState.togglingId = product.id;
            return async ({ result, update }) => {
                productsState.togglingId = null;
                if (result.type === 'success') await invalidate('app:vendor-products');
                await update();
            };
        }}>
            <input type="hidden" name="product_id" value={product.id} />
            <input type="hidden" name="is_available" value={isAvailable.toString()} />
            <button type="submit" class="action-btn toggle-btn" disabled={productsState.togglingId === product.id} title={isAvailable ? i18n.t.listings.markUnavailable : i18n.t.listings.markAvailable}>
                {#if isAvailable}
                    <ToggleRight size={13} /> {i18n.t.listings.available}
                {:else}
                    <ToggleLeft size={13} /> {i18n.t.listings.unavailable}
                {/if}
            </button>
        </form>

        <!-- Delete -->
        <button type="button" class="action-btn delete-btn" onclick={() => productsState.productToDelete = product}>
            <Trash2 size={13} /> {i18n.t.common.delete}
        </button>
    </div>
</div>
