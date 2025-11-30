import { useState, useEffect, useCallback } from "react";
import { traceabilityService } from "../services/traceability";
import { useWebSocket } from "./useWebSocket";
import type {
  TraceabilityItem,
  TraceabilityResponse,
  SearchQuery,
  SearchResults,
  KnowledgeGraph,
} from "../types";

export interface UseTraceabilityReturn {
  // Data state
  items: TraceabilityItem[];
  selectedItem: TraceabilityItem | null;
  traceData: TraceabilityResponse | null;
  searchResults: SearchResults<TraceabilityItem> | null;
  knowledgeGraph: KnowledgeGraph | null;

  // Loading states
  loading: boolean;
  searchLoading: boolean;
  traceLoading: boolean;
  graphLoading: boolean;

  // Error states
  error: string | null;
  searchError: string | null;
  traceError: string | null;
  graphError: string | null;

  // Actions
  loadItems: (query?: SearchQuery) => Promise<void>;
  searchItems: (
    searchTerm: string,
    filters?: SearchQuery["filters"],
  ) => Promise<void>;
  selectItem: (itemId: string) => Promise<void>;
  loadItemTrace: (itemId: string) => Promise<void>;
  loadKnowledgeGraph: (itemIds: string[]) => Promise<void>;
  validateItem: (itemId: string) => Promise<{
    is_authentic: boolean;
    integrity_score: number;
    validation_details: {
      signature_valid: boolean;
      chain_intact: boolean;
      data_consistent: boolean;
      timestamp_valid: boolean;
    };
    validation_time_ms: number;
  }>;
  refresh: () => Promise<void>;
  clearSelection: () => void;
  clearSearch: () => void;
}

export const useTraceability = (): UseTraceabilityReturn => {
  // Data state
  const [items, setItems] = useState<TraceabilityItem[]>([]);
  const [selectedItem, setSelectedItem] = useState<TraceabilityItem | null>(
    null,
  );
  const [traceData, setTraceData] = useState<TraceabilityResponse | null>(null);
  const [searchResults, setSearchResults] =
    useState<SearchResults<TraceabilityItem> | null>(null);
  const [knowledgeGraph, setKnowledgeGraph] = useState<KnowledgeGraph | null>(
    null,
  );

  // Loading states
  const [loading, setLoading] = useState(false);
  const [searchLoading, setSearchLoading] = useState(false);
  const [traceLoading, setTraceLoading] = useState(false);
  const [graphLoading, setGraphLoading] = useState(false);

  // Error states
  const [error, setError] = useState<string | null>(null);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [traceError, setTraceError] = useState<string | null>(null);
  const [graphError, setGraphError] = useState<string | null>(null);

  // WebSocket integration
  const { onItemUpdate, isConnected } = useWebSocket();

  // Load items with optional query
  const loadItems = useCallback(async (query?: SearchQuery) => {
    setLoading(true);
    setError(null);

    try {
      const result = await traceabilityService.getItems(query);
      setItems(result.items);

      // If this was a search query, also update search results
      if (query?.query) {
        setSearchResults(result);
      }
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load items";
      setError(errorMessage);
      console.error("Error loading items:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Search items
  const searchItems = useCallback(
    async (searchTerm: string, filters?: SearchQuery["filters"]) => {
      setSearchLoading(true);
      setSearchError(null);

      try {
        const result = await traceabilityService.searchItems(
          searchTerm,
          filters,
        );
        setSearchResults(result);
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "Search failed";
        setSearchError(errorMessage);
        console.error("Error searching items:", err);
      } finally {
        setSearchLoading(false);
      }
    },
    [],
  );

  // Select and load detailed item information
  const selectItem = useCallback(async (itemId: string) => {
    setLoading(true);
    setError(null);

    try {
      const item = await traceabilityService.getItem(itemId);
      setSelectedItem(item);

      // Automatically load trace data for selected item
      await loadItemTrace(itemId);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load item";
      setError(errorMessage);
      console.error("Error selecting item:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Load item trace data
  const loadItemTrace = useCallback(async (itemId: string) => {
    setTraceLoading(true);
    setTraceError(null);

    try {
      const trace = await traceabilityService.getItemTrace(itemId);
      setTraceData(trace);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load trace data";
      setTraceError(errorMessage);
      console.error("Error loading trace data:", err);
    } finally {
      setTraceLoading(false);
    }
  }, []);

  // Load knowledge graph
  const loadKnowledgeGraph = useCallback(async (itemIds: string[]) => {
    setGraphLoading(true);
    setGraphError(null);

    try {
      const graph = await traceabilityService.getKnowledgeGraph(itemIds);
      setKnowledgeGraph(graph);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load knowledge graph";
      setGraphError(errorMessage);
      console.error("Error loading knowledge graph:", err);
    } finally {
      setGraphLoading(false);
    }
  }, []);

  // Validate item
  const validateItem = useCallback(async (itemId: string) => {
    try {
      return await traceabilityService.validateItem(itemId);
    } catch (err) {
      console.error("Error validating item:", err);
      throw err;
    }
  }, []);

  // Refresh current data
  const refresh = useCallback(async () => {
    if (selectedItem) {
      await selectItem(selectedItem.id);
    } else {
      await loadItems();
    }
  }, [selectedItem, selectItem, loadItems]);

  // Clear selection
  const clearSelection = useCallback(() => {
    setSelectedItem(null);
    setTraceData(null);
    setTraceError(null);
  }, []);

  // Clear search results
  const clearSearch = useCallback(() => {
    setSearchResults(null);
    setSearchError(null);
  }, []);

  // WebSocket event handlers
  useEffect(() => {
    if (!isConnected) return;

    const unsubscribeItemUpdate = onItemUpdate(
      (updatedItem: TraceabilityItem) => {
        // Update items list if the updated item is in current items
        setItems((prevItems) =>
          prevItems.map((item) =>
            item.id === updatedItem.id ? updatedItem : item,
          ),
        );

        // Update search results if applicable
        setSearchResults((prevResults) => {
          if (!prevResults) return prevResults;

          return {
            ...prevResults,
            items: prevResults.items.map((item) =>
              item.id === updatedItem.id ? updatedItem : item,
            ),
          };
        });

        // Update selected item if it matches
        setSelectedItem((prevSelected) =>
          prevSelected?.id === updatedItem.id ? updatedItem : prevSelected,
        );

        // If this item is currently being traced, reload trace data
        if (selectedItem?.id === updatedItem.id) {
          loadItemTrace(updatedItem.id);
        }
      },
    );

    return () => {
      unsubscribeItemUpdate();
    };
  }, [isConnected, onItemUpdate, selectedItem, loadItemTrace]);

  // Load initial data on mount
  useEffect(() => {
    loadItems();
  }, [loadItems]);

  return {
    // Data state
    items,
    selectedItem,
    traceData,
    searchResults,
    knowledgeGraph,

    // Loading states
    loading,
    searchLoading,
    traceLoading,
    graphLoading,

    // Error states
    error,
    searchError,
    traceError,
    graphError,

    // Actions
    loadItems,
    searchItems,
    selectItem,
    loadItemTrace,
    loadKnowledgeGraph,
    validateItem,
    refresh,
    clearSelection,
    clearSearch,
  };
};
