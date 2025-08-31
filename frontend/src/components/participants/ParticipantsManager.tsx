import React, { useState, useEffect } from 'react';
import { Users, Plus, Search, Filter, Edit, Trash2, Shield, Key, Calendar, Clock, X } from 'lucide-react';
import LoadingSpinner from '../ui/LoadingSpinner';
import Card from '../ui/Card';
import Badge from '../ui/Badge';
import Button from '../ui/Button';
import Input from '../ui/Input';
import type { Participant, ParticipantType, Permission } from '../../types';

interface ParticipantFilters {
  searchQuery: string;
  type: ParticipantType | 'all';
  status: 'active' | 'inactive' | 'suspended' | 'all';
  permissions: Permission[];
}

interface NewParticipant {
  name: string;
  type: ParticipantType;
  address: string;
  email: string;
  phone: string;
  organization: string;
  location: string;
  permissions: Permission[];
}

const ParticipantsManager: React.FC = () => {
  const [participants, setParticipants] = useState<Participant[]>([]);
  const [filteredParticipants, setFilteredParticipants] = useState<Participant[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [filters, setFilters] = useState<ParticipantFilters>({
    searchQuery: '',
    type: 'all',
    status: 'all',
    permissions: []
  });
  const [isFiltersOpen, setIsFiltersOpen] = useState(false);
  const [selectedParticipant, setSelectedParticipant] = useState<Participant | null>(null);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [newParticipant, setNewParticipant] = useState<NewParticipant>({
    name: '',
    type: 'Producer',
    address: '',
    email: '',
    phone: '',
    organization: '',
    location: '',
    permissions: ['read']
  });

  const participantTypes: ParticipantType[] = [
    'Producer',
    'Manufacturer',
    'LogisticsProvider',
    'QualityLab',
    'Auditor',
    'Retailer',
    'Administrator'
  ];

  const allPermissions: Permission[] = ['read', 'write', 'validate', 'admin', 'audit'];

  useEffect(() => {
    loadParticipants();
  }, []);

  useEffect(() => {
    applyFilters();
  }, [participants, filters]);

  const loadParticipants = async () => {
    setIsLoading(true);
    try {
      const token = localStorage.getItem('authToken');
      
      // Try to load participants from API
      try {
        const response = await fetch('http://localhost:8080/api/participants', {
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
          }
        });

        if (response.ok) {
          const data: Participant[] = await response.json();
          setParticipants(data);
        } else {
          // Fallback to mock data if API not available
          loadMockParticipants();
        }
      } catch (error) {
        console.warn('API not available, using mock data:', error);
        loadMockParticipants();
      }
    } catch (error) {
      console.error('Error loading participants:', error);
      loadMockParticipants();
    } finally {
      setIsLoading(false);
    }
  };

  const loadMockParticipants = () => {
    const mockParticipants: Participant[] = [
      {
        id: 'participant-1',
        name: 'Organic Farms Co.',
        type: 'Producer',
        address: '0x1234567890abcdef1234567890abcdef12345678',
        public_key: 'ed25519:ABC123...',
        permissions: ['read', 'write'],
        created_at: '2025-01-15T10:00:00Z',
        last_active: '2025-08-31T08:30:00Z',
        status: 'active'
      },
      {
        id: 'participant-2',
        name: 'Global Manufacturing Ltd.',
        type: 'Manufacturer',
        address: '0x2345678901bcdef12345678901bcdef123456789',
        public_key: 'ed25519:DEF456...',
        permissions: ['read', 'write', 'validate'],
        created_at: '2025-01-20T14:30:00Z',
        last_active: '2025-08-31T07:45:00Z',
        status: 'active'
      },
      {
        id: 'participant-3',
        name: 'Swift Logistics',
        type: 'LogisticsProvider',
        address: '0x3456789012cdef123456789012cdef1234567890',
        public_key: 'ed25519:GHI789...',
        permissions: ['read', 'write'],
        created_at: '2025-02-01T09:15:00Z',
        last_active: '2025-08-30T16:20:00Z',
        status: 'active'
      },
      {
        id: 'participant-4',
        name: 'Quality Assurance Labs',
        type: 'QualityLab',
        address: '0x456789013def1234567890123def12345678901',
        public_key: 'ed25519:JKL012...',
        permissions: ['read', 'write', 'validate', 'audit'],
        created_at: '2025-02-10T11:45:00Z',
        last_active: '2025-08-31T09:10:00Z',
        status: 'active'
      },
      {
        id: 'participant-5',
        name: 'Independent Auditors',
        type: 'Auditor',
        address: '0x56789014ef123456789014ef123456789012345',
        public_key: 'ed25519:MNO345...',
        permissions: ['read', 'audit', 'validate'],
        created_at: '2025-02-15T13:20:00Z',
        last_active: '2025-08-29T14:30:00Z',
        status: 'inactive'
      },
      {
        id: 'participant-6',
        name: 'Retail Chain Corp',
        type: 'Retailer',
        address: '0x6789015f12345678905f123456789012345678',
        public_key: 'ed25519:PQR678...',
        permissions: ['read'],
        created_at: '2025-03-01T08:00:00Z',
        last_active: '2025-08-31T06:15:00Z',
        status: 'active'
      },
      {
        id: 'participant-7',
        name: 'System Administrator',
        type: 'Administrator',
        address: '0x789016123456789016123456789012345678901',
        public_key: 'ed25519:STU901...',
        permissions: ['read', 'write', 'validate', 'admin', 'audit'],
        created_at: '2025-01-01T00:00:00Z',
        last_active: '2025-08-31T09:00:00Z',
        status: 'active'
      }
    ];
    setParticipants(mockParticipants);
  };

  const applyFilters = () => {
    let filtered = [...participants];

    // Apply search query
    if (filters.searchQuery) {
      const query = filters.searchQuery.toLowerCase();
      filtered = filtered.filter(participant =>
        participant.name.toLowerCase().includes(query) ||
        participant.type.toLowerCase().includes(query) ||
        participant.address.toLowerCase().includes(query)
      );
    }

    // Apply type filter
    if (filters.type !== 'all') {
      filtered = filtered.filter(participant => participant.type === filters.type);
    }

    // Apply status filter
    if (filters.status !== 'all') {
      filtered = filtered.filter(participant => participant.status === filters.status);
    }

    // Apply permissions filter
    if (filters.permissions.length > 0) {
      filtered = filtered.filter(participant =>
        filters.permissions.every(permission => participant.permissions.includes(permission))
      );
    }

    setFilteredParticipants(filtered);
  };

  const handleFilterChange = (key: keyof ParticipantFilters, value: string | Permission[]) => {
    setFilters(prev => ({ ...prev, [key]: value }));
  };

  const clearFilters = () => {
    setFilters({
      searchQuery: '',
      type: 'all',
      status: 'all',
      permissions: []
    });
  };

  const getStatusColor = (status: string): 'default' | 'success' | 'warning' | 'primary' | 'secondary' | 'danger' | 'info' => {
    switch (status) {
      case 'active': return 'success';
      case 'inactive': return 'warning';
      case 'suspended': return 'danger';
      default: return 'default';
    }
  };

  const getTypeColor = (type: ParticipantType): 'default' | 'success' | 'warning' | 'primary' | 'secondary' | 'danger' | 'info' => {
    switch (type) {
      case 'Administrator': return 'danger';
      case 'Auditor': return 'info';
      case 'QualityLab': return 'warning';
      case 'Manufacturer': return 'primary';
      case 'Producer': return 'success';
      default: return 'secondary';
    }
  };

  const getPermissionColor = (permission: Permission): 'default' | 'success' | 'warning' | 'primary' | 'secondary' | 'danger' | 'info' => {
    switch (permission) {
      case 'admin': return 'danger';
      case 'audit': return 'info';
      case 'validate': return 'warning';
      case 'write': return 'primary';
      case 'read': return 'success';
      default: return 'default';
    }
  };

  const formatLastActive = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffHours < 1) return 'Just now';
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const handleAddParticipant = async () => {
    try {
      const token = localStorage.getItem('authToken');
      const participantData = {
        ...newParticipant,
        id: `participant-${Date.now()}`,
        public_key: `ed25519:${Math.random().toString(36).substring(2, 15)}...`,
        created_at: new Date().toISOString(),
        last_active: new Date().toISOString(),
        status: 'active' as const
      };

      // Try to add via API
      try {
        const response = await fetch('http://localhost:8080/api/participants', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify(participantData)
        });

        if (response.ok) {
          const newParticipantData: Participant = await response.json();
          setParticipants(prev => [...prev, newParticipantData]);
        } else {
          // Fallback to local addition
          setParticipants(prev => [...prev, participantData]);
        }
      } catch (error) {
        console.warn('API not available, adding locally:', error);
        setParticipants(prev => [...prev, participantData]);
      }

      // Reset form and close modal
      setNewParticipant({
        name: '',
        type: 'Producer',
        address: '',
        email: '',
        phone: '',
        organization: '',
        location: '',
        permissions: ['read']
      });
      setIsAddModalOpen(false);
    } catch (error) {
      console.error('Error adding participant:', error);
    }
  };

  const handleDeleteParticipant = async (participantId: string) => {
    if (!confirm('Are you sure you want to delete this participant?')) return;

    try {
      const token = localStorage.getItem('authToken');
      
      // Try to delete via API
      try {
        const response = await fetch(`http://localhost:8080/api/participants/${participantId}`, {
          method: 'DELETE',
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
          }
        });

        if (response.ok) {
          setParticipants(prev => prev.filter(p => p.id !== participantId));
        } else {
          // Fallback to local deletion
          setParticipants(prev => prev.filter(p => p.id !== participantId));
        }
      } catch (error) {
        console.warn('API not available, deleting locally:', error);
        setParticipants(prev => prev.filter(p => p.id !== participantId));
      }
    } catch (error) {
      console.error('Error deleting participant:', error);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                Participants Management
              </h1>
              <p className="text-gray-600 dark:text-gray-300">
                Manage blockchain participants, permissions, and access control
              </p>
            </div>
            <Button
              onClick={() => setIsAddModalOpen(true)}
              className="flex items-center gap-2"
            >
              <Plus className="w-4 h-4" />
              Add Participant
            </Button>
          </div>
        </div>

        {/* Filters */}
        <Card className="mb-6">
          <div className="flex flex-col lg:flex-row gap-4">
            {/* Search */}
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <Input
                  type="text"
                  placeholder="Search participants..."
                  value={filters.searchQuery}
                  onChange={(e) => handleFilterChange('searchQuery', e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>

            {/* Quick Filters */}
            <div className="flex gap-2">
              <select
                value={filters.type}
                onChange={(e) => handleFilterChange('type', e.target.value as ParticipantType | 'all')}
                className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                <option value="all">All Types</option>
                {participantTypes.map(type => (
                  <option key={type} value={type}>{type}</option>
                ))}
              </select>

              <select
                value={filters.status}
                onChange={(e) => handleFilterChange('status', e.target.value as 'active' | 'inactive' | 'suspended' | 'all')}
                className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                <option value="all">All Status</option>
                <option value="active">Active</option>
                <option value="inactive">Inactive</option>
                <option value="suspended">Suspended</option>
              </select>

              <Button
                variant="outline"
                onClick={() => setIsFiltersOpen(!isFiltersOpen)}
                className="flex items-center gap-2"
              >
                <Filter className="w-4 h-4" />
                Filters
                {filters.permissions.length > 0 && (
                  <Badge variant="primary" className="ml-1">
                    {filters.permissions.length}
                  </Badge>
                )}
              </Button>
            </div>
          </div>

          {/* Advanced Filters */}
          {isFiltersOpen && (
            <div className="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Required Permissions
                </label>
                <div className="flex flex-wrap gap-2">
                  {allPermissions.map(permission => (
                    <label key={permission} className="flex items-center">
                      <input
                        type="checkbox"
                        checked={filters.permissions.includes(permission)}
                        onChange={(e) => {
                          if (e.target.checked) {
                            handleFilterChange('permissions', [...filters.permissions, permission]);
                          } else {
                            handleFilterChange('permissions', filters.permissions.filter(p => p !== permission));
                          }
                        }}
                        className="mr-2"
                      />
                      <Badge variant={getPermissionColor(permission)} className="capitalize">
                        {permission}
                      </Badge>
                    </label>
                  ))}
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

        {/* Participants List */}
        {isLoading ? (
          <div className="flex justify-center py-12">
            <LoadingSpinner size="lg" message="Loading participants..." />
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredParticipants.map(participant => (
              <Card key={participant.id} className="hover:shadow-md transition-shadow">
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center gap-3">
                    <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                      <Users className="w-6 h-6 text-white" />
                    </div>
                    <div>
                      <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {participant.name}
                      </h3>
                      <div className="flex items-center gap-2 mt-1">
                        <Badge variant={getTypeColor(participant.type)}>
                          {participant.type}
                        </Badge>
                        <Badge variant={getStatusColor(participant.status)}>
                          {participant.status}
                        </Badge>
                      </div>
                    </div>
                  </div>
                  <div className="flex gap-1">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setSelectedParticipant(participant);
                        setIsEditModalOpen(true);
                      }}
                    >
                      <Edit className="w-4 h-4" />
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDeleteParticipant(participant.id)}
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>

                <div className="space-y-3">
                  <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
                    <Key className="w-4 h-4" />
                    <span className="font-mono text-xs">
                      {participant.address.substring(0, 20)}...
                    </span>
                  </div>

                  <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
                    <Clock className="w-4 h-4" />
                    <span>Last active: {formatLastActive(participant.last_active)}</span>
                  </div>

                  <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
                    <Calendar className="w-4 h-4" />
                    <span>Joined: {new Date(participant.created_at).toLocaleDateString()}</span>
                  </div>

                  <div>
                    <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300 mb-2">
                      <Shield className="w-4 h-4" />
                      <span>Permissions:</span>
                    </div>
                    <div className="flex flex-wrap gap-1">
                      {participant.permissions.map(permission => (
                        <Badge key={permission} variant={getPermissionColor(permission)} size="sm">
                          {permission}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setSelectedParticipant(participant)}
                    className="w-full"
                  >
                    View Details
                  </Button>
                </div>
              </Card>
            ))}

            {/* Empty State */}
            {filteredParticipants.length === 0 && (
              <div className="col-span-full text-center py-12">
                <Users className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                  No participants found
                </h3>
                <p className="text-gray-600 dark:text-gray-300 mb-4">
                  {filters.searchQuery || filters.type !== 'all' || filters.status !== 'all' || filters.permissions.length > 0
                    ? 'Try adjusting your filters'
                    : 'Get started by adding your first participant'
                  }
                </p>
                {filters.searchQuery || filters.type !== 'all' || filters.status !== 'all' || filters.permissions.length > 0 ? (
                  <Button variant="outline" onClick={clearFilters}>
                    Clear Filters
                  </Button>
                ) : (
                  <Button onClick={() => setIsAddModalOpen(true)}>
                    <Plus className="w-4 h-4 mr-2" />
                    Add Participant
                  </Button>
                )}
              </div>
            )}
          </div>
        )}

        {/* Add Participant Modal */}
        {isAddModalOpen && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
            <Card className="max-w-2xl w-full max-h-[80vh] overflow-y-auto">
              <div className="flex items-start justify-between mb-6">
                <div>
                  <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
                    Add New Participant
                  </h3>
                  <p className="text-gray-600 dark:text-gray-300">
                    Create a new participant with appropriate permissions
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setIsAddModalOpen(false)}
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>

              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Name *
                    </label>
                    <Input
                      type="text"
                      value={newParticipant.name}
                      onChange={(e) => setNewParticipant(prev => ({ ...prev, name: e.target.value }))}
                      placeholder="Organization name"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Type *
                    </label>
                    <select
                      value={newParticipant.type}
                      onChange={(e) => setNewParticipant(prev => ({ ...prev, type: e.target.value as ParticipantType }))}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                    >
                      {participantTypes.map(type => (
                        <option key={type} value={type}>{type}</option>
                      ))}
                    </select>
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Blockchain Address *
                  </label>
                  <Input
                    type="text"
                    value={newParticipant.address}
                    onChange={(e) => setNewParticipant(prev => ({ ...prev, address: e.target.value }))}
                    placeholder="0x..."
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Email
                    </label>
                    <Input
                      type="email"
                      value={newParticipant.email}
                      onChange={(e) => setNewParticipant(prev => ({ ...prev, email: e.target.value }))}
                      placeholder="contact@example.com"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Phone
                    </label>
                    <Input
                      type="tel"
                      value={newParticipant.phone}
                      onChange={(e) => setNewParticipant(prev => ({ ...prev, phone: e.target.value }))}
                      placeholder="+1 (555) 123-4567"
                    />
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Organization
                    </label>
                    <Input
                      type="text"
                      value={newParticipant.organization}
                      onChange={(e) => setNewParticipant(prev => ({ ...prev, organization: e.target.value }))}
                      placeholder="Company name"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Location
                    </label>
                    <Input
                      type="text"
                      value={newParticipant.location}
                      onChange={(e) => setNewParticipant(prev => ({ ...prev, location: e.target.value }))}
                      placeholder="City, Country"
                    />
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Permissions *
                  </label>
                  <div className="flex flex-wrap gap-2">
                    {allPermissions.map(permission => (
                      <label key={permission} className="flex items-center">
                        <input
                          type="checkbox"
                          checked={newParticipant.permissions.includes(permission)}
                          onChange={(e) => {
                            if (e.target.checked) {
                              setNewParticipant(prev => ({
                                ...prev,
                                permissions: [...prev.permissions, permission]
                              }));
                            } else {
                              setNewParticipant(prev => ({
                                ...prev,
                                permissions: prev.permissions.filter(p => p !== permission)
                              }));
                            }
                          }}
                          className="mr-2"
                        />
                        <Badge variant={getPermissionColor(permission)} className="capitalize">
                          {permission}
                        </Badge>
                      </label>
                    ))}
                  </div>
                </div>
              </div>

              <div className="flex justify-end gap-2 mt-6">
                <Button
                  variant="outline"
                  onClick={() => setIsAddModalOpen(false)}
                >
                  Cancel
                </Button>
                <Button
                  onClick={handleAddParticipant}
                  disabled={!newParticipant.name || !newParticipant.address || newParticipant.permissions.length === 0}
                >
                  Add Participant
                </Button>
              </div>
            </Card>
          </div>
        )}

        {/* Participant Detail Modal */}
        {selectedParticipant && !isEditModalOpen && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
            <Card className="max-w-2xl w-full max-h-[80vh] overflow-y-auto">
              <div className="flex items-start justify-between mb-6">
                <div>
                  <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
                    {selectedParticipant.name}
                  </h3>
                  <div className="flex items-center gap-2">
                    <Badge variant={getTypeColor(selectedParticipant.type)}>
                      {selectedParticipant.type}
                    </Badge>
                    <Badge variant={getStatusColor(selectedParticipant.status)}>
                      {selectedParticipant.status}
                    </Badge>
                  </div>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setSelectedParticipant(null)}
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>

              <div className="space-y-6">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Address</h4>
                    <p className="text-gray-600 dark:text-gray-300 font-mono text-sm break-all">
                      {selectedParticipant.address}
                    </p>
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Public Key</h4>
                    <p className="text-gray-600 dark:text-gray-300 font-mono text-sm">
                      {selectedParticipant.public_key}
                    </p>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Created</h4>
                    <p className="text-gray-600 dark:text-gray-300">
                      {new Date(selectedParticipant.created_at).toLocaleString()}
                    </p>
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Last Active</h4>
                    <p className="text-gray-600 dark:text-gray-300">
                      {formatLastActive(selectedParticipant.last_active)}
                    </p>
                  </div>
                </div>

                <div>
                  <h4 className="font-semibold text-gray-900 dark:text-white mb-2">Permissions</h4>
                  <div className="flex flex-wrap gap-2">
                    {selectedParticipant.permissions.map(permission => (
                      <Badge key={permission} variant={getPermissionColor(permission)}>
                        {permission}
                      </Badge>
                    ))}
                  </div>
                </div>

                <div className="flex justify-end gap-2">
                  <Button
                    variant="outline"
                    onClick={() => {
                      setIsEditModalOpen(true);
                    }}
                  >
                    <Edit className="w-4 h-4 mr-2" />
                    Edit
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => handleDeleteParticipant(selectedParticipant.id)}
                  >
                    <Trash2 className="w-4 h-4 mr-2" />
                    Delete
                  </Button>
                </div>
              </div>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
};

export default ParticipantsManager;
