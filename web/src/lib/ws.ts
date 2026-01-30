type WsEventHandler = (event: WsEvent) => void;

export interface WsEvent {
  type: string;
  data: unknown;
}

class WebSocketClient {
  private ws: WebSocket | null = null;
  private handlers: Set<WsEventHandler> = new Set();
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private url: string;

  constructor(url: string) {
    this.url = url;
  }

  connect() {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log("[WS] Connected");
      if (this.reconnectTimer) {
        clearTimeout(this.reconnectTimer);
        this.reconnectTimer = null;
      }
    };

    this.ws.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data) as WsEvent;
        this.handlers.forEach((handler) => handler(parsed));
      } catch {
        console.warn("[WS] Failed to parse message:", event.data);
      }
    };

    this.ws.onclose = () => {
      console.log("[WS] Disconnected, reconnecting...");
      this.scheduleReconnect();
    };

    this.ws.onerror = () => {
      this.ws?.close();
    };
  }

  disconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
  }

  subscribe(handler: WsEventHandler): () => void {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, 3000);
  }
}

const WS_URL = process.env.NEXT_PUBLIC_WS_URL ?? "ws://localhost:3101/ws";
export const wsClient = new WebSocketClient(WS_URL);
