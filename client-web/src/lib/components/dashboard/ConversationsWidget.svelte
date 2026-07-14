<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import Button from "$lib/components/ui/Button.svelte";
  import { formatDate } from "$lib/utils/localize.js";

  let { conversations = [] } = $props<{ conversations?: any[] }>();
</script>

<section
  class="bg-white rounded-2xl border border-[var(--color-border)] p-6 shadow-sm flex flex-col gap-4"
  aria-labelledby="recent-chats-title"
>
  <div class="pb-3 border-b border-[var(--color-border)] flex justify-between items-center">
    <div>
      <h2
        id="recent-chats-title"
        class="font-display text-lg font-bold text-[var(--color-secondary)]"
      >
        {m.auto_recent_conversations()}
      </h2>
      <p class="text-xs text-[var(--color-muted)]">
        {m.auto_direct_communication()}
      </p>
    </div>
    <Button href="/dashboard/messages" variant="ghost" size="sm" class="text-xs">
      {m.auto_view_all()}
    </Button>
  </div>

  {#if conversations.length === 0}
    <div class="py-6 text-center text-[var(--color-muted)] border border-dashed border-[var(--color-border)] rounded-xl bg-[var(--color-surface-alt)]">
      <span class="text-2xl mb-1 block">💬</span>
      <p class="font-semibold text-sm">
        {m.auto_no_active_chats()}
      </p>
      <p class="text-xs mt-0.5">
        {m.auto_contact_a_vendor_dir()}
      </p>
    </div>
  {:else}
    <div class="flex flex-col gap-3">
      {#each conversations.slice(0, 3) as conv}
        <a
          href="/dashboard/messages"
          class="flex items-center justify-between p-4 rounded-xl border border-[var(--color-border)] hover:bg-[var(--color-surface-alt)]/50 transition-all"
        >
          <div class="flex items-center gap-4 min-w-0">
            <div class="w-10 h-10 rounded-full bg-[var(--color-primary-light)] flex items-center justify-center text-[var(--color-primary-dark)] font-bold text-sm shrink-0 border border-[var(--color-primary)]">
              {conv.otherParticipant?.name?.charAt(0).toUpperCase() || "V"}
            </div>
            <div class="min-w-0">
              <span class="font-bold text-sm text-[var(--color-secondary)] block truncate">
                {conv.otherParticipant?.name || "Vendor"}
              </span>
              <span class="text-xs text-[var(--color-muted)] truncate block mt-0.5">
                {conv.lastMessage?.body || m.auto_no_messages_yet()}
              </span>
            </div>
          </div>
          <div class="flex items-center gap-3 shrink-0">
            {#if conv.unread_count > 0}
              <span class="bg-[var(--color-primary)] text-white text-[10px] font-extrabold px-2 py-0.5 rounded-full shrink-0">
                {conv.unread_count} {m.auto_new()}
              </span>
            {/if}
            <span class="text-[10px] text-[var(--color-muted)]">
              {conv.lastMessage?.createdAt ? formatDate(conv.lastMessage.createdAt) : ""}
            </span>
          </div>
        </a>
      {/each}
    </div>
  {/if}
</section>
