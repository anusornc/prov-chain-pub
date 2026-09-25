import type {
  KnowledgeGraph,
  KnowledgeGraphEdge,
  KnowledgeGraphNode,
  SearchResults,
  TraceStep,
  TraceabilityItem,
} from "./index";

export type TraceGraphConfidence = "known" | "inferred" | "unknown" | "missing-data";
export type TraceGraphCompleteness = "known" | "inferred" | "unknown" | "missing-data";
export type TraceGraphDirection = "upstream" | "downstream" | "related" | "unknown";

export interface BackendProductRecord {
  id?: string;
  name?: string;
  type?: string;
  status?: string;
  participant?: string;
  current_owner?: string;
  location?: string;
  timestamp?: string;
  created_at?: string;
  description?: string;
  properties?: Record<string, unknown>;
  relationships?: unknown[];
  trace_steps?: BackendTraceStepRecord[];
  [key: string]: unknown;
}

export interface BackendProductsResponse {
  items?: BackendProductRecord[];
  total?: number;
  total_count?: number;
  page?: number;
  limit?: number;
  total_pages?: number;
  has_more?: boolean;
  [key: string]: unknown;
}

export interface BackendTraceStepRecord {
  id?: string;
  step_number?: number;
  timestamp?: string;
  transaction_id?: string;
  action?: string;
  participant?: string;
  location?: string;
  status?: string;
  source?: string;
  metadata?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface BackendTraceResponse {
  product_id?: string;
  trace_steps?: BackendTraceStepRecord[];
  total_steps?: number;
  start_timestamp?: string;
  end_timestamp?: string;
  [key: string]: unknown;
}

export interface BackendKnowledgeGraphNode {
  id?: string;
  label?: string;
  type?: string;
  properties?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface BackendKnowledgeGraphEdge {
  id?: string;
  source?: string;
  target?: string;
  type?: string;
  label?: string;
  properties?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface BackendKnowledgeGraphMetadata {
  total_nodes?: number;
  total_edges?: number;
  node_types?: Record<string, number>;
  edge_types?: Record<string, number>;
  created_at?: string;
  query_timestamp?: string;
  query_time_ms?: number;
  [key: string]: unknown;
}

export interface BackendKnowledgeGraphResponse {
  nodes?: BackendKnowledgeGraphNode[];
  edges?: BackendKnowledgeGraphEdge[];
  metadata?: BackendKnowledgeGraphMetadata;
  [key: string]: unknown;
}

export interface TraceGraphNode extends KnowledgeGraphNode {
  confidence: TraceGraphConfidence;
  completeness: TraceGraphCompleteness;
  isTarget?: boolean;
  isUnknown?: boolean;
}

export interface TraceGraphEdge extends KnowledgeGraphEdge {
  direction: TraceGraphDirection;
  confidence: TraceGraphConfidence;
  completeness: TraceGraphCompleteness;
  isUnknown?: boolean;
}

export interface TraceGraphMetadata {
  target_id: string;
  total_nodes: number;
  total_edges: number;
  trace_step_count: number;
  known_edge_count: number;
  unknown_edge_count: number;
  generated_at: string;
  source_timestamps: {
    trace_start?: string;
    trace_end?: string;
    knowledge_graph_query?: string;
  };
}

export interface TraceGraphViewModel {
  target: TraceabilityItem;
  targetId: string;
  nodes: TraceGraphNode[];
  edges: TraceGraphEdge[];
  traceSteps: TraceStep[];
  completeness: TraceGraphCompleteness;
  warnings: string[];
  metadata: TraceGraphMetadata;
}

export interface TraceGraphBuildInput {
  target: TraceabilityItem;
  traceResponse?: BackendTraceResponse | null;
  traceabilityResponse?: {
    trace_path?: TraceStep[];
    knowledge_graph?: KnowledgeGraph;
  } | null;
  knowledgeGraph?: KnowledgeGraph | null;
  generatedAt?: string;
}

export interface TraceGraphSharePayload {
  target_id: string;
  target_name: string;
  filters: Record<string, unknown>;
  layout: string;
  generated_at: string;
  graph_summary: {
    total_nodes: number;
    total_edges: number;
    completeness: TraceGraphCompleteness;
    warnings: string[];
  };
}

export type NormalizedTraceabilityList = SearchResults<TraceabilityItem>;
