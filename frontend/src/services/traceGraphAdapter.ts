import type {
  GraphMetadata,
  KnowledgeGraph,
  KnowledgeGraphEdge,
  KnowledgeGraphNode,
  TraceStep,
  TraceabilityItem,
  TraceabilityResponse,
  Transaction,
} from "../types";
import type {
  BackendKnowledgeGraphResponse,
  BackendProductRecord,
  BackendProductsResponse,
  BackendTraceResponse,
  BackendTraceStepRecord,
  NormalizedTraceabilityList,
  TraceGraphBuildInput,
  TraceGraphCompleteness,
  TraceGraphConfidence,
  TraceGraphDirection,
  TraceGraphEdge,
  TraceGraphNode,
  TraceGraphSharePayload,
  TraceGraphViewModel,
} from "../types/traceGraph";

const DEFAULT_TIMESTAMP = "1970-01-01T00:00:00.000Z";
const UNKNOWN_NODE_ID = "unknown-links";

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const asString = (value: unknown, defaultValue = ""): string =>
  typeof value === "string" && value.trim() ? value : defaultValue;

const asNumber = (value: unknown, defaultValue: number): number =>
  typeof value === "number" && Number.isFinite(value) ? value : defaultValue;

const timestampOrDefault = (value: unknown): string =>
  asString(value, DEFAULT_TIMESTAMP);

const ownProperties = (record: Record<string, unknown>, excluded: string[]) =>
  Object.fromEntries(
    Object.entries(record).filter(([key]) => !excluded.includes(key)),
  );

const normalizeRelationshipType = (value: unknown): TraceabilityItem["relationships"][number]["type"] => {
  const allowed: TraceabilityItem["relationships"][number]["type"][] = [
    "produced_from",
    "processed_into",
    "transported_by",
    "quality_tested",
    "transferred_to",
  ];
  return allowed.includes(value as TraceabilityItem["relationships"][number]["type"])
    ? (value as TraceabilityItem["relationships"][number]["type"])
    : "processed_into";
};

const normalizeRelationships = (
  relationships: unknown,
): TraceabilityItem["relationships"] => {
  if (!Array.isArray(relationships)) return [];

  return relationships.filter(isRecord).map((relationship, index) => ({
    type: normalizeRelationshipType(relationship.type),
    target_item: asString(relationship.target_item ?? relationship.target, `unknown-${index}`),
    timestamp: timestampOrDefault(relationship.timestamp),
    transaction_id: asString(
      relationship.transaction_id ?? relationship.id,
      `relationship-${index + 1}`,
    ),
    metadata: isRecord(relationship.metadata) ? relationship.metadata : {},
  }));
};

export const normalizeTraceTarget = (
  product: BackendProductRecord | TraceabilityItem,
): TraceabilityItem => {
  const record = product as Record<string, unknown>;
  const id = asString(record.id, "unknown-product");
  const timestamp = timestampOrDefault(record.created_at ?? record.timestamp);
  const directProperties = isRecord(record.properties) ? record.properties : {};

  return {
    id,
    name: asString(record.name, id),
    type: asString(record.type, "entity"),
    current_owner: asString(record.current_owner ?? record.participant, "Unknown participant"),
    created_at: timestamp,
    location: asString(record.location, "") || undefined,
    properties: {
      ...ownProperties(record, [
        "id",
        "name",
        "type",
        "current_owner",
        "participant",
        "created_at",
        "timestamp",
        "location",
        "properties",
        "relationships",
      ]),
      ...directProperties,
      source_participant: record.participant,
      source_timestamp: record.timestamp,
    },
    relationships: normalizeRelationships(record.relationships),
  };
};

export const normalizeTraceabilityListResponse = (
  response: BackendProductsResponse | TraceabilityItem[],
): NormalizedTraceabilityList => {
  const rawItems = Array.isArray(response) ? response : response.items ?? [];
  const items = rawItems.map((item) => normalizeTraceTarget(item));
  const page = Array.isArray(response) ? 1 : asNumber(response.page, 1);
  const limit = Array.isArray(response)
    ? items.length || 20
    : asNumber(response.limit, items.length || 20);
  const total = Array.isArray(response)
    ? items.length
    : asNumber(response.total ?? response.total_count, items.length);
  const totalPages = Array.isArray(response)
    ? Math.max(1, Math.ceil(total / Math.max(1, limit)))
    : asNumber(response.total_pages, Math.ceil(total / Math.max(1, limit)));

  return {
    items,
    total,
    page,
    limit,
    has_more: Array.isArray(response)
      ? page < totalPages
      : typeof response.has_more === "boolean"
        ? response.has_more
        : page < totalPages,
  };
};

