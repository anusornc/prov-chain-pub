import React, { useState, useEffect } from 'react';
import { 
  ArrowLeft, 
  Copy, 
  ExternalLink, 
  Clock, 
  Database, 
  Activity, 
  CheckCircle, 
  AlertCircle
} from 'lucide-react';
import Button from '../ui/Button';
import Card from '../ui/Card';
import Badge from '../ui/Badge';
import LoadingSpinner from '../ui/LoadingSpinner';
import useBlockchain from '../../hooks/useBlockchain';
import type { Block, Transaction, RdfSummary, ValidationStatus } from '../../types';

interface BlockDetailsProps {
  block: Block;
  onBack: () => void;
  onTransactionSelect?: (transaction: Transaction) => void;
}

interface TransactionListItemProps {
  transaction: Transaction;
  onClick: (transaction: Transaction) => void;
}

const TransactionListItem: React.FC<TransactionListItemProps> = ({ transaction, onClick }) => {
  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'confirmed': return 'text-green-600 dark:text-green-400';
      case 'pending': return 'text-yellow-600 dark:text-yellow-400';
      case 'failed': return 'text-red-600 dark:text-red-400';
      default: return 'text-gray-600 dark:text-gray-400';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'confirmed': return <CheckCircle className="w-4 h-4" />;
      case 'pending': return <Clock className="w-4 h-4" />;
      case 'failed': return <AlertCircle className="w-4 h-4" />;
      default: return <Activity className="w-4 h-4" />;
    }
  };

  return (
    <Card className="p-4 hover:shadow-md transition-shadow cursor-pointer" onClick={() => onClick(transaction)}>
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="flex items-center justify-center w-8 h-8 bg-blue-100 dark:bg-blue-900 rounded-lg">
            <Activity className="w-4 h-4 text-blue-600 dark:text-blue-400" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <p className="font-medium text-gray-900 dark:text-white">
                {transaction.id.slice(0, 12)}...
              </p>
              <Badge variant="secondary" size="sm">
                {transaction.type}
              </Badge>
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {formatTimestamp(transaction.timestamp)}
            </p>
          </div>
        </div>
        
        <div className="flex items-center space-x-4">
          <div className="text-right">
            <p className="text-sm text-gray-600 dark:text-gray-400">
              From: {transaction.from.slice(0, 8)}...
            </p>
            {transaction.to && (
              <p className="text-sm text-gray-600 dark:text-gray-400">
                To: {transaction.to.slice(0, 8)}...
              </p>
            )}
          </div>
          <div className={`flex items-center space-x-1 ${getStatusColor(transaction.status)}`}>
            {getStatusIcon(transaction.status)}
            <span className="text-sm font-medium capitalize">{transaction.status}</span>
          </div>
        </div>
      </div>
    </Card>
  );
};

