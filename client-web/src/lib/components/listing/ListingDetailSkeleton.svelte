<script lang="ts">
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
</script>

<div class="min-h-screen bg-[var(--color-surface)] py-8 px-4 sm:px-6 lg:px-8">
  <div class="max-w-7xl mx-auto space-y-8 animate-pulse">
    <!-- Gallery Skeleton -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 rounded-3xl overflow-hidden bg-white p-3 border border-[var(--color-border)] shadow-sm">
      <div class="lg:col-span-8 h-80 sm:h-96 lg:h-[450px] bg-gradient-to-br from-amber-50 to-gray-100 rounded-2xl relative overflow-hidden">
        <Skeleton class="w-full h-full" height="h-full" rounded />
      </div>
      <div class="lg:col-span-4 grid grid-cols-2 lg:grid-cols-1 gap-4 h-full">
        <Skeleton class="w-full h-40 lg:h-[215px]" height="h-full" rounded />
        <Skeleton class="w-full h-40 lg:h-[215px]" height="h-full" rounded />
      </div>
    </div>

    <!-- Main Content & Sidebar Skeleton Grid -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
      <!-- Main Content Skeleton -->
      <div class="lg:col-span-8 space-y-8">
        <!-- Title & Header Info -->
        <div class="bg-white rounded-2xl p-6 border border-[var(--color-border)] shadow-sm space-y-4">
          <div class="flex items-center gap-3">
            <Skeleton width="w-24" height="h-6" rounded />
            <Skeleton width="w-32" height="h-6" rounded />
          </div>
          <Skeleton width="w-3/4" height="h-8" rounded />
          <Skeleton width="w-1/2" height="h-5" rounded />
          <div class="flex gap-4 pt-2">
            <Skeleton width="w-28" height="h-8" rounded />
            <Skeleton width="w-28" height="h-8" rounded />
            <Skeleton width="w-28" height="h-8" rounded />
          </div>
        </div>

        <!-- Attributes Grid Skeleton -->
        <div class="bg-white rounded-2xl p-6 border border-[var(--color-border)] shadow-sm space-y-4">
          <Skeleton width="w-40" height="h-6" rounded />
          <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
            {#each Array(6) as _}
              <div class="p-4 rounded-xl bg-gray-50 border border-gray-100 space-y-2">
                <Skeleton width="w-16" height="h-4" rounded />
                <Skeleton width="w-24" height="h-5" rounded />
              </div>
            {/each}
          </div>
        </div>

        <!-- Description Builder Skeleton -->
        <div class="bg-white rounded-2xl p-6 border border-[var(--color-border)] shadow-sm space-y-4">
          <Skeleton width="w-48" height="h-6" rounded />
          <Skeleton width="w-full" height="h-4" rounded />
          <Skeleton width="w-11/12" height="h-4" rounded />
          <Skeleton width="w-4/5" height="h-4" rounded />
        </div>
      </div>

      <!-- Sidebar Skeleton -->
      <div class="lg:col-span-4 space-y-6">
        <div class="bg-white rounded-2xl p-6 border border-[var(--color-border)] shadow-sm space-y-6 sticky top-24">
          <div class="space-y-2">
            <Skeleton width="w-20" height="h-4" rounded />
            <Skeleton width="w-36" height="h-8" rounded />
          </div>
          <div class="space-y-3 pt-4 border-t border-gray-100">
            <Skeleton width="w-full" height="h-12" rounded />
            <Skeleton width="w-full" height="h-12" rounded />
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
