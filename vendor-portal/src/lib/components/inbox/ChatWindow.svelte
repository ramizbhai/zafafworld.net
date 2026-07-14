<script lang="ts">
    import { Send, Image as ImageIcon, Store, ArrowLeft } from 'lucide-svelte';
    import { onMount, tick } from 'svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { getApiBaseUrl } from '$lib/utils/api';
    import { resolveMediaUrl } from '$lib/shared/utils/media';
    import SkeletonLoader from '../SkeletonLoader.svelte';

    let {
        conversation = null,
        messages = [],
        vendorUserId = '',
        onSendMessage,
        onBack = null,
        loadingMessages = false
    } = $props();
    const i18n = getI18n();
    const API_BASE = getApiBaseUrl();
    let messageText = $state('');
    let messagesContainer = $state<HTMLElement | null>(null);

    // Scroll to bottom when messages change
    $effect(() => {
        if (messages.length && messagesContainer) {
            tick().then(() => {
                if (messagesContainer) {
                    messagesContainer.scrollTop = messagesContainer.scrollHeight;
                }
            });
        }
    });

    let fileInput = $state<HTMLInputElement | null>(null);
    let filesToUpload = $state<FileList | null>(null);
    let uploadPreviews = $state<string[]>([]);

    async function handleSend() {
        if (!messageText.trim() && (!filesToUpload || filesToUpload.length === 0)) return;
        
        const textToSend = messageText.trim();
        const currentFiles = filesToUpload;
        
        messageText = ''; // clear input immediately
        filesToUpload = null;
        uploadPreviews = [];

        await onSendMessage(textToSend, currentFiles);
    }

    function handleFileSelect(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files.length > 0) {
            filesToUpload = input.files;
            
            // Create preview URLs
            uploadPreviews = Array.from(input.files).map(file => URL.createObjectURL(file));
        }
    }

    function removePreview(index: number) {
        const newPreviews = [...uploadPreviews];
        newPreviews.splice(index, 1);
        uploadPreviews = newPreviews;

        if (fileInput && uploadPreviews.length === 0) {
            fileInput.value = '';
            filesToUpload = null;
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSend();
        }
    }

    function formatDateTime(isoString: string) {
        if (!isoString) return '';
        const d = new Date(isoString);
        return d.toLocaleDateString() + ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }
</script>