const BlockDetails: React.FC<BlockDetailsProps> = ({ block, onBack, onTransactionSelect }) => {
  const { fetchBlock } = useBlockchain();
  const [detailedBlock, setDetailedBlock] = useState<Block | null>(block);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [rdfSummary, setRdfSummary] = useState<RdfSummary | null>(null);
  const [validationStatus, setValidationStatus] = useState<ValidationStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchBlockDetails = async () => {
      try {
        setLoading(true);
        setError(null);

        // Fetch detailed block information
        const blockData = await fetchBlock(block.index);
        if (blockData) {
          setDetailedBlock(blockData);
        }

        // Mock transaction data - replace with actual API call
        const transactionTypes: Array<'Production' | 'Processing' | 'Transport' | 'Quality'> = ['Production', 'Processing', 'Transport', 'Quality'];
        const transactionStatuses: Array<'confirmed' | 'pending' | 'failed'> = ['confirmed', 'pending', 'failed'];
        
        const mockTransactions: Transaction[] = Array.from({ length: block.transaction_count }, (_, i) => ({
          id: `tx_${block.index}_${i}`,
          type: transactionTypes[i % 4],
          from: `addr_${Math.random().toString(36).substr(2, 8)}`,
          to: `addr_${Math.random().toString(36).substr(2, 8)}`,
          timestamp: new Date(Date.now() - Math.random() * 86400000).toISOString(),
          block_index: block.index,
          signature: `sig_${Math.random().toString(36).substr(2, 16)}`,
          data: { amount: Math.floor(Math.random() * 1000) },
          status: transactionStatuses[Math.floor(Math.random() * 3)],
          gas_used: Math.floor(Math.random() * 50000),
          gas_price: Math.floor(Math.random() * 100)
        }));
        setTransactions(mockTransactions);

        // Mock RDF summary
        setRdfSummary({
          triple_count: Math.floor(Math.random() * 1000) + 100,
          subject_count: Math.floor(Math.random() * 100) + 10,
          predicate_count: Math.floor(Math.random() * 50) + 5,
          object_count: Math.floor(Math.random() * 200) + 20,
          namespaces: ['http://provchain.org/', 'http://www.w3.org/ns/prov#', 'http://example.org/']
        });

        // Mock validation status
        setValidationStatus({
          is_valid: Math.random() > 0.1,
          validation_time_ms: Math.floor(Math.random() * 100) + 10,
          errors: [],
          warnings: Math.random() > 0.7 ? ['Minor validation warning'] : []
        });

      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to fetch block details';
        setError(errorMessage);
        console.error('Error fetching block details:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchBlockDetails();
  }, [block.index, fetchBlock]);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    // TODO: Add toast notification
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  const formatHash = (hash: string) => {
    return `${hash.slice(0, 16)}...${hash.slice(-16)}`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
        <span className="ml-2 text-gray-600 dark:text-gray-400">Loading block details...</span>
      </div>
    );
  }

  if (error) {
    return (
      <Card className="p-8 text-center border-red-200 dark:border-red-800">
        <AlertCircle className="w-12 h-12 text-red-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
          Error Loading Block Details
        </h3>
        <p className="text-gray-600 dark:text-gray-400 mb-4">{error}</p>
        <Button onClick={onBack} variant="outline">
          Go Back
        </Button>
      </Card>
    );
  }

  if (!detailedBlock) {
    return (
      <Card className="p-8 text-center">
        <Database className="w-12 h-12 text-gray-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
          Block Not Found
        </h3>
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          The requested block could not be found.
        </p>
        <Button onClick={onBack} variant="outline">
          Go Back
        </Button>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Button variant="outline" onClick={onBack}>
            <ArrowLeft className="w-4 h-4 mr-2" />
            Back to Explorer
          </Button>
          <div>
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
              Block #{detailedBlock.index}
            </h1>
            <p className="text-gray-600 dark:text-gray-400">
              {formatTimestamp(detailedBlock.timestamp)}
            </p>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          {validationStatus?.is_valid ? (
            <Badge variant="secondary" className="text-green-600 bg-green-100 dark:bg-green-900">
              <CheckCircle className="w-3 h-3 mr-1" />
              Valid
            </Badge>
          ) : (
            <Badge variant="secondary" className="text-red-600 bg-red-100 dark:bg-red-900">
              <AlertCircle className="w-3 h-3 mr-1" />
              Invalid
            </Badge>
          )}
        </div>
      </div>

      {/* Block Information */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Block Information
          </h3>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">Block Hash</span>
              <div className="flex items-center space-x-2">
                <code className="text-sm font-mono text-gray-900 dark:text-white">
                  {formatHash(detailedBlock.hash)}
                </code>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => copyToClipboard(detailedBlock.hash)}
                >
                  <Copy className="w-3 h-3" />
                </Button>
              </div>
            </div>
            
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">Previous Hash</span>
              <div className="flex items-center space-x-2">
                <code className="text-sm font-mono text-gray-900 dark:text-white">
                  {formatHash(detailedBlock.previous_hash)}
                </code>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => copyToClipboard(detailedBlock.previous_hash)}
                >
                  <Copy className="w-3 h-3" />
                </Button>
              </div>
            </div>
            
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">Timestamp</span>
              <span className="text-sm font-medium text-gray-900 dark:text-white">
                {formatTimestamp(detailedBlock.timestamp)}
              </span>
            </div>
            
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">Size</span>
              <span className="text-sm font-medium text-gray-900 dark:text-white">
                {(detailedBlock.size / 1024).toFixed(2)} KB
              </span>
            </div>
            
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600 dark:text-gray-400">Transaction Count</span>
              <span className="text-sm font-medium text-gray-900 dark:text-white">
                {detailedBlock.transaction_count}
              </span>
            </div>
            
            {detailedBlock.validator && (
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600 dark:text-gray-400">Validator</span>
                <code className="text-sm font-mono text-gray-900 dark:text-white">
                  {detailedBlock.validator.slice(0, 16)}...
                </code>
              </div>
            )}
          </div>
        </Card>

        {/* RDF Summary */}
        {rdfSummary && (
          <Card className="p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              RDF Data Summary
            </h3>
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="text-center">
                  <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                    {rdfSummary.triple_count.toLocaleString()}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400">Triples</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                    {rdfSummary.subject_count.toLocaleString()}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400">Subjects</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
                    {rdfSummary.predicate_count.toLocaleString()}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400">Predicates</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                    {rdfSummary.object_count.toLocaleString()}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400">Objects</div>
                </div>
              </div>
              
              <div>
                <h4 className="text-sm font-medium text-gray-900 dark:text-white mb-2">
                  Namespaces
                </h4>
                <div className="space-y-1">
                  {rdfSummary.namespaces.map((namespace, index) => (
                    <code key={index} className="block text-xs text-gray-600 dark:text-gray-400">
                      {namespace}
                    </code>
                  ))}
                </div>
              </div>
            </div>
          </Card>
        )}
      </div>

      {/* Validation Status */}
      {validationStatus && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Validation Status
          </h3>
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className={`flex items-center space-x-2 ${validationStatus.is_valid ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>
                {validationStatus.is_valid ? (
                  <CheckCircle className="w-5 h-5" />
                ) : (
                  <AlertCircle className="w-5 h-5" />
                )}
                <span className="font-medium">
                  {validationStatus.is_valid ? 'Valid' : 'Invalid'}
                </span>
              </div>
              <span className="text-sm text-gray-600 dark:text-gray-400">
                Validated in {validationStatus.validation_time_ms}ms
              </span>
            </div>
            
            {validationStatus.warnings.length > 0 && (
              <div className="text-sm text-yellow-600 dark:text-yellow-400">
                {validationStatus.warnings.length} warning(s)
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Transactions */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Transactions ({transactions.length})
          </h3>
          <Button variant="outline" size="sm">
            <ExternalLink className="w-4 h-4 mr-2" />
            View All
          </Button>
        </div>
        
        {transactions.length === 0 ? (
          <div className="text-center py-8">
            <Activity className="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <p className="text-gray-600 dark:text-gray-400">No transactions in this block</p>
          </div>
        ) : (
          <div className="space-y-3">
            {transactions.slice(0, 10).map((transaction) => (
              <TransactionListItem
                key={transaction.id}
                transaction={transaction}
                onClick={(tx) => onTransactionSelect?.(tx)}
              />
            ))}
            {transactions.length > 10 && (
              <div className="text-center pt-4">
                <Button variant="outline" size="sm">
                  Load More Transactions
                </Button>
              </div>
            )}
          </div>
        )}
      </Card>
    </div>
  );
};

export default BlockDetails;
