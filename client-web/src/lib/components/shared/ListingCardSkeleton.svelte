<script lang="ts">
  interface Props { count?: number; }
  let { count = 9 }: Props = $props();
</script>

{#each Array(count) as _}
  <div class="rounded-2xl overflow-hidden bg-white border border-[var(--color-border)] flex flex-col animate-pulse">
    <!-- Image placeholder -->
    <div class="aspect-[4/3] w-full bg-gradient-to-br from-gray-100 to-gray-200"></div>
    <!-- Content placeholder -->
    <div class="p-5 flex flex-col gap-3">
      <!-- Vendor + city row -->
      <div class="flex items-center justify-between">
        <div class="h-3 bg-amber-100 rounded-full w-24"></div>
        <div class="h-3 bg-gray-100 rounded-full w-16"></div>
      </div>
      <!-- Name -->
      <div class="h-5 bg-gray-200 rounded-full w-3/4"></div>
      <div class="h-4 bg-gray-100 rounded-full w-1/2"></div>
      <!-- Stars -->
      <div class="flex gap-1">
        {#each Array(5) as _}
          <div class="w-3.5 h-3.5 rounded-full bg-gray-200"></div>
        {/each}
      </div>
      <!-- Capacity -->
      <div class="h-3 bg-gray-100 rounded-full w-28"></div>
      <!-- Price + CTA -->
      <div class="flex items-end justify-between mt-2">
        <div class="flex flex-col gap-1.5">
          <div class="h-3 bg-gray-100 rounded-full w-16"></div>
          <div class="h-6 bg-gray-200 rounded-full w-28"></div>
        </div>
        <div class="h-10 bg-amber-100 rounded-xl w-28"></div>
      </div>
    </div>
  </div>
{/each}
