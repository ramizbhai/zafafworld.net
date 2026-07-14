<script lang="ts">
  import { Search, MessageSquare } from "lucide-svelte";
  import { t, lang } from "$lib/i18n/index.js";
  import type { MessagesState } from "../../../features/admin/messages/messagesState.svelte.js";

  let { state } = $props<{ state: MessagesState }>();

  function formatTime(dt: string) {
    return new Date(dt).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
    });
  }
  function formatDate(dt: string) {
    return new Date(dt).toLocaleDateString($lang === 'ar' ? 'ar-SA' : 'en-US', {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  }
  function formatRelative(dt: string) {
    const d = new Date(dt);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    if (diff < 86400000) return formatTime(dt);
    if (diff < 604800000) return formatDate(dt);
    return d.toLocaleDateString();
  }
</script>

<aside class="conv-sidebar {state.selectedId ? 'hidden md:flex' : 'flex'}">
  <!-- Search -->
  <div class="search-wrap">
    <div class="search-box">
      <Search size={15} class="text-[var(--text-ghost)]" />
      <input
        type="text"
        bind:value={state.searchQuery}
        placeholder={$lang === 'ar' ? 'البحث في المحادثات...' : 'Search conversations…'}
      />
    </div>
  </div>

  <!-- List -->
  <div class="conv-list">
    {#if state.filteredConversations.length === 0}
      <div class="empty-state">
        <MessageSquare size={32} />
        <p>{$lang === 'ar' ? 'لا توجد محادثات' : 'No conversations found'}</p>
      </div>
    {:else}
      {#each state.filteredConversations as conv}
        {@const isSelected = state.selectedId === conv.id}
        {@const convTitle =
          conv.participants
            ?.map((p: any) => (p.name || p.email)?.split(" ")[0])
            .join(" & ") || "Unknown"}

        <button
          class="conv-item {isSelected ? 'selected' : ''}"
          onclick={() => state.selectConversation(conv.id)}
        >
          <div class="flex-1 min-w-0">
            <div class="flex justify-between items-center mb-1">
              <span class="conv-title truncate pr-2">{convTitle}</span>
              {#if conv.lastMessage?.createdAt}
                <span class="conv-time whitespace-nowrap shrink-0 ml-2"
                  >{formatRelative(conv.lastMessage.createdAt)}</span
                >
              {/if}
            </div>
            <div class="flex justify-between items-start gap-3">
              <p class="conv-preview truncate">
                {conv.lastMessage?.body || "Started a conversation"}
              </p>
              {#if conv.unreadCount > 0}
                <span class="conv-unread mt-1"></span>
              {/if}
            </div>
          </div>
        </button>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .conv-sidebar {
    width: 340px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-raised);
    border-inline-end: 1px solid var(--glass-border);
  }
  .search-wrap {
    padding: 16px;
    border-bottom: 1px solid var(--glass-border);
    background: var(--bg-elevated);
  }
  .search-box {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-base);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
  }
  .search-box input {
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 13.5px;
    width: 100%;
  }
  .conv-list {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }
  .conv-item {
    width: 100%;
    text-align: start;
    padding: 14px 16px;
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    margin-bottom: 8px;
    background: var(--bg-base);
    border: 1px solid var(--glass-border);
    transition: all var(--dur-base) var(--ease-smooth);
    position: relative;
    overflow: hidden;
    cursor: pointer;
  }
  :global(html[dir="rtl"]) .conv-item {
    text-align: right;
  }
  .conv-item:hover {
    background: var(--bg-hover);
    border-color: var(--glass-border-hover);
  }
  .conv-item.selected {
    background: var(--purple-dim);
    border-color: var(--purple-border);
  }
  .conv-item::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    width: 4px;
    background: transparent;
    transition: background var(--dur-base);
  }
  :global(html[dir="ltr"]) .conv-item::before {
    left: 0;
  }
  :global(html[dir="rtl"]) .conv-item::before {
    right: 0;
  }
  .conv-item.selected::before {
    background: var(--purple);
  }
  .conv-title {
    font-size: 13.5px;
    font-weight: 750;
    color: var(--text-primary);
  }
  .conv-item.selected .conv-title {
    color: var(--purple);
  }
  .conv-time {
    font-size: 11.5px;
    color: var(--text-ghost);
  }
  .conv-preview {
    font-size: 12.5px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0;
  }
  .conv-item.selected .conv-preview {
    color: var(--text-primary);
    opacity: 0.8;
  }
  .conv-unread {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--purple);
    box-shadow: 0 0 6px var(--purple);
    flex-shrink: 0;
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 16px;
    color: var(--text-ghost);
    gap: 12px;
    text-align: center;
    height: 100%;
  }
  .empty-state p {
    font-size: 13.5px;
    margin: 0;
    color: var(--text-ghost);
  }
  .conv-list::-webkit-scrollbar {
    width: 4px;
  }
  .conv-list::-webkit-scrollbar-thumb {
    background: rgba(91, 33, 182, 0.08);
    border-radius: 2px;
  }
  .conv-list::-webkit-scrollbar-thumb:hover {
    background: rgba(91, 33, 182, 0.16);
  }
</style>
