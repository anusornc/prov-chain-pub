// Core blockchain types
export interface Block {
  index: number;
  timestamp: string;
  hash: string;
  previous_hash: string;
  rdf_data: string;
  transaction_count: number;
  size: number;
  validator?: string;
}

export interface Transaction {
  id: string;
  type: TransactionType;
  from: string;
  to?: string;
  timestamp: string;
  block_index: number;
  signature: string;
  data: Record<string, unknown>;
  status: 'pending' | 'confirmed' | 'failed';
  gas_used?: number;
  gas_price?: number;
}

export type TransactionType = 
  | 'Production'
  | 'Processing'
  | 'Transport'
  | 'Quality'
  | 'Transfer'
  | 'Environmental'
  | 'Compliance'
  | 'Governance';

// Traceability types
export interface TraceabilityItem {
  id: string;
  name: string;
  type: string;
  current_owner: string;
  created_at: string;
  location?: string;
  properties: Record<string, unknown>;
  relationships: TraceabilityRelationship[];
}

export interface TraceabilityRelationship {
  type: 'produced_from' | 'processed_into' | 'transported_by' | 'quality_tested' | 'transferred_to';
  target_item: string;
  timestamp: string;
  transaction_id: string;
  metadata?: Record<string, unknown>;
}

export interface TraceStep {
  step_number: number;
  timestamp: string;
  transaction_id: string;
  action: string;
  participant: string;
  location?: string;
  metadata: Record<string, unknown>;
}

// Knowledge Graph types
export interface KnowledgeGraphNode {
  id: string;
  label: string;
  type: 'item' | 'participant' | 'location' | 'process';
  properties: Record<string, unknown>;
  x?: number;
  y?: number;
  size?: number;
  color?: string;
}

export interface KnowledgeGraphEdge {
  id: string;
  source: string;
  target: string;
  type: string;
  label: string;
  properties: Record<string, unknown>;
  weight?: number;
  color?: string;
}

export interface KnowledgeGraph {
  nodes: KnowledgeGraphNode[];
  edges: KnowledgeGraphEdge[];
  metadata: GraphMetadata;
}

export interface GraphMetadata {
  total_nodes: number;
  total_edges: number;
  node_types: Record<string, number>;
  edge_types: Record<string, number>;
  created_at: string;
  query_time_ms: number;
}

// Search and filtering types
export interface SearchFilters {
  type?: string;
  dateRange?: [Date, Date];
  participant?: string;
  location?: string;
  status?: string;
  minAmount?: number;
  maxAmount?: number;
}

export interface SearchQuery {
  query: string;
  filters: SearchFilters;
  page: number;
  limit: number;
  sort_by?: string;
  sort_order?: 'asc' | 'desc';
}

export interface SearchResults<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  has_more: boolean;
}

// Dashboard and metrics types
export interface DashboardMetrics {
  total_blocks: number;
  total_transactions: number;
  total_items: number;
  active_participants: number;
  network_status: 'healthy' | 'warning' | 'error';
  last_block_time: string;
  avg_block_time: number;
  transactions_per_second: number;
  network_hash_rate?: number;
}

export interface NetworkHealth {
  status: 'healthy' | 'warning' | 'error';
  uptime: number;
  peer_count: number;
  sync_status: 'synced' | 'syncing' | 'behind';
  last_block_age: number;
  validation_errors: number;
}

// API Response types
export interface BlockExplorerResponse {
  block: Block;
  transactions: Transaction[];
  rdf_summary: RdfSummary;
  validation_status: ValidationStatus;
}

export interface TraceabilityResponse {
  item: TraceabilityItem;
  trace_path: TraceStep[];
  knowledge_graph: KnowledgeGraph;
  related_transactions: Transaction[];
}

export interface RdfSummary {
  triple_count: number;
  subject_count: number;
  predicate_count: number;
  object_count: number;
  namespaces: string[];
}

export interface ValidationStatus {
  is_valid: boolean;
  validation_time_ms: number;
  errors: string[];
  warnings: string[];
}

// WebSocket types
export interface WebSocketMessage {
  message_type: MessageType;
  data: unknown;
  timestamp: string;
}

export type MessageType = 
  | 'NewBlock'
  | 'NewTransaction'
  | 'ItemUpdate'
  | 'NetworkStatus'
  | 'ValidationAlert';

// UI Component types
export interface TabConfig {
  id: string;
  label: string;
  icon: string;
  component: React.ComponentType;
  badge?: number;
}

export interface NavigationItem {
  id: string;
  label: string;
  icon: string;
  path: string;
  children?: NavigationItem[];
  badge?: number;
}

export interface ChartDataPoint {
  timestamp: string;
  value: number;
  label?: string;
}

export interface TimeSeriesData {
  name: string;
  data: ChartDataPoint[];
  color?: string;
}

// Form types
export interface FormField {
  name: string;
  label: string;
  type: 'text' | 'number' | 'select' | 'date' | 'textarea' | 'checkbox';
  required?: boolean;
  options?: { value: string; label: string }[];
  validation?: {
    min?: number;
    max?: number;
    pattern?: string;
    message?: string;
  };
}

// Participant types
export interface Participant {
  id: string;
  name: string;
  type: ParticipantType;
  address: string;
  public_key: string;
  permissions: Permission[];
  created_at: string;
  last_active: string;
  status: 'active' | 'inactive' | 'suspended';
}

export type ParticipantType = 
  | 'Producer'
  | 'Manufacturer'
  | 'LogisticsProvider'
  | 'QualityLab'
  | 'Auditor'
  | 'Retailer'
  | 'Administrator';

export type Permission = 
  | 'read'
  | 'write'
  | 'validate'
  | 'admin'
  | 'audit';

// SPARQL types
export interface SPARQLQuery {
  id?: string;
  name?: string;
  query: string;
  description?: string;
  created_at?: string;
  created_by?: string;
  is_favorite?: boolean;
  execution_time_ms?: number;
}

export interface SPARQLResult {
  head: {
    vars: string[];
  };
  results: {
    bindings: Record<string, {
      type: string;
      value: string;
      datatype?: string;
      'xml:lang'?: string;
    }>[];
  };
}

// Error types
export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
  timestamp: string;
}

// Pagination types
export interface PaginationInfo {
  current_page: number;
  total_pages: number;
  total_items: number;
  items_per_page: number;
  has_previous: boolean;
  has_next: boolean;
}

// Theme types
export interface ThemeConfig {
  mode: 'light' | 'dark' | 'system';
  primaryColor: string;
  accentColor: string;
  fontSize: 'small' | 'medium' | 'large';
  compactMode: boolean;
}
