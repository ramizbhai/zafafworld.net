<script lang="ts">
    import { fade, fly } from 'svelte/transition';
    import { X, MessageSquare, AlertCircle, Send } from 'lucide-svelte';
    import { env } from '$env/dynamic/public';
    import { onDestroy } from 'svelte';

    let { vendorsState } = $props<{ vendorsState: any }>();

    function closeChat() {
        vendorsState.chatVendor = null;
        vendorsState.chatMessages = [];
        if (vendorsState.pollInterval) {
            clearInterval(vendorsState.pollInterval);
            vendorsState.pollInterval = null;
        }
    }

    async function fetchMessages() {
        if (!vendorsState.chatVendor) return;
        vendorsState.isFetchingMessages = true;
        try {
            const res = await fetch(`/dashboard/vendors/${vendorsState.chatVendor.id}/chat`);
            if (res.ok) {
                const data = await res.json();
                vendorsState.chatMessages = data.messages || [];
                
                // Trigger optimistic unread count update for Admin
                if (typeof window !== 'undefined' && (window as any).__updateAdminUnreadCounts) {
                    (window as any).__updateAdminUnreadCounts();
                }
            } else {
                vendorsState.chatError = 'Failed to load support thread messages';
            }
        } catch (e) {
            vendorsState.chatError = 'Backend service connection error';
        } finally {
            vendorsState.isFetchingMessages = false;
        }
    }

    async function sendChatMessage() {
        if (!vendorsState.chatVendor || !vendorsState.chatInput.trim() || vendorsState.isSendingMessage) return;
        vendorsState.isSendingMessage = true;
        vendorsState.chatError = '';
        const bodyText = vendorsState.chatInput.trim();
        try {
            const res = await fetch(`/dashboard/vendors/${vendorsState.chatVendor.id}/chat`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ body: bodyText })
            });
            if (res.ok) {
                vendorsState.chatInput = '';
                await fetchMessages();
                setTimeout(() => {
                    const container = document.querySelector('.chat-history');
                    if (container) {
                        container.scrollTop = container.scrollHeight;
                    }
                }, 100);
            } else {
                vendorsState.chatError = 'Failed to dispatch message';
            }
        } catch (e) {
            vendorsState.chatError = 'Backend service connection error';
        } finally {
            vendorsState.isSendingMessage = false;
        }
    }

    $effect(() => {
        const handleVisibility = () => {
            if (typeof document === 'undefined') return;
            if (document.hidden) {
                if (vendorsState.pollInterval) {
                    clearInterval(vendorsState.pollInterval);
                    vendorsState.pollInterval = null;
                }
            } else if (vendorsState.chatVendor) {
                if (!vendorsState.pollInterval) {
                    fetchMessages();
                    vendorsState.pollInterval = setInterval(fetchMessages, 4000);
                }
            }
        };
        if (typeof document !== 'undefined') {
            document.addEventListener('visibilitychange', handleVisibility);
        }
        return () => {
            if (typeof document !== 'undefined') {
                document.removeEventListener('visibilitychange', handleVisibility);
            }
            if (vendorsState.pollInterval) {
                clearInterval(vendorsState.pollInterval);
                vendorsState.pollInterval = null;
            }
        };
    });

    onDestroy(() => {
        if (vendorsState.pollInterval) clearInterval(vendorsState.pollInterval);
    });

    let apiUrl = $derived(env.PUBLIC_API_URL || 'http://localhost:8080');
</script>

{#if vendorsState.chatVendor}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="chat-drawer-backdrop" role="button" tabindex="-1" onclick={closeChat} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') closeChat(); }} transition:fade></div>
    <div class="chat-drawer" transition:fly={{ x: 450, duration: 250 }}>
        
        <div class="chat-header">
            <div class="chat-header-info">
                <h3>{vendorsState.chatVendor.name_en}</h3>
                {#if vendorsState.chatVendor.name_ar}
                    <span class="arabic-subtext" dir="rtl">{vendorsState.chatVendor.name_ar}</span>
                {/if}
            </div>
            <button class="close-btn" onclick={closeChat} title="Close drawer">
                <X size={20} />
            </button>
        </div>

        <div class="chat-messages-container">
            <div class="chat-history">
                {#if vendorsState.isFetchingMessages && vendorsState.chatMessages.length === 0}
                    <div class="chat-loading">
                        <span class="spinner"></span> Loading support thread...
                    </div>
                {:else if vendorsState.chatMessages.length === 0}
                    <div class="chat-empty">
                        <MessageSquare size={36} class="empty-chat-icon" />
                        <p>No support messages yet.</p>
                        <p class="sub">Open a dialog thread with this vendor. Responses will be displayed live on their Pending Wall.</p>
                    </div>
                {:else}
                    {#each vendorsState.chatMessages as msg (msg.id)}
                        <div class="message-bubble-wrapper" class:admin={msg.sender === 'admin'}>
                            <div class="message-bubble">
                                {#if msg.body}
                                    <p class="message-body">{msg.body}</p>
                                {/if}
                                {#if msg.file_url}
                                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                                    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                                    <img 
                                        src={msg.file_url ? (msg.file_url.startsWith('http') ? msg.file_url : `${apiUrl}${msg.file_url}`) : ''} 
                                        alt="Uploaded attachment" 
                                        class="attached-img"
                                        onclick={() => { if (msg.file_url) vendorsState.lightboxUrl = msg.file_url.startsWith('http') ? msg.file_url : `${apiUrl}${msg.file_url}`; }}
                                    />
                                {/if}
                                <span class="message-time">
                                    {new Date(msg.created_at).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                                </span>
                            </div>
                        </div>
                    {/each}
                {/if}
            </div>

            {#if vendorsState.chatError}
                <div class="chat-error">
                    <AlertCircle size={14} />
                    <span>{vendorsState.chatError}</span>
                </div>
            {/if}
        </div>

        <!-- Quick reply shortcuts -->
        <div class="quick-replies">
            <span class="quick-reply-label">Quick Templates / ردود سريعة:</span>
            <div class="quick-reply-buttons">
                <button onclick={() => vendorsState.chatInput = "Hello! We are currently reviewing your onboarding details. Please stand by."}>Reviewing</button>
                <button onclick={() => vendorsState.chatInput = "Please upload higher resolution photos of your products and halls to proceed."}>HQ Photos</button>
                <button onclick={() => vendorsState.chatInput = "Your subscription has expired. Please contact us to update your details."}>Expired</button>
            </div>
        </div>

        <!-- Chat Input area -->
        <form class="chat-input-area" onsubmit={(e) => { e.preventDefault(); sendChatMessage(); }}>
            <input 
                type="text" 
                placeholder="Type reply to vendor..." 
                bind:value={vendorsState.chatInput} 
                disabled={vendorsState.isSendingMessage} 
            />
            <button type="submit" class="btn-send" disabled={vendorsState.isSendingMessage || !vendorsState.chatInput.trim()}>
                {#if vendorsState.isSendingMessage}
                    <span class="spinner"></span>
                {:else}
                    <Send size={16} />
                {/if}
            </button>
        </form>

    </div>
{/if}

<!-- Full Screen Image Lightbox Modal -->
{#if vendorsState.lightboxUrl}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="lightbox" onclick={() => vendorsState.lightboxUrl = ''} transition:fade>
        <button class="lightbox-close" onclick={() => vendorsState.lightboxUrl = ''} aria-label="Close image"><X size={24} /></button>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <img src={vendorsState.lightboxUrl} alt="Full screen attachment" onclick={(e) => e.stopPropagation()} />
    </div>
{/if}
