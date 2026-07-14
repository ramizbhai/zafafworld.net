<script lang="ts">
  import { env } from "$env/dynamic/public";
  import { ChevronLeft, Info, MessageSquare, Paperclip, ShieldAlert, Trash2 } from "lucide-svelte";
  import { t, lang } from "$lib/i18n/index.js";
  import { RBACService, type User } from "../../../../core/auth/rbac.service.js";
  import type { MessagesState } from "../../../features/admin/messages/messagesState.svelte.js";

  let { state, user } = $props<{ state: MessagesState, user?: User | null }>();

  const API_BASE = env.PUBLIC_API_URL || "https://api.zafafworld.net";

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
</script>

<main class="chat-panel {state.selectedId ? 'flex' : 'hidden md:flex'}">
  {#if !state.selectedId}
    <!-- Empty state -->
    <div class="empty-state-chat">
      <div class="empty-chat-icon">
        <MessageSquare size={32} class="text-[var(--text-ghost)]" />
      </div>
      <div>
        <h3 class="text-[16px] font-[750] text-[var(--text-primary)] mb-2">
          {$lang === 'ar' ? 'حدد محادثة' : 'Select a conversation'}
        </h3>
        <p class="text-[13.5px] text-[var(--text-secondary)] max-w-xs mx-auto leading-relaxed">
          {$lang === 'ar' ? 'اختر محادثة من القائمة لعرض الرسائل وتفاصيل أطراف الحوار.' : 'Choose a thread to review messages, participant details, and context.'}
        </p>
      </div>
    </div>
  {:else}
    <!-- Chat Header -->
    <div class="chat-header">
      <div class="flex items-center gap-3">
        <button
          class="back-btn md:hidden"
          onclick={() => (state.selectedId = null)}
        >
          <ChevronLeft size={18} />
        </button>
        <div>
          <h2 class="chat-title">
            {state.activeConv?.participants
              ?.map((p: any) => p.name || p.email)
              .join(" ↔ ")}
          </h2>
          <p class="chat-subtitle">
            {$lang === 'ar' ? 'محادثة بين العميل والمورد' : 'Client & Vendor Conversation'}
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        {#if RBACService.canDeleteThread(user)}
          <button
            onclick={() => state.deleteThread(state.selectedId!, user)}
            class="btn-icon text-red-500 hover:bg-red-500/10"
            title={$lang === 'ar' ? 'حذف المحادثة' : 'Delete thread'}
          >
            <Trash2 size={16} />
          </button>
        {/if}
        <button
          onclick={() => (state.isDrawerOpen = !state.isDrawerOpen)}
          class="btn-icon"
          title="Toggle details"
        >
          <Info size={16} />
        </button>
      </div>
    </div>

    <!-- Messages Area -->
    <div
      bind:this={state.messageContainer}
      class="chat-body"
    >
      {#if state.messages.length === 0}
        <div class="empty-state-chat">
          <MessageSquare size={32} class="opacity-20 text-[var(--text-ghost)]" />
          <p class="text-[13.5px]">{$lang === 'ar' ? 'لا توجد رسائل في هذه المحادثة' : 'No messages in this thread'}</p>
        </div>
      {:else}
        {#each state.messages as msg, i}
          {@const sender = state.resolveParticipant(msg.senderId)}
          {@const isVendor = sender.role === "vendor"}
          {@const prevMsg = i > 0 ? state.messages[i - 1] : null}
          {@const prevSender = prevMsg
            ? state.resolveParticipant(prevMsg.senderId)
            : null}
          {@const sameGroup = prevSender?.name === sender.name}
          {@const prevDate = prevMsg ? formatDate(prevMsg.createdAt) : null}
          {@const thisDate = formatDate(msg.createdAt)}
          {@const isRedacted = msg.isRedacted || msg.status === "redacted"}

          <!-- Date separator -->
          {#if !prevMsg || prevDate !== thisDate}
            <div class="flex justify-center my-3">
              <span class="badge badge-muted text-[10.5px] px-2.5 py-0.5 font-[600]">
                {thisDate}
              </span>
            </div>
          {/if}

          <!-- System / Redacted Message -->
          {#if isRedacted}
            <div class="flex justify-center w-full my-3">
              <div
                class="flex items-center gap-2.5 px-4 py-2.5 bg-[var(--danger-dim)] rounded-[var(--radius-md)] max-w-[80%] border border-[var(--danger-border)] shadow-xs"
              >
                <ShieldAlert size={15} class="text-[var(--danger)] shrink-0" />
                <span class="text-[13px] text-[var(--danger)] font-[600]"
                  >{$lang === 'ar' ? 'تمت إزالة الرسالة بواسطة الإشراف' : 'Message removed by moderation'}</span
                >
              </div>
            </div>
          {:else}
            <!-- Regular Message -->
            <div
              class="flex w-full {isVendor ? 'justify-end' : 'justify-start'} {sameGroup ? 'mt-1' : 'mt-4'} group relative"
            >
              <div
                class="flex flex-col {isVendor ? 'items-end' : 'items-start'} max-w-[75%] min-w-[120px]"
              >
                <!-- Bubble -->
                <div
                  class="msg-bubble {isVendor ? 'msg-vendor' : 'msg-client'}"
                >
                  <!-- Text content -->
                  {#if msg.body}
                    <p class="whitespace-pre-wrap break-words">
                      {msg.body}
                    </p>
                  {/if}

                  <!-- Attachments -->
                  {#if msg.attachments?.length > 0}
                    <div class="flex flex-col gap-2 mb-1">
                      {#each msg.attachments as att}
                        {#if att.fileUrl}
                          <img
                            src={att.fileUrl.startsWith("http")
                              ? att.fileUrl
                              : `${API_BASE}${att.fileUrl}`}
                            alt="attachment"
                            class="rounded-[var(--radius-sm)] max-w-full h-auto object-cover border border-[var(--glass-border)]"
                          />
                        {:else}
                          <div
                            class="flex items-center gap-2 bg-[var(--bg-elevated)] border border-[var(--glass-border)] p-2 rounded-[var(--radius-sm)] text-[12px]"
                          >
                            <Paperclip size={14} class="text-[var(--text-ghost)]" />
                            <span class="truncate font-[500] text-[var(--text-secondary)]"
                              >{att.fileName || "File"}</span
                            >
                          </div>
                        {/if}
                      {/each}
                    </div>
                  {/if}

                  <!-- Footer containing timestamp and status -->
                  <div class="flex justify-between items-center w-full mt-1">
                    {#if msg.status === "sending"}
                      <span class="text-[9px] text-[var(--text-ghost)] italic">
                        {$lang === 'ar' ? 'جاري الإرسال...' : 'Sending...'}
                      </span>
                    {:else if msg.status === "failed"}
                      <span class="text-[9px] text-red-500 font-semibold">
                        {$lang === 'ar' ? 'فشل الإرسال' : 'Failed'}
                      </span>
                    {/if}
                    <span class="msg-time ml-auto">
                      {formatTime(msg.createdAt)}
                    </span>
                  </div>
                </div>
              </div>

              <!-- Quick moderation action to Redact/Block a message -->
              {#if RBACService.canBlockMessage(user) && msg.status !== "sending" && msg.status !== "failed"}
                <div class="absolute top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity px-2 {isVendor ? 'left-[-40px]' : 'right-[-40px]'}">
                  <button
                    onclick={() => state.blockMessage(msg.id, user)}
                    class="p-1.5 rounded-full bg-red-500/10 hover:bg-red-500/20 text-red-500 border border-red-500/20 transition-all"
                    title={$lang === 'ar' ? 'حظر الرسالة' : 'Block Message'}
                  >
                    <ShieldAlert size={14} />
                  </button>
                </div>
              {/if}
            </div>
          {/if}
        {/each}
      {/if}
    </div>
  {/if}
</main>

<style>
  .chat-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--bg-base);
  }
  .chat-header {
    height: 64px;
    border-bottom: 1px solid var(--glass-border);
    padding: 0 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg-elevated);
  }
  .chat-title {
    font-size: 15px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 2px 0;
    line-height: 1.2;
  }
  .chat-subtitle {
    font-size: 12px;
    color: var(--text-tertiary);
    margin: 0;
    line-height: 1.2;
  }
  .chat-body {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--bg-base);
  }
  .empty-state-chat {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 24px;
    color: var(--text-ghost);
    gap: 16px;
    text-align: center;
    flex: 1;
    background: var(--bg-base);
  }
  .empty-chat-icon {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-xl);
    background: var(--bg-raised);
    border: 1px solid var(--glass-border);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: var(--shadow-sm);
  }
  .msg-bubble {
    position: relative;
    border-radius: var(--radius-lg);
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    max-width: 100%;
    box-shadow: var(--shadow-xs);
    border: 1px solid var(--glass-border);
  }
  .msg-bubble p {
    font-size: 13.5px;
    line-height: 1.5;
    margin: 0 0 3px 0;
    color: var(--text-primary);
  }
  .msg-client {
    background: var(--bg-elevated);
    border-color: var(--glass-border);
    border-bottom-left-radius: 4px;
  }
  .msg-vendor {
    background: var(--purple-dim);
    border-color: var(--purple-border);
    border-bottom-right-radius: 4px;
  }
  .msg-time {
    font-size: 10px;
    color: var(--text-ghost);
    align-self: flex-end;
    margin-top: 3px;
    font-weight: 500;
  }
  .back-btn {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 1px solid var(--glass-border);
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 160ms;
  }
  .back-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .btn-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: transparent;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 160ms;
  }
  .btn-icon:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .chat-body::-webkit-scrollbar {
    width: 4px;
  }
  .chat-body::-webkit-scrollbar-thumb {
    background: rgba(91, 33, 182, 0.08);
    border-radius: 2px;
  }
  .chat-body::-webkit-scrollbar-thumb:hover {
    background: rgba(91, 33, 182, 0.16);
  }
</style>
