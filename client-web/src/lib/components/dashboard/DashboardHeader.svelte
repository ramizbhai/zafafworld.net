<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "$lib/components/ui/Button.svelte";

  let { user, conversations = [] } = $props<{ user: any, conversations?: any[] }>();
</script>

<div class="bg-white border-b border-[var(--color-border)] py-8 shadow-sm">
  <div class="container-page flex flex-col md:flex-row justify-between items-start md:items-center gap-6">
    <div>
      <span class="divider-gold"></span>
      <h1 class="font-display text-3xl font-bold text-[var(--color-secondary)] mt-3 mb-1">
        {getLocale() === "ar" ? `مرحباً، ${user.first_name}` : `Welcome back, ${user.first_name}`}
      </h1>
      <p class="text-sm text-[var(--color-muted)]">
        {m.auto_track_your_wedding_p()}
      </p>
    </div>
    <div class="flex gap-3 flex-wrap">
      <Button href="/dashboard/messages" variant="outline" size="sm" class="flex items-center gap-1.5 relative">
        <span>💬</span>
        {m.auto_my_conversations()}
        {#if conversations.some((c: any) => c.unread_count > 0)}
          <span class="absolute -top-1 -end-1 bg-[var(--color-primary)] text-white text-[10px] w-4 h-4 rounded-full flex items-center justify-center font-bold">
            {conversations.filter((c: any) => c.unread_count > 0).length}
          </span>
        {/if}
      </Button>
      <Button href="/venues" variant="primary" size="sm" class="btn-royal-purple">
        {m.auto_browse_venues()}
      </Button>
      <Button href="/vendors" variant="outline" size="sm">
        {m.auto_view_vendors()}
      </Button>
    </div>
  </div>
</div>

<style>
  :global(.btn-royal-purple) {
    background-color: #5b21b6 !important; /* Royal Purple */
    color: #ffffff !important;
    box-shadow: 0 4px 14px rgba(91, 33, 182, 0.2) !important;
    border: none !important;
  }
  :global(.btn-royal-purple:hover) {
    background-color: #4c1d95 !important;
  }
</style>
