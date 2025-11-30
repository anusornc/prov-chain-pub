import React, { useState, useEffect } from "react";
import { useTraceability } from "../../hooks/useTraceability";
import {
  Search,
  Filter,
  RefreshCw,
  Package,
  MapPin,
  User,
  Calendar,
  ChevronRight,
  AlertCircle,
  CheckCircle,
  Clock,
} from "lucide-react";
import Button from "../ui/Button";
import Input from "../ui/Input";
import Card from "../ui/Card";
import Badge from "../ui/Badge";
import LoadingSpinner from "../ui/LoadingSpinner";
import Alert from "../ui/Alert";
import type { TraceabilityItem, SearchQuery } from "../../types";

interface TraceabilityExplorerProps {
  onItemSelect?: (item: TraceabilityItem) => void;
}

const TraceabilityExplorer: React.FC<TraceabilityExplorerProps> = ({
  onItemSelect,
}) => {
  const {
    items,
    searchResults,
    loading,
    searchLoading,
    error,
    searchError,
    searchItems,
    loadItems,
    refresh,
    clearSearch,
  } = useTraceability();

  const [searchTerm, setSearchTerm] = useState("");
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState<SearchQuery["filters"]>({});
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage] = useState(20);

  // Get current items to display (search results or all items)
  const currentItems = searchResults ? searchResults.items : items;
  const totalItems = searchResults ? searchResults.total : items.length;
  const hasMore = searchResults ? searchResults.has_more : false;

  // Pagination
  const startIndex = (currentPage - 1) * itemsPerPage;
  const endIndex = startIndex + itemsPerPage;
  const paginatedItems = searchResults
    ? currentItems
    : currentItems.slice(startIndex, endIndex);
  const totalPages = Math.ceil(totalItems / itemsPerPage);

  const handleSearch = async () => {
    if (!searchTerm.trim()) {
      clearSearch();
      return;
    }

    await searchItems(searchTerm, filters);
    setCurrentPage(1);
  };

  const handleClearSearch = () => {
    setSearchTerm("");
    clearSearch();
    setCurrentPage(1);
  };

  const handleFilterChange = (key: string, value: string) => {
    const newFilters = { ...filters };
    if (value) {
      if (key === "type") newFilters.type = value;
      else if (key === "location") newFilters.location = value;
      else if (key === "participant") newFilters.participant = value;
      else if (key === "status") newFilters.status = value;
    } else {
      if (key === "type") delete newFilters.type;
      else if (key === "location") delete newFilters.location;
      else if (key === "participant") delete newFilters.participant;
      else if (key === "status") delete newFilters.status;
    }
    setFilters(newFilters);
  };

  const handleItemClick = (item: TraceabilityItem) => {
    if (onItemSelect) {
      onItemSelect(item);
    }
  };

  const handleRefresh = async () => {
    if (searchResults) {
      await handleSearch();
    } else {
      await refresh();
    }
  };

  const getStatusColor = (
    item: TraceabilityItem,
  ): "success" | "info" | "warning" | "default" => {
    // Determine status based on item properties
    const hasLocation = !!item.location;
    const hasOwner = !!item.current_owner;
    const timestamp = item.created_at;
    const isRecent = timestamp
      ? new Date(timestamp) > new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)
      : false;

    if (hasLocation && hasOwner && isRecent) return "success";
    if (hasLocation && hasOwner) return "info";
    if (hasOwner) return "warning";
    return "default";
  };

  const getStatusIcon = (item: TraceabilityItem) => {
    const color = getStatusColor(item);
    if (color === "success")
      return <CheckCircle className="w-4 h-4 text-green-500" />;
    if (color === "info") return <Clock className="w-4 h-4 text-blue-500" />;
    if (color === "warning")
      return <AlertCircle className="w-4 h-4 text-yellow-500" />;
    return <AlertCircle className="w-4 h-4 text-gray-400" />;
  };

  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  useEffect(() => {
    if (!items.length && !loading) {
      loadItems();
    }
  }, [items.length, loading, loadItems]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-7xl mx-auto p-6">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                Traceability Explorer
              </h1>
              <p className="text-gray-600 dark:text-gray-300 mt-1">
                Track and trace items through the supply chain
              </p>
            </div>
            <Button
              onClick={handleRefresh}
              disabled={loading || searchLoading}
              className="flex items-center gap-2"
            >
              <RefreshCw
                className={`w-4 h-4 ${loading || searchLoading ? "animate-spin" : ""}`}
              />
              Refresh
            </Button>
          </div>

          {/* Search and Filters */}
          <Card className="p-6">
            <div className="flex flex-col lg:flex-row gap-4">
              <div className="flex-1 flex gap-2">
                <div className="flex-1 relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                  <Input
                    type="text"
                    placeholder="Search items by name, ID, or properties..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    onKeyPress={(e) => e.key === "Enter" && handleSearch()}
                    className="pl-10"
                  />
                </div>
                <Button onClick={handleSearch} disabled={searchLoading}>
                  {searchLoading ? <LoadingSpinner size="sm" /> : "Search"}
                </Button>
                {(searchTerm || searchResults) && (
                  <Button variant="outline" onClick={handleClearSearch}>
                    Clear
                  </Button>
                )}
              </div>
              <Button
                variant="outline"
                onClick={() => setShowFilters(!showFilters)}
                className="flex items-center gap-2"
              >
                <Filter className="w-4 h-4" />
                Filters
              </Button>
            </div>

            {/* Filters Panel */}
            {showFilters && (
              <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Type
                    </label>
                    <select
                      value={filters.type || ""}
                      onChange={(e) =>
                        handleFilterChange("type", e.target.value)
                      }
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                    >
                      <option value="">All Types</option>
                      <option value="raw_material">Raw Material</option>
                      <option value="component">Component</option>
                      <option value="product">Product</option>
                      <option value="batch">Batch</option>
                      <option value="shipment">Shipment</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Location
                    </label>
                    <Input
                      type="text"
                      placeholder="Filter by location..."
                      value={filters.location || ""}
                      onChange={(e) =>
                        handleFilterChange("location", e.target.value)
                      }
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Participant
                    </label>
                    <Input
                      type="text"
                      placeholder="Filter by participant..."
                      value={filters.participant || ""}
                      onChange={(e) =>
                        handleFilterChange("participant", e.target.value)
                      }
                    />
                  </div>
                </div>
              </div>
            )}
          </Card>
        </div>

        {/* Error Display */}
        {(error || searchError) && (
          <div className="mb-6">
            <Alert
              variant="error"
              message={error || searchError || "An error occurred"}
            />
          </div>
        )}

        {/* Results Summary */}
        {(searchResults || items.length > 0) && (
          <div className="mb-4 flex items-center justify-between">
            <div className="text-sm text-gray-600 dark:text-gray-300">
              {searchResults ? (
                <>
                  Found {searchResults.total} items
                  {searchTerm && ` for "${searchTerm}"`}
                </>
              ) : (
                `Showing ${startIndex + 1}-${Math.min(endIndex, totalItems)} of ${totalItems} items`
              )}
            </div>
            {hasMore && <Badge variant="info">More results available</Badge>}
          </div>
        )}

        {/* Items Grid */}
        {loading && !paginatedItems.length ? (
          <div className="flex justify-center py-12">
            <LoadingSpinner size="lg" message="Loading traceability items..." />
          </div>
        ) : paginatedItems.length === 0 ? (
          <Card className="p-12 text-center">
            <Package className="w-16 h-16 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
              {searchResults ? "No items found" : "No items available"}
            </h3>
            <p className="text-gray-600 dark:text-gray-300">
              {searchResults
                ? "Try adjusting your search terms or filters"
                : "Items will appear here once they are added to the blockchain"}
            </p>
          </Card>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {paginatedItems.map((item) => (
              <Card
                key={item.id}
                className="p-6 hover:shadow-lg transition-shadow cursor-pointer"
                onClick={() => handleItemClick(item)}
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center gap-2">
                    {getStatusIcon(item)}
                    <h3 className="font-semibold text-gray-900 dark:text-white truncate">
                      {item.name}
                    </h3>
                  </div>
                  <ChevronRight className="w-4 h-4 text-gray-400 flex-shrink-0" />
                </div>

                <div className="space-y-2 mb-4">
                  <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
                    <Package className="w-4 h-4" />
                    <span className="truncate">{item.type}</span>
                  </div>

                  {item.current_owner && (
                    <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
                      <User className="w-4 h-4" />
                      <span className="truncate">{item.current_owner}</span>
                    </div>
                  )}

                  {item.location && (
                    <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
                      <MapPin className="w-4 h-4" />
                      <span className="truncate">{item.location}</span>
                    </div>
                  )}

                  <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
                    <Calendar className="w-4 h-4" />
                    <span>{formatDate(item.created_at)}</span>
                  </div>
                </div>

                <div className="flex items-center justify-between">
                  <Badge variant={getStatusColor(item)}>
                    {item.relationships?.length || 0} relationships
                  </Badge>
                  <span className="text-xs text-gray-500 dark:text-gray-400 font-mono">
                    {item.id.slice(0, 8)}...
                  </span>
                </div>
              </Card>
            ))}
          </div>
        )}

        {/* Pagination */}
        {!searchResults && totalPages > 1 && (
          <div className="mt-8 flex items-center justify-center gap-2">
            <Button
              variant="outline"
              onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
              disabled={currentPage === 1}
            >
              Previous
            </Button>

            <div className="flex items-center gap-1">
              {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                const page = i + Math.max(1, currentPage - 2);
                if (page > totalPages) return null;

                return (
                  <Button
                    key={page}
                    variant={page === currentPage ? "primary" : "outline"}
                    onClick={() => setCurrentPage(page)}
                    className="w-10 h-10 p-0"
                  >
                    {page}
                  </Button>
                );
              })}
            </div>

            <Button
              variant="outline"
              onClick={() =>
                setCurrentPage(Math.min(totalPages, currentPage + 1))
              }
              disabled={currentPage === totalPages}
            >
              Next
            </Button>
          </div>
        )}
      </div>
    </div>
  );
};

export default TraceabilityExplorer;
