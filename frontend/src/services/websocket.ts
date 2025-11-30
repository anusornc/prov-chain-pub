// Native WebSocket implementation (replaced Socket.IO)
import type {
  WebSocketMessage,
  MessageType,
  Block,
  Transaction,
  TraceabilityItem,
  NetworkHealth,
} from "../types";

type WebSocketData =
  | Block
  | Transaction
  | TraceabilityItem
  | NetworkHealth
  | Record<string, unknown>;
type WebSocketCallback = (data: WebSocketData) => void;

class WebSocketService {
  private socket: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private listeners: Map<MessageType, Set<WebSocketCallback>> = new Map();
  private isConnectedState = false;
  private pollingInterval: NodeJS.Timeout | null = null;

  constructor() {
    // Try to connect to real WebSocket first, fall back to simulation if needed
    this.initializeSocket();
  }

  private simulateConnection() {
    // Simulate a successful connection for development
    this.isConnectedState = true;
    console.log(
      "WebSocket service initialized (simulation mode - real WebSocket unavailable)",
    );

    // Start polling for updates instead of WebSocket
    this.startPolling();
  }

  private startPolling() {
    // Poll for updates every 5 seconds
    this.pollingInterval = setInterval(() => {
      // Simulate network status updates
      this.notifyListeners("NetworkStatus", {
        status: "healthy",
        uptime: Math.floor(Date.now() / 1000),
        peer_count: 1,
        sync_status: "synced",
        last_block_age: 30,
      });
    }, 5000);
  }

  private initializeSocket() {
    // Try to connect to native WebSocket server
    console.log("Attempting WebSocket connection to ws://localhost:8080/ws");

    try {
      const ws = new WebSocket("ws://localhost:8080/ws");

      ws.onopen = () => {
        console.log("WebSocket connected");
        this.isConnectedState = true;
        this.reconnectAttempts = 0;
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          console.log("Received WebSocket message:", message);
          this.handleNativeMessage(message);
        } catch (error) {
          console.error("Error parsing WebSocket message:", error);
        }
      };

      ws.onclose = () => {
        console.log("WebSocket disconnected");
        this.isConnectedState = false;
        this.handleReconnection();
      };

      ws.onerror = (error) => {
        console.error("WebSocket error:", error);
        this.isConnectedState = false;
        this.handleReconnection();
      };

      // Store the native WebSocket
      this.socket = ws;
    } catch (error) {
      console.error("Failed to initialize WebSocket connection:", error);
      this.simulateConnection();
    }
  }

  private handleNativeMessage(message: unknown) {
    // Handle native WebSocket messages with proper type guards
    if (this.isWebSocketMessage(message)) {
      this.notifyListeners(message.message_type, message.data as WebSocketData);
    } else if (this.isTypeMessage(message)) {
      // Handle different message formats
      switch (message.type) {
        case "new_block":
          this.notifyListeners(
            "NewBlock",
            (message.data || message) as WebSocketData,
          );
          break;
        case "new_transaction":
          this.notifyListeners(
            "NewTransaction",
            (message.data || message) as WebSocketData,
          );
          break;
        case "item_update":
          this.notifyListeners(
            "ItemUpdate",
            (message.data || message) as WebSocketData,
          );
          break;
        case "network_status":
          this.notifyListeners(
            "NetworkStatus",
            (message.data || message) as WebSocketData,
          );
          break;
        case "validation_alert":
          this.notifyListeners(
            "ValidationAlert",
            (message.data || message) as WebSocketData,
          );
          break;
        default:
          console.log("Unknown message type:", message.type);
      }
    } else {
      // Direct data message
      console.log("Received direct WebSocket data:", message);
    }
  }

  private isWebSocketMessage(message: unknown): message is WebSocketMessage {
    return (
      typeof message === "object" &&
      message !== null &&
      "message_type" in message &&
      "data" in message
    );
  }

  private isTypeMessage(
    message: unknown,
  ): message is { type: string; data?: WebSocketData } {
    return (
      typeof message === "object" &&
      message !== null &&
      "type" in message &&
      typeof (message as { type: unknown }).type === "string"
    );
  }

  // Native WebSocket doesn't need separate event handler setup
  // All event handlers are set up in initializeSocket()

  private notifyListeners(messageType: MessageType, data: WebSocketData) {
    const listeners = this.listeners.get(messageType);
    if (listeners) {
      listeners.forEach((callback) => {
        try {
          callback(data);
        } catch (error) {
          console.error(
            `Error in WebSocket listener for ${messageType}:`,
            error,
          );
        }
      });
    }
  }

  private handleReconnection() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(
        `Attempting to reconnect... (${this.reconnectAttempts}/${this.maxReconnectAttempts})`,
      );

      setTimeout(() => {
        this.initializeSocket();
      }, this.reconnectDelay * this.reconnectAttempts);
    } else {
      console.error(
        "Max reconnection attempts reached, falling back to simulation",
      );
      this.simulateConnection();
    }
  }

  // Public methods
  public subscribe(
    messageType: MessageType,
    callback: WebSocketCallback,
  ): () => void {
    if (!this.listeners.has(messageType)) {
      this.listeners.set(messageType, new Set());
    }

    const listeners = this.listeners.get(messageType)!;
    listeners.add(callback);

    // Return unsubscribe function
    return () => {
      listeners.delete(callback);
      if (listeners.size === 0) {
        this.listeners.delete(messageType);
      }
    };
  }

  public unsubscribe(messageType: MessageType, callback: WebSocketCallback) {
    const listeners = this.listeners.get(messageType);
    if (listeners) {
      listeners.delete(callback);
      if (listeners.size === 0) {
        this.listeners.delete(messageType);
      }
    }
  }

  public emit(event: string, data: WebSocketData) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      const message = {
        type: event,
        data: data,
      };
      this.socket.send(JSON.stringify(message));
    } else {
      console.warn("WebSocket not connected, cannot emit event:", event);
    }
  }

  public isConnected(): boolean {
    return (
      this.isConnectedState ||
      this.socket?.readyState === WebSocket.OPEN ||
      false
    );
  }

  public disconnect() {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
    if (this.pollingInterval) {
      clearInterval(this.pollingInterval);
      this.pollingInterval = null;
    }
    this.listeners.clear();
    this.isConnectedState = false;
    // this.useSimulation = false; // Reset simulation flag
  }

  public reconnect() {
    this.disconnect();
    this.reconnectAttempts = 0;
    this.initializeSocket();
  }

  // Specific subscription methods for type safety
  public onNewBlock(callback: (block: Block) => void) {
    return this.subscribe("NewBlock", callback as WebSocketCallback);
  }

  public onNewTransaction(callback: (transaction: Transaction) => void) {
    return this.subscribe("NewTransaction", callback as WebSocketCallback);
  }

  public onItemUpdate(callback: (item: TraceabilityItem) => void) {
    return this.subscribe("ItemUpdate", callback as WebSocketCallback);
  }

  public onNetworkStatus(callback: (status: NetworkHealth) => void) {
    return this.subscribe("NetworkStatus", callback as WebSocketCallback);
  }

  public onValidationAlert(callback: (alert: Record<string, unknown>) => void) {
    return this.subscribe("ValidationAlert", callback as WebSocketCallback);
  }
}

// Create singleton instance
const webSocketService = new WebSocketService();

export default webSocketService;
export { WebSocketService };
