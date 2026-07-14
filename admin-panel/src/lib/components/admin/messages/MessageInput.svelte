<script lang="ts">
  import { Send } from "lucide-svelte";
  import { t, lang } from "$lib/i18n/index.js";
  import type { MessagesState } from "../../../features/admin/messages/messagesState.svelte.js";
  import type { User } from "../../../../core/auth/rbac.service.js";

  let { messagesState, user } = $props<{ messagesState: MessagesState, user?: User | null }>();

  let text = $state("");

  function handleSend() {
    const val = text.trim();
    if (!val) return;
    messagesState.sendMessage(val, user);
    text = "";
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }
</script>

{#if messagesState.selectedId}
  <div class="chat-input-container">
    <div class="chat-input-wrap">
      <textarea
        bind:value={text}
        onkeydown={handleKeyDown}
        placeholder={$lang === 'ar' ? 'اكتب رسالة...' : 'Type a message…'}
        rows="1"
      ></textarea>
      
      <button
        onclick={handleSend}
        disabled={!text.trim()}
        class="send-btn"
        title={$lang === 'ar' ? 'إرسال' : 'Send'}
      >
        <Send size={16} />
      </button>
    </div>
  </div>
{/if}

<style>
  .chat-input-container {
    padding: 16px 20px;
    background: var(--bg-elevated);
    border-top: 1px solid var(--glass-border);
  }
  .chat-input-wrap {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--bg-base);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 8px 12px;
  }
  .chat-input-wrap textarea {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 13.5px;
    resize: none;
    line-height: 1.5;
    max-height: 100px;
    font-family: inherit;
  }
  .send-btn {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--purple);
    color: white;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 160ms;
  }
  .send-btn:hover:not(:disabled) {
    background: var(--purple-hover);
    transform: scale(1.05);
  }
  .send-btn:disabled {
    background: var(--bg-hover);
    color: var(--text-ghost);
    cursor: not-allowed;
  }
</style>
