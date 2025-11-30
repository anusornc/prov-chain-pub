import { useState, useEffect, useCallback } from "react";
import { blockchainAPI, transactionAPI } from "../services/api";
import useWebSocket from "./useWebSocket";
import type {
  Block,
  Transaction,
  DashboardMetrics,
  NetworkHealth,
} from "../types";

interface UseBlockchainReturn {
  // Data
  blocks: Block[];
  transactions: Transaction[];
  metrics: DashboardMetrics | null;
  networkHealth: NetworkHealth | null;

  // Loading states
  loading: boolean;
  blocksLoading: boolean;
  transactionsLoading: boolean;
  metricsLoading: boolean;

  // Error states
  error: string | null;

  // Actions
  fetchBlocks: () => Promise<void>;
  fetchTransactions: () => Promise<void>;
  fetchMetrics: () => Promise<void>;
  fetchBlock: (index: number) => Promise<Block | null>;
  validateBlockchain: () => Promise<boolean>;
  refresh: () => Promise<void>;
}

export const useBlockchain = (): UseBlockchainReturn => {
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [metrics, setMetrics] = useState<DashboardMetrics | null>(null);
  const [networkHealth, setNetworkHealth] = useState<NetworkHealth | null>(
    null,
  );

  const [loading, setLoading] = useState(true);
  const [blocksLoading, setBlocksLoading] = useState(false);
  const [transactionsLoading, setTransactionsLoading] = useState(false);
  const [metricsLoading, setMetricsLoading] = useState(false);

  const [error, setError] = useState<string | null>(null);

  const { onNewBlock, onNewTransaction, onNetworkStatus } = useWebSocket();

  // Fetch blocks from API
  const fetchBlocks = useCallback(async () => {
    try {
      setBlocksLoading(true);
      setError(null);

      const response = await blockchainAPI.getBlocks();
      setBlocks(response.data.blocks || []);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to fetch blocks";
      setError(errorMessage);
      console.error("Error fetching blocks:", err);
    } finally {
      setBlocksLoading(false);
    }
  }, []);

  // Fetch transactions from API
  const fetchTransactions = useCallback(async () => {
    try {
      setTransactionsLoading(true);
      setError(null);

      const response = await transactionAPI.getRecent();
      setTransactions(response.data.transactions || []);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to fetch transactions";
      setError(errorMessage);
      console.error("Error fetching transactions:", err);
    } finally {
      setTransactionsLoading(false);
    }
  }, []);

  // Fetch dashboard metrics
  const fetchMetrics = useCallback(async () => {
    try {
      setMetricsLoading(true);
      setError(null);

      const [statusResponse, blocksResponse, transactionsResponse] =
        await Promise.all([
          blockchainAPI.getStatus(),
          blockchainAPI.getBlocks(),
          transactionAPI.getRecent(),
        ]);

      const status = statusResponse.data;
      const blocksData = blocksResponse.data;
      const transactionsData = transactionsResponse.data;

      // Calculate metrics from API responses
      const dashboardMetrics: DashboardMetrics = {
        total_blocks: blocksData.total_blocks || blocks.length,
        total_transactions:
          transactionsData.total_transactions || transactions.length,
        total_items: status.total_items || 0,
        active_participants: status.active_participants || 0,
        network_status: status.network_status || "healthy",
        last_block_time: status.last_block_time || new Date().toISOString(),
        avg_block_time: status.avg_block_time || 0,
        transactions_per_second: status.transactions_per_second || 0,
        network_hash_rate: status.network_hash_rate || 0,
      };

      const health: NetworkHealth = {
        status: status.network_status || "healthy",
        uptime: status.uptime || 0,
        peer_count: status.peer_count || 0,
        sync_status: status.sync_status || "synced",
        last_block_age: status.last_block_age || 0,
        validation_errors: status.validation_errors || 0,
      };

      setMetrics(dashboardMetrics);
      setNetworkHealth(health);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to fetch metrics";
      setError(errorMessage);
      console.error("Error fetching metrics:", err);

      // Set fallback metrics if API fails
      setMetrics({
        total_blocks: blocks.length,
        total_transactions: transactions.length,
        total_items: 0,
        active_participants: 0,
        network_status: "error",
        last_block_time: new Date().toISOString(),
        avg_block_time: 0,
        transactions_per_second: 0,
        network_hash_rate: 0,
      });

      setNetworkHealth({
        status: "error",
        uptime: 0,
        peer_count: 0,
        sync_status: "behind",
        last_block_age: 0,
        validation_errors: 1,
      });
    } finally {
      setMetricsLoading(false);
    }
  }, [blocks.length, transactions.length]);

  // Fetch single block
  const fetchBlock = useCallback(
    async (index: number): Promise<Block | null> => {
      try {
        const response = await blockchainAPI.getBlock(index);
        return response.data.block || null;
      } catch (err) {
        console.error(`Error fetching block ${index}:`, err);
        return null;
      }
    },
    [],
  );

  // Validate blockchain
  const validateBlockchain = useCallback(async (): Promise<boolean> => {
    try {
      const response = await blockchainAPI.validate();
      return response.data.is_valid || false;
    } catch (err) {
      console.error("Error validating blockchain:", err);
      return false;
    }
  }, []);

  // Refresh all data
  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      await Promise.all([fetchBlocks(), fetchTransactions(), fetchMetrics()]);
    } finally {
      setLoading(false);
    }
  }, [fetchBlocks, fetchTransactions, fetchMetrics]);

  // Set up WebSocket listeners for real-time updates
  useEffect(() => {
    const unsubscribeNewBlock = onNewBlock((block: Block) => {
      setBlocks((prevBlocks) => {
        // Add new block if it doesn't exist
        const exists = prevBlocks.some((b) => b.index === block.index);
        if (!exists) {
          return [...prevBlocks, block].sort((a, b) => b.index - a.index);
        }
        return prevBlocks;
      });

      // Update metrics when new block arrives
      setMetrics((prevMetrics) => {
        if (!prevMetrics) return null;
        return {
          ...prevMetrics,
          total_blocks: prevMetrics.total_blocks + 1,
          last_block_time: block.timestamp,
        };
      });
    });

    const unsubscribeNewTransaction = onNewTransaction(
      (transaction: Transaction) => {
        setTransactions((prevTransactions) => {
          // Add new transaction if it doesn't exist
          const exists = prevTransactions.some((t) => t.id === transaction.id);
          if (!exists) {
            return [transaction, ...prevTransactions].slice(0, 100); // Keep only latest 100
          }
          return prevTransactions;
        });

        // Update metrics when new transaction arrives
        setMetrics((prevMetrics) => {
          if (!prevMetrics) return null;
          return {
            ...prevMetrics,
            total_transactions: prevMetrics.total_transactions + 1,
          };
        });
      },
    );

    const unsubscribeNetworkStatus = onNetworkStatus(
      (status: NetworkHealth) => {
        setNetworkHealth(status);

        // Update metrics with network status
        setMetrics((prevMetrics) => {
          if (!prevMetrics) return null;
          return {
            ...prevMetrics,
            network_status: status.status,
          };
        });
      },
    );

    return () => {
      unsubscribeNewBlock();
      unsubscribeNewTransaction();
      unsubscribeNetworkStatus();
    };
  }, [onNewBlock, onNewTransaction, onNetworkStatus]);

  // Initial data fetch
  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    // Data
    blocks,
    transactions,
    metrics,
    networkHealth,

    // Loading states
    loading,
    blocksLoading,
    transactionsLoading,
    metricsLoading,

    // Error state
    error,

    // Actions
    fetchBlocks,
    fetchTransactions,
    fetchMetrics,
    fetchBlock,
    validateBlockchain,
    refresh,
  };
};

export default useBlockchain;
