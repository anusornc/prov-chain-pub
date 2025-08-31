import React, { useState, useEffect } from 'react';
import { 
  Search, 
  Filter, 
  RefreshCw, 
  ChevronRight, 
  Clock, 
  Database,
  Activity,
  AlertCircle,
  CheckCircle
} from 'lucide-react';
import Button from '../ui/Button';
import Input from '../ui/Input';
import Card from '../ui/Card';
import Badge from '../ui/Badge';
import LoadingSpinner from '../ui/LoadingSpinner';
import useBlockchain from '../../hooks/useBlockchain';
import type { Block, SearchFilters } from '../../types';

interface BlockListItemProps {
  block: Block;
  onClick: (block: Block) => void;
}

const BlockListItem: React.FC<BlockListItemProps> = ({ block, onClick }) => {
  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  const formatHash = (hash: string) => {
    return `${hash.slice(0, 8)}...${hash.slice(-8)}`;
  };

  return (
    <Card className="p-4 hover:shadow-md transition-shadow cursor-pointer" onClick={() => onClick(block)}>
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="flex items-center justify-center w-10 h-10 bg-primary-100 dark:bg-primary-900 rounded-lg">
            <Database className="w-5 h-5 text-primary-600 dark:text-primary-400" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Block #{block.index}
              </h3>
              <Badge variant="secondary" size="sm">
                {block.transaction_count} txns
              </Badge>
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {formatTimestamp(block.timestamp)}
            </p>
          </div>
        </div>
        
        <div className="flex items-center space-x-4">
          <div className="text-right">
            <p className="text-sm font-medium text-gray-900 dark:text-white">
              Hash: {formatHash(block.hash)}
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              Size: {(block.size / 1024).toFixed(1)} KB
            </p>
          </div>
          <ChevronRight className="w-5 h-5 text-gray-400" />
        </div>
      </div>
    </Card>
  );
};

interface SearchBarProps {
  onSearch: (query: string) => void;
  onFilterChange: (filters: SearchFilters) => void;
  loading?: boolean;
}

const SearchBar: React.FC<SearchBarProps> = ({ onSearch, loading = false }) => {
  const [query, setQuery] = useState('');
  const [showFilters, setShowFilters] = useState(false);

  const handleSearch = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    onSearch(query);
  };

  return (
    <div className="space-y-4">
      <form onSubmit={handleSearch} className="flex space-x-2">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <Input
            type="text"
            placeholder="Search by block number, hash, or transaction ID..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        <Button type="submit" disabled={loading}>
          {loading ? <LoadingSpinner size="sm" /> : 'Search'}
        </Button>
        <Button
          type="button"
          variant="outline"
          onClick={() => setShowFilters(!showFilters)}
        >
          <Filter className="w-4 h-4" />
        </Button>
      </form>

      {showFilters && (
        <Card className="p-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Date Range
              </label>
              <Input type="date" />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Min Transactions
              </label>
              <Input type="number" placeholder="0" />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Validator
              </label>
              <Input type="text" placeholder="Validator address" />
            </div>
          </div>
          <div className="flex justify-end space-x-2 mt-4">
            <Button variant="outline" size="sm" onClick={() => setShowFilters(false)}>
              Cancel
            </Button>
            <Button size="sm">Apply Filters</Button>
          </div>
        </Card>
      )}
    </div>
  );
};

interface BlockExplorerProps {
  onBlockSelect?: (block: Block) => void;
}

