export interface WebSocketOptions {
  url: string;
  onMessage?: (data: any) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Event) => void;
  reconnect?: boolean;
}

type WsMessageHandler = (payload: any) => void;

export class WsClient {
  private socket: WebSocket | null = null;
  private url: string = '';
  private reconnecting: boolean = false;
  private retryCount: number = 0;
  private maxRetries: number = 7; // Cap retries at 7 attempts
  private baseBackoffMs: number = 1000;
  private listeners: Record<string, WsMessageHandler[]> = {};
  
  private onConnectCallback?: () => void;
  private onDisconnectCallback?: () => void;
  private onErrorCallback?: (error: Event) => void;

  public connect(options: WebSocketOptions): void {
    if (this.socket && (this.socket.readyState === WebSocket.OPEN || this.socket.readyState === WebSocket.CONNECTING)) {
      return;
    }

    this.url = options.url;
    this.onConnectCallback = options.onConnect;
    this.onDisconnectCallback = options.onDisconnect;
    this.onErrorCallback = options.onError;
    
    if (options.onMessage) {
      this.on('*', options.onMessage);
    }

    this.initSocket();
  }

  private initSocket(): void {
    if (typeof window === 'undefined') return; // Do not attempt connection in SSR

    try {
      // The browser automatically attaches cookies (e.g. zafaf_admin_session) 
      // if the endpoint is same-origin or credentials are allowed.
      this.socket = new WebSocket(this.url);

      this.socket.onopen = () => {
        this.reconnecting = false;
        this.retryCount = 0;
        this.onConnectCallback?.();
      };

      this.socket.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          const type = data.type || '*';
          
          if (this.listeners[type]) {
            this.listeners[type].forEach(handler => handler(data));
          }
          if (type !== '*' && this.listeners['*']) {
            this.listeners['*'].forEach(handler => handler(data));
          }
        } catch (e) {
          if (this.listeners['*']) {
            this.listeners['*'].forEach(handler => handler(event.data));
          }
        }
      };

      this.socket.onclose = (event) => {
        this.onDisconnectCallback?.();
        this.socket = null;
        
        if (event.code === 4401 || event.code === 4403) {
          console.error(`WebSocket auth failed (Code ${event.code}). Halting reconnects.`);
          this.retryCount = this.maxRetries; // Halt reconnects
        } else {
          this.handleReconnect();
        }
      };

      this.socket.onerror = (error) => {
        this.onErrorCallback?.(error);
      };
    } catch (e) {
      console.error('Failed to initialize WebSocket', e);
      this.handleReconnect();
    }
  }

  private handleReconnect(): void {
    if (this.reconnecting || this.retryCount >= this.maxRetries) return;
    this.reconnecting = true;
    
    // Exponential backoff capped at 30 seconds
    const delay = Math.min(this.baseBackoffMs * Math.pow(2, this.retryCount), 30000);
    this.retryCount++;
    
    console.log(`WebSocket reconnecting in ${delay}ms... (Attempt ${this.retryCount}/${this.maxRetries})`);
    setTimeout(() => {
      this.reconnecting = false;
      this.initSocket();
    }, delay);
  }

  public on(messageType: string, handler: WsMessageHandler): void {
    if (!this.listeners[messageType]) {
      this.listeners[messageType] = [];
    }
    this.listeners[messageType].push(handler);
  }

  public off(messageType: string, handler: WsMessageHandler): void {
    if (!this.listeners[messageType]) return;
    this.listeners[messageType] = this.listeners[messageType].filter(h => h !== handler);
  }

  public send(payload: any): void {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(typeof payload === 'string' ? payload : JSON.stringify(payload));
    } else {
      console.warn('WebSocket is not connected. Cannot send message.');
    }
  }

  public destroy(): void {
    this.retryCount = this.maxRetries; // Prevent further reconnection attempts
    this.listeners = {};
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
  }
}

export const wsClient = new WsClient();
