import React, { useState, useEffect } from 'react';
import { Activity, Clock, User, Hash, ArrowRight, Filter, Search, CheckCircle } from 'lucide-react';
import LoadingSpinner from '../ui/LoadingSpinner';
import type { Transaction, TransactionType } from '../../types';

interface TransactionExplorerProps {
  onTransactionSelect?: (transaction: Transaction) => void;
}

const TransactionExplorer: React.FC<TransactionExplorerProps> = ({ onTransactionSelect }) => {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'pending' | 'confirmed'>('all');

  useEffect(() => {
    fetchTransactions();
  }, []);

  const fetchTransactions = async () => {
    try {
      setLoading(true);
      const token = localStorage.getItem('authToken');
      const response = await fetch('http://localhost:8080/api/transactions/recent', {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      setTransactions(data.transactions || []);
    } catch (err) {
      console.error('Error fetching transactions:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch transactions');
      // Fallback to mock data for development
      setTransactions(generateMockTransactions());
    } finally {
      setLoading(false);
    }
  };

  const generateMockTransactions = (): Transaction[] => {
    const mockTransactions: Transaction[] = [];
    const types: TransactionType[] = ['Production', 'Processing', 'Transport', 'Quality', 'Transfer', 'Environmental', 'Compliance', 'Governance'];
    const statuses: ('confirmed' | 'pending' | 'failed')[] = ['confirmed', 'pending', 'failed'];
    const participants = ['Producer_A', 'Manufacturer_B', 'LogisticsProvider_C', 'QualityLab_D', 'Retailer_E'];
    
    for (let i = 0; i < 20; i++) {
      const type = types[Math.floor(Math.random() * types.length)];
      const fromParticipant = participants[Math.floor(Math.random() * participants.length)];
      const toParticipant = participants[Math.floor(Math.random() * participants.length)];
      
      mockTransactions.push({
        id: `tx_${i.toString().padStart(3, '0')}`,
        type,
        from: fromParticipant,
        to: Math.random() > 0.3 ? toParticipant : undefined,
        timestamp: new Date(Date.now() - Math.random() * 7 * 24 * 60 * 60 * 1000).toISOString(),
        block_index: Math.floor(Math.random() * 1000) + 1000,
        signature: `sig_${Math.random().toString(36).substr(2, 16)}`,
        status: statuses[Math.floor(Math.random() * statuses.length)],
        data: {
          rdf_data: `<http://provchain.org/item/batch${i}> <http://provchain.org/ontology#hasStatus> "${type}" .`,
          subject: `http://provchain.org/item/batch${i}`,
          predicate: `http://provchain.org/ontology#${type.toLowerCase()}`,
          object: `Batch ${i} processed`,
          triple_count: Math.floor(Math.random() * 10) + 1,
          description: `${type} transaction for batch ${i}`,
          location: `Location_${Math.floor(Math.random() * 5) + 1}`,
          metadata: {
            temperature: type === 'Transport' ? `${Math.floor(Math.random() * 10) + 15}°C` : undefined,
            quality_score: type === 'Quality' ? Math.floor(Math.random() * 100) + 1 : undefined,
            quantity: Math.floor(Math.random() * 1000) + 100
          }
        },
      });
    }
    
    return mockTransactions.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
  };

  const filteredTransactions = transactions.filter(tx => {
    const matchesSearch = searchTerm === '' || 
      tx.signature.toLowerCase().includes(searchTerm.toLowerCase()) ||
      tx.from.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (tx.to && tx.to.toLowerCase().includes(searchTerm.toLowerCase())) ||
      tx.id.toLowerCase().includes(searchTerm.toLowerCase());
    
    const matchesFilter = filterType === 'all' || tx.status === filterType;
    
    return matchesSearch && matchesFilter;
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'confirmed':
        return 'text-green-600 bg-green-100 dark:text-green-400 dark:bg-green-900/30';
      case 'pending':
        return 'text-yellow-600 bg-yellow-100 dark:text-yellow-400 dark:bg-yellow-900/30';
      case 'failed':
        return 'text-red-600 bg-red-100 dark:text-red-400 dark:bg-red-900/30';
      default:
        return 'text-gray-600 bg-gray-100 dark:text-gray-400 dark:bg-gray-900/30';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'Production':
        return <Activity className="w-4 h-4" />;
      case 'Processing':
        return <Hash className="w-4 h-4" />;
      case 'Transport':
        return <ArrowRight className="w-4 h-4" />;
      case 'Quality':
        return <CheckCircle className="w-4 h-4" />;
      case 'Transfer':
        return <ArrowRight className="w-4 h-4" />;
      case 'Environmental':
        return <Activity className="w-4 h-4" />;
      case 'Compliance':
        return <Hash className="w-4 h-4" />;
      case 'Governance':
        return <User className="w-4 h-4" />;
      default:
        return <Hash className="w-4 h-4" />;
    }
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  const truncateHash = (hash: string, length = 10) => {
    return `${hash.slice(0, length)}...${hash.slice(-6)}`;
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <LoadingSpinner size="lg" message="Loading transactions..." />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Transaction Explorer</h1>
              <p className="text-gray-600 dark:text-gray-300 mt-2">
                Browse and analyze blockchain transactions
              </p>
            </div>
            <div className="flex items-center space-x-4">
              <div className="bg-white dark:bg-gray-800 px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700">
                <div className="flex items-center space-x-2">
                  <Activity className="w-5 h-5 text-primary-600 dark:text-primary-400" />
                  <span className="text-sm font-medium text-gray-900 dark:text-white">
                    {filteredTransactions.length} Transactions
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Filters */}
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 mb-6">
          <div className="flex flex-col sm:flex-row gap-4">
            {/* Search */}
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <input
                  type="text"
                  placeholder="Search by hash, address, or transaction ID..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                />
              </div>
            </div>

            {/* Status Filter */}
            <div className="flex items-center space-x-2">
              <Filter className="w-5 h-5 text-gray-400" />
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value as 'all' | 'pending' | 'confirmed')}
                className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              >
                <option value="all">All Status</option>
                <option value="confirmed">Confirmed</option>
                <option value="pending">Pending</option>
                <option value="failed">Failed</option>
              </select>
            </div>
          </div>
        </div>

        {/* Error State */}
        {error && (
          <div className="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6">
            <div className="flex items-center">
              <div className="text-red-600 dark:text-red-400">
                <Activity className="w-5 h-5" />
              </div>
              <div className="ml-3">
                <p className="text-red-800 dark:text-red-200 font-medium">Error loading transactions</p>
                <p className="text-red-600 dark:text-red-400 text-sm">{error}</p>
                <p className="text-red-600 dark:text-red-400 text-sm">Showing mock data for development.</p>
              </div>
            </div>
          </div>
        )}

        {/* Transactions List */}
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-50 dark:bg-gray-700">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Transaction
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Type
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Participants
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    RDF Data
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    Time
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {filteredTransactions.map((transaction) => (
                  <tr
                    key={transaction.id}
                    onClick={() => onTransactionSelect?.(transaction)}
                    className="hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer transition-colors"
                  >
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <Hash className="w-4 h-4 text-gray-400 mr-2" />
                        <div>
                          <div className="text-sm font-medium text-gray-900 dark:text-white">
                            {truncateHash(transaction.signature)}
                          </div>
                          <div className="text-sm text-gray-500 dark:text-gray-400">
                            Block #{transaction.block_index}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        {getTypeIcon(transaction.type)}
                        <span className="ml-2 text-sm text-gray-900 dark:text-white capitalize">
                          {transaction.type}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm">
                        <div className="text-gray-900 dark:text-white">
                          <User className="w-3 h-3 inline mr-1" />
                          {transaction.from}
                        </div>
                        {transaction.to && (
                          <div className="text-gray-500 dark:text-gray-400">
                            <ArrowRight className="w-3 h-3 inline mr-1" />
                            {transaction.to}
                          </div>
                        )}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm">
                        <div className="text-gray-900 dark:text-white">
                          {((transaction.data as Record<string, unknown>)?.triple_count as number) || 0} triples
                        </div>
                        <div className="text-gray-500 dark:text-gray-400 truncate max-w-xs">
                          {String((transaction.data as Record<string, unknown>)?.subject || 'N/A')}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${getStatusColor(transaction.status)}`}>
                        {transaction.status}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center text-sm text-gray-500 dark:text-gray-400">
                        <Clock className="w-4 h-4 mr-1" />
                        {formatTimestamp(transaction.timestamp)}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {filteredTransactions.length === 0 && (
            <div className="text-center py-12">
              <Activity className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">No transactions found</h3>
              <p className="text-gray-500 dark:text-gray-400">
                {searchTerm || filterType !== 'all' 
                  ? 'Try adjusting your search or filter criteria.'
                  : 'No transactions available at the moment.'
                }
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TransactionExplorer;
