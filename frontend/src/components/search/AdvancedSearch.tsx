import React, { useState, useEffect } from "react";
import {
  Search,
  Filter,
  Tag,
  User,
  Package,
  Activity,
  AlertCircle,
  Clock,
  X,
} from "lucide-react";
import LoadingSpinner from "../ui/LoadingSpinner";
import Card from "../ui/Card";
import Badge from "../ui/Badge";
import Button from "../ui/Button";
import Input from "../ui/Input";
import type { Block, Transaction, TraceabilityItem } from "../../types";
import { API_ENDPOINTS } from "../../config/api";

interface SearchFilters {
  query: string;
  type: "all" | "blocks" | "transactions" | "traceability" | "participants";
  dateRange: {
    start: string;
    end: string;
  };
  status: string;
  participant: string;
  tags: string[];
}

interface SearchResult {
  id: string;
  type: "block" | "transaction" | "traceability" | "participant";
  title: string;
  description: string;
  timestamp: string;
  status?: string;
  participant?: string;
  tags?: string[];
  data: Block | Transaction | TraceabilityItem | Record<string, unknown>;
}

const AdvancedSearch: React.FC = () => {
  const [filters, setFilters] = useState<SearchFilters>({
    query: "",
    type: "all",
    dateRange: {
      start: "",
      end: "",
    },
    status: "",
    participant: "",
    tags: [],
  });

  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isFiltersOpen, setIsFiltersOpen] = useState(false);
  const [totalResults, setTotalResults] = useState(0);
  const [currentPage, setCurrentPage] = useState(1);
  const [sortBy, setSortBy] = useState<"relevance" | "date" | "type">(
    "relevance",
  );

  const resultsPerPage = 10;

  useEffect(() => {
    if (
      filters.query.trim() ||
      filters.type !== "all" ||
      filters.status ||
      filters.participant
    ) {
      performSearch();
    } else {
      setResults([]);
      setTotalResults(0);
    }
  }, [filters, currentPage, sortBy]);

  const performSearch = async () => {
    setIsLoading(true);
    try {
      const token = localStorage.getItem("authToken");
      const searchParams = new URLSearchParams({
        q: filters.query,
        type: filters.type,
        page: currentPage.toString(),
        limit: resultsPerPage.toString(),
        sort: sortBy,
        ...(filters.dateRange.start && { start_date: filters.dateRange.start }),
        ...(filters.dateRange.end && { end_date: filters.dateRange.end }),
        ...(filters.status && { status: filters.status }),
        ...(filters.participant && { participant: filters.participant }),
        ...(filters.tags.length > 0 && { tags: filters.tags.join(",") }),
      });

      // Search across different data types
      const searchResults: SearchResult[] = [];

      // Search blocks if type is 'all' or 'blocks'
      if (filters.type === "all" || filters.type === "blocks") {
        try {
          const blocksResponse = await fetch(
            `${API_ENDPOINTS.API}/blocks?${searchParams}`,
            {
              headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
              },
            },
          );

          if (blocksResponse.ok) {
            const blocks: Block[] = await blocksResponse.json();
            const blockResults = blocks
              .filter(
                (block) =>
                  !filters.query ||
                  block.hash
                    .toLowerCase()
                    .includes(filters.query.toLowerCase()) ||
                  block.rdf_data
                    .toLowerCase()
                    .includes(filters.query.toLowerCase()),
              )
              .map((block) => ({
                id: `block-${block.index}`,
                type: "block" as const,
                title: `Block #${block.index}`,
                description: `Hash: ${block.hash.substring(0, 16)}... | ${block.rdf_data.length} bytes`,
                timestamp: block.timestamp,
                data: block,
              }));
            searchResults.push(...blockResults);
          }
        } catch (error) {
          console.warn("Error searching blocks:", error);
        }
      }

      // Search transactions if type is 'all' or 'transactions'
      if (filters.type === "all" || filters.type === "transactions") {
        try {
          const transactionsResponse = await fetch(
            `${API_ENDPOINTS.API}/transactions?${searchParams}`,
            {
              headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
              },
            },
          );

          if (transactionsResponse.ok) {
            const transactions: Transaction[] =
              await transactionsResponse.json();
            const transactionResults = transactions
              .filter(
                (tx) =>
                  !filters.query ||
                  tx.id.toLowerCase().includes(filters.query.toLowerCase()) ||
                  tx.from.toLowerCase().includes(filters.query.toLowerCase()) ||
                  (tx.to &&
                    tx.to.toLowerCase().includes(filters.query.toLowerCase())),
              )
              .map((tx) => ({
                id: `transaction-${tx.id}`,
                type: "transaction" as const,
                title: `Transaction ${tx.id.substring(0, 8)}...`,
                description: `From: ${tx.from} → To: ${tx.to || "N/A"} | Type: ${tx.type}`,
                timestamp: tx.timestamp,
                status: tx.status,
                data: tx,
              }));
            searchResults.push(...transactionResults);
          }
        } catch (error) {
          console.warn("Error searching transactions:", error);
        }
      }

      // Search traceability items if type is 'all' or 'traceability'
      if (filters.type === "all" || filters.type === "traceability") {
        try {
          const traceabilityResponse = await fetch(
            `${API_ENDPOINTS.API}/traceability/items?${searchParams}`,
            {
              headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
              },
            },
          );

          if (traceabilityResponse.ok) {
            const items: TraceabilityItem[] = await traceabilityResponse.json();
            const traceabilityResults = items
              .filter(
                (item) =>
                  !filters.query ||
                  item.id.toLowerCase().includes(filters.query.toLowerCase()) ||
                  item.name
                    .toLowerCase()
                    .includes(filters.query.toLowerCase()) ||
                  item.type.toLowerCase().includes(filters.query.toLowerCase()),
              )
              .map((item) => ({
                id: `traceability-${item.id}`,
                type: "traceability" as const,
                title: item.name,
                description: `Type: ${item.type} | Location: ${item.location || "Unknown"}`,
                timestamp: item.created_at,
                status: "active",
                participant: item.current_owner,
                tags: Object.keys(item.properties),
                data: item,
              }));
            searchResults.push(...traceabilityResults);
          }
        } catch (error) {
          console.warn("Error searching traceability items:", error);
        }
      }

      // Apply additional filters
      let filteredResults = searchResults;

      if (filters.status) {
        filteredResults = filteredResults.filter(
          (result) => result.status === filters.status,
        );
      }

      if (filters.participant) {
        filteredResults = filteredResults.filter((result) =>
          result.participant
            ?.toLowerCase()
            .includes(filters.participant.toLowerCase()),
        );
      }

      if (filters.tags.length > 0) {
        filteredResults = filteredResults.filter((result) =>
          result.tags?.some((tag) => filters.tags.includes(tag)),
        );
      }

      // Apply date range filter
      if (filters.dateRange.start || filters.dateRange.end) {
        filteredResults = filteredResults.filter((result) => {
          const resultDate = new Date(result.timestamp);
          const startDate = filters.dateRange.start
            ? new Date(filters.dateRange.start)
            : null;
          const endDate = filters.dateRange.end
            ? new Date(filters.dateRange.end)
            : null;

          if (startDate && resultDate < startDate) return false;
          if (endDate && resultDate > endDate) return false;
          return true;
        });
      }

      // Sort results
      filteredResults.sort((a, b) => {
        switch (sortBy) {
          case "date": {
            return (
              new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
            );
          }
          case "type": {
            return a.type.localeCompare(b.type);
          }
          case "relevance":
          default: {
            // Simple relevance scoring based on query match
            if (!filters.query) return 0;
            const aScore =
              (a.title.toLowerCase().includes(filters.query.toLowerCase())
                ? 2
                : 0) +
              (a.description.toLowerCase().includes(filters.query.toLowerCase())
                ? 1
                : 0);
            const bScore =
              (b.title.toLowerCase().includes(filters.query.toLowerCase())
                ? 2
                : 0) +
              (b.description.toLowerCase().includes(filters.query.toLowerCase())
                ? 1
                : 0);
            return bScore - aScore;
          }
        }
      });

      setTotalResults(filteredResults.length);

      // Paginate results
      const startIndex = (currentPage - 1) * resultsPerPage;
      const paginatedResults = filteredResults.slice(
        startIndex,
        startIndex + resultsPerPage,
      );

      setResults(paginatedResults);
    } catch (error) {
      console.error("Search error:", error);
      setResults([]);
      setTotalResults(0);
    } finally {
      setIsLoading(false);
    }
  };

  const handleFilterChange = (
    key: keyof SearchFilters,
    value: string | string[] | { start: string; end: string },
  ) => {
    setFilters((prev) => ({ ...prev, [key]: value }));
    setCurrentPage(1); // Reset to first page when filters change
  };

  const addTag = (tag: string) => {
    if (tag && !filters.tags.includes(tag)) {
      handleFilterChange("tags", [...filters.tags, tag]);
    }
  };

  const removeTag = (tag: string) => {
    handleFilterChange(
      "tags",
      filters.tags.filter((t) => t !== tag),
    );
  };

  const clearFilters = () => {
    setFilters({
      query: "",
      type: "all",
      dateRange: { start: "", end: "" },
      status: "",
      participant: "",
      tags: [],
    });
    setCurrentPage(1);
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case "block":
        return <Package className="w-4 h-4" />;
      case "transaction":
        return <Activity className="w-4 h-4" />;
      case "traceability":
        return <Tag className="w-4 h-4" />;
      case "participant":
        return <User className="w-4 h-4" />;
      default:
        return <Search className="w-4 h-4" />;
    }
  };

  const getStatusColor = (
    status?: string,
  ):
    | "default"
    | "success"
    | "warning"
    | "primary"
    | "secondary"
    | "danger"
    | "info" => {
    switch (status) {
      case "completed":
        return "success";
      case "pending":
        return "warning";
      case "failed":
        return "danger";
      case "confirmed":
        return "success";
      case "active":
        return "primary";
      default:
        return "default";
    }
  };

  const totalPages = Math.ceil(totalResults / resultsPerPage);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Advanced Search
          </h1>
          <p className="text-gray-600 dark:text-gray-300">
            Search across blocks, transactions, traceability items, and
            participants
          </p>
        </div>

        {/* Search Bar */}
        <Card className="mb-6">
          <div className="flex flex-col lg:flex-row gap-4">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <Input
                  type="text"
                  placeholder="Search blocks, transactions, items, participants..."
                  value={filters.query}
                  onChange={(e) => handleFilterChange("query", e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={() => setIsFiltersOpen(!isFiltersOpen)}
                className="flex items-center gap-2"
              >
                <Filter className="w-4 h-4" />
                Filters
                {(filters.type !== "all" ||
                  filters.status ||
                  filters.participant ||
                  filters.tags.length > 0) && (
                  <Badge variant="primary" className="ml-1">
                    {[
                      filters.type !== "all" ? 1 : 0,
                      filters.status ? 1 : 0,
                      filters.participant ? 1 : 0,
                      filters.tags.length,
                    ].reduce((a, b) => a + b, 0)}
                  </Badge>
                )}
              </Button>
              <select
                value={sortBy}
                onChange={(e) =>
                  setSortBy(e.target.value as "relevance" | "date" | "type")
                }
                className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                <option value="relevance">Sort by Relevance</option>
                <option value="date">Sort by Date</option>
                <option value="type">Sort by Type</option>
              </select>
            </div>
          </div>

          {/* Advanced Filters */}
          {isFiltersOpen && (
            <div className="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {/* Type Filter */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Type
                  </label>
                  <select
                    value={filters.type}
                    onChange={(e) => handleFilterChange("type", e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                  >
                    <option value="all">All Types</option>
                    <option value="blocks">Blocks</option>
                    <option value="transactions">Transactions</option>
                    <option value="traceability">Traceability</option>
                    <option value="participants">Participants</option>
                  </select>
                </div>

                {/* Status Filter */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Status
                  </label>
                  <select
                    value={filters.status}
                    onChange={(e) =>
                      handleFilterChange("status", e.target.value)
                    }
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                  >
                    <option value="">All Statuses</option>
                    <option value="completed">Completed</option>
                    <option value="pending">Pending</option>
                    <option value="failed">Failed</option>
                    <option value="active">Active</option>
                  </select>
                </div>

                {/* Participant Filter */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Participant
                  </label>
                  <Input
                    type="text"
                    placeholder="Filter by participant"
                    value={filters.participant}
                    onChange={(e) =>
                      handleFilterChange("participant", e.target.value)
                    }
                  />
                </div>

                {/* Date Range */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Date Range
                  </label>
                  <div className="flex gap-2">
                    <Input
                      type="date"
                      value={filters.dateRange.start}
                      onChange={(e) =>
                        handleFilterChange("dateRange", {
                          ...filters.dateRange,
                          start: e.target.value,
                        })
                      }
                      className="text-sm"
                    />
                    <Input
                      type="date"
                      value={filters.dateRange.end}
                      onChange={(e) =>
                        handleFilterChange("dateRange", {
                          ...filters.dateRange,
                          end: e.target.value,
                        })
                      }
                      className="text-sm"
                    />
                  </div>
                </div>
              </div>

              {/* Tags */}
              <div className="mt-4">
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Tags
                </label>
                <div className="flex flex-wrap gap-2 mb-2">
                  {filters.tags.map((tag) => (
                    <Badge
                      key={tag}
                      variant="primary"
                      className="flex items-center gap-1"
                    >
                      {tag}
                      <button onClick={() => removeTag(tag)}>
                        <X className="w-3 h-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
                <div className="flex gap-2">
                  <Input
                    type="text"
                    placeholder="Add tag..."
                    onKeyPress={(e) => {
                      if (e.key === "Enter") {
                        addTag((e.target as HTMLInputElement).value);
                        (e.target as HTMLInputElement).value = "";
                      }
                    }}
                    className="flex-1"
                  />
                  <Button variant="outline" onClick={clearFilters}>
                    Clear All
                  </Button>
                </div>
              </div>
            </div>
          )}
        </Card>

        {/* Results */}
        <div className="space-y-6">
          {/* Results Header */}
          {(results.length > 0 || isLoading) && (
            <div className="flex items-center justify-between">
              <div className="text-sm text-gray-600 dark:text-gray-300">
                {isLoading
                  ? "Searching..."
                  : `${totalResults} results found ${filters.query ? `for "${filters.query}"` : ""}`}
              </div>
              {totalPages > 1 && (
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() =>
                      setCurrentPage((prev) => Math.max(1, prev - 1))
                    }
                    disabled={currentPage === 1}
                  >
                    Previous
                  </Button>
                  <span className="text-sm text-gray-600 dark:text-gray-300">
                    Page {currentPage} of {totalPages}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() =>
                      setCurrentPage((prev) => Math.min(totalPages, prev + 1))
                    }
                    disabled={currentPage === totalPages}
                  >
                    Next
                  </Button>
                </div>
              )}
            </div>
          )}

          {/* Loading State */}
          {isLoading && (
            <div className="flex justify-center py-12">
              <LoadingSpinner size="lg" message="Searching..." />
            </div>
          )}

          {/* Results List */}
          {!isLoading && results.length > 0 && (
            <div className="space-y-4">
              {results.map((result) => (
                <Card
                  key={result.id}
                  className="hover:shadow-md transition-shadow cursor-pointer"
                >
                  <div className="flex items-start gap-4">
                    <div className="flex-shrink-0 p-2 bg-gray-100 dark:bg-gray-800 rounded-lg">
                      {getTypeIcon(result.type)}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between">
                        <div>
                          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">
                            {result.title}
                          </h3>
                          <p className="text-gray-600 dark:text-gray-300 mb-2">
                            {result.description}
                          </p>
                          <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
                            <div className="flex items-center gap-1">
                              <Clock className="w-4 h-4" />
                              {new Date(result.timestamp).toLocaleString()}
                            </div>
                            {result.participant && (
                              <div className="flex items-center gap-1">
                                <User className="w-4 h-4" />
                                {result.participant}
                              </div>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <Badge variant="secondary">{result.type}</Badge>
                          {result.status && (
                            <Badge variant={getStatusColor(result.status)}>
                              {result.status}
                            </Badge>
                          )}
                        </div>
                      </div>
                      {result.tags && result.tags.length > 0 && (
                        <div className="flex flex-wrap gap-1 mt-2">
                          {result.tags.map((tag) => (
                            <Badge key={tag} variant="secondary" size="sm">
                              {tag}
                            </Badge>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          )}

          {/* No Results */}
          {!isLoading &&
            results.length === 0 &&
            (filters.query || filters.type !== "all") && (
              <div className="text-center py-12">
                <AlertCircle className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                  No results found
                </h3>
                <p className="text-gray-600 dark:text-gray-300 mb-4">
                  Try adjusting your search criteria or filters
                </p>
                <Button variant="outline" onClick={clearFilters}>
                  Clear Filters
                </Button>
              </div>
            )}

          {/* Empty State */}
          {!isLoading &&
            results.length === 0 &&
            !filters.query &&
            filters.type === "all" && (
              <div className="text-center py-12">
                <Search className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                  Start searching
                </h3>
                <p className="text-gray-600 dark:text-gray-300">
                  Enter a search term or apply filters to find blocks,
                  transactions, and more
                </p>
              </div>
            )}
        </div>
      </div>
    </div>
  );
};

export default AdvancedSearch;
