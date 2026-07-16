import { tick } from "svelte";
import { ui } from "$lib/stores/ui.store";
import { RBACService, type User } from "../../../../core/auth/rbac.service.js";

export function createMessagesState(getData: () => any) {
  let conversations = $state<any[]>([]);
  let selectedId = $state<string | null>(null);
  let messages = $state<any[]>([]);
  let searchQuery = $state("");
  let isLoading = $state(false);
  let errorMsg = $state("");
  let isDrawerOpen = $state(true);
  let messageContainer = $state<HTMLElement | null>(null);

  // Derived Stats
  const unreadCount = $derived(conversations.filter(c => c.unreadCount > 0).length);
  const activeCount = $derived(conversations.filter(c => c.status === 'active' || c.status === 'open' || !c.status).length);
  const afrahCount = $derived(conversations.filter(c => c.title === 'Afrah Concierge').length);

  const filteredConversations = $derived.by(() => {
    if (!searchQuery.trim()) return conversations;
    const q = searchQuery.toLowerCase();
    return conversations.filter((c) => {
      const matchId = c.id.toLowerCase().includes(q);
      const matchStatus = c.status?.toLowerCase().includes(q);
      const matchParticipants =
        c.participants?.some(
          (p: any) =>
            p.name?.toLowerCase().includes(q) ||
            p.email?.toLowerCase().includes(q),
        ) || false;
      const matchBody = c.lastMessage?.body?.toLowerCase().includes(q) || false;
      return matchId || matchStatus || matchParticipants || matchBody;
    });
  });

  const activeConv = $derived(conversations.find((c) => c.id === selectedId));

  async function scrollToBottom(behavior: ScrollBehavior = "smooth") {
    await tick();
    if (messageContainer) {
      messageContainer.scrollTo({
        top: messageContainer.scrollHeight,
        behavior,
      });
    }
  }

  async function loadConversations() {
    ui.setLoading(true);
    errorMsg = "";
    isLoading = true;
    try {
      const res = await fetch(`/api/v1/conversations`);
      if (res.ok) {
        const body = await res.json();
        if (body.status === "success") conversations = body.conversations;
      } else {
        errorMsg = "Failed to load conversations.";
      }
    } catch (e) {
      errorMsg = "Network error.";
    } finally {
      ui.setLoading(false);
      isLoading = false;
    }
  }

  async function selectConversation(id: string) {
    selectedId = id;
    messages = [];
    ui.setLoading(true);
    errorMsg = "";
    try {
      const res = await fetch(
        `/api/v1/admin/conversations/${id}/messages`
      );
      if (res.ok) {
        const body = await res.json();
        if (body.status === "success") {
          messages = body.data || [];
          
          if (typeof window !== 'undefined' && (window as any).__updateAdminUnreadCounts) {
            (window as any).__updateAdminUnreadCounts();
          }
          
          await scrollToBottom("smooth");
        }
      } else {
        errorMsg = "Failed to load messages.";
      }
    } catch (e) {
      errorMsg = "Network error.";
    } finally {
      ui.setLoading(false);
    }
  }

  function resolveParticipant(senderId: string) {
    if (!selectedId)
      return { name: "Unknown", email: "", role: "user", initials: "U" };
    const part = activeConv?.participants?.find((p: any) => p.userId === senderId);
    if (!part)
      return { name: "Unknown", email: "", role: "user", initials: "?" };
    const name = part.name || part.email || "Unknown";
    const initials = name
      .split(" ")
      .map((w: string) => w[0])
      .join("")
      .slice(0, 2)
      .toUpperCase();
    return {
      name,
      email: part.email || "",
      role: (part.role || "user").toLowerCase(),
      initials,
    };
  }

  async function sendMessage(body: string, currentUser?: User | null) {
    if (!selectedId || !body.trim()) return;
    const tempId = `temp-${Date.now()}`;
    const newMsg = {
      id: tempId,
      senderId: currentUser?.id || "admin",
      body: body.trim(),
      createdAt: new Date().toISOString(),
      status: "sending",
      attachments: []
    };
    
    messages = [...messages, newMsg];
    await scrollToBottom("smooth");

    try {
      const res = await fetch(`/api/v1/admin/conversations/${selectedId}/messages`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ body: body.trim() })
      });
      if (res.ok) {
        const resBody = await res.json();
        if (resBody.status === "success") {
          messages = messages.map(m => m.id === tempId ? resBody.message : m);
        } else {
          messages = messages.map(m => m.id === tempId ? { ...m, status: "failed" } : m);
        }
      } else {
        messages = messages.map(m => m.id === tempId ? { ...m, status: "failed" } : m);
      }
    } catch (e) {
      messages = messages.map(m => m.id === tempId ? { ...m, status: "failed" } : m);
    }
  }

  async function deleteThread(id: string, currentUser?: User | null) {
    if (!RBACService.canDeleteThread(currentUser)) {
      errorMsg = "Unauthorized to delete conversations.";
      return;
    }
    ui.setLoading(true);
    try {
      const res = await fetch(`/api/v1/admin/conversations/${id}`, {
        method: "DELETE"
      });
      if (res.ok) {
        conversations = conversations.filter(c => c.id !== id);
        if (selectedId === id) {
          selectedId = null;
          messages = [];
        }
      } else {
        errorMsg = "Failed to delete conversation thread.";
      }
    } catch (e) {
      errorMsg = "Network error deleting conversation.";
    } finally {
      ui.setLoading(false);
    }
  }

  async function flagUser(userId: string, currentUser?: User | null) {
    if (!RBACService.canFlagUser(currentUser)) {
      errorMsg = "Unauthorized to flag users.";
      return;
    }
    ui.setLoading(true);
    try {
      const res = await fetch(`/api/v1/admin/users/${userId}/flag`, {
        method: "PATCH"
      });
      if (res.ok) {
        // Success feedback
        errorMsg = "User flagged successfully.";
      } else {
        errorMsg = "Failed to flag user.";
      }
    } catch (e) {
      errorMsg = "Network error flagging user.";
    } finally {
      ui.setLoading(false);
    }
  }

  async function blockMessage(messageId: string, currentUser?: User | null) {
    if (!RBACService.canBlockMessage(currentUser)) {
      errorMsg = "Unauthorized to block messages.";
      return;
    }
    ui.setLoading(true);
    try {
      const res = await fetch(`/api/v1/admin/messages/${messageId}`, {
        method: "DELETE"
      });
      if (res.ok) {
        messages = messages.map(m => m.id === messageId ? { ...m, status: "redacted", isRedacted: true } : m);
      } else {
        errorMsg = "Failed to block message.";
      }
    } catch (e) {
      errorMsg = "Network error blocking message.";
    } finally {
      ui.setLoading(false);
    }
  }

  function exportTranscript() {
    if (!selectedId || messages.length === 0) return;
    const exportData = {
      conversationId: selectedId,
      exportedAt: new Date().toISOString(),
      participants: activeConv?.participants || [],
      messages: messages.map((m) => ({
        id: m.id,
        senderId: m.senderId,
        body: m.body,
        createdAt: m.createdAt,
        attachments: m.attachments || [],
      })),
    };
    const blob = new Blob([JSON.stringify(exportData, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `transcript_${selectedId}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  return {
    get conversations() { return conversations; },
    set conversations(v) { conversations = v; },

    get selectedId() { return selectedId; },
    set selectedId(v) { selectedId = v; },

    get messages() { return messages; },
    set messages(v) { messages = v; },

    get searchQuery() { return searchQuery; },
    set searchQuery(v) { searchQuery = v; },

    get isLoading() { return isLoading; },
    set isLoading(v) { isLoading = v; },

    get errorMsg() { return errorMsg; },
    set errorMsg(v) { errorMsg = v; },

    get isDrawerOpen() { return isDrawerOpen; },
    set isDrawerOpen(v) { isDrawerOpen = v; },

    get messageContainer() { return messageContainer; },
    set messageContainer(v) { messageContainer = v; },

    get unreadCount() { return unreadCount; },
    get activeCount() { return activeCount; },
    get afrahCount() { return afrahCount; },
    get filteredConversations() { return filteredConversations; },
    get activeConv() { return activeConv; },

    loadConversations,
    selectConversation,
    resolveParticipant,
    sendMessage,
    deleteThread,
    flagUser,
    blockMessage,
    exportTranscript,
  };
}

export type MessagesState = ReturnType<typeof createMessagesState>;