export const normalizeTraceSteps = (
  steps: BackendTraceStepRecord[] | TraceStep[] | undefined,
  productId = "unknown-product",
): TraceStep[] =>
  (steps ?? []).map((step, index) => {
    const record = step as Record<string, unknown>;
    const stepNumber = asNumber(record.step_number, index + 1);
    const transactionId = asString(
      record.transaction_id ?? record.tx_id ?? record.id,
      `${productId}-step-${stepNumber}`,
    );

    return {
      step_number: stepNumber,
      timestamp: timestampOrDefault(record.timestamp),
      transaction_id: transactionId,
      action: asString(record.action ?? record.type, "Trace event"),
      participant: asString(record.participant ?? record.source, "Unknown participant"),
      location: asString(record.location, "") || undefined,
      metadata: {
        ...(isRecord(record.metadata) ? record.metadata : {}),
        status: record.status,
        source: record.source,
        backend_id: record.id,
      },
    };
  });

const nodeTypeCounts = (nodes: KnowledgeGraphNode[]): Record<string, number> =>
  nodes.reduce<Record<string, number>>((counts, node) => {
    counts[node.type] = (counts[node.type] ?? 0) + 1;
    return counts;
  }, {});

const edgeTypeCounts = (edges: KnowledgeGraphEdge[]): Record<string, number> =>
  edges.reduce<Record<string, number>>((counts, edge) => {
    counts[edge.type] = (counts[edge.type] ?? 0) + 1;
    return counts;
  }, {});

const normalizeNodeType = (value: unknown): KnowledgeGraphNode["type"] => {
  const type = asString(value, "entity");
  if (
    type === "item" ||
    type === "participant" ||
    type === "location" ||
    type === "process" ||
    type === "entity" ||
    type === "unknown"
  ) {
    return type;
  }
  return "entity";
};

export const normalizeKnowledgeGraphResponse = (
  response: BackendKnowledgeGraphResponse | KnowledgeGraph | null | undefined,
): KnowledgeGraph => {
  const nodes = (response?.nodes ?? []).map((node, index) => {
    const id = asString(node.id, `kg-node-${index + 1}`);
    return {
      id,
      label: asString(node.label, id),
      type: normalizeNodeType(node.type),
      properties: isRecord(node.properties) ? node.properties : {},
    } satisfies KnowledgeGraphNode;
  });

  const edges = (response?.edges ?? [])
    .filter((edge) => asString(edge.source) && asString(edge.target))
    .map((edge, index) => {
      const source = asString(edge.source);
      const target = asString(edge.target);
      const type = asString(edge.type, "related");
      return {
        id: asString(edge.id, `${source}--${type}--${target}--${index + 1}`),
        source,
        target,
        type,
        label: asString(edge.label, type),
        properties: isRecord(edge.properties) ? edge.properties : {},
      } satisfies KnowledgeGraphEdge;
    });

  const metadataRecord = (response?.metadata ?? {}) as Record<string, unknown>;
  const metadata: GraphMetadata = {
    total_nodes: asNumber(metadataRecord.total_nodes, nodes.length),
    total_edges: asNumber(metadataRecord.total_edges, edges.length),
    node_types: (metadataRecord.node_types as Record<string, number> | undefined) ?? nodeTypeCounts(nodes),
    edge_types: (metadataRecord.edge_types as Record<string, number> | undefined) ?? edgeTypeCounts(edges),
    created_at: asString(
      metadataRecord.created_at ?? metadataRecord.query_timestamp,
      new Date(0).toISOString(),
    ),
    query_time_ms: asNumber(metadataRecord.query_time_ms, 0),
  };

  return { nodes, edges, metadata };
};

const traceStepToNode = (step: TraceStep): TraceGraphNode => ({
  id: `trace-step-${step.step_number}-${step.transaction_id}`,
  label: `${step.step_number}. ${step.action}`,
  type: "process",
  properties: {
    ...step.metadata,
    timestamp: step.timestamp,
    participant: step.participant,
    location: step.location,
    transaction_id: step.transaction_id,
    step_number: step.step_number,
  },
  confidence: "known",
  completeness: "known",
});

const graphNodeToTraceNode = (
  node: KnowledgeGraphNode,
  targetId: string,
): TraceGraphNode => ({
  ...node,
  confidence: "known",
  completeness: "known",
  isTarget: node.id === targetId,
  isUnknown: node.type === "unknown",
});

const graphEdgeToTraceEdge = (edge: KnowledgeGraphEdge): TraceGraphEdge => ({
  ...edge,
  direction: inferDirection(edge.type),
  confidence: "known",
  completeness: "known",
  isUnknown: edge.type === "unknown",
});

const inferDirection = (type: string): TraceGraphDirection => {
  if (/(from|input|source|upstream|produced_from|produced_by)/i.test(type)) return "upstream";
  if (/(into|output|target|downstream|processed_into)/i.test(type)) return "downstream";
  if (/(related|associated|participant|location)/i.test(type)) return "related";
  return "unknown";
};

