import { wsClient } from '../../../core/websocket/wsClient';

export interface ChatMessage {
  id: string;
  threadId: string;
  senderId: string;
  content: string;
  timestamp: string;
  [key: string]: any;
}

class ChatStore {
  public unreadThreads = $state(0);
  public messages = $state<Record<string, ChatMessage[]>>({});

  // Update from REST/polling mechanism
  public updateFromRest(threadId: string, msgs: ChatMessage[], unread?: number) {
    this.messages[threadId] = msgs;
    if (unread !== undefined) {
      this.unreadThreads = unread;
    }
  }

  // Update from WebSocket push
  public updateFromWs(payload: any) {
    if (payload.unreadThreads !== undefined) {
      this.unreadThreads = payload.unreadThreads;
    }
    if (payload.message) {
      const msg = payload.message as ChatMessage;
      const threadId = msg.threadId;
      
      if (!this.messages[threadId]) {
        this.messages[threadId] = [];
      }
      // Add the message to the thread
      this.messages[threadId].push(msg);
    }
  }

  public initWsListener() {
    // Wire up to the generic wsClient
    wsClient.on('CHAT_MESSAGE', (payload) => this.updateFromWs(payload));
  }
}

export const chatStore = new ChatStore();
