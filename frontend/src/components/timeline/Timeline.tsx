import React, { useState, useEffect } from 'react';
import { Calendar, Clock, User, Package, Activity, Filter, Search, ZoomIn, ZoomOut, RotateCcw } from 'lucide-react';
import LoadingSpinner from '../ui/LoadingSpinner';
import Card from '../ui/Card';
import Badge from '../ui/Badge';
import Button from '../ui/Button';
import Input from '../ui/Input';
import type { TraceabilityItem, Transaction, Block } from '../../types';

interface TimelineEvent {
  id: string;
  timestamp: string;
  type: 'block' | 'transaction' | 'traceability' | 'quality' | 'transport' | 'production';
  title: string;
  description: string;
  participant?: string;
  location?: string;
  status: 'completed' | 'pending' | 'failed' | 'active';
  data: Block | Transaction | TraceabilityItem | Record<string, unknown>;
  relatedEvents?: string[];
}

interface TimelineFilters {
  dateRange: {
    start: string;
    end: string;
  };
  eventTypes: string[];
  participants: string[];
  locations: string[];
  searchQuery: string;
}

const Timeline: React.FC = () => {
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [filteredEvents, setFilteredEvents] = useState<TimelineEvent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [filters, setFilters] = useState<TimelineFilters>({
    dateRange: {
      start: '',
      end: ''
    },
    eventTypes: [],
    participants: [],
    locations: [],
    searchQuery: ''
  });
  const [isFiltersOpen, setIsFiltersOpen] = useState(false);
  const [selectedEvent, setSelectedEvent] = useState<TimelineEvent | null>(null);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [viewMode, setViewMode] = useState<'timeline' | 'gantt' | 'calendar'>('timeline');

  useEffect(() => {
    loadTimelineData();
  }, []);

  useEffect(() => {
    applyFilters();
  }, [events, filters]);

  const loadTimelineData = async () => {
    setIsLoading(true);
    try {
      const token = localStorage.getItem('authToken');
      const timelineEvents: TimelineEvent[] = [];

      // Load blocks
      try {
        const blocksResponse = await fetch('http://localhost:8080/api/blocks', {
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
          }
        });

        if (blocksResponse.ok) {
          const blocks: Block[] = await blocksResponse.json();
          const blockEvents = blocks.map(block => ({
            id: `block-${block.index}`,
            timestamp: block.timestamp,
            type: 'block' as const,
            title: `Block #${block.index} Created`,
            description: `New block added to blockchain with ${block.transaction_count} transactions`,
            participant: block.validator || 'System',
            status: 'completed' as const,
            data: block
          }));
          timelineEvents.push(...blockEvents);
        }
      } catch (error) {
        console.warn('Error loading blocks for timeline:', error);
      }

      // Load transactions
      try {
        const transactionsResponse = await fetch('http://localhost:8080/api/transactions', {
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
          }
        });

        if (transactionsResponse.ok) {
          const transactions: Transaction[] = await transactionsResponse.json();
          const transactionEvents = transactions.map(tx => ({
            id: `transaction-${tx.id}`,
            timestamp: tx.timestamp,
            type: tx.type.toLowerCase() as 'transaction' | 'production' | 'transport' | 'quality',
            title: `${tx.type} Transaction`,
            description: `Transaction from ${tx.from} to ${tx.to || 'N/A'}`,
            participant: tx.from,
            status: tx.status === 'confirmed' ? 'completed' as const : tx.status as 'pending' | 'failed',
            data: tx
          }));
          timelineEvents.push(...transactionEvents);
        }
      } catch (error) {
        console.warn('Error loading transactions for timeline:', error);
      }

      // Load traceability items
      try {
        const traceabilityResponse = await fetch('http://localhost:8080/api/traceability/items', {
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
          }
        });

        if (traceabilityResponse.ok) {
          const items: TraceabilityItem[] = await traceabilityResponse.json();
          const traceabilityEvents = items.map(item => ({
            id: `traceability-${item.id}`,
            timestamp: item.created_at,
            type: 'traceability' as const,
            title: `${item.name} Created`,
            description: `New ${item.type} item created`,
            participant: item.current_owner,
            location: item.location,
            status: 'active' as const,
            data: item
          }));
          timelineEvents.push(...traceabilityEvents);
        }
      } catch (error) {
        console.warn('Error loading traceability items for timeline:', error);
      }

      // Sort events by timestamp
      timelineEvents.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
      
      setEvents(timelineEvents);
    } catch (error) {
      console.error('Error loading timeline data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const applyFilters = () => {
    let filtered = [...events];

    // Apply search query
    if (filters.searchQuery) {
      const query = filters.searchQuery.toLowerCase();
      filtered = filtered.filter(event =>
        event.title.toLowerCase().includes(query) ||
        event.description.toLowerCase().includes(query) ||
        event.participant?.toLowerCase().includes(query) ||
        event.location?.toLowerCase().includes(query)
      );
    }

    // Apply date range
    if (filters.dateRange.start || filters.dateRange.end) {
      filtered = filtered.filter(event => {
        const eventDate = new Date(event.timestamp);
        const startDate = filters.dateRange.start ? new Date(filters.dateRange.start) : null;
        const endDate = filters.dateRange.end ? new Date(filters.dateRange.end) : null;

        if (startDate && eventDate < startDate) return false;
        if (endDate && eventDate > endDate) return false;
        return true;
      });
    }

    // Apply event type filters
    if (filters.eventTypes.length > 0) {
      filtered = filtered.filter(event => filters.eventTypes.includes(event.type));
    }

    // Apply participant filters
    if (filters.participants.length > 0) {
      filtered = filtered.filter(event => 
        event.participant && filters.participants.some(p => 
          event.participant!.toLowerCase().includes(p.toLowerCase())
        )
      );
    }

    // Apply location filters
    if (filters.locations.length > 0) {
      filtered = filtered.filter(event => 
        event.location && filters.locations.some(l => 
          event.location!.toLowerCase().includes(l.toLowerCase())
        )
      );
    }

    setFilteredEvents(filtered);
  };

  const handleFilterChange = (key: keyof TimelineFilters, value: string | string[] | { start: string; end: string }) => {
    setFilters(prev => ({ ...prev, [key]: value }));
  };

  const clearFilters = () => {
    setFilters({
      dateRange: { start: '', end: '' },
      eventTypes: [],
      participants: [],
      locations: [],
      searchQuery: ''
    });
  };

  const getEventIcon = (type: string) => {
    switch (type) {
      case 'block': return <Package className="w-4 h-4" />;
      case 'transaction': return <Activity className="w-4 h-4" />;
      case 'production': return <Package className="w-4 h-4" />;
      case 'transport': return <Activity className="w-4 h-4" />;
      case 'quality': return <Activity className="w-4 h-4" />;
      case 'traceability': return <Package className="w-4 h-4" />;
      default: return <Clock className="w-4 h-4" />;
    }
  };

  const getEventColor = (type: string, status: string) => {
    if (status === 'failed') return 'bg-red-500';
    if (status === 'pending') return 'bg-yellow-500';
    
    switch (type) {
      case 'block': return 'bg-blue-500';
      case 'transaction': return 'bg-green-500';
      case 'production': return 'bg-purple-500';
      case 'transport': return 'bg-orange-500';
      case 'quality': return 'bg-indigo-500';
      case 'traceability': return 'bg-teal-500';
      default: return 'bg-gray-500';
    }
  };

  const getStatusBadgeVariant = (status: string): 'default' | 'success' | 'warning' | 'primary' | 'secondary' | 'danger' | 'info' => {
    switch (status) {
      case 'completed': return 'success';
      case 'pending': return 'warning';
      case 'failed': return 'danger';
      case 'active': return 'primary';
      default: return 'default';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return {
      date: date.toLocaleDateString(),
      time: date.toLocaleTimeString(),
      relative: getRelativeTime(date)
    };
  };

  const getRelativeTime = (date: Date) => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const groupEventsByDate = (events: TimelineEvent[]) => {
    const groups: { [key: string]: TimelineEvent[] } = {};
    events.forEach(event => {
      const date = new Date(event.timestamp).toDateString();
      if (!groups[date]) groups[date] = [];
      groups[date].push(event);
    });
    return groups;
  };

  const eventGroups = groupEventsByDate(filteredEvents);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Traceability Timeline
          </h1>
          <p className="text-gray-600 dark:text-gray-300">
            Interactive timeline visualization of blockchain events and traceability activities
          </p>
        </div>

        {/* Controls */}
        <Card className="mb-6">
          <div className="flex flex-col lg:flex-row gap-4">
            {/* Search */}
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <Input
                  type="text"
                  placeholder="Search events, participants, locations..."
                  value={filters.searchQuery}
                  onChange={(e) => handleFilterChange('searchQuery', e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>

            {/* View Mode */}
            <div className="flex gap-2">
              <select
                value={viewMode}
                onChange={(e) => setViewMode(e.target.value as 'timeline' | 'gantt' | 'calendar')}
                className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                <option value="timeline">Timeline View</option>
                <option value="gantt">Gantt View</option>
                <option value="calendar">Calendar View</option>
              </select>

              <Button
                variant="outline"
                onClick={() => setIsFiltersOpen(!isFiltersOpen)}
                className="flex items-center gap-2"
              >
                <Filter className="w-4 h-4" />
                Filters
                {(filters.eventTypes.length > 0 || filters.participants.length > 0 || filters.locations.length > 0) && (
                  <Badge variant="primary" className="ml-1">
                    {filters.eventTypes.length + filters.participants.length + filters.locations.length}
                  </Badge>
                )}
              </Button>
            </div>

            {/* Zoom Controls */}
            <div className="flex gap-1">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setZoomLevel(prev => Math.min(3, prev + 0.5))}
              >
                <ZoomIn className="w-4 h-4" />
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setZoomLevel(prev => Math.max(0.5, prev - 0.5))}
              >
                <ZoomOut className="w-4 h-4" />
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setZoomLevel(1)}
              >
                <RotateCcw className="w-4 h-4" />
              </Button>
            </div>
          </div>

          {/* Advanced Filters */}
          {isFiltersOpen && (
            <div className="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {/* Date Range */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Date Range
                  </label>
                  <div className="flex gap-2">
                    <Input
                      type="date"
                      value={filters.dateRange.start}
                      onChange={(e) => handleFilterChange('dateRange', { ...filters.dateRange, start: e.target.value })}
                      className="text-sm"
                    />
                    <Input
                      type="date"
                      value={filters.dateRange.end}
                      onChange={(e) => handleFilterChange('dateRange', { ...filters.dateRange, end: e.target.value })}
                      className="text-sm"
                    />
                  </div>
                </div>

                {/* Event Types */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Event Types
                  </label>
                  <div className="space-y-1">
                    {['block', 'transaction', 'production', 'transport', 'quality', 'traceability'].map(type => (
                      <label key={type} className="flex items-center">
                        <input
                          type="checkbox"
                          checked={filters.eventTypes.includes(type)}
                          onChange={(e) => {
                            if (e.target.checked) {
                              handleFilterChange('eventTypes', [...filters.eventTypes, type]);
                            } else {
                              handleFilterChange('eventTypes', filters.eventTypes.filter(t => t !== type));
                            }
                          }}
                          className="mr-2"
                        />
                        <span className="text-sm capitalize">{type}</span>
                      </label>
                    ))}
                  </div>
                </div>

                {/* Participants */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Participants
                  </label>
                  <Input
                    type="text"
                    placeholder="Filter by participant"
                    onKeyPress={(e) => {
                      if (e.key === 'Enter') {
                        const value = (e.target as HTMLInputElement).value;
                        if (value && !filters.participants.includes(value)) {
                          handleFilterChange('participants', [...filters.participants, value]);
                          (e.target as HTMLInputElement).value = '';
                        }
                      }
                    }}
                  />
                  <div className="flex flex-wrap gap-1 mt-2">
                    {filters.participants.map(participant => (
                      <Badge key={participant} variant="primary" className="flex items-center gap-1">
                        {participant}
                        <button onClick={() => handleFilterChange('participants', filters.participants.filter(p => p !== participant))}>
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                </div>

                {/* Locations */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Locations
                  </label>
                  <Input
                    type="text"
                    placeholder="Filter by location"
                    onKeyPress={(e) => {
                      if (e.key === 'Enter') {
                        const value = (e.target as HTMLInputElement).value;
                        if (value && !filters.locations.includes(value)) {
                          handleFilterChange('locations', [...filters.locations, value]);
                          (e.target as HTMLInputElement).value = '';
                        }
                      }
                    }}
                  />
                  <div className="flex flex-wrap gap-1 mt-2">
                    {filters.locations.map(location => (
                      <Badge key={location} variant="primary" className="flex items-center gap-1">
                        {location}
                        <button onClick={() => handleFilterChange('locations', filters.locations.filter(l => l !== location))}>
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                </div>
              </div>

              <div className="mt-4 flex justify-end">
                <Button variant="outline" onClick={clearFilters}>
                  Clear All Filters
                </Button>
              </div>
            </div>
          )}
        </Card>

        {/* Timeline Content */}
        {isLoading ? (
          <div className="flex justify-center py-12">
            <LoadingSpinner size="lg" message="Loading timeline..." />
          </div>
        ) : (
          <div className="space-y-8" style={{ transform: `scale(${zoomLevel})`, transformOrigin: 'top left' }}>
            {Object.entries(eventGroups).map(([date, dayEvents]) => (
              <div key={date} className="relative">
                {/* Date Header */}
                <div className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-900 py-2 mb-4">
                  <div className="flex items-center gap-4">
                    <div className="flex items-center gap-2">
                      <Calendar className="w-5 h-5 text-gray-500" />
                      <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {new Date(date).toLocaleDateString('en-US', { 
                          weekday: 'long', 
                          year: 'numeric', 
                          month: 'long', 
                          day: 'numeric' 
                        })}
                      </h3>
                    </div>
                    <Badge variant="secondary">{dayEvents.length} events</Badge>
                  </div>
                </div>

                {/* Timeline Line */}
                <div className="absolute left-8 top-16 bottom-0 w-0.5 bg-gray-300 dark:bg-gray-600"></div>

                {/* Events */}
                <div className="space-y-6">
                  {dayEvents.map((event) => {
                    const timeInfo = formatTimestamp(event.timestamp);
                    return (
                      <div key={event.id} className="relative flex items-start gap-6">
                        {/* Timeline Dot */}
                        <div className={`relative z-10 flex items-center justify-center w-8 h-8 rounded-full ${getEventColor(event.type, event.status)}`}>
                          <div className="text-white">
                            {getEventIcon(event.type)}
                          </div>
                        </div>

                        {/* Event Card */}
                        <Card 
                          className="flex-1 hover:shadow-md transition-shadow cursor-pointer"
                          onClick={() => setSelectedEvent(event)}
                        >
                          <div className="flex items-start justify-between">
                            <div className="flex-1">
                              <div className="flex items-center gap-2 mb-2">
                                <h4 className="text-lg font-semibold text-gray-900 dark:text-white">
                                  {event.title}
                                </h4>
                                <Badge variant={getStatusBadgeVariant(event.status)}>
                                  {event.status}
                                </Badge>
                                <Badge variant="secondary" className="capitalize">
                                  {event.type}
                                </Badge>
                              </div>
                              
                              <p className="text-gray-600 dark:text-gray-300 mb-3">
                                {event.description}
                              </p>

                              <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
                                <div className="flex items-center gap-1">
                                  <Clock className="w-4 h-4" />
                                  {timeInfo.time}
                                </div>
                                {event.participant && (
                                  <div className="flex items-center gap-1">
                                    <User className="w-4 h-4" />
                                    {event.participant}
                                  </div>
                                )}
                                {event.location && (
                                  <div className="flex items-center gap-1">
                                    <Package className="w-4 h-4" />
                                    {event.location}
                                  </div>
                                )}
                              </div>
                            </div>

                            <div className="text-right text-sm text-gray-500 dark:text-gray-400">
                              <div>{timeInfo.relative}</div>
                              <div className="text-xs">{timeInfo.date}</div>
                            </div>
                          </div>
                        </Card>
                      </div>
                    );
                  })}
                </div>
              </div>
            ))}

            {/* Empty State */}
            {filteredEvents.length === 0 && (
              <div className="text-center py-12">
                <Calendar className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                  No events found
                </h3>
                <p className="text-gray-600 dark:text-gray-300 mb-4">
                  Try adjusting your filters or date range
                </p>
                <Button variant="outline" onClick={clearFilters}>
                  Clear Filters
                </Button>
              </div>
            )}
          </div>
        )}

        {/* Event Detail Modal */}
        {selectedEvent && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
            <Card className="max-w-2xl w-full max-h-[80vh] overflow-y-auto">
              <div className="flex items-start justify-between mb-4">
                <div>
                  <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
                    {selectedEvent.title}
                  </h3>
                  <div className="flex items-center gap-2">
                    <Badge variant={getStatusBadgeVariant(selectedEvent.status)}>
                      {selectedEvent.status}
                    </Badge>
                    <Badge variant="secondary" className="capitalize">
                      {selectedEvent.type}
                    </Badge>
                  </div>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setSelectedEvent(null)}
                >
                  ×
                </Button>
              </div>

              <div className="space-y-4">
                <div>
                  <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Description</h4>
                  <p className="text-gray-600 dark:text-gray-300">{selectedEvent.description}</p>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Timestamp</h4>
                    <p className="text-gray-600 dark:text-gray-300">
                      {new Date(selectedEvent.timestamp).toLocaleString()}
                    </p>
                  </div>
                  {selectedEvent.participant && (
                    <div>
                      <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Participant</h4>
                      <p className="text-gray-600 dark:text-gray-300">{selectedEvent.participant}</p>
                    </div>
                  )}
                  {selectedEvent.location && (
                    <div>
                      <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Location</h4>
                      <p className="text-gray-600 dark:text-gray-300">{selectedEvent.location}</p>
                    </div>
                  )}
                </div>

                <div>
                  <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Event Data</h4>
                  <pre className="bg-gray-100 dark:bg-gray-800 p-3 rounded-lg text-sm overflow-x-auto">
                    {JSON.stringify(selectedEvent.data, null, 2)}
                  </pre>
                </div>
              </div>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
};

export default Timeline;