const relationshipLabel = (type: string): string => type.replace(/_/g, " ");

const relationshipTargetNode = (
  relationship: TraceabilityItem["relationships"][number],
): TraceGraphNode => ({
  id: relationship.target_item,
  label: relationship.target_item,
  type: "unknown",
  properties: {
    reason: "Target relationship was returned without a matching knowledge-graph node.",
    relationship_type: relationship.type,
    timestamp: relationship.timestamp,
    transaction_id: relationship.transaction_id,
    ...relationship.metadata,
  },
  confidence: "missing-data",
  completeness: "missing-data",
  isUnknown: true,
});

const relationshipToTraceEdge = (
  target: TraceabilityItem,
  relationship: TraceabilityItem["relationships"][number],
  index: number,
): TraceGraphEdge => ({
  id: `${target.id}--${relationship.type}--${relationship.target_item}--${relationship.transaction_id || index + 1}`,
  source: target.id,
  target: relationship.target_item,
  type: relationship.type,
  label: relationshipLabel(relationship.type),
  properties: {
    timestamp: relationship.timestamp,
    transaction_id: relationship.transaction_id,
    ...relationship.metadata,
  },
  direction: inferDirection(relationship.type),
  confidence: "known",
  completeness: "known",
});

const mergeNodes = (nodes: TraceGraphNode[]): TraceGraphNode[] => {
  const byId = new Map<string, TraceGraphNode>();
  nodes.forEach((node) => {
    const previous = byId.get(node.id);
    byId.set(node.id, previous ? { ...previous, ...node, properties: { ...previous.properties, ...node.properties } } : node);
  });
  return Array.from(byId.values());
};

const mergeEdges = (edges: TraceGraphEdge[]): TraceGraphEdge[] => {
  const byId = new Map<string, TraceGraphEdge>();
  edges.forEach((edge) => byId.set(edge.id, edge));
  return Array.from(byId.values());
};

const createUnknownLinkNode = (): TraceGraphNode => ({
  id: UNKNOWN_NODE_ID,
  label: "Unknown links",
  type: "unknown",
  properties: {
    reason: "Backend did not return explicit relationship edges for this trace target.",
  },
  confidence: "missing-data",
  completeness: "missing-data",
  isUnknown: true,
});

const deriveCompleteness = (
  warnings: string[],
  knownEdges: number,
  unknownEdges: number,
): TraceGraphCompleteness => {
  if (unknownEdges > 0) return "missing-data";
  if (warnings.length > 0) return "unknown";
  return knownEdges > 0 ? "known" : "inferred";
};

export const buildTraceGraphViewModel = ({
  target,
  traceResponse,
  traceabilityResponse,
  knowledgeGraph,
  generatedAt = new Date().toISOString(),
}: TraceGraphBuildInput): TraceGraphViewModel => {
  const normalizedGraph = normalizeKnowledgeGraphResponse(
    knowledgeGraph ?? traceabilityResponse?.knowledge_graph ?? null,
  );
  const rawTraceSteps =
    traceabilityResponse?.trace_path ?? traceResponse?.trace_steps ?? [];
  const traceSteps = normalizeTraceSteps(rawTraceSteps, target.id);
  const warnings: string[] = [];

  const targetNode: TraceGraphNode = {
    id: target.id,
    label: target.name,
    type: "item",
    properties: {
      ...target.properties,
      current_owner: target.current_owner,
      location: target.location,
      created_at: target.created_at,
    },
    confidence: "known",
    completeness: "known",
    isTarget: true,
  };

  const traceNodes = traceSteps.map(traceStepToNode);
  const traceEdges = traceNodes.map((node, index) => ({
    id: `${target.id}--trace-step--${node.id}`,
    source: index === 0 ? target.id : traceNodes[index - 1].id,
    target: node.id,
    type: "trace_step",
    label: "trace step",
    properties: {
      step_number: node.properties.step_number,
      timestamp: node.properties.timestamp,
    },
    direction: "downstream" as TraceGraphDirection,
    confidence: "inferred" as TraceGraphConfidence,
    completeness: "inferred" as TraceGraphCompleteness,
  }));

  const graphNodes = normalizedGraph.nodes.map((node) =>
    graphNodeToTraceNode(node, target.id),
  );
  const graphEdges = normalizedGraph.edges.map(graphEdgeToTraceEdge);
  const relationshipNodes = target.relationships.map(relationshipTargetNode);
  const relationshipEdges = target.relationships.map((relationship, index) =>
    relationshipToTraceEdge(target, relationship, index),
  );
  const nodes = mergeNodes([
    ...relationshipNodes,
    targetNode,
    ...graphNodes,
    ...traceNodes,
  ]);
  const edges = mergeEdges([...graphEdges, ...relationshipEdges, ...traceEdges]);

  if (!traceSteps.length) {
    warnings.push("Trace endpoint returned no trace_steps for this target.");
  }

  if (!normalizedGraph.edges.length && !relationshipEdges.length) {
    warnings.push("No explicit relationship edges were returned; unknown link marker added.");
    nodes.push(createUnknownLinkNode());
    edges.push({
      id: `${target.id}--unknown-links`,
      source: target.id,
      target: UNKNOWN_NODE_ID,
      type: "unknown",
      label: "unknown relationship",
      properties: {
        reason: "relationship edges missing from backend response",
      },
      direction: "unknown",
      confidence: "missing-data",
      completeness: "missing-data",
      isUnknown: true,
    });
  }

  const knownEdgeCount = edges.filter((edge) => edge.confidence === "known").length;
  const unknownEdgeCount = edges.filter((edge) => edge.isUnknown).length;
  const completeness = deriveCompleteness(warnings, knownEdgeCount, unknownEdgeCount);

  return {
    target,
    targetId: target.id,
    nodes: mergeNodes(nodes),
    edges: mergeEdges(edges),
    traceSteps,
    completeness,
    warnings,
    metadata: {
      target_id: target.id,
      total_nodes: nodes.length,
      total_edges: edges.length,
      trace_step_count: traceSteps.length,
      known_edge_count: knownEdgeCount,
      unknown_edge_count: unknownEdgeCount,
      generated_at: generatedAt,
      source_timestamps: {
        trace_start: traceResponse?.start_timestamp,
        trace_end: traceResponse?.end_timestamp,
        knowledge_graph_query: normalizedGraph.metadata.created_at,
      },
    },
  };
};

