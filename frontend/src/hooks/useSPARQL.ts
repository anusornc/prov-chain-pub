import { useState, useEffect, useCallback } from "react";
import {
  sparqlService,
  type QueryTemplate,
  type QueryBuilderConfig,
} from "../services/sparql";
import type { SPARQLQuery, SPARQLResult } from "../types";

export interface UseSPARQLReturn {
  // Query execution
  executeQuery: (query: string) => Promise<SPARQLResult>;
  currentQuery: string;
  setCurrentQuery: (query: string) => void;
  queryResult: SPARQLResult | null;
  executionTime: number;
  isExecuting: boolean;
  executionError: string | null;

  // Query management
  savedQueries: SPARQLQuery[];
  saveQuery: (query: SPARQLQuery) => Promise<void>;
  deleteQuery: (queryId: string) => Promise<void>;
  toggleFavorite: (queryId: string) => Promise<void>;
  loadSavedQueries: () => Promise<void>;
  savedQueriesLoading: boolean;
  savedQueriesError: string | null;

  // Query builder configuration
  config: QueryBuilderConfig | null;
  templates: QueryTemplate[];
  loadConfig: () => Promise<void>;
  configLoading: boolean;
  configError: string | null;

  // Query validation
  validateQuery: (
    query: string,
  ) => Promise<{ is_valid: boolean; errors: string[]; warnings: string[] }>;
  validationResult: {
    is_valid: boolean;
    errors: string[];
    warnings: string[];
  } | null;
  isValidating: boolean;

  // Template management
  loadTemplate: (templateId: string) => void;
  fillTemplate: (
    template: QueryTemplate,
    parameters: Record<string, string>,
  ) => string;

  // Results formatting
  formatResults: (results: SPARQLResult) => {
    headers: string[];
    rows: string[][];
    totalRows: number;
  };

  // Utility functions
  clearResults: () => void;
  clearError: () => void;
}

