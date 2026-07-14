<script lang="ts">
    let { productsState } = $props<{ productsState: any }>();
</script>

<div class="quota-bar-container">
    <div class="quota-header">
        <span class="quota-title">Listing Quota</span>
        <span class="quota-text">You have used {productsState.usedProducts} out of {productsState.maxProducts < 0 ? 'Unlimited' : productsState.maxProducts} allowed listings</span>
    </div>
    {#if productsState.maxProducts > 0}
        <div class="progress-bar-bg">
            <div class="progress-bar-fill" style="width: {productsState.quotaPercentage}%" class:warning={productsState.quotaPercentage > 80} class:danger={productsState.quotaPercentage >= 100}></div>
        </div>
    {/if}
</div>