export const traceGraphViewModelToKnowledgeGraph = (
  viewModel: TraceGraphViewModel,
): KnowledgeGraph => ({
  nodes: viewModel.nodes.map((node) => ({
    id: node.id,
    label: node.label,
    type: node.type,
    properties: {
      ...node.properties,
      confidence: node.confidence,
      completeness: node.completeness,
      isTarget: node.isTarget,
      isUnknown: node.isUnknown,
    },
    size: node.isTarget ? 60 : node.isUnknown ? 48 : 42,
  })),
  edges: viewModel.edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    type: edge.type,
    label: edge.label,
    properties: {
      ...edge.properties,
      confidence: edge.confidence,
      completeness: edge.completeness,
      direction: edge.direction,
      isUnknown: edge.isUnknown,
    },
  })),
  metadata: {
    total_nodes: viewModel.nodes.length,
    total_edges: viewModel.edges.length,
    node_types: nodeTypeCounts(viewModel.nodes),
    edge_types: edgeTypeCounts(viewModel.edges),
    created_at: viewModel.metadata.generated_at,
    query_time_ms: 0,
  },
});

export const normalizeTraceResponse = (
  response: BackendTraceResponse | TraceabilityResponse,
  target?: TraceabilityItem,
): TraceabilityResponse => {
  const candidate = response as Partial<TraceabilityResponse>;
  if (isRecord(candidate.item) && Array.isArray(candidate.trace_path)) {
    const item = normalizeTraceTarget(candidate.item);
    return {
      item,
      trace_path: normalizeTraceSteps(candidate.trace_path, item.id),
      knowledge_graph: normalizeKnowledgeGraphResponse(candidate.knowledge_graph),
      related_transactions: candidate.related_transactions ?? [],
    };
  }

  const backendResponse = response as BackendTraceResponse;
  const productId = asString(backendResponse.product_id, target?.id ?? "unknown-product");
  const item = target ??
    normalizeTraceTarget({
      id: productId,
      name: productId,
      type: "entity",
      participant: "Unknown participant",
      timestamp: backendResponse.end_timestamp ?? backendResponse.start_timestamp,
    });
  const tracePath = normalizeTraceSteps(backendResponse.trace_steps, productId);
  const graphView = buildTraceGraphViewModel({ target: item, traceResponse: backendResponse });

  return {
    item,
    trace_path: tracePath,
    knowledge_graph: traceGraphViewModelToKnowledgeGraph(graphView),
    related_transactions: [] as Transaction[],
  };
};

export const createTraceGraphSharePayload = (
  viewModel: TraceGraphViewModel,
  filters: Record<string, unknown> = {},
  layout = "dagre",
): TraceGraphSharePayload => ({
  target_id: viewModel.targetId,
  target_name: viewModel.target.name,
  filters,
  layout,
  generated_at: new Date().toISOString(),
  graph_summary: {
    total_nodes: viewModel.nodes.length,
    total_edges: viewModel.edges.length,
    completeness: viewModel.completeness,
    warnings: viewModel.warnings,
  },
});
