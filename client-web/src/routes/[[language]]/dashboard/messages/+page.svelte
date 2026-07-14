<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { page } from '$app/stores';
  import { env } from '$env/dynamic/public';
  import { fade, slide } from 'svelte/transition';
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { resolveMediaUrl, getOptimizedImage } from '$lib/shared/utils/media.js';

  interface Props {
    data: {
      token: string;
      // user is now fetched server-side in +page.server.ts — no longer undefined
      user: {
        id: string;
        email: string;
        first_name?: string;
        last_name?: string;
      } | null;
    };
  }
  let { data }: Props = $props();

  // AbortController for all in-flight fetch requests.
  // Aborted in onDestroy() so that rapid sidebar navigation never leaves orphaned
  // rejected promises that crash the SvelteKit navigation lifecycle.
  let fetchController = new AbortController();

  const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
  const WS_BASE = env.PUBLIC_WS_URL || API_BASE.replace('http://', 'ws://').replace('https://', 'wss://');

  // App State
  let conversations = $state<any[]>([]);
  let selectedId = $state<string | null>(null);
  let messages = $state<any[]>([]);
  let messageText = $state('');
  let filesToUpload = $state<FileList | null>(null);
  let uploadPreviews = $state<string[]>([]);
  let isSending = $state(false);
  let errorMsg = $state('');
  let inputEl = $state<HTMLTextAreaElement | null>(null);
  let messagesContainer = $state<HTMLElement | null>(null);

  // Selected conversation computed helper
  let selectedConv = $derived(conversations.find(c => c.id === selectedId) || null);

  // Reactive URL checking to auto-select conversation
  $effect(() => {
    const urlChatId = $page.url.searchParams.get('chatId');
    if (urlChatId && urlChatId !== selectedId) {
      selectConversation(urlChatId);
    }
  });

  // Auto-focus input when conversation is selected
  $effect(() => {
    if (selectedId && inputEl) {
      inputEl.focus();
    }
  });

  // Scroll messages container to bottom
  $effect(() => {
    if (messages.length && messagesContainer) {
      tick().then(() => {
        if (messagesContainer) {
          messagesContainer.scrollTop = messagesContainer.scrollHeight;
        }
      });
    }
  });

  // Resiliency & Telemetry State
  let transport = $state<'WebSocket (Primary)' | 'SSE (Fallback)' | 'Connecting...'>('Connecting...');
  let reconnectAttempts = $state(0);
  let latencyMs = $state<number | null>(null);
  let deliveryRate = $state<number>(100);
  let activeUsers = $state<string[]>([]);

  let ws: WebSocket | null = null;
  let sse: EventSource | null = null;
  let reconnectTimer: any = null;
  let backoffDelay = 1000;

  // Latency & delivery tracking map
  let sentTimestamps = new Map<string, number>();
  let messagesAttempted = 0;
  let messagesDelivered = 0;

  // Fetch conversations (State Synchronization Pattern)
  async function syncConversations() {
    try {
      const res = await fetch(`${API_BASE}/api/v1/client/conversations`, {
        headers: { 'Authorization': `Bearer ${data.token}` },
        signal: fetchController.signal
      });
      if (res.ok) {
        const body = await res.json();
        if (body.status === 'success') {
          conversations = body.data || [];
          const urlChatId = $page.url.searchParams.get('chatId');
          if (urlChatId && !selectedId) {
            selectConversation(urlChatId);
          }
        }
      }
    } catch (e: any) {
      // AbortError is expected on navigation — don't pollute the console
      if (e?.name !== 'AbortError') {
        console.error('Failed to sync conversations:', e);
      }
    }
  }

  // Fetch messages for selected conversation
  async function selectConversation(id: string) {
    selectedId = id;
    messages = [];
    errorMsg = '';
    try {
      const res = await fetch(`${API_BASE}/api/v1/conversations/${id}/messages`, {
        headers: { 'Authorization': `Bearer ${data.token}` },
        signal: fetchController.signal
      });
      if (res.ok) {
        const body = await res.json();
        if (body.status === 'success') {
          messages = body.data || [];
          if (messages.length > 0) {
            const lastMsg = messages[messages.length - 1];
            // Guard: data.user may be null if server profile fetch timed out
            if (data.user && lastMsg.senderId !== data.user.id) {
              await markAsRead(lastMsg.id);
            }
          }
        }
      }
    } catch (e: any) {
      if (e?.name !== 'AbortError') {
        console.error('Failed to load messages:', e);
      }
    }
  }

  async function markAsRead(messageId: string) {
    try {
      await fetch(`${API_BASE}/api/v1/messages/${messageId}/read`, {
        method: 'PATCH',
        headers: { 'Authorization': `Bearer ${data.token}` },
        signal: fetchController.signal
      });
      // Locally update unread status for the conversation
      const convIndex = conversations.findIndex(c => c.id === selectedId);
      if (convIndex !== -1) {
        conversations[convIndex].unread_count = 0;
      }
    } catch (e: any) {
      if (e?.name !== 'AbortError') {
        console.error('Failed to mark message read:', e);
      }
    }
  }

  // File picker selection change
  function handleFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files) {
      filesToUpload = input.files;
      uploadPreviews = [];
      for (let i = 0; i < input.files.length; i++) {
        const file = input.files[i];
        if (!file.type.startsWith('image/') && !file.type.startsWith('video/')) {
          errorMsg = 'Only image and video files are allowed.';
          filesToUpload = null;
          uploadPreviews = [];
          return;
        }
        if (file.size > 20 * 1024 * 1024) {
          errorMsg = 'File size exceeds 20MB limit.';
          filesToUpload = null;
          uploadPreviews = [];
          return;
        }
        errorMsg = '';
        const reader = new FileReader();
        reader.onload = (ev) => {
          if (ev.target?.result) uploadPreviews.push(ev.target.result as string);
        };
        reader.readAsDataURL(file);
      }
    }
  }

  // Send message flow with Optimistic UI
  async function sendMessage(e: Event) {
    e.preventDefault();
    if (!selectedId || (messageText.trim() === '' && !filesToUpload)) return;

    const bodyText = messageText;
    const files = filesToUpload;

    // Clear inputs immediately for instant feedback
    messageText = '';
    filesToUpload = null;
    uploadPreviews = [];
    isSending = true;
    errorMsg = '';

    const tempId = crypto.randomUUID();
    sentTimestamps.set(tempId, performance.now());
    messagesAttempted++;

    // Generate optimistic attachments
    const optimisticAttachments = files ? Array.from(files).map(f => ({
      fileName: f.name,
      fileUrl: URL.createObjectURL(f),
      fileType: f.type,
      fileSize: f.size,
      isOptimistic: true
    })) : [];

    // Append optimistic message
    const optimisticMessage = {
      id: tempId,
      tempId: tempId,
      senderId: data.user?.id ?? '',
      body: bodyText,
      createdAt: new Date().toISOString(),
      attachments: optimisticAttachments,
      status: 'sending',
      readReceipts: []
    };

    messages = [...messages, optimisticMessage];

    try {
      // 1. Upload attachments if any
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
          
          if (!attachRes.ok) {
            throw new Error('Attachment upload failed');
          }
          
          const attachJson = await attachRes.json();
          attachmentsData.push({
            file_name: attachJson.file_name,
            file_url: attachJson.file_url,
            file_type: attachJson.file_type,
            file_size: attachJson.file_size
          });
        }
      }

      // 2. Send text message + attachments
      const res = await fetch(`${API_BASE}/api/v1/conversations/${selectedId}/messages`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${data.token}`
        },
        body: JSON.stringify({ 
          body: bodyText,
          tempId: tempId,
          attachments: attachmentsData.length > 0 ? attachmentsData : undefined
        })
      });

      if (!res.ok) {
        throw new Error('Server returned an error when sending message');
      }

      const result = await res.json();
      const realMessageId = result.message_id || result.messageId;

      // Update optimistic message with real message details
      messages = messages.map(m => {
        if (m.tempId === tempId) {
          return {
            ...m,
            id: realMessageId,
            status: 'sent',
            attachments: attachmentsData.map(att => ({
              fileName: att.file_name,
              fileUrl: att.file_url,
              fileType: att.file_type,
              fileSize: att.file_size
            }))
          };
        }
        return m;
      });

      messagesDelivered++;
      deliveryRate = Math.round((messagesDelivered / messagesAttempted) * 100);

      // Sync latest state from server
      await syncConversations();

    } catch (e: any) {
      console.error(e);
      errorMsg = e.message || 'Failed to send message.';
      
      // Update optimistic message to show failure
      messages = messages.map(m => {
        if (m.tempId === tempId) {
          return { ...m, status: 'failed' };
        }
        return m;
      });

      deliveryRate = Math.round((messagesDelivered / messagesAttempted) * 100);
    } finally {
      isSending = false;
    }
  }

  // Real-time synchronization handlers
  function handleIncomingMessage(msg: any) {
    if (msg.conversationId === selectedId) {
      // Find index of message with the same tempId or id
      const existingIdx = messages.findIndex(m => m.id === msg.id || (msg.tempId && m.tempId === msg.tempId));
      if (existingIdx !== -1) {
        // Update it
        messages[existingIdx] = {
          ...messages[existingIdx],
          id: msg.id,
          status: 'sent',
          attachments: msg.attachments,
          body: msg.body,
          createdAt: msg.createdAt
        };
      } else {
        // Append it
        messages = [...messages, msg];
        if (msg.senderId !== data.user?.id) {
          markAsRead(msg.id);
        }
      }
    }

    // Update conversation list snippet
    const convIndex = conversations.findIndex(c => c.id === msg.conversationId);
    if (convIndex !== -1) {
      conversations[convIndex].lastMessage = {
        body: msg.body,
        createdAt: msg.createdAt,
        senderId: msg.senderId
      };
      if (msg.senderId !== data.user?.id && msg.conversationId !== selectedId) {
        conversations[convIndex].unread_count = (conversations[convIndex].unread_count || 0) + 1;
      }
      // Re-order conversations
      conversations = [
        conversations[convIndex],
        ...conversations.filter((_, idx) => idx !== convIndex)
      ];
    } else {
      syncConversations();
    }
  }

  function handleIncomingReadReceipt(receipt: any) {
    if (receipt.conversationId === selectedId) {
      if (receipt.userId !== data.user?.id) {
        messages = messages.map(m => {
          if (m.senderId === data.user?.id) {
            return { ...m, readReceipts: [{ userId: receipt.userId, readAt: receipt.readAt }] };
          }
          return m;
        });
      }
    }
  }

  // Setup WebSocket Connection
  function setupWebSocket() {
    // Guard: if fetchController is already aborted, we're being destroyed — skip
    if (fetchController.signal.aborted) return;

    try {
      if (sse) {
        sse.close();
        sse = null;
      }
      if (ws) ws.close();

      transport = 'Connecting...';
      ws = new WebSocket(`${WS_BASE}/api/v1/ws?token=${data.token}`);

      ws.onopen = () => {
        if (fetchController.signal.aborted) { ws?.close(); return; }
        transport = 'WebSocket (Primary)';
        reconnectAttempts = 0;
        backoffDelay = 1000;
        syncConversations();
        if (selectedId) selectConversation(selectedId);
      };

      ws.onmessage = (event) => {
        if (fetchController.signal.aborted) return;
        try {
          const data = JSON.parse(event.data);
          if (data.type === 'NEW_MESSAGE') {
            handleIncomingMessage(data.message);
          } else if (data.type === 'READ_RECEIPT') {
            handleIncomingReadReceipt(data);
          }
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      ws.onclose = () => {
        // Only attempt reconnect if we're not being destroyed
        if (!fetchController.signal.aborted && transport !== 'SSE (Fallback)') {
          triggerReconnect();
        }
      };

      ws.onerror = (e) => {
        console.error('WebSocket encountered an error:', e);
        ws?.close();
      };
    } catch (e) {
      // Catches WebSocket constructor errors (e.g. invalid URL) — prevents
      // an unhandled exception from freezing SvelteKit's navigation flow
      console.error('Failed to create WebSocket connection:', e);
      triggerReconnect();
    }
  }

  // Reconnection with Exponential Backoff
  function triggerReconnect() {
    if (reconnectAttempts >= 5) {
      console.warn('Max WebSocket reconnect attempts reached. Falling back to SSE.');
      setupSSE();
      return;
    }

    transport = 'Connecting...';
    reconnectAttempts++;
    clearTimeout(reconnectTimer);
    reconnectTimer = setTimeout(() => {
      setupWebSocket();
    }, backoffDelay);
    backoffDelay *= 2;
  }

  // Setup SSE Fallback
  function setupSSE() {
    if (ws) {
      ws.close();
      ws = null;
    }
    if (sse) sse.close();

    transport = 'SSE (Fallback)';
    sse = new EventSource(`${API_BASE}/api/v1/events/stream`, {
      withCredentials: true
    });

    sse.addEventListener('NEW_MESSAGE_RECEIVED', (event: any) => {
      try {
        const body = JSON.parse(event.data);
        handleIncomingMessage({
          id: body.messageId,
          conversationId: body.conversationId,
          senderId: body.senderId,
          body: body.body,
          createdAt: body.createdAt,
          attachments: body.attachments
        });
      } catch (e) {
        console.error('Failed to parse SSE payload:', e);
      }
    });

    sse.addEventListener('READ_RECEIPT_RECEIVED', (event: any) => {
      try {
        const body = JSON.parse(event.data);
        handleIncomingReadReceipt({
          conversationId: body.conversationId,
          messageId: body.messageId,
          userId: body.userId,
          readAt: body.readAt
        });
      } catch (e) {
        console.error('Failed to parse SSE read receipt:', e);
      }
    });

    sse.onerror = (e) => {
      console.error('SSE connection error:', e);
      // Reconnect with backoff — SSE doesn't auto-reconnect after onerror in all browsers
      if (sse) {
        sse.close();
        sse = null;
      }
      const sseBackoff = Math.min(backoffDelay, 30000);
      backoffDelay *= 2;
      setTimeout(() => {
        console.log('[SSE] Attempting reconnection...');
        setupSSE();
      }, sseBackoff);
    };

    syncConversations();
    if (selectedId) selectConversation(selectedId);
  }

  onMount(() => {
    // Create a fresh AbortController for this mount lifecycle.
    // This ensures each component instance gets its own controller.
    fetchController = new AbortController();
    setupWebSocket();
    syncConversations();
  });

  onDestroy(() => {
    // Abort all in-flight fetch requests immediately so they don't
    // reject after the component is gone and freeze navigation.
    fetchController.abort();
    if (ws) ws.close();
    if (sse) sse.close();
    clearTimeout(reconnectTimer);
  });
</script>

<svelte:head>
  <title>{getLocale() === 'ar' ? 'محادثاتي' : 'My Conversations'} | {m.meta_siteName()}</title>
  <meta name="description" content={getLocale() === 'ar' ? 'تواصل مع مزودي خدمات الزفاف عبر المنصة.' : 'Stay in touch with wedding service vendors entirely inside the platform.'} />
</svelte:head>

<div class="bg-[var(--color-surface)] lg:pt-0 md:py-12 sm:px-6 lg:px-8 font-sans {selectedId ? 'h-[calc(100dvh-80px)] p-0 overflow-hidden lg:h-auto' : 'min-h-[calc(100vh-80px)] py-6 px-4'}">
  <div class="w-full lg:w-[95%] max-w-[1600px] mx-auto {selectedId ? 'h-full flex flex-col lg:h-auto lg:block' : ''}">
    <!-- Header -->
    <div class="mb-6 md:mb-10 flex-col md:flex-row md:items-center md:justify-between gap-6 {selectedId ? 'hidden lg:flex' : 'flex'}">
      <div>
        <h1 class="text-3xl font-display text-[var(--color-secondary)]">My Conversations</h1>
        <p class="text-[var(--color-muted)] mt-1">Direct messaging channel with wedding vendors.</p>
      </div>

      <!-- Telemetry Resiliency Panel -->
      <div class="bg-white/80 backdrop-blur-md border border-[var(--color-border)] rounded-2xl p-4 shadow-sm flex flex-wrap gap-6 items-center">
        <div>
          <div class="text-[var(--color-muted)] text-xs font-semibold uppercase tracking-wider">Transport layer</div>
          <div class="flex items-center gap-2 mt-1">
            <span class="w-2.5 h-2.5 rounded-full" 
                  class:bg-green-500={transport.includes('WebSocket')}
                  class:bg-amber-500={transport.includes('SSE')}
                  class:bg-red-500={transport.includes('Connecting')}></span>
            <span class="font-bold text-[var(--color-secondary)] text-sm">{transport}</span>
          </div>
        </div>

        {#if latencyMs !== null}
          <div>
            <div class="text-[var(--color-muted)] text-xs font-semibold uppercase tracking-wider">Latency</div>
            <div class="font-bold text-[var(--color-secondary)] text-sm mt-1">{latencyMs} ms</div>
          </div>
        {/if}

        <div>
          <div class="text-[var(--color-muted)] text-xs font-semibold uppercase tracking-wider">Delivery rate</div>
          <div class="font-bold text-[var(--color-secondary)] text-sm mt-1">{deliveryRate}%</div>
        </div>

        {#if reconnectAttempts > 0}
          <div>
            <div class="text-[var(--color-muted)] text-xs font-semibold uppercase tracking-wider">Reconnect attempts</div>
            <div class="font-bold text-[var(--color-secondary)] text-sm mt-1">{reconnectAttempts}</div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Responsive Split-Pane Container -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-0 lg:gap-8 bg-white border-0 lg:border border-[var(--color-border)] lg:rounded-3xl overflow-hidden shadow-none lg:shadow-lg min-h-[600px] {selectedId ? 'flex-1 rounded-none' : 'rounded-2xl border shadow-lg'}">
      <!-- Conversation List (Left Panel) -->
      <div class="lg:col-span-4 border-e border-[var(--color-border)] flex-col bg-white {selectedId !== null ? 'hidden lg:flex' : 'flex'}">
        <div class="p-6 border-b border-[var(--color-border)]">
          <h2 class="font-semibold text-lg text-[var(--color-secondary)]">Recent Messages</h2>
        </div>

        <div class="flex-1 overflow-y-auto divide-y divide-[var(--color-border)] max-h-[600px]">
          {#if conversations.length === 0}
            <div class="p-8 text-center text-[var(--color-muted)]">
              No conversations found. Contact vendors directly from their profile pages.
            </div>
          {:else}
            {#each conversations as conv}
              <button 
                class="w-full text-start p-5 hover:bg-[var(--color-surface-alt)]/50 transition-colors flex items-start gap-4 border-s-4 border-transparent"
                class:bg-[var(--color-surface-alt)]={selectedId === conv.id}
                class:border-l-[var(--color-primary)]={selectedId === conv.id}
                onclick={() => selectConversation(conv.id)}>
                
                <!-- Avatar / Listing cover image -->
                {#if conv.productCoverImage}
                  <img src={resolveMediaUrl(getOptimizedImage(conv.productCoverImage, 'thumb'))} alt="Listing" class="w-12 h-12 rounded-xl object-cover border border-[var(--color-border)]" />
                {:else}
                  <div class="w-12 h-12 rounded-full bg-[var(--color-primary-light)] flex items-center justify-center text-[var(--color-primary-dark)] font-bold text-lg border border-[var(--color-primary)] shrink-0">
                    {conv.otherParticipant?.name?.charAt(0).toUpperCase() || 'V'}
                  </div>
                {/if}

                <!-- Text Snippet -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center justify-between gap-2">
                    <span class="font-bold text-[var(--color-secondary)] truncate">{conv.otherParticipant?.name || 'Vendor'}</span>
                    {#if conv.unread_count > 0}
                      <span class="bg-[var(--color-primary)] text-white text-xs font-extrabold px-2.5 py-1 rounded-full shrink-0">{conv.unread_count}</span>
                    {/if}
                  </div>
                  {#if conv.productTitle}
                    <div class="text-[var(--color-muted)] text-[11px] uppercase tracking-wider font-semibold truncate mt-0.5">
                      Inquiry: {conv.productTitle}
                    </div>
                  {/if}
                  <p class="text-[var(--color-muted)] text-sm truncate mt-1">
                    {conv.lastMessage?.body || 'No messages yet.'}
                  </p>
                </div>
              </button>
            {/each}
          {/if}
        </div>
      </div>

      <!-- Chat Thread (Right Panel) -->
      <div class="lg:col-span-8 flex-col bg-slate-50/20 {selectedId === null ? 'hidden lg:flex min-h-[500px]' : 'flex h-full lg:h-auto lg:min-h-[500px]'}">
        {#if !selectedId}
          <div class="flex-1 flex flex-col items-center justify-center p-12 text-center">
            <span class="text-5xl mb-4">💬</span>
            <h3 class="font-bold text-xl text-[var(--color-secondary)]">Select a Conversation</h3>
            <p class="text-[var(--color-muted)] mt-1">Choose a conversation from the list to start messaging.</p>
          </div>
        {:else}
          <!-- Chat Header -->
          <div class="p-6 border-b border-[var(--color-border)] flex items-center gap-4 bg-white shadow-sm shrink-0">
            <!-- Mobile Back Button -->
            <button 
              class="lg:hidden p-2 rounded-full hover:bg-[var(--color-surface-alt)] me-1 text-xl font-bold text-[var(--color-secondary)] transition-colors"
              onclick={() => selectedId = null}
              aria-label="Back to messages">
              ←
            </button>

            <div class="w-10 h-10 rounded-full bg-[var(--color-primary-light)] flex items-center justify-center text-[var(--color-primary-dark)] font-bold shrink-0">
              {selectedConv?.otherParticipant?.name?.charAt(0).toUpperCase() || 'V'}
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="font-bold text-[var(--color-secondary)] truncate">
                {selectedConv?.otherParticipant?.name || 'Vendor'}
              </h3>
              <p class="text-xs text-green-600 mt-0.5 flex items-center gap-1">
                <span class="inline-block w-1.5 h-1.5 rounded-full bg-green-500"></span> Active Channel
              </p>
            </div>
          </div>

          <!-- Sticky Listing Context Preview Card -->
          {#if selectedConv?.productId}
            <div class="bg-amber-50/80 border-b border-amber-100 p-4 flex items-center justify-between gap-4 shrink-0 shadow-inner backdrop-blur-md">
              <div class="flex items-center gap-3 min-w-0">
                {#if selectedConv.productCoverImage}
                  <img src={resolveMediaUrl(getOptimizedImage(selectedConv.productCoverImage, 'thumb'))} alt="Listing" class="w-12 h-12 rounded-lg object-cover border border-amber-200" />
                {/if}
                <div class="min-w-0">
                  <div class="text-xs font-semibold text-amber-800 uppercase tracking-wider">Inquiring about:</div>
                  <div class="font-bold text-amber-950 truncate text-sm">{selectedConv.productTitle || 'Wedding Listing'}</div>
                </div>
              </div>
              {#if selectedConv.productPrice}
                <div class="text-end shrink-0">
                  <div class="text-[10px] text-amber-700">Base Price</div>
                  <div class="font-display font-extrabold text-amber-950 text-sm">SAR {selectedConv.productPrice.toLocaleString()}</div>
                </div>
              {/if}
            </div>
          {/if}

          <!-- Message History -->
          <div class="flex-1 overflow-y-auto p-6 space-y-4 max-h-none lg:max-h-[500px]" bind:this={messagesContainer}>
            {#if messages.length === 0}
              <div class="flex flex-col items-center justify-center h-full text-center text-[var(--color-muted)] py-12">
                <span class="text-3xl mb-2">✉️</span>
                <p>No messages yet. Send the first message!</p>
              </div>
            {:else}
              {#each messages as msg}
                {@const isMine = msg.senderId === (data.user?.id ?? '')}
                <div class="flex flex-col" class:items-end={isMine} class:items-start={!isMine}>
                  <div class="max-w-[70%] rounded-2xl p-4 shadow-sm transition-all"
                       class:bg-[var(--color-primary)]={isMine && msg.status !== 'failed'}
                       class:text-white={isMine && msg.status !== 'failed'}
                       class:bg-red-500={msg.status === 'failed'}
                       class:bg-white={!isMine}
                       class:text-[var(--color-text)]={!isMine}
                       class:border={!isMine}
                       class:border-[var(--color-border)]={!isMine}>
                    
                    <!-- Message text -->
                    {#if msg.body}
                      <p class="text-sm leading-relaxed whitespace-pre-wrap">{msg.body}</p>
                    {/if}

                    <!-- Attachments inside message bubble -->
                    {#if msg.attachments && msg.attachments.length > 0}
                      <div class="mt-3 space-y-2">
                        {#each msg.attachments as att}
                          {#if att.fileType?.startsWith('image/')}
                            <a href={resolveMediaUrl(att.fileUrl)} target="_blank" class="block rounded-lg overflow-hidden border border-slate-200/20 max-w-xs hover:opacity-90">
                              <img src={resolveMediaUrl(getOptimizedImage(att.fileUrl, 'card'))} alt={att.fileName || 'Image attachment'} class="w-full object-cover max-h-48 hover:scale-102 transition-transform" />
                            </a>
                          {:else}
                            <a href={resolveMediaUrl(att.fileUrl)} target="_blank" class="flex items-center gap-2 text-xs bg-black/10 hover:bg-black/20 p-2.5 rounded-lg transition-colors border border-black/5">
                              <span class="text-base">📎</span>
                              <span class="truncate max-w-[180px] font-medium">{att.fileName || 'Download attachment'}</span>
                            </a>
                          {/if}
                        {/each}
                      </div>
                    {/if}

                    <!-- Message Metadata (Time, Status and Receipts) -->
                    <div class="flex items-center justify-end gap-1.5 mt-2 text-[10px] {isMine ? 'text-white/80' : 'text-[var(--color-muted)]'}">
                      <span>{new Date(msg.createdAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                      
                      {#if isMine}
                        {#if msg.status === 'sending'}
                          <span class="animate-pulse">⏳</span>
                        {:else if msg.status === 'failed'}
                          <span class="text-red-200" title="Click to retry?">⚠️ failed</span>
                        {:else if msg.readReceipts && msg.readReceipts.length > 0}
                          <span class="text-blue-300 font-extrabold">✓✓</span>
                        {:else}
                          <span class="opacity-70">✓</span>
                        {/if}
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            {/if}
          </div>

          <!-- Bottom Editor -->
          <div class="p-4 md:p-6 border-t border-[var(--color-border)] bg-white shadow-lg shrink-0">
            {#if errorMsg}
              <div class="mb-4 text-xs text-[var(--color-error)] bg-red-50 border border-red-200 p-3 rounded-xl flex items-center justify-between" transition:slide>
                <span>{errorMsg}</span>
                <button onclick={() => errorMsg = ''} class="font-extrabold hover:text-red-700 ms-2">×</button>
              </div>
            {/if}

            <!-- File Upload Previews -->
            {#if uploadPreviews.length > 0}
              <div class="flex gap-3 mb-4 overflow-x-auto py-2" transition:slide>
                {#each uploadPreviews as preview}
                  <div class="relative w-20 h-20 rounded-xl overflow-hidden border border-[var(--color-border)] group">
                    <img src={preview} alt="Preview" class="w-full h-full object-cover" />
                  </div>
                {/each}
              </div>
            {/if}

            <form onsubmit={sendMessage} class="flex items-end gap-4">
              <!-- Upload Trigger -->
              <label class="flex items-center justify-center p-3 rounded-full hover:bg-[var(--color-surface-alt)] border border-[var(--color-border)] cursor-pointer transition-colors bg-white shadow-sm shrink-0">
                <span class="text-xl">📷</span>
                <input type="file" accept="image/*,video/*" multiple class="hidden" onchange={handleFileChange} />
              </label>

              <!-- Text Input -->
              <div class="flex-1 relative">
                <textarea 
                  bind:this={inputEl}
                  bind:value={messageText} 
                  placeholder="Type your message here..."
                  class="w-full border border-[var(--color-border)] rounded-2xl px-5 py-3 pe-10 text-sm focus:outline-none focus:border-[var(--color-primary)] focus:ring-1 focus:ring-[var(--color-primary)] transition-all resize-none shadow-inner"
                  rows="2"
                  onkeydown={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      sendMessage(e);
                    }
                  }}></textarea>
              </div>

              <!-- Submit -->
              <button 
                type="submit" 
                disabled={isSending || (messageText.trim() === '' && !filesToUpload)}
                class="gradient-gold text-white font-bold text-sm px-6 py-3.5 rounded-full shadow-[var(--shadow-sm)] hover:shadow-[var(--shadow-md)] transition-all disabled:opacity-50 flex items-center gap-2 shrink-0">
                {#if isSending}
                  <span class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
                {:else}
                  <span>Send</span>
                {/if}
              </button>
            </form>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
