import React, { useState, useEffect } from 'react';
import { 
  ArrowLeft, 
  Package, 
  User, 
  MapPin, 
  Calendar, 
  Shield, 
  AlertCircle, 
  CheckCircle, 
  ExternalLink,
  Copy,
  RefreshCw,
  Activity,
  Link,
  FileText
} from 'lucide-react';
import Button from '../ui/Button';
import Card from '../ui/Card';
import Badge from '../ui/Badge';
import LoadingSpinner from '../ui/LoadingSpinner';
import Alert from '../ui/Alert';
import { useTraceability } from '../../hooks/useTraceability';

interface ItemDetailsProps {
  itemId: string;
  onBack?: () => void;
  onRelatedItemSelect?: (itemId: string) => void;
}

interface ValidationResult {
  is_authentic: boolean;
  integrity_score: number;
  validation_details: {
    signature_valid: boolean;
    chain_intact: boolean;
    data_consistent: boolean;
    timestamp_valid: boolean;
  };
  validation_time_ms: number;
}

const ItemDetails: React.FC<ItemDetailsProps> = ({ 
  itemId, 
  onBack, 
  onRelatedItemSelect 
}) => {
  const {
    selectedItem,
    traceData,
    traceLoading,
    traceError,
    selectItem,
    validateItem,
    refresh
  } = useTraceability();

  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [validationLoading, setValidationLoading] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'provenance' | 'relationships' | 'transactions'>('overview');
  const [copiedField, setCopiedField] = useState<string | null>(null);

  // Load item data when component mounts or itemId changes
  useEffect(() => {
    if (itemId && (!selectedItem || selectedItem.id !== itemId)) {
      selectItem(itemId);
    }
  }, [itemId, selectedItem, selectItem]);

  // Handle validation
  const handleValidation = async () => {
    if (!selectedItem) return;

    setValidationLoading(true);
    setValidationError(null);

    try {
      const result = await validateItem(selectedItem.id);
      setValidationResult(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Validation failed';
      setValidationError(errorMessage);
    } finally {
      setValidationLoading(false);
    }
  };

  // Handle copy to clipboard
  const handleCopy = async (text: string, field: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedField(field);
      setTimeout(() => setCopiedField(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  // Format date
  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  };

  // Get validation status color
  const getValidationColor = (result: ValidationResult): 'success' | 'warning' | 'danger' => {
    if (result.integrity_score >= 0.9) return 'success';
    if (result.integrity_score >= 0.7) return 'warning';
    return 'danger';
  };

  // Get validation status text
  const getValidationText = (result: ValidationResult): string => {
    if (result.integrity_score >= 0.9) return 'Highly Trusted';
    if (result.integrity_score >= 0.7) return 'Moderately Trusted';
    return 'Low Trust';
  };

  if (traceLoading && !selectedItem) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <LoadingSpinner size="lg" message="Loading item details..." />
      </div>
    );
  }

  if (traceError && !selectedItem) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
        <div className="max-w-4xl mx-auto">
          <Alert
            variant="error"
            message={traceError}
          />
          {onBack && (
            <Button onClick={onBack} className="flex items-center gap-2">
              <ArrowLeft className="w-4 h-4" />
              Back to Explorer
            </Button>
          )}
        </div>
      </div>
    );
  }

  if (!selectedItem) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
        <div className="max-w-4xl mx-auto text-center py-12">
          <Package className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
            Item not found
          </h3>
          <p className="text-gray-600 dark:text-gray-300 mb-6">
            The requested item could not be loaded.
          </p>
          {onBack && (
            <Button onClick={onBack} className="flex items-center gap-2">
              <ArrowLeft className="w-4 h-4" />
              Back to Explorer
            </Button>
          )}
        </div>
      </div>
    );
  }

  const tabs = [
    { id: 'overview', label: 'Overview', icon: Package },
    { id: 'provenance', label: 'Provenance Chain', icon: Activity },
    { id: 'relationships', label: 'Relationships', icon: Link },
    { id: 'transactions', label: 'Transactions', icon: FileText }
  ];

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-6xl mx-auto p-6">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-4">
              {onBack && (
                <Button
                  variant="outline"
                  onClick={onBack}
                  className="flex items-center gap-2"
                >
                  <ArrowLeft className="w-4 h-4" />
                  Back
                </Button>
              )}
              <div>
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                  {selectedItem.name}
                </h1>
                <p className="text-gray-600 dark:text-gray-300 mt-1">
                  Item Details & Provenance
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Button
                onClick={handleValidation}
                disabled={validationLoading}
                className="flex items-center gap-2"
                variant="outline"
              >
                <Shield className={`w-4 h-4 ${validationLoading ? 'animate-pulse' : ''}`} />
                {validationLoading ? 'Validating...' : 'Validate'}
              </Button>
              <Button
                onClick={refresh}
                disabled={traceLoading}
                className="flex items-center gap-2"
              >
                <RefreshCw className={`w-4 h-4 ${traceLoading ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            </div>
          </div>

          {/* Item Summary Card */}
          <Card className="p-6 mb-6">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
                  <Package className="w-5 h-5 text-blue-600 dark:text-blue-400" />
                </div>
                <div>
                  <p className="text-sm text-gray-600 dark:text-gray-300">Type</p>
                  <p className="font-semibold text-gray-900 dark:text-white">{selectedItem.type}</p>
                </div>
              </div>

              <div className="flex items-center gap-3">
                <div className="p-2 bg-green-100 dark:bg-green-900 rounded-lg">
                  <User className="w-5 h-5 text-green-600 dark:text-green-400" />
                </div>
                <div>
                  <p className="text-sm text-gray-600 dark:text-gray-300">Current Owner</p>
                  <p className="font-semibold text-gray-900 dark:text-white">{selectedItem.current_owner}</p>
                </div>
              </div>

              {selectedItem.location && (
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-purple-100 dark:bg-purple-900 rounded-lg">
                    <MapPin className="w-5 h-5 text-purple-600 dark:text-purple-400" />
                  </div>
                  <div>
                    <p className="text-sm text-gray-600 dark:text-gray-300">Location</p>
                    <p className="font-semibold text-gray-900 dark:text-white">{selectedItem.location}</p>
                  </div>
                </div>
              )}

              <div className="flex items-center gap-3">
                <div className="p-2 bg-orange-100 dark:bg-orange-900 rounded-lg">
                  <Calendar className="w-5 h-5 text-orange-600 dark:text-orange-400" />
                </div>
                <div>
                  <p className="text-sm text-gray-600 dark:text-gray-300">Created</p>
                  <p className="font-semibold text-gray-900 dark:text-white">
                    {formatDate(selectedItem.created_at)}
                  </p>
                </div>
              </div>
            </div>

            {/* Item ID with copy functionality */}
            <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-600 dark:text-gray-300">Item ID</p>
                  <p className="font-mono text-sm text-gray-900 dark:text-white">{selectedItem.id}</p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleCopy(selectedItem.id, 'id')}
                  className="flex items-center gap-2"
                >
                  <Copy className="w-4 h-4" />
                  {copiedField === 'id' ? 'Copied!' : 'Copy'}
                </Button>
              </div>
            </div>
          </Card>

          {/* Validation Results */}
          {validationResult && (
            <Card className="p-6 mb-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                  Validation Results
                </h3>
                <Badge variant={getValidationColor(validationResult)}>
                  {getValidationText(validationResult)}
                </Badge>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
                <div className="flex items-center gap-2">
                  {validationResult.validation_details.signature_valid ? (
                    <CheckCircle className="w-5 h-5 text-green-500" />
                  ) : (
                    <AlertCircle className="w-5 h-5 text-red-500" />
                  )}
                  <span className="text-sm text-gray-700 dark:text-gray-300">
                    Signature Valid
                  </span>
                </div>

                <div className="flex items-center gap-2">
                  {validationResult.validation_details.chain_intact ? (
                    <CheckCircle className="w-5 h-5 text-green-500" />
                  ) : (
                    <AlertCircle className="w-5 h-5 text-red-500" />
                  )}
                  <span className="text-sm text-gray-700 dark:text-gray-300">
                    Chain Intact
                  </span>
                </div>

                <div className="flex items-center gap-2">
                  {validationResult.validation_details.data_consistent ? (
                    <CheckCircle className="w-5 h-5 text-green-500" />
                  ) : (
                    <AlertCircle className="w-5 h-5 text-red-500" />
                  )}
                  <span className="text-sm text-gray-700 dark:text-gray-300">
                    Data Consistent
                  </span>
                </div>

                <div className="flex items-center gap-2">
                  {validationResult.validation_details.timestamp_valid ? (
                    <CheckCircle className="w-5 h-5 text-green-500" />
                  ) : (
                    <AlertCircle className="w-5 h-5 text-red-500" />
                  )}
                  <span className="text-sm text-gray-700 dark:text-gray-300">
                    Timestamp Valid
                  </span>
                </div>
              </div>

              <div className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-300">
                <span>
                  Integrity Score: {Math.round(validationResult.integrity_score * 100)}%
                </span>
                <span>
                  Validation Time: {validationResult.validation_time_ms}ms
                </span>
              </div>
            </Card>
          )}

          {validationError && (
            <Alert
              variant="error"
              message={validationError}
            />
          )}
        </div>

        {/* Tabs */}
        <div className="mb-6">
          <div className="border-b border-gray-200 dark:border-gray-700">
            <nav className="-mb-px flex space-x-8">
              {tabs.map((tab) => {
                const Icon = tab.icon;
                const isActive = activeTab === tab.id;
                
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id as typeof activeTab)}
                    className={`
                      flex items-center gap-2 py-2 px-1 border-b-2 font-medium text-sm
                      ${isActive
                        ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                        : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
                      }
                    `}
                  >
                    <Icon className="w-4 h-4" />
                    {tab.label}
                  </button>
                );
              })}
            </nav>
          </div>
        </div>

        {/* Tab Content */}
        <div className="space-y-6">
          {activeTab === 'overview' && (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Properties */}
              <Card className="p-6">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                  Properties
                </h3>
                {Object.keys(selectedItem.properties).length > 0 ? (
                  <div className="space-y-3">
                    {Object.entries(selectedItem.properties).map(([key, value]) => (
                      <div key={key} className="flex justify-between items-start">
                        <span className="text-sm font-medium text-gray-600 dark:text-gray-300 capitalize">
                          {key.replace(/_/g, ' ')}:
                        </span>
                        <span className="text-sm text-gray-900 dark:text-white text-right max-w-xs">
                          {typeof value === 'object' ? JSON.stringify(value) : String(value)}
                        </span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-gray-500 dark:text-gray-400 text-sm">
                    No additional properties available
                  </p>
                )}
              </Card>

              {/* Quick Stats */}
              <Card className="p-6">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                  Quick Stats
                </h3>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600 dark:text-gray-300">Relationships</span>
                    <Badge variant="info">{selectedItem.relationships.length}</Badge>
                  </div>
                  
                  {traceData && (
                    <>
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-gray-600 dark:text-gray-300">Provenance Steps</span>
                        <Badge variant="info">{traceData.trace_path.length}</Badge>
                      </div>
                      
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-gray-600 dark:text-gray-300">Related Transactions</span>
                        <Badge variant="info">{traceData.related_transactions.length}</Badge>
                      </div>
                      
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-gray-600 dark:text-gray-300">Knowledge Graph Nodes</span>
                        <Badge variant="info">{traceData.knowledge_graph.nodes.length}</Badge>
                      </div>
                    </>
                  )}
                </div>
              </Card>
            </div>
          )}

          {activeTab === 'provenance' && (
            <Card className="p-6">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">
                Provenance Chain
              </h3>
              
              {traceLoading ? (
                <div className="flex justify-center py-8">
                  <LoadingSpinner size="md" message="Loading provenance data..." />
                </div>
              ) : traceData?.trace_path.length ? (
                <div className="space-y-4">
                  {traceData.trace_path.map((step, index) => (
                    <div key={step.step_number} className="relative">
                      {index < traceData.trace_path.length - 1 && (
                        <div className="absolute left-6 top-12 w-0.5 h-8 bg-gray-300 dark:bg-gray-600" />
                      )}
                      
                      <div className="flex items-start gap-4">
                        <div className="flex-shrink-0 w-12 h-12 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
                          <span className="text-sm font-semibold text-blue-600 dark:text-blue-400">
                            {step.step_number}
                          </span>
                        </div>
                        
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center justify-between mb-2">
                            <h4 className="text-sm font-semibold text-gray-900 dark:text-white">
                              {step.action}
                            </h4>
                            <span className="text-xs text-gray-500 dark:text-gray-400">
                              {formatDate(step.timestamp)}
                            </span>
                          </div>
                          
                          <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-sm text-gray-600 dark:text-gray-300">
                            <div>
                              <span className="font-medium">Participant:</span> {step.participant}
                            </div>
                            {step.location && (
                              <div>
                                <span className="font-medium">Location:</span> {step.location}
                              </div>
                            )}
                          </div>
                          
                          {Object.keys(step.metadata).length > 0 && (
                            <div className="mt-2 p-2 bg-gray-50 dark:bg-gray-800 rounded text-xs">
                              <details>
                                <summary className="cursor-pointer text-gray-700 dark:text-gray-300">
                                  View metadata
                                </summary>
                                <pre className="mt-2 text-gray-600 dark:text-gray-400 whitespace-pre-wrap">
                                  {JSON.stringify(step.metadata, null, 2)}
                                </pre>
                              </details>
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                  No provenance data available
                </p>
              )}
            </Card>
          )}

          {activeTab === 'relationships' && (
            <Card className="p-6">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">
                Item Relationships
              </h3>
              
              {selectedItem.relationships.length > 0 ? (
                <div className="space-y-4">
                  {selectedItem.relationships.map((relationship, index) => (
                    <div
                      key={index}
                      className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
                    >
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                          <Badge variant="info">{relationship.type.replace(/_/g, ' ')}</Badge>
                          <span className="text-sm text-gray-600 dark:text-gray-300">
                            → {relationship.target_item}
                          </span>
                        </div>
                        {onRelatedItemSelect && (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onRelatedItemSelect(relationship.target_item)}
                            className="flex items-center gap-1"
                          >
                            <ExternalLink className="w-3 h-3" />
                            View
                          </Button>
                        )}
                      </div>
                      
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        {formatDate(relationship.timestamp)} • Transaction: {relationship.transaction_id}
                      </div>
                      
                      {relationship.metadata && Object.keys(relationship.metadata).length > 0 && (
                        <div className="mt-2 text-xs">
                          <details>
                            <summary className="cursor-pointer text-gray-600 dark:text-gray-400">
                              View metadata
                            </summary>
                            <pre className="mt-1 text-gray-500 dark:text-gray-500 whitespace-pre-wrap">
                              {JSON.stringify(relationship.metadata, null, 2)}
                            </pre>
                          </details>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                  No relationships found
                </p>
              )}
            </Card>
          )}

          {activeTab === 'transactions' && (
            <Card className="p-6">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">
                Related Transactions
              </h3>
              
              {traceLoading ? (
                <div className="flex justify-center py-8">
                  <LoadingSpinner size="md" message="Loading transaction data..." />
                </div>
              ) : traceData?.related_transactions.length ? (
                <div className="space-y-4">
                  {traceData.related_transactions.map((transaction) => (
                    <div
                      key={transaction.id}
                      className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg"
                    >
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center gap-2">
                          <Badge variant="info">{transaction.type}</Badge>
                          <Badge 
                            variant={
                              transaction.status === 'confirmed' ? 'success' : 
                              transaction.status === 'failed' ? 'danger' : 'warning'
                            }
                          >
                            {transaction.status}
                          </Badge>
                        </div>
                        <span className="text-xs text-gray-500 dark:text-gray-400">
                          Block #{transaction.block_index}
                        </span>
                      </div>
                      
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-sm text-gray-600 dark:text-gray-300 mb-2">
                        <div>
                          <span className="font-medium">From:</span> {transaction.from}
                        </div>
                        {transaction.to && (
                          <div>
                            <span className="font-medium">To:</span> {transaction.to}
                          </div>
                        )}
                      </div>
                      
                      <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
                        <span>{formatDate(transaction.timestamp)}</span>
                        <div className="flex items-center gap-2">
                          <span>ID: {transaction.id.slice(0, 8)}...</span>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleCopy(transaction.id, `tx-${transaction.id}`)}
                            className="p-1"
                          >
                            <Copy className="w-3 h-3" />
                          </Button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                  No related transactions found
                </p>
              )}
            </Card>
          )}
        </div>
      </div>
    </div>
  );
};

export default ItemDetails;
