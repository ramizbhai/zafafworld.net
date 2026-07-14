<script lang="ts">
    import { MessageCircle } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { getApiBaseUrl } from '$lib/utils/api';
    import { resolveMediaUrl } from '$lib/shared/utils/media';

    let {
        conversations = [],
        selectedId = null,
        onSelect
    } = $props();

    const i18n = getI18n();
    const API_BASE = getApiBaseUrl();
    function formatTime(isoString: string) {
        if (!isoString) return '';
        const date = new Date(isoString);
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }
</script>

<div class="inbox-sidebar">
    <div class="inbox-header">
        <h2>{i18n.t.inboxChat.title}</h2>
        <span class="badge">{conversations.length}</span>
    </div>

    <div class="conversation-list">
        {#if conversations.length === 0}
            <div class="empty-state">
                <MessageCircle size={32} class="empty-icon" />
                <p>{i18n.t.inboxChat.noConversations}</p>
            </div>
        {:else}
            {#each conversations as conv}
                <button
                    class="conv-card {selectedId === conv.id ? 'active' : ''}"
                    onclick={() => onSelect(conv)}
                >
                    {#if conv.productCoverImage}
                        <img src={resolveMediaUrl(conv.productCoverImage, 'thumb')} alt="Listing" class="conv-avatar-img" />
                    {:else}
                        <div class="conv-avatar">
                            {conv.otherParticipant?.name?.[0]?.toUpperCase() || 'C'}
                        </div>
                    {/if}

                    <div class="conv-details">
                        <div class="conv-top">
                            <span class="client-name">
                                {conv.otherParticipant?.name || (i18n.locale === 'ar' ? 'عميل' : 'Client')}
                            </span>
                            <span class="conv-time">
                                {conv.lastMessage?.createdAt ? formatTime(conv.lastMessage.createdAt) : ''}
                            </span>
                        </div>
                        {#if conv.productTitle}
                            <div class="conv-product">
                                {i18n.t.inboxChat.inquiryFor} <strong>{conv.productTitle}</strong>
                            </div>
                        {/if}
                        <div class="conv-snippet">
                            {conv.lastMessage?.body || i18n.t.inboxChat.startOfConversation}
                        </div>
                    </div>
                </button>
            {/each}
        {/if}
    </div>
</div>

<style>
    .inbox-sidebar {
        width: 100%;
        background: #fff;
        display: flex;
        flex-direction: column;
        height: 100%;
    }

    .inbox-header {
        padding: 20px;
        border-bottom: 1px solid #e2e8f0;
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .inbox-header h2 {
        font-size: 1.1rem;
        font-weight: 600;
        margin: 0;
        color: #1e293b;
    }

    .badge {
        background: hsl(162, 72%, 40%);
        color: white;
        font-size: 0.75rem;
        padding: 2px 8px;
        border-radius: 12px;
        font-weight: 600;
    }

    .conversation-list {
        flex: 1;
        overflow-y: auto;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: #94a3b8;
        gap: 12px;
        padding: 20px;
        text-align: center;
    }

    :global(.empty-icon) {
        opacity: 0.5;
    }

    .conv-card {
        width: 100%;
        display: flex;
        gap: 12px;
        padding: 16px 20px;
        border: none;
        background: transparent;
        border-bottom: 1px solid #f1f5f9;
        cursor: pointer;
        text-align: start;
        transition: all 0.2s ease;
    }

    .conv-card:hover {
        background: #f8fafc;
    }

    .conv-card.active {
        background: hsl(162, 72%, 96%);
        border-inline-start: 3px solid hsl(162, 72%, 40%);
    }

    .conv-avatar {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background: #e2e8f0;
        color: #475569;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 600;
        font-size: 1.1rem;
        flex-shrink: 0;
    }

    .conv-avatar-img {
        width: 40px;
        height: 40px;
        border-radius: 8px;
        object-fit: cover;
        border: 1px solid #e2e8f0;
        flex-shrink: 0;
    }

    .active .conv-avatar {
        background: hsl(162, 72%, 40%);
        color: white;
    }

    .conv-details {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .conv-top {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .client-name {
        font-weight: 600;
        color: #1e293b;
        font-size: 0.95rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .conv-time {
        font-size: 0.75rem;
        color: #94a3b8;
    }

    .conv-product {
        font-size: 0.8rem;
        color: #64748b;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .conv-product strong {
        color: #475569;
    }

    .conv-snippet {
        font-size: 0.85rem;
        color: #94a3b8;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
