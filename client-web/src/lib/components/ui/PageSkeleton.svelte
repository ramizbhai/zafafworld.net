<script lang="ts">
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
</script>

<div class="min-h-[calc(100vh-120px)] bg-[var(--color-surface)] py-8 px-4 sm:px-6 lg:px-8 w-full">
  <div class="max-w-7xl mx-auto space-y-8 animate-pulse">
    <!-- Header Banner Skeleton -->
    <div class="w-full h-48 sm:h-64 bg-gradient-to-r from-amber-50/70 via-stone-100 to-amber-50/70 rounded-3xl p-8 flex flex-col justify-center space-y-4 border border-[var(--color-border)]">
      <Skeleton width="w-48" height="h-8" rounded />
      <Skeleton width="w-96" height="h-5" rounded />
    </div>

    <!-- Grid Layout Skeleton -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each Array(6) as _}
        <div class="bg-white rounded-2xl p-5 border border-[var(--color-border)] shadow-sm space-y-4">
          <Skeleton width="w-full" height="h-48" rounded />
          <Skeleton width="w-3/4" height="h-6" rounded />
          <Skeleton width="w-1/2" height="h-4" rounded />
          <div class="flex justify-between items-center pt-2">
            <Skeleton width="w-24" height="h-5" rounded />
            <Skeleton width="w-20" height="h-8" rounded />
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
