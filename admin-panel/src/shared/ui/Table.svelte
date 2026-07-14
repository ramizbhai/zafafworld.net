<script lang="ts">
  import type { Snippet } from 'svelte';

  // Table generic implementation
  // Expected usage:
  // <Table data={items} columns={['name', 'email']}>
  //   {#snippet header(col)} {col.toUpperCase()} {/snippet}
  //   {#snippet cell({ item, col })} {item[col]} {/snippet}
  // </Table>

  type T = $$Generic;

  let {
    data = [],
    columns = [],
    class: className = '',
    header,
    cell,
    empty
  }: {
    data: T[];
    columns: string[];
    class?: string;
    header?: Snippet<[string]>;
    cell?: Snippet<[{ item: T; col: string }]>;
    empty?: Snippet;
  } = $props();

</script>

<div class="bg-[var(--glass-sm)] border border-solid border-[var(--glass-border)] rounded-[var(--radius-lg)] overflow-hidden {className}">
  <div class="overflow-x-auto overflow-y-hidden webkit-overflow-scrolling-touch">
    <table class="w-full border-collapse min-w-[560px] text-left text-[13.5px]">
      <thead>
        <tr>
          {#each columns as col}
            <th class="px-[16px] py-[11px] text-[11px] font-bold text-[var(--text-tertiary)] uppercase tracking-[0.6px] border-b border-solid border-[var(--glass-border)] bg-[rgba(255,255,255,0.012)] whitespace-nowrap select-none">
              {#if header}
                {@render header(col)}
              {:else}
                {col}
              {/if}
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#if data.length === 0}
          <tr>
            <td colspan={columns.length} class="px-[16px] py-[32px] text-center text-[var(--text-ghost)] border-b-0">
              {#if empty}
                {@render empty()}
              {:else}
                No data available
              {/if}
            </td>
          </tr>
        {:else}
          {#each data as item}
            <tr class="hover:bg-[var(--bg-hover-sm)] transition-colors duration-[var(--dur-fast)] ease-[var(--ease-smooth)]">
              {#each columns as col}
                <td class="px-[16px] py-[12px] text-[var(--text-primary)] border-b border-solid border-[rgba(255,255,255,0.025)] align-middle last:border-b-0">
                  {#if cell}
                    {@render cell({ item, col })}
                  {:else}
                    {item[col as keyof typeof item]}
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>
