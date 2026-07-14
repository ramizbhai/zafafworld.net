<script lang="ts">
  import { onMount } from "svelte";
  import { t, lang } from "$lib/i18n/index.js";
  import { RefreshCw, Download, AlertTriangle } from "lucide-svelte";
  
  import { createMessagesState } from "$lib/features/admin/messages/messagesState.svelte.js";
  import MessageList from "$lib/components/admin/messages/MessageList.svelte";
  import MessageThread from "$lib/components/admin/messages/MessageThread.svelte";
  import MessageInput from "$lib/components/admin/messages/MessageInput.svelte";
  import ThreadInfoPanel from "$lib/components/admin/messages/ThreadInfoPanel.svelte";

  let { data } = $props<{ data: any }>();
  const user = $derived(data.user);

  const state = createMessagesState(() => data);

  onMount(() => {
    state.loadConversations();
  });
</script>

<svelte:head>
  <title>Communication Center | ZafafWorld Admin</title>
</svelte:head>

<div class="fade-in">
  <!-- HEADER -->
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.messages')}</h1>
      <p class="page-subtitle">
        {$lang === 'ar' ? 'مراقبة وإدارة المحادثات النشطة بين العملاء والموردين' : 'Monitor and manage active conversations between clients and vendors'}
      </p>
    </div>
    <div class="page-header-right">
      {#if state.selectedId}
        <button
          onclick={() => state.exportTranscript()}
          class="btn btn-outline btn-sm"
        >
          <Download size={14} /> {$lang === 'ar' ? 'تصدير' : 'Export'}
        </button>
      {/if}
      <button
        onclick={() => state.loadConversations()}
        disabled={state.isLoading}
        class="btn btn-primary btn-sm"
      >
        <RefreshCw size={14} class={state.isLoading ? "animate-spin" : ""} />
        {state.isLoading ? ($lang === 'ar' ? 'تحديث...' : 'Refreshing…') : ($lang === 'ar' ? 'تحديث' : 'Refresh')}
      </button>
    </div>
  </div>

  {#if state.errorMsg}
    <div
      class="m-6 flex items-center gap-3 px-6 py-4 bg-[var(--danger-dim)] text-[var(--danger)] text-[14px] rounded-[var(--radius-md)] border border-[var(--danger-border)] shadow-md absolute z-20 top-[80px] left-1/2 -translate-x-1/2"
    >
      <AlertTriangle size={18} />
      {state.errorMsg}
    </div>
  {/if}

  <!-- STATS CARDS -->
  <div class="mod-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي المحادثات' : 'Total Conversations'}</span>
      <span class="mini-stat-value">{state.conversations.length}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'رسائل غير مقروءة' : 'Unread Conversations'}</span>
      <span class="mini-stat-value" style="color:var(--purple)">{state.unreadCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'محادثات نشطة' : 'Active Conversations'}</span>
      <span class="mini-stat-value" style="color:var(--success)">{state.activeCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'طلبات الأفراح VIP' : 'Afrah VIP Desk'}</span>
      <span class="mini-stat-value" style="color:var(--gold)">{state.afrahCount}</span>
    </div>
  </div>

  <!-- WORKSPACE -->
  <div class="chat-workspace">
    <!-- Sidebar Listing -->
    <MessageList {state} />

    <!-- Chat thread panel -->
    <div class="flex-1 flex flex-col min-w-0">
      <MessageThread {state} {user} />
      <MessageInput messagesState={state} {user} />
    </div>

    <!-- Participants drawer -->
    <ThreadInfoPanel {state} {user} />
  </div>
</div>

<style>
  /* Stats Grid */
  .mod-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-bottom: 20px;
  }
  .mini-stat {
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .mini-stat-label {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--text-ghost);
  }
  .mini-stat-value {
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.4px;
  }
  @media (max-width: 900px) {
    .mod-stats {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  /* Chat Workspace Container */
  .chat-workspace {
    display: flex;
    height: calc(100vh - 310px);
    min-height: 480px;
    background: var(--glass-sm);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
  }
</style>
