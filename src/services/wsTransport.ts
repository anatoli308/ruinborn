// ── WebSocket Transport Layer ─────────────────────────────────
// Replaces Tauri IPC invoke/listen with WebSocket to dedicated server.
// Handles connection, reconnection, and message routing.

const DEFAULT_SERVER_URL = "ws://localhost:9000";

type MessageHandler = (data: unknown) => void;

interface WsTransport {
  connect: (url?: string) => void;
  send: (msg: Record<string, unknown>) => void;
  onMessage: (handler: MessageHandler) => void;
  disconnect: () => void;
  isConnected: () => boolean;
}

let socket: WebSocket | null = null;
let messageHandler: MessageHandler | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let serverUrl: string = DEFAULT_SERVER_URL;

const RECONNECT_DELAY_MS = 2000;

function connect(url?: string): void {
  if (url) serverUrl = url;
  if (socket?.readyState === WebSocket.OPEN || socket?.readyState === WebSocket.CONNECTING) return;

  // Close stale socket before creating a new one
  if (socket) {
    socket.onclose = null;
    socket.close();
  }

  socket = new WebSocket(serverUrl);

  socket.onopen = () => {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
  };

  socket.onmessage = (event: MessageEvent) => {
    if (!messageHandler) return;
    try {
      const data: unknown = JSON.parse(event.data as string);
      messageHandler(data);
    } catch {
      // Ignore malformed messages
    }
  };

  socket.onclose = () => {
    scheduleReconnect();
  };

  socket.onerror = () => {
    socket?.close();
  };
}

function scheduleReconnect(): void {
  if (reconnectTimer) return;
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    connect();
  }, RECONNECT_DELAY_MS);
}

function send(msg: Record<string, unknown>): void {
  if (socket?.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify(msg));
  }
}

function onMessage(handler: MessageHandler): void {
  messageHandler = handler;
}

function disconnect(): void {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  socket?.close();
  socket = null;
}

function isConnected(): boolean {
  return socket?.readyState === WebSocket.OPEN;
}

export const wsTransport: WsTransport = {
  connect,
  send,
  onMessage,
  disconnect,
  isConnected,
};
