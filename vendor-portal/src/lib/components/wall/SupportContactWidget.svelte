<script lang="ts">
    import { fade } from 'svelte/transition';
    import { MessageSquare, Send, Paperclip, X } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { getApiUrl } from '$lib/utils/api';
    import type { WallState } from '../../features/vendor/wallState.svelte.js';

    let { wallState, title, description, emptyMessage }: { wallState: WallState; title?: string; description?: string; emptyMessage?: string } = $props();
    const i18n = getI18n();
    
    let fileInputEl = $state<HTMLInputElement | null>(null);

    function formatTime(dateStr: string) {
        try {
            const date = new Date(dateStr);
            return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        } catch (e) {
            return '';
        }
    }
</script>

<div class="chat-card">
    <div class="chat-header">
        <MessageSquare size={20} class="chat-header-icon" />
        <div>
            <h3>{title || i18n.t.pendingWall.onboardingAssistance}</h3>
            <p>{description || i18n.t.pendingWall.directChannel}</p>
        </div>
    </div>

    <div class="chat-messages" bind:this={wallState.chatContainer}>
        {#if wallState.loadingMessages && wallState.messages.length === 0}
            <div class="loading-state">
                <div class="loader-spinner"></div>
                <p>{i18n.t.pendingWall.connectingSupport}</p>
            </div>
        {:else if wallState.messages.length === 0}
            <div class="empty-state">
                <MessageSquare size={36} class="empty-icon" />
                <p>{emptyMessage || i18n.t.pendingWall.noMessages}</p>
            </div>
        {:else}
            <div class="message-list">
                {#each wallState.messages as msg (msg.id)}
                    <div class="message-row" class:my-message={msg.sender === 'vendor'}>
                        <div class="message-bubble">
                            {#if msg.body}
                                <p>{msg.body}</p>
                            {/if}
                            
                            {#if msg.file_url}
                                <!-- Render image file attachments -->
                                <!-- svelte-ignore a11y_click_events_have_key_events -->
                                <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                                <img 
                                    src={msg.file_url.startsWith('http') ? msg.file_url : getApiUrl(msg.file_url)} 
                                    alt="Attachment" 
                                    class="max-w-[200px] max-h-[150px] rounded-lg border cursor-pointer object-cover hover:opacity-90 transition-opacity"
                                    onclick={() => { if (msg.file_url) wallState.lightboxUrl = msg.file_url.startsWith('http') ? msg.file_url : getApiUrl(msg.file_url); }}
                                />
                            {/if}

                            <span class="message-time">{formatTime(msg.created_at)}</span>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Optional Attachment Preview Bar -->
    {#if wallState.attachedPreviewUrl}
        <div class="attachment-preview-bar" transition:fade>
            <div class="preview-thumb-wrapper">
                <img src={wallState.attachedPreviewUrl} alt="Selected attachment preview" />
                <button class="remove-preview-btn" onclick={() => wallState.removeAttachment(fileInputEl)} title={i18n.t.pendingWall.removeImage}>
                    <X size={12} />
                </button>
            </div>
            <span class="preview-filename">{wallState.attachedFile?.name}</span>
        </div>
    {/if}

    <!-- Input Form Area -->
    <form class="chat-input-form" onsubmit={(e) => { e.preventDefault(); wallState.sendMessage(fileInputEl); }}>
        <!-- Hidden file input -->
        <input 
            type="file" 
            accept="image/*" 
            class="hidden-input" 
            bind:this={fileInputEl}
            onchange={(e) => wallState.handleFileChange(e)}
        />
        
        <!-- Attach paperclip trigger button -->
        <button 
            type="button" 
            class="btn-attach" 
            onclick={() => fileInputEl?.click()}
            disabled={wallState.sendingMessage}
            title={i18n.t.pendingWall.attachImage}
        >
            <Paperclip size={18} />
        </button>

        <input 
            type="text" 
            placeholder={i18n.t.pendingWall.typeMessagePl} 
            bind:value={wallState.newMessage}
            disabled={wallState.sendingMessage}
        />

        <button type="submit" class="btn btn-send" disabled={(!wallState.newMessage.trim() && !wallState.attachedFile) || wallState.sendingMessage}>
            {#if wallState.sendingMessage}
                <span class="spinner-inline"></span>
            {:else}
                <Send size={16} />
            {/if}
        </button>
    </form>
</div>

<!-- Full Screen Image Lightbox Modal -->
{#if wallState.lightboxUrl}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="lightbox" onclick={() => wallState.lightboxUrl = ''} transition:fade>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="lightbox-content" onclick={(e) => e.stopPropagation()}>
            <img src={wallState.lightboxUrl} alt="Document preview in full size" />
            <button class="lightbox-close" onclick={() => wallState.lightboxUrl = ''} title={i18n.t.pendingWall.closeFullView}>
                <X size={24} />
            </button>
        </div>
    </div>
{/if}

<style>
    /* Chat Card Styles */
    .chat-card {
        background: #ffffff;
        border: 1px solid rgba(0, 0, 0, 0.06);
        border-radius: 1.25rem;
        display: flex;
        flex-direction: column;
        height: 520px;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.03);
        overflow: hidden;
    }

    @media (min-width: 1024px) {
        .chat-card {
            height: auto;
            max-height: 620px;
        }
    }

    .chat-header {
        display: flex;
        gap: 1rem;
        align-items: center;
        padding: 1.25rem 1.5rem;
        background-color: #ffffff;
        border-bottom: 1px solid rgba(0, 0, 0, 0.06);
    }

    :global(.chat-header-icon) {
        color: var(--color-primary, #5b21b6);
    }

    .chat-header h3 {
        margin: 0 0 0.15rem 0;
        font-size: 1rem;
        font-weight: 750;
        color: var(--color-text-dark, #1e293b);
    }

    .chat-header p {
        margin: 0;
        font-size: 0.8rem;
        color: #64748b;
    }

    .chat-messages {
        flex: 1;
        overflow-y: auto;
        padding: 1.5rem;
        background-color: #fafaf9;
    }

    .loading-state, .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: #64748b;
        gap: 0.75rem;
        text-align: center;
    }

    .loader-spinner {
        width: 1.75rem;
        height: 1.75rem;
        border: 2px solid rgba(91, 33, 182, 0.15);
        border-top-color: var(--color-primary, #5b21b6);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }

    :global(.empty-icon) {
        color: rgba(91, 33, 182, 0.12);
        margin-bottom: 0.25rem;
    }

    .message-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .message-row {
        display: flex;
        width: 100%;
        justify-content: flex-start;
    }

    .message-row.my-message {
        justify-content: flex-end;
    }

    .message-bubble {
        max-width: 80%;
        background-color: #ffffff;
        border: 1px solid rgba(0, 0, 0, 0.06);
        border-radius: 12px 12px 12px 2px;
        padding: 0.75rem 1rem;
        color: var(--color-text-dark, #1e293b);
        font-size: 0.9rem;
        line-height: 1.45;
        box-shadow: 0 2px 5px rgba(0, 0, 0, 0.02);
    }

    .my-message .message-bubble {
        background: linear-gradient(135deg, var(--color-primary, #5b21b6) 0%, #7c3aed 100%);
        border: none;
        border-radius: 12px 12px 2px 12px;
        color: #ffffff;
        box-shadow: 0 4px 10px rgba(91, 33, 182, 0.15);
    }

    .message-bubble p {
        margin: 0;
        word-break: break-word;
        color: inherit;
    }

    .message-time {
        display: block;
        font-size: 0.68rem;
        color: #64748b;
        text-align: end;
        margin-top: 0.35rem;
    }

    .my-message .message-time {
        color: rgba(255, 255, 255, 0.75);
    }

    /* Attachment Preview Bar */
    .attachment-preview-bar {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 0.5rem 1.5rem;
        background-color: #f1f5f9;
        border-top: 1px solid rgba(0, 0, 0, 0.05);
    }

    .preview-thumb-wrapper {
        position: relative;
        width: 42px;
        height: 42px;
        border-radius: 6px;
        overflow: hidden;
        border: 1px solid rgba(0, 0, 0, 0.1);
        flex-shrink: 0;
    }

    .preview-thumb-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .remove-preview-btn {
        position: absolute;
        top: 2px;
        inset-inline-end: 2px;
        background-color: rgba(0, 0, 0, 0.6);
        border: none;
        color: #ffffff;
        border-radius: 50%;
        width: 16px;
        height: 16px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 0;
        transition: background-color 0.2s;
    }

    .remove-preview-btn:hover {
        background-color: rgba(239, 68, 68, 0.85);
    }

    .preview-filename {
        font-size: 0.75rem;
        color: #475569;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 180px;
    }

    /* Input Form Styles */
    .chat-input-form {
        display: flex;
        gap: 0.75rem;
        padding: 1rem 1.5rem;
        background-color: #ffffff;
        border-top: 1px solid rgba(0, 0, 0, 0.06);
        align-items: center;
    }

    .hidden-input {
        display: none !important;
    }

    .btn-attach {
        background-color: transparent;
        border: 1px solid rgba(0, 0, 0, 0.12);
        color: #475569;
        width: 2.5rem;
        height: 2.5rem;
        border-radius: 0.5rem;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        flex-shrink: 0;
        transition: all 0.2s;
    }

    .btn-attach:hover:not(:disabled) {
        border-color: var(--color-primary, #5b21b6);
        color: var(--color-primary, #5b21b6);
        background-color: rgba(91, 33, 182, 0.03);
    }

    .chat-input-form input[type="text"] {
        flex: 1;
        background-color: #f8fafc;
        border: 1px solid rgba(0, 0, 0, 0.1);
        border-radius: 0.65rem;
        padding: 0.65rem 1rem;
        color: var(--color-text-dark, #1e293b);
        font-size: 0.9rem;
        outline: none;
        transition: all 0.2s;
    }

    .chat-input-form input[type="text"]:focus {
        border-color: var(--color-primary, #5b21b6);
        background-color: #ffffff;
        box-shadow: 0 0 0 2px rgba(91, 33, 182, 0.12);
    }

    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        font-size: 0.85rem;
        font-weight: 700;
        border-radius: 0.5rem;
        border: none;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-send {
        background-color: var(--color-primary, #5b21b6);
        color: #ffffff;
        width: 2.5rem;
        height: 2.5rem;
        border-radius: 0.5rem;
        padding: 0;
        flex-shrink: 0;
    }

    .btn-send:hover:not(:disabled) {
        background-color: var(--color-primary-hover, #4c1d95);
        box-shadow: 0 4px 12px rgba(91, 33, 182, 0.2);
    }

    .btn-send:disabled {
        opacity: 0.45;
        cursor: not-allowed;
        background-color: #cbd5e1;
        color: #94a3b8;
    }

    .spinner-inline {
        width: 14px;
        height: 14px;
        border: 2px solid rgba(255, 255, 255, 0.3);
        border-radius: 50%;
        border-top-color: #ffffff;
        animation: spin 1s linear infinite;
    }

    /* Lightbox Modal */
    .lightbox {
        position: fixed;
        inset: 0;
        background-color: rgba(10, 10, 15, 0.85);
        backdrop-filter: blur(8px);
        z-index: 2000;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2rem;
    }

    .lightbox-content {
        position: relative;
        max-width: 90%;
        max-height: 90%;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .lightbox-content img {
        display: block;
        max-width: 100%;
        max-height: 90vh;
        border-radius: 12px;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .lightbox-close {
        position: absolute;
        top: -40px;
        inset-inline-end: 0;
        background: transparent;
        border: none;
        color: #ffffff;
        cursor: pointer;
        padding: 0.5rem;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.2s;
    }

    .lightbox-close:hover {
        transform: scale(1.1);
    }
</style>