<div class="chat-window">
    {#if !conversation}
        <div class="empty-chat">
            <Store size={48} class="empty-icon" />
            <h3>{i18n.t.inboxChat.selectConversation}</h3>
            <p>{i18n.t.inboxChat.chooseChatDesc}</p>
        </div>
    {:else}
        <!-- Context Header -->
        <div class="chat-header">
            <div class="header-info flex items-center gap-3">
                {#if onBack}
                    <button class="back-btn lg:hidden" onclick={onBack} aria-label={i18n.t.inboxChat.backToConversations}>
                        <ArrowLeft size={20} />
                    </button>
                {/if}
                <div>
                    <h3>{conversation.otherParticipant?.name || (i18n.locale === 'ar' ? 'عميل' : 'Client')}</h3>
                    <p class="text-xs text-emerald-600 font-medium">{i18n.t.inboxChat.activeChannel}</p>
                </div>
            </div>
        </div>

        <!-- Sticky Listing Context Preview Card -->
        {#if conversation.productId}
            <div class="sticky-context-card">
                <div class="context-details">
                    {#if conversation.productCoverImage}
                        <img src={resolveMediaUrl(conversation.productCoverImage, 'thumb')} alt="Listing" class="context-img" />
                    {/if}
                    <div class="context-text">
                        <span class="context-label">{i18n.t.inboxChat.inquiryContext}</span>
                        <h4 class="context-title">{conversation.productTitle || i18n.t.inboxChat.weddingServiceListing}</h4>
                    </div>
                </div>
                {#if conversation.productPrice}
                    <div class="context-price">
                        <span class="price-label">{i18n.t.inboxChat.basePrice}</span>
                        <span class="price-val">{i18n.locale === 'ar' ? `${conversation.productPrice.toLocaleString()} ريال` : `SAR ${conversation.productPrice.toLocaleString()}`}</span>
                    </div>
                {/if}
            </div>
        {/if}

        <!-- Messages Area -->
        <div class="messages-area" bind:this={messagesContainer}>
            {#if loadingMessages}
                <div class="message-wrapper theirs">
                    <div class="message-bubble" style="background: var(--color-surface-raised); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 8px; max-width: 70%;">
                        <SkeletonLoader width="140px" height="12px" />
                        <SkeletonLoader width="220px" height="12px" />
                        <div style="margin-top: 4px; display: flex; justify-content: flex-end;">
                            <SkeletonLoader width="40px" height="8px" />
                        </div>
                    </div>
                </div>
                <div class="message-wrapper mine">
                    <div class="message-bubble" style="background: var(--teal-light); display: flex; flex-direction: column; gap: 8px; max-width: 70%;">
                        <SkeletonLoader width="180px" height="12px" />
                        <SkeletonLoader width="120px" height="12px" />
                        <div style="margin-top: 4px; display: flex; justify-content: flex-end;">
                            <SkeletonLoader width="40px" height="8px" />
                        </div>
                    </div>
                </div>
                <div class="message-wrapper theirs">
                    <div class="message-bubble" style="background: var(--color-surface-raised); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 8px; max-width: 70%;">
                        <SkeletonLoader width="100px" height="12px" />
                        <div style="margin-top: 4px; display: flex; justify-content: flex-end;">
                            <SkeletonLoader width="40px" height="8px" />
                        </div>
                    </div>
                </div>
            {:else if messages.length === 0}
                <div class="empty-messages">
                  <span class="text-4xl">✉️</span>
                  <p class="mt-2">{i18n.t.inboxChat.noMessages}</p>
                </div>
            {:else}
                {#each messages as msg}
                    {@const isMine = msg.senderId === vendorUserId}
                    <div class="message-wrapper {isMine ? 'mine' : 'theirs'}">
                        <div class="message-bubble {msg.status === 'failed' ? 'failed' : ''}">
                            <!-- Attachment renders -->
                            {#if msg.attachments && msg.attachments.length > 0}
                                <div class="message-attachments">
                                    {#each msg.attachments as att}
                                        {#if att.fileType?.startsWith('image/')}
                                            <a href={resolveMediaUrl(att.fileUrl)} target="_blank" class="attachment-link">
                                                <img src={resolveMediaUrl(att.fileUrl, 'card')} alt="Attachment" class="message-image" />
                                            </a>
                                        {:else if att.fileType?.startsWith('video/')}
                                            <div class="attachment-video">
                                                <!-- svelte-ignore a11y_media_has_caption -->
                                                <video src={resolveMediaUrl(att.fileUrl)} controls class="message-video"></video>
                                            </div>
                                        {:else}
                                            <a href={resolveMediaUrl(att.fileUrl)} target="_blank" class="attachment-file">
                                                <span>📎</span>
                                                <span class="truncate">{att.fileName || i18n.t.inboxChat.downloadAttachment}</span>
                                            </a>
                                        {/if}
                                    {/each}
                                </div>
                            {/if}
                            
                            {#if msg.body}
                                <p>{msg.body}</p>
                            {/if}
                            
                            <div class="message-meta">
                                <span class="message-time">
                                    {msg.createdAt ? formatDateTime(msg.createdAt) : ''}
                                </span>
                                {#if isMine}
                                    {#if msg.status === 'sending'}
                                        <span class="status-indicator">⏳</span>
                                    {:else if msg.status === 'failed'}
                                        <span class="status-indicator text-red-200">⚠️ {i18n.t.inboxChat.failed}</span>
                                    {:else if msg.readReceipts && msg.readReceipts.length > 0}
                                        <span class="status-indicator read text-blue-200">✓✓</span>
                                    {:else}
                                        <span class="status-indicator sent opacity-75">✓</span>
                                    {/if}
                                {/if}
                            </div>
                        </div>
                    </div>
                {/each}
            {/if}
        </div>

        <!-- Image Previews -->
        {#if uploadPreviews.length > 0}
            <div class="previews-area">
                {#each uploadPreviews as preview, i}
                    <div class="preview-item">
                        <!-- svelte-ignore a11y_missing_attribute -->
                        <img src={preview} />
                        <button class="remove-preview" onclick={() => removePreview(i)}>×</button>
                    </div>
                {/each}
            </div>
        {/if}

        <!-- Input Area -->
        <div class="input-area">
            <input 
                type="file" 
                accept="image/*,video/*" 
                bind:this={fileInput} 
                onchange={handleFileSelect} 
                style="display: none" 
                multiple
            />
            <button class="attach-btn" title="Attach media" onclick={() => fileInput?.click()}>
                <ImageIcon size={20} />
            </button>
            <textarea
                class="chat-input"
                placeholder={i18n.t.inboxChat.typeMessage}
                bind:value={messageText}
                onkeydown={handleKeydown}
                rows="1"
            ></textarea>
            <button class="send-btn" onclick={handleSend} disabled={!messageText.trim() && (!filesToUpload || filesToUpload.length === 0)}>
                <Send size={18} />
                <span>{i18n.t.inboxChat.send}</span>
            </button>
        </div>
    {/if}
</div>

<style>
    .chat-window {
        flex: 1;
        display: flex;
        flex-direction: column;
        background: #f8fafc;
        height: 100%;
        min-height: 500px;
    }

    .empty-chat {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: #94a3b8;
    }

    .empty-chat h3 {
        margin-top: 16px;
        color: #475569;
    }

    .chat-header {
        background: #fff;
        padding: 16px 24px;
        border-bottom: 1px solid var(--border-color, #e2e8f0);
        display: flex;
        align-items: center;
        justify-content: space-between;
        flex-shrink: 0;
    }

    .header-info h3 {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 700;
        color: #1e293b;
    }

    .back-btn {
        background: transparent;
        border: none;
        color: #64748b;
        cursor: pointer;
        padding: 4px;
        margin-inline-end: 8px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .back-btn:hover {
        background: #f1f5f9;
        color: #1e293b;
    }

    .sticky-context-card {
        background: hsl(40, 100%, 97%);
        border-bottom: 1px solid hsl(40, 96%, 82%);
        padding: 12px 24px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        flex-shrink: 0;
    }

    .context-details {
        display: flex;
        align-items: center;
        gap: 12px;
        min-width: 0;
    }

    .context-img {
        width: 44px;
        height: 44px;
        border-radius: 6px;
        object-fit: cover;
        border: 1px solid hsl(40, 96%, 82%);
    }

    .context-text {
        min-width: 0;
    }

    .context-label {
        font-size: 0.75rem;
        font-weight: 600;
        color: hsl(36, 90%, 36%);
        display: block;
    }

    .context-title {
        margin: 2px 0 0 0;
        font-size: 0.9rem;
        font-weight: 700;
        color: hsl(36, 95%, 15%);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .context-price {
        text-align: end;
        flex-shrink: 0;
    }

    .price-label {
        font-size: 0.7rem;
        color: hsl(36, 70%, 40%);
        display: block;
    }

    .price-val {
        font-size: 0.85rem;
        font-weight: 800;
        color: hsl(36, 95%, 15%);
    }

    .messages-area {
        flex: 1;
        padding: 24px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .empty-messages {
        text-align: center;
        color: #94a3b8;
        margin-top: auto;
        margin-bottom: auto;
        padding: 40px 20px;
    }

    .message-wrapper {
        display: flex;
        width: 100%;
    }

    .message-wrapper.mine {
        justify-content: flex-end;
    }

    .message-wrapper.theirs {
        justify-content: flex-start;
    }

    .message-bubble {
        max-width: 65%;
        padding: 12px 16px;
        border-radius: 16px;
        position: relative;
        box-shadow: 0 1px 2px rgba(0,0,0,0.05);
        display: flex;
        flex-direction: column;
    }

    .message-bubble.failed {
        background: #fee2e2 !important;
        border-color: #fca5a5 !important;
        color: #991b1b !important;
    }

    .message-wrapper.theirs .message-bubble {
        background: #fff;
        color: #1e293b;
        border-end-start-radius: 4px;
        border: 1px solid #e2e8f0;
    }

    .message-wrapper.mine .message-bubble {
        background: hsl(162, 72%, 40%);
        color: #fff;
        border-end-end-radius: 4px;
    }

    .message-bubble p {
        margin: 0;
        line-height: 1.5;
        font-size: 0.95rem;
        white-space: pre-wrap;
        word-break: break-word;
    }

    .message-attachments {
        margin-bottom: 8px;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .message-image {
        max-width: 100%;
        max-height: 200px;
        border-radius: 8px;
        object-fit: cover;
    }

    .message-video {
        max-width: 100%;
        max-height: 200px;
        border-radius: 8px;
    }

    .attachment-file {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        background: rgba(0, 0, 0, 0.05);
        padding: 8px 12px;
        border-radius: 8px;
        font-size: 0.8rem;
        color: inherit;
        text-decoration: none;
    }

    .message-meta {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 6px;
        margin-top: 6px;
    }

    .message-time {
        font-size: 0.7rem;
        opacity: 0.75;
    }

    .status-indicator {
        font-size: 0.75rem;
    }

    .input-area {
        background: #fff;
        padding: 16px 24px;
        border-top: 1px solid #e2e8f0;
        display: flex;
        align-items: flex-end;
        gap: 12px;
        flex-shrink: 0;
    }

    .attach-btn {
        background: transparent;
        border: none;
        color: #94a3b8;
        padding: 10px;
        cursor: pointer;
        border-radius: 50%;
        transition: background 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .attach-btn:hover {
        background: #f1f5f9;
        color: #64748b;
    }

    .chat-input {
        flex: 1;
        border: 1px solid #cbd5e1;
        border-radius: 20px;
        padding: 12px 16px;
        font-size: 0.95rem;
        font-family: inherit;
        resize: none;
        max-height: 120px;
        outline: none;
        transition: border-color 0.2s;
    }

    .chat-input:focus {
        border-color: hsl(162, 72%, 40%);
    }

    .send-btn {
        background: hsl(162, 72%, 40%);
        color: white;
        border: none;
        padding: 10px 20px;
        border-radius: 20px;
        font-weight: 600;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        transition: background 0.2s;
        flex-shrink: 0;
    }

    .send-btn:hover:not(:disabled) {
        background: hsl(162, 72%, 35%);
    }

    .send-btn:disabled {
        background: #cbd5e1;
        cursor: not-allowed;
    }

    .previews-area {
        padding: 12px 24px 0 24px;
        background: #fff;
        display: flex;
        gap: 12px;
        flex-wrap: wrap;
        border-top: 1px solid #e2e8f0;
        flex-shrink: 0;
    }

    .preview-item {
        position: relative;
        width: 60px;
        height: 60px;
        border-radius: 8px;
        overflow: hidden;
        border: 1px solid #cbd5e1;
    }

    .preview-item img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .remove-preview {
        position: absolute;
        top: 2px;
        inset-inline-end: 2px;
        background: rgba(0,0,0,0.6);
        color: white;
        border: none;
        border-radius: 50%;
        width: 18px;
        height: 18px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 14px;
        cursor: pointer;
        padding: 0;
        line-height: 1;
    }

    .remove-preview:hover {
        background: rgba(239, 68, 68, 0.9);
    }
</style>
