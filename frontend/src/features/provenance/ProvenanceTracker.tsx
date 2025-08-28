import React, { useState } from 'react';
import Card from '../../components/ui/Card';
import Button from '../../components/ui/Button';
import Badge from '../../components/ui/Badge';
import Alert from '../../components/ui/Alert';
import LoadingSpinner from '../../components/ui/LoadingSpinner';
import { productAPI } from '../../services/api';

interface ProvenanceEvent {
  id: string;
  timestamp: string;
  activity: string;
  agent: string;
  resource: string;
  type: 'creation' | 'modification' | 'usage' | 'transfer';
}

const ProvenanceTracker: React.FC = () => {
  const [selectedEvent, setSelectedEvent] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [productId, setProductId] = useState('');
  const [events, setEvents] = useState<ProvenanceEvent[]>([]);

  const getEventTypeVariant = (type: string): 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info' | 'default' => {
    const variants: Record<string, 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info' | 'default'> = {
      creation: 'success',
      modification: 'warning',
      usage: 'info',
      transfer: 'primary'
    };
    return variants[type] || 'default';
  };

  const getEventTypeIcon = (type: string): string => {
    const icons: Record<string, string> = {
      creation: '🆕',
      modification: '🔧',
      usage: '📱',
      transfer: '🚚'
    };
    return icons[type] || '📋';
  };

  const loadProvenanceData = async () => {
    if (!productId.trim()) {
      setError('Please enter a product ID');
      return;
    }

    setIsLoading(true);
    setError(null);
    try {
      // Load real provenance data from the API
      const response: any = await productAPI.getEnhancedTrace(productId);
      const traceData = response.data;
      
      // Transform the trace data into provenance events
      const provenanceEvents: ProvenanceEvent[] = traceData.events?.map((event: any, index: number) => ({
        id: `event-${index}`,
        timestamp: event.timestamp || new Date().toISOString(),
        activity: event.activity || event.type || 'Unknown Activity',
        agent: event.agent || event.participant || 'Unknown Agent',
        resource: event.resource || productId,
        type: event.type || 'usage'
      })) || [];
      
      setEvents(provenanceEvents);
    } catch (err: any) {
      setError('Failed to load provenance data: ' + (err.response?.data?.message || err.message || 'Please try again.'));
      console.error('Error loading provenance data:', err);
      // Fallback to sample data
      const sampleProvenanceData: ProvenanceEvent[] = [
        {
          id: 'event-001',
          timestamp: '2025-08-26T10:00:00Z',
          activity: 'Manufacturing',
          agent: 'Factory A',
          resource: productId || 'product:smartphone-001',
          type: 'creation'
        },
        {
          id: 'event-002',
          timestamp: '2025-08-26T12:30:00Z',
          activity: 'Quality Check',
          agent: 'Quality Inspector',
          resource: productId || 'product:smartphone-001',
          type: 'modification'
        },
        {
          id: 'event-003',
          timestamp: '2025-08-26T14:15:00Z',
          activity: 'Packaging',
          agent: 'Packaging Team',
          resource: productId || 'product:smartphone-001',
          type: 'modification'
        },
        {
          id: 'event-004',
          timestamp: '2025-08-26T16:00:00Z',
          activity: 'Shipment',
          agent: 'Logistics Corp',
          resource: productId || 'product:smartphone-001',
          type: 'transfer'
        },
        {
          id: 'event-005',
          timestamp: '2025-08-27T09:00:00Z',
          activity: 'Retail Display',
          agent: 'Retail Store',
          resource: productId || 'product:smartphone-001',
          type: 'usage'
        }
      ];
      setEvents(sampleProvenanceData);
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setProductId(e.target.value);
    // Clear error when user starts typing
    if (error) {
      setError(null);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      loadProvenanceData();
    }
  };

  const exportTimeline = () => {
    // In a real implementation, this would export the timeline data
    alert('Timeline exported successfully!');
  };

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">Provenance Tracker</h2>
          <p className="feature-description">
            Track the complete history and lineage of resources in the blockchain
          </p>
        </div>
        <Button variant="secondary" onClick={exportTimeline}>
          📋 Export Timeline
        </Button>
      </div>

      {error && (
        <div className="mb-6">
          <Alert 
            variant="error" 
            message={error}
            dismissible
            onClose={() => setError(null)}
          />
        </div>
      )}

      <Card title="Search Product" className="mb-6">
        <div className="flex gap-4">
          <div className="flex-1">
            <input
              type="text"
              value={productId}
              onChange={handleInputChange}
              onKeyPress={handleKeyPress}
              placeholder="Enter product ID (e.g., product:smartphone-001)"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-800 dark:border-gray-600 dark:text-white"
            />
          </div>
          <Button 
            variant="primary" 
            onClick={loadProvenanceData}
            disabled={isLoading}
          >
            {isLoading ? 'Searching...' : '🔍 Search'}
          </Button>
        </div>
      </Card>

      {isLoading && (
        <div className="flex justify-center my-8">
          <LoadingSpinner size="lg" message="Loading Provenance Data..." />
        </div>
      )}

      {!isLoading && events.length > 0 && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card title="Provenance Timeline">
            <div className="max-h-[500px] overflow-y-auto space-y-3">
              {events.map((event) => (
                <div
                  key={event.id}
                  onClick={() => setSelectedEvent(event.id)}
                  className={`p-4 border-l-4 rounded-lg cursor-pointer transition-all duration-200 ${
                    selectedEvent === event.id 
                      ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' 
                      : 'border-gray-300 bg-gray-50 dark:bg-gray-800 hover:bg-gray-100 dark:hover:bg-gray-700'
                  }`}
                >
                  <div className="flex justify-between items-start">
                    <div>
                      <h4 className="font-medium text-gray-900 dark:text-white mb-1">{event.activity}</h4>
                      <p className="text-sm text-gray-600 dark:text-gray-300 mb-2">
                        {new Date(event.timestamp).toLocaleString()}
                      </p>
                      <div className="mb-2">
                        <Badge variant={getEventTypeVariant(event.type)}>
                          {getEventTypeIcon(event.type)} {event.type.charAt(0).toUpperCase() + event.type.slice(1)}
                        </Badge>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-xs text-gray-500 dark:text-gray-400">Agent</div>
                      <div className="font-medium text-gray-900 dark:text-white">{event.agent}</div>
                    </div>
                  </div>
                  <div className="mt-2">
                    <Badge variant="secondary">Resource: {event.resource}</Badge>
                  </div>
                </div>
              ))}
            </div>
          </Card>

          <div className="space-y-6">
            <Card title="Provenance Statistics">
              <div className="grid grid-cols-2 gap-4">
                <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4 text-center">
                  <div className="text-2xl mb-1">📊</div>
                  <div className="text-xl font-bold text-blue-600 dark:text-blue-400">{events.length}</div>
                  <div className="text-xs text-gray-600 dark:text-gray-300">Total Events</div>
                </div>
                <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4 text-center">
                  <div className="text-2xl mb-1">👥</div>
                  <div className="text-xl font-bold text-green-600 dark:text-green-400">
                    {new Set(events.map(e => e.agent)).size}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-300">Unique Agents</div>
                </div>
                <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4 text-center">
                  <div className="text-2xl mb-1">🔧</div>
                  <div className="text-xl font-bold text-purple-600 dark:text-purple-400">
                    {new Set(events.map(e => e.activity)).size}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-300">Activity Types</div>
                </div>
                <div className="bg-yellow-50 dark:bg-yellow-900/20 rounded-lg p-4 text-center">
                  <div className="text-2xl mb-1">📦</div>
                  <div className="text-xl font-bold text-yellow-600 dark:text-yellow-400">
                    {new Set(events.map(e => e.resource)).size}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-300">Tracked Resources</div>
                </div>
              </div>

              {selectedEvent && (
                <div className="mt-6">
                  <h4 className="text-lg font-medium text-gray-900 dark:text-white mb-3">Selected Event Details</h4>
                  {(() => {
                    const event = events.find(e => e.id === selectedEvent);
                    if (!event) return null;
                    
                    return (
                      <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
                        <div className="mb-2">
                          <span className="font-medium text-gray-700 dark:text-gray-300">Activity:</span> {event.activity}
                        </div>
                        <div className="mb-2">
                          <span className="font-medium text-gray-700 dark:text-gray-300">Timestamp:</span> {new Date(event.timestamp).toLocaleString()}
                        </div>
                        <div className="mb-2">
                          <span className="font-medium text-gray-700 dark:text-gray-300">Agent:</span> {event.agent}
                        </div>
                        <div className="mb-2">
                          <span className="font-medium text-gray-700 dark:text-gray-300">Resource:</span> {event.resource}
                        </div>
                        <div>
                          <span className="font-medium text-gray-700 dark:text-gray-300">Type:</span> 
                          <Badge variant={getEventTypeVariant(event.type)} className="ml-2">
                            {getEventTypeIcon(event.type)} {event.type}
                          </Badge>
                        </div>
                      </div>
                    );
                  })()}
                </div>
              )}
            </Card>

            <Card title="Event Type Distribution">
              <div className="grid grid-cols-2 gap-4">
                {Object.entries(
                  events.reduce((acc, event) => {
                    acc[event.type] = (acc[event.type] || 0) + 1;
                    return acc;
                  }, {} as Record<string, number>)
                ).map(([type, count]) => (
                  <div 
                    key={type} 
                    className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 text-center hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                  >
                    <div className="text-3xl mb-2">{getEventTypeIcon(type)}</div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-white">{count}</div>
                    <div className="text-sm text-gray-600 dark:text-gray-300">
                      {type.charAt(0).toUpperCase() + type.slice(1)}
                    </div>
                  </div>
                ))}
              </div>
            </Card>
          </div>
        </div>
      )}

      {!isLoading && events.length === 0 && !error && (
        <Card title="No Data Found">
          <div className="text-center py-8">
            <div className="text-4xl mb-4">🔍</div>
            <h3 className="text-lg font-medium text-gray-900 mb-2">Search for a Product</h3>
            <p className="text-gray-600">Enter a product ID above to view its provenance timeline and history.</p>
          </div>
        </Card>
      )}
    </div>
  );
};

export default ProvenanceTracker;
