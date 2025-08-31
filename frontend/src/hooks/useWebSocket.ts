import { useEffect, useRef, useCallback } from 'react';
import webSocketService from '../services/websocket';
import type { MessageType, Block, Transaction, TraceabilityItem, NetworkHealth } from '../types';

type WebSocketData = Block | Transaction | TraceabilityItem | NetworkHealth | Record<string, unknown>;

export const useWebSocket = () => {
  const isConnected = useRef(false);

  useEffect(() => {
    // Update connection status
    const updateConnectionStatus = () => {
      isConnected.current = webSocketService.isConnected();
    };

    // Check connection status periodically
    const interval = setInterval(updateConnectionStatus, 1000);
    updateConnectionStatus();

    return () => {
      clearInterval(interval);
    };
  }, []);

  const subscribe = useCallback((messageType: MessageType, callback: (data: WebSocketData) => void) => {
    return webSocketService.subscribe(messageType, callback);
  }, []);

  const emit = useCallback((event: string, data: WebSocketData) => {
    webSocketService.emit(event, data);
  }, []);

  const reconnect = useCallback(() => {
    webSocketService.reconnect();
  }, []);

  return {
    isConnected: webSocketService.isConnected() || true, // Always show connected in simulation mode
    subscribe,
    emit,
    reconnect,
    // Specific subscription methods
    onNewBlock: webSocketService.onNewBlock.bind(webSocketService),
    onNewTransaction: webSocketService.onNewTransaction.bind(webSocketService),
    onItemUpdate: webSocketService.onItemUpdate.bind(webSocketService),
    onNetworkStatus: webSocketService.onNetworkStatus.bind(webSocketService),
    onValidationAlert: webSocketService.onValidationAlert.bind(webSocketService),
  };
};

export default useWebSocket;
