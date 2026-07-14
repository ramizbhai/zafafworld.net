import { wsClient } from '../../../core/websocket/wsClient';

export interface Notification {
  id: string;
  message: string;
  read: boolean;
  createdAt: string;
  [key: string]: any;
}

class NotificationStore {
  public unreadCount = $state(0);
  public notifications = $state<Notification[]>([]);

  // Update from existing REST/polling mechanism
  public updateFromRest(data: { unreadCount?: number; notifications?: Notification[] }) {
    if (data.unreadCount !== undefined) {
      this.unreadCount = data.unreadCount;
    }
    if (data.notifications) {
      this.notifications = data.notifications;
    }
  }

  // Update from WebSocket push
  public updateFromWs(payload: any) {
    if (payload.unreadCount !== undefined) {
      this.unreadCount = payload.unreadCount;
    }
    if (payload.notification) {
      // Prepend new notification to the list
      this.notifications = [payload.notification, ...this.notifications];
    }
  }

  public initWsListener() {
    // Wire up to the generic wsClient
    wsClient.on('NOTIFICATION_UPDATE', (payload) => this.updateFromWs(payload));
  }
}

export const notificationStore = new NotificationStore();
