<script lang="ts">
    import InboxList from '$lib/components/inbox/InboxList.svelte';
    import ChatWindow from '$lib/components/inbox/ChatWindow.svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { getApiBaseUrl, getWsUrl } from '$lib/utils/api';
    import { onMount, onDestroy } from 'svelte';

    let { data } = $props();
    const i18n = getI18n();
    const API_BASE = getApiBaseUrl();
    const WS_BASE = getWsUrl();

    // svelte-ignore state_referenced_locally
    let initialConversations = data.conversations || [];
    let conversations = $state<any[]>(initialConversations);
    let selectedConversation = $state<any>(null);
    let currentMessages = $state<any[]>([]);
    let loadingMessages = $state(false);
    let ws = $state<WebSocket | null>(null);
    let wsStatus = $state<'connecting' | 'connected' | 'disconnected'>('disconnected');
    let reconnectAttempts = 0;

    onMount(() => {
        setupWebSocket();
    });

    onDestroy(() => {
        if (ws) {
            ws.close();
        }
    });

    // Mark preceding messages in selected conversation as read
    async function markAsRead(messageId: string) {
        try {
            await fetch(`${API_BASE}/api/v1/messages/${messageId}/read`, {
                method: 'PATCH',
                headers: { 'Authorization': `Bearer ${data.token}` }
            });
        } catch (e) {
            console.error('Failed to mark message read:', e);
        }
    }

    function setupWebSocket() {
        if (wsStatus === 'connected') return;
        
        wsStatus = 'connecting';
        
        try {
            const socket = new WebSocket(`${WS_BASE}/api/v1/ws?token=${data.token}`);
            
            socket.onopen = () => {
                wsStatus = 'connected';
                reconnectAttempts = 0;
            };
            
            socket.onmessage = (event) => {
                try {
                    const parsed = JSON.parse(event.data);
                    
                    if (parsed.type === 'NEW_MESSAGE') {
                        const newMsg = parsed.message;
                        
                        // Update current messages if it belongs to selected conversation
                        if (selectedConversation && newMsg.conversationId === selectedConversation.id) {
                            const existingIdx = currentMessages.findIndex(m => m.id === newMsg.id || (newMsg.tempId && m.tempId === newMsg.tempId));
                            if (existingIdx !== -1) {
                                currentMessages[existingIdx] = {
                                    ...currentMessages[existingIdx],
                                    id: newMsg.id,
                                    isOptimistic: false,
                                    status: 'sent',
                                    attachments: newMsg.attachments,
                                    body: newMsg.body,
                                    createdAt: newMsg.createdAt
                                };
                            } else {
                                currentMessages = [...currentMessages, newMsg];
                                if (newMsg.senderId !== data.vendorUserId) {
                                    markAsRead(newMsg.id);
                                }
                            }
                        }
                        
                        // Update snippet in conversations list
                        const convIndex = conversations.findIndex((c: any) => c.id === newMsg.conversationId);
                        if (convIndex !== -1) {
                            conversations[convIndex].lastMessage = {
                                body: newMsg.body,
                                createdAt: newMsg.createdAt,
                                senderId: newMsg.senderId
                            };
                            
                            // Re-order list
                            const updatedConv = conversations[convIndex];
                            conversations.splice(convIndex, 1);
                            conversations.unshift(updatedConv);
                        } else {
                            // Sync conversations list to fetch newly created one
                            syncConversationsList();
                        }
                    } else if (parsed.type === 'READ_RECEIPT') {
                        if (selectedConversation && parsed.conversationId === selectedConversation.id) {
                            if (parsed.userId !== data.vendorUserId) {
                                currentMessages = currentMessages.map(m => {
                                    if (m.senderId === data.vendorUserId) {
                                        return { ...m, readReceipts: [{ userId: parsed.userId, readAt: parsed.readAt }] };
                                    }
                                    return m;
                                });
                            }
                        }
                    }
                } catch (e) {
                    console.error("Failed to parse WS message", e);
                }
            };
            
            socket.onclose = () => {
                wsStatus = 'disconnected';
                // Auto reconnect with exponential backoff
                const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 15000);
                reconnectAttempts++;
                setTimeout(setupWebSocket, delay);
            };
            
            socket.onerror = (err) => {
                console.error("WS Error:", err);
                socket.close();
            };
            
            ws = socket;
        } catch (e) {
            console.error("Failed to setup WebSocket", e);
            wsStatus = 'disconnected';
        }
    }

    async function syncConversationsList() {
        try {
            const res = await fetch(`${API_BASE}/api/v1/client/conversations`, {
                headers: { 'Authorization': `Bearer ${data.token}` }
            });
            if (res.ok) {
                const body = await res.json();
                if (body.status === 'success') {
                    conversations = body.data || [];
                }
            }
        } catch (err) {
            console.error("Error syncing conversations:", err);
        }
    }

    async function handleSelectConversation(conv: any) {
        selectedConversation = conv;
        currentMessages = []; // clear while loading
        loadingMessages = true;

        try {
            const res = await fetch(`${API_BASE}/api/v1/conversations/${conv.id}/messages`, {
                headers: { 'Authorization': `Bearer ${data.token}` }
            });
            if (res.ok) {
                const json = await res.json();
                currentMessages = json.data || [];
                
                // Mark latest received message read
                if (currentMessages.length > 0) {
                    const lastMsg = currentMessages[currentMessages.length - 1];
                    if (lastMsg.senderId !== data.vendorUserId) {
                        await markAsRead(lastMsg.id);
                    }
                }
            } else {
                console.error("Failed to load messages");
            }
        } catch (err) {
            console.error("Error fetching messages:", err);
        } finally {
            loadingMessages = false;
        }
    }

    async function handleSendMessage(body: string, files: FileList | null = null) {
        if (!selectedConversation) return;

        // optimistic UI update
        const tempId = crypto.randomUUID();
        
        // Optimistic attachments
        const optimisticAttachments = files ? Array.from(files).map(f => ({
            fileName: f.name,
            fileUrl: URL.createObjectURL(f),
            fileType: f.type,
            fileSize: f.size,
            isOptimistic: true
        })) : [];

        const optimisticMsg = {
            id: tempId,
            tempId: tempId,
            senderId: data.vendorUserId,
            body: body,
            createdAt: new Date().toISOString(),
            isOptimistic: true,
            status: 'sending',
            attachments: optimisticAttachments,
            readReceipts: []
        };
        currentMessages = [...currentMessages, optimisticMsg];

        // Also update snippet in sidebar
        const convIndex = conversations.findIndex((c: any) => c.id === selectedConversation.id);
        if (convIndex !== -1) {
            conversations[convIndex].lastMessage = {
                body: body || (files && files.length > 0 ? "Sent a media attachment" : ""),
                createdAt: optimisticMsg.createdAt,
                senderId: data.vendorUserId
            };
            // Move to top
            const updatedConv = conversations[convIndex];
            conversations.splice(convIndex, 1);
            conversations.unshift(updatedConv);
        }

        try {
            let attachmentsData: any[] = [];
            if (files && files.length > 0) {
                for (let i = 0; i < files.length; i++) {
                    const formData = new FormData();
                    formData.append('file', files[i]);

                    const attachRes = await fetch(`${API_BASE}/api/v1/attachments/upload`, {
                        method: 'POST',
                        headers: { 'Authorization': `Bearer ${data.token}` },
                        body: formData
                    });

                    if (attachRes.ok) {
                        const attachJson = await attachRes.json();
                        attachmentsData.push({
                            file_name: attachJson.file_name,
                            file_url: attachJson.file_url,
                            file_type: attachJson.file_type,
                            file_size: attachJson.file_size
                        });
                    }
                }
            }

            const res = await fetch(`${API_BASE}/api/v1/conversations/${selectedConversation.id}/messages`, {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${data.token}`
                },
                body: JSON.stringify({ 
                    body,
                    tempId,
                    attachments: attachmentsData.length > 0 ? attachmentsData : undefined
                })
            });

            if (res.ok) {
                const result = await res.json();
                const realId = result.message_id || result.messageId;
                
                // Update optimistic message with real message details
                currentMessages = currentMessages.map(m => 
                    m.tempId === tempId ? {
                        ...m,
                        id: realId,
                        isOptimistic: false,
                        status: 'sent',
                        attachments: attachmentsData.map(att => ({
                            fileName: att.file_name,
                            fileUrl: att.file_url,
                            fileType: att.file_type,
                            fileSize: att.file_size
                        }))
                    } : m
                );
            } else {
                console.error("Failed to send message");
                currentMessages = currentMessages.map(m => 
                    m.tempId === tempId ? { ...m, status: 'failed' } : m
                );
            }
        } catch (err) {
            console.error("Error sending message:", err);
            currentMessages = currentMessages.map(m => 
                m.tempId === tempId ? { ...m, status: 'failed' } : m
            );
        }
    }
</script>

<svelte:head>
    <title>{i18n.t.inboxChat.title} | {i18n.t.common.appName}</title>
</svelte:head>

<div class="inbox-layout">
    <!-- Sidebar Container: hidden on mobile if conversation is selected -->
    <div class="sidebar-container {selectedConversation ? 'hidden lg:flex' : 'flex'}">
        <div class="status-indicator-container">
            <div class="status-dot {wsStatus}" aria-label="Connection status"></div>
            <span class="status-text">{wsStatus === 'connected' ? i18n.t.inboxChat.statusOnline : wsStatus === 'connecting' ? i18n.t.inboxChat.statusConnecting : i18n.t.inboxChat.statusOffline}</span>
        </div>
        <InboxList
            conversations={conversations}
            selectedId={selectedConversation?.id}
            onSelect={handleSelectConversation}
        />
    </div>

    <!-- Chat Container: hidden on mobile if no conversation is selected -->
    <div class="chat-container {selectedConversation ? 'flex' : 'hidden lg:flex'}">
        <div class="chat-inner-wrap" dir={i18n.locale === 'ar' ? 'rtl' : 'ltr'}>
            <ChatWindow
                conversation={selectedConversation}
                messages={currentMessages}
                vendorUserId={data.vendorUserId}
                onSendMessage={handleSendMessage}
                onBack={() => selectedConversation = null}
                loadingMessages={loadingMessages}
            />
        </div>
    </div>
</div>

<style>
    .inbox-layout {
        display: flex;
        height: calc(100vh - 120px);
        background: #fff;
        border-radius: 16px;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05), 0 2px 4px -1px rgba(0, 0, 0, 0.03);
        overflow: hidden;
        margin-top: -10px;
    }

    .sidebar-container {
        flex-shrink: 0;
        z-index: 10;
        display: flex;
        flex-direction: column;
        width: 320px;
        border-inline-end: 1px solid #e2e8f0;
    }

    @media (max-width: 1023px) {
        .sidebar-container {
            width: 100%;
            border-inline-end: none;
        }
        .chat-container {
            width: 100%;
        }
    }

    .status-indicator-container {
        padding: 12px 16px;
        border-bottom: 1px solid #e2e8f0;
        display: flex;
        align-items: center;
        gap: 8px;
        background: #f8fafc;
    }

    .status-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
    }
    .status-dot.connected { background: #10b981; box-shadow: 0 0 6px #10b981; }
    .status-dot.connecting { background: #f59e0b; animation: pulse 1.5s infinite; }
    .status-dot.disconnected { background: #ef4444; }

    .status-text {
        font-size: 12px;
        font-weight: 500;
        color: #64748b;
    }

    @keyframes pulse {
        0% { opacity: 0.5; }
        50% { opacity: 1; }
        100% { opacity: 0.5; }
    }

    .chat-container {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
    }

    .chat-inner-wrap {
        flex: 1;
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100%;
        min-width: 0;
    }
</style>