export const useSPARQL = (): UseSPARQLReturn => {
  // Query execution state
  const [currentQuery, setCurrentQuery] = useState("");
  const [queryResult, setQueryResult] = useState<SPARQLResult | null>(null);
  const [executionTime, setExecutionTime] = useState(0);
  const [isExecuting, setIsExecuting] = useState(false);
  const [executionError, setExecutionError] = useState<string | null>(null);

  // Saved queries state
  const [savedQueries, setSavedQueries] = useState<SPARQLQuery[]>([]);
  const [savedQueriesLoading, setSavedQueriesLoading] = useState(false);
  const [savedQueriesError, setSavedQueriesError] = useState<string | null>(
    null,
  );

  // Configuration state
  const [config, setConfig] = useState<QueryBuilderConfig | null>(null);
  const [configLoading, setConfigLoading] = useState(false);
  const [configError, setConfigError] = useState<string | null>(null);

  // Validation state
  const [validationResult, setValidationResult] = useState<{
    is_valid: boolean;
    errors: string[];
    warnings: string[];
  } | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  /**
   * Execute a SPARQL query
   */
  const executeQuery = useCallback(
    async (query: string): Promise<SPARQLResult> => {
      setIsExecuting(true);
      setExecutionError(null);
      const startTime = Date.now();

      try {
        const result = await sparqlService.executeQuery(query);
        setQueryResult(result);
        setExecutionTime(Date.now() - startTime);
        return result;
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : "Unknown error occurred";
        setExecutionError(errorMessage);
        throw error;
      } finally {
        setIsExecuting(false);
      }
    },
    [],
  );

  /**
   * Save a SPARQL query
   */
  const saveQuery = useCallback(async (query: SPARQLQuery): Promise<void> => {
    try {
      const savedQuery = await sparqlService.saveQuery(query);
      setSavedQueries((prev) => [...prev, savedQuery]);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to save query";
      setSavedQueriesError(errorMessage);
      throw error;
    }
  }, []);

  /**
   * Delete a saved query
   */
  const deleteQuery = useCallback(async (queryId: string): Promise<void> => {
    try {
      await sparqlService.deleteQuery(queryId);
      setSavedQueries((prev) => prev.filter((q) => q.id !== queryId));
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to delete query";
      setSavedQueriesError(errorMessage);
      throw error;
    }
  }, []);

  /**
   * Toggle favorite status of a query
   */
  const toggleFavorite = useCallback(async (queryId: string): Promise<void> => {
    try {
      const updatedQuery = await sparqlService.toggleFavorite(queryId);
      setSavedQueries((prev) =>
        prev.map((q) => (q.id === queryId ? updatedQuery : q)),
      );
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to toggle favorite";
      setSavedQueriesError(errorMessage);
      throw error;
    }
  }, []);

  /**
   * Load saved queries
   */
  const loadSavedQueries = useCallback(async (): Promise<void> => {
    setSavedQueriesLoading(true);
    setSavedQueriesError(null);

    try {
      const queries = await sparqlService.getSavedQueries();
      setSavedQueries(queries);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to load saved queries";
      setSavedQueriesError(errorMessage);
    } finally {
      setSavedQueriesLoading(false);
    }
  }, []);

  /**
   * Load query builder configuration
   */
  const loadConfig = useCallback(async (): Promise<void> => {
    setConfigLoading(true);
    setConfigError(null);

    try {
      const builderConfig = await sparqlService.getQueryBuilderConfig();
      setConfig(builderConfig);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to load configuration";
      setConfigError(errorMessage);
    } finally {
      setConfigLoading(false);
    }
  }, []);

  /**
   * Validate SPARQL query syntax
   */
  const validateQuery = useCallback(
    async (
      query: string,
    ): Promise<{ is_valid: boolean; errors: string[]; warnings: string[] }> => {
      setIsValidating(true);

      try {
        const result = await sparqlService.validateQuery(query);
        setValidationResult(result);
        return result;
      } catch (error) {
        const result = {
          is_valid: false,
          errors: [
            error instanceof Error ? error.message : "Validation failed",
          ],
          warnings: [],
        };
        setValidationResult(result);
        return result;
      } finally {
        setIsValidating(false);
      }
    },
    [],
  );

  /**
   * Load a template into the current query
   */
  const loadTemplate = useCallback(
    (templateId: string): void => {
      if (!config) return;

      const template = config.templates.find((t) => t.id === templateId);
      if (template) {
        setCurrentQuery(template.query);
        setQueryResult(null);
        setExecutionError(null);
        setValidationResult(null);
      }
    },
    [config],
  );

  /**
   * Fill template parameters
   */
  const fillTemplate = useCallback(
    (template: QueryTemplate, parameters: Record<string, string>): string => {
      let filledQuery = template.query;

      // Replace template parameters
      Object.entries(parameters).forEach(([key, value]) => {
        const placeholder = `{{${key}}}`;
        filledQuery = filledQuery.replace(new RegExp(placeholder, "g"), value);
      });

      return filledQuery;
    },
    [],
  );

  /**
   * Format SPARQL results for display
   */
  const formatResults = useCallback((results: SPARQLResult) => {
    return sparqlService.formatResults(results);
  }, []);

  /**
   * Clear query results
   */
  const clearResults = useCallback((): void => {
    setQueryResult(null);
    setExecutionError(null);
    setValidationResult(null);
    setExecutionTime(0);
  }, []);

  /**
   * Clear execution error
   */
  const clearError = useCallback((): void => {
    setExecutionError(null);
    setSavedQueriesError(null);
    setConfigError(null);
  }, []);

  // Load initial data
  useEffect(() => {
    loadConfig();
    loadSavedQueries();
  }, [loadConfig, loadSavedQueries]);

  return {
    // Query execution
    executeQuery,
    currentQuery,
    setCurrentQuery,
    queryResult,
    executionTime,
    isExecuting,
    executionError,

    // Query management
    savedQueries,
    saveQuery,
    deleteQuery,
    toggleFavorite,
    loadSavedQueries,
    savedQueriesLoading,
    savedQueriesError,

    // Query builder configuration
    config,
    templates: config?.templates || [],
    loadConfig,
    configLoading,
    configError,

    // Query validation
    validateQuery,
    validationResult,
    isValidating,

    // Template management
    loadTemplate,
    fillTemplate,

    // Results formatting
    formatResults,

    // Utility functions
    clearResults,
    clearError,
  };
};
