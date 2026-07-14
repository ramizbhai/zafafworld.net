<script lang="ts">
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  interface Props { count?: number; }
  let { count = 6 }: Props = $props();
</script>

{#each Array(count) as _}
  <div class="rounded-2xl overflow-hidden bg-white border border-[var(--color-border)] flex flex-col">
    <Skeleton class="aspect-[4/3] w-full" height="" rounded={false} />
    <div class="p-5 flex flex-col gap-3">
      <Skeleton height="h-3" width="w-24" />
      <Skeleton height="h-5" width="w-3/4" />
      <Skeleton height="h-4" width="w-32" />
      <Skeleton height="h-3" width="w-28" />
      <div class="flex items-end justify-between mt-2">
        <div class="flex flex-col gap-1">
          <Skeleton height="h-3" width="w-16" />
          <Skeleton height="h-6" width="w-28" />
        </div>
        <Skeleton height="h-10" width="w-28" rounded />
      </div>
    </div>
  </div>
{/each}