const BlockExplorer: React.FC<BlockExplorerProps> = ({ onBlockSelect }) => {
  const { 
    blocks, 
    loading, 
    blocksLoading, 
    error, 
    refresh 
  } = useBlockchain();

  const [filteredBlocks, setFilteredBlocks] = useState<Block[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage] = useState(10);

  // Update filtered blocks when blocks change
  useEffect(() => {
    setFilteredBlocks(blocks);
  }, [blocks]);

  // Handle search
  const handleSearch = (query: string) => {
    setSearchQuery(query);
    setCurrentPage(1);

    if (!query.trim()) {
      setFilteredBlocks(blocks);
      return;
    }

    const filtered = blocks.filter(block => {
      const queryLower = query.toLowerCase();
      return (
        block.index.toString().includes(queryLower) ||
        block.hash.toLowerCase().includes(queryLower) ||
        block.previous_hash.toLowerCase().includes(queryLower)
      );
    });

    setFilteredBlocks(filtered);
  };

  // Handle filter changes
  const handleFilterChange = (filters: SearchFilters) => {
    // TODO: Implement filtering logic
    console.log('Filters changed:', filters);
  };

  // Handle block selection
  const handleBlockClick = (block: Block) => {
    if (onBlockSelect) {
      onBlockSelect(block);
    }
  };

  // Pagination
  const totalPages = Math.ceil(filteredBlocks.length / itemsPerPage);
  const startIndex = (currentPage - 1) * itemsPerPage;
  const endIndex = startIndex + itemsPerPage;
  const currentBlocks = filteredBlocks.slice(startIndex, endIndex);

  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
        <span className="ml-2 text-gray-600 dark:text-gray-400">Loading blockchain data...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Block Explorer
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Browse and search blockchain blocks and transactions
          </p>
        </div>
        <div className="flex space-x-2">
          <Button
            variant="outline"
            onClick={refresh}
            disabled={blocksLoading}
          >
            <RefreshCw className={`w-4 h-4 ${blocksLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Search and Filters */}
      <SearchBar
        onSearch={handleSearch}
        onFilterChange={handleFilterChange}
        loading={blocksLoading}
      />

      {/* Error State */}
      {error && (
        <Card className="p-4 border-red-200 dark:border-red-800">
          <div className="flex items-center space-x-2 text-red-600 dark:text-red-400">
            <AlertCircle className="w-5 h-5" />
            <span>Error: {error}</span>
          </div>
        </Card>
      )}

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card className="p-4">
          <div className="flex items-center space-x-2">
            <Database className="w-5 h-5 text-blue-500" />
            <div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Total Blocks</p>
              <p className="text-xl font-semibold text-gray-900 dark:text-white">
                {blocks.length.toLocaleString()}
              </p>
            </div>
          </div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center space-x-2">
            <Activity className="w-5 h-5 text-green-500" />
            <div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Total Transactions</p>
              <p className="text-xl font-semibold text-gray-900 dark:text-white">
                {blocks.reduce((sum, block) => sum + block.transaction_count, 0).toLocaleString()}
              </p>
            </div>
          </div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center space-x-2">
            <Clock className="w-5 h-5 text-yellow-500" />
            <div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Latest Block</p>
              <p className="text-xl font-semibold text-gray-900 dark:text-white">
                #{blocks[0]?.index || 0}
              </p>
            </div>
          </div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center space-x-2">
            <CheckCircle className="w-5 h-5 text-purple-500" />
            <div>
              <p className="text-sm text-gray-600 dark:text-gray-400">Search Results</p>
              <p className="text-xl font-semibold text-gray-900 dark:text-white">
                {filteredBlocks.length.toLocaleString()}
              </p>
            </div>
          </div>
        </Card>
      </div>

      {/* Block List */}
      <div className="space-y-4">
        {currentBlocks.length === 0 ? (
          <Card className="p-8 text-center">
            <Database className="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
              No blocks found
            </h3>
            <p className="text-gray-600 dark:text-gray-400">
              {searchQuery ? 'Try adjusting your search criteria.' : 'No blocks available yet.'}
            </p>
          </Card>
        ) : (
          currentBlocks.map((block) => (
            <BlockListItem
              key={block.index}
              block={block}
              onClick={handleBlockClick}
            />
          ))
        )}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between">
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Showing {startIndex + 1} to {Math.min(endIndex, filteredBlocks.length)} of{' '}
            {filteredBlocks.length} blocks
          </p>
          <div className="flex space-x-1">
            <Button
              variant="outline"
              size="sm"
              onClick={() => handlePageChange(currentPage - 1)}
              disabled={currentPage === 1}
            >
              Previous
            </Button>
            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
              const page = i + 1;
              return (
                <Button
                  key={page}
                  variant={currentPage === page ? 'primary' : 'outline'}
                  size="sm"
                  onClick={() => handlePageChange(page)}
                >
                  {page}
                </Button>
              );
            })}
            <Button
              variant="outline"
              size="sm"
              onClick={() => handlePageChange(currentPage + 1)}
              disabled={currentPage === totalPages}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
};

export default BlockExplorer;
