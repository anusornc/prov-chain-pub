import {
  buildTraceGraphViewModel,
  createTraceGraphSharePayload,
  normalizeKnowledgeGraphResponse,
  normalizeTraceResponse,
  normalizeTraceTarget,
  normalizeTraceabilityListResponse,
  traceGraphViewModelToKnowledgeGraph,
} from "../traceGraphAdapter";
import type {
  BackendKnowledgeGraphResponse,
  BackendProductsResponse,
  BackendTraceResponse,
} from "../../types/traceGraph";

const productsResponse: BackendProductsResponse = {
  items: [
    {
      id: "batch-001",
      name: "Organic Mango Batch",
      type: "batch",
      participant: "Farm Cooperative A",
      timestamp: "2026-05-01T09:00:00Z",
      location: "Chiang Mai",
      status: "in_transit",
    },
  ],
  total_count: 1,
  total_pages: 1,
  page: 1,
  limit: 20,
};

const productDetail = {
  id: "batch-001",
  name: "Organic Mango Batch",
  type: "batch",
  participant: "Farm Cooperative A",
  timestamp: "2026-05-01T09:00:00Z",
  trace_steps: [
    {
      id: "step-1",
      timestamp: "2026-05-01T09:00:00Z",
      participant: "Farm Cooperative A",
      action: "Harvested",
      status: "complete",
    },
  ],
};

const traceResponse: BackendTraceResponse = {
  product_id: "batch-001",
  trace_steps: [
    {
      id: "event-1",
      timestamp: "2026-05-01T09:00:00Z",
      location: "Chiang Mai",
      participant: "Farm Cooperative A",
      action: "Harvested",
      status: "complete",
      metadata: { temperature_c: 24 },
      source: "blockchain",
    },
  ],
  total_steps: 1,
  start_timestamp: "2026-05-01T09:00:00Z",
  end_timestamp: "2026-05-01T12:00:00Z",
};

const knowledgeGraphResponse: BackendKnowledgeGraphResponse = {
  nodes: [
    {
      id: "batch-001",
      label: "Organic Mango Batch",
      type: "entity",
      properties: { category: "batch" },
    },
    {
      id: "farm-a",
      label: "Farm Cooperative A",
      type: "entity",
      properties: { role: "producer" },
    },
  ],
  edges: [
    {
      source: "farm-a",
      target: "batch-001",
      label: "produced",
      type: "produced_by",
    },
  ],
  metadata: {
    query_timestamp: "2026-05-01T12:30:00Z",
    total_nodes: 2,
    total_edges: 1,
  },
};

describe("trace graph adapter", () => {
  it("normalizes /api/products list shape without requiring relationships", () => {
    const normalized = normalizeTraceabilityListResponse(productsResponse);

    expect(normalized.total).toBe(1);
    expect(normalized.has_more).toBe(false);
    expect(normalized.items[0]).toMatchObject({
      id: "batch-001",
      current_owner: "Farm Cooperative A",
      created_at: "2026-05-01T09:00:00Z",
      relationships: [],
    });
  });

  it("normalizes product detail participant, timestamp, and trace_steps drift", () => {
    const normalized = normalizeTraceTarget(productDetail);

    expect(normalized.current_owner).toBe("Farm Cooperative A");
    expect(normalized.created_at).toBe("2026-05-01T09:00:00Z");
    expect(normalized.relationships).toEqual([]);
    expect(normalized.properties.trace_steps).toHaveLength(1);
  });

  it("normalizes /api/products/:id/trace product_id and trace_steps shape", () => {
    const normalized = normalizeTraceResponse(traceResponse);

    expect(normalized.item.id).toBe("batch-001");
    expect(normalized.trace_path).toHaveLength(1);
    expect(normalized.trace_path[0]).toMatchObject({
      step_number: 1,
      transaction_id: "event-1",
      participant: "Farm Cooperative A",
      action: "Harvested",
    });
  });

  it("normalizes /api/knowledge-graph entity nodes, edge ids/properties, and query_timestamp metadata", () => {
    const normalized = normalizeKnowledgeGraphResponse(knowledgeGraphResponse);

    expect(normalized.nodes[0].type).toBe("entity");
    expect(normalized.edges[0]).toMatchObject({
      id: "farm-a--produced_by--batch-001--1",
      source: "farm-a",
      target: "batch-001",
      properties: {},
    });
    expect(normalized.metadata.created_at).toBe("2026-05-01T12:30:00Z");
    expect(normalized.metadata.total_nodes).toBe(2);
    expect(normalized.metadata.total_edges).toBe(1);
  });

  it("builds trace graph view models with direction and known confidence when edges exist", () => {
    const target = normalizeTraceTarget(productsResponse.items?.[0] ?? {});
    const graph = normalizeKnowledgeGraphResponse(knowledgeGraphResponse);
    const viewModel = buildTraceGraphViewModel({
      target,
      traceResponse,
      knowledgeGraph: graph,
      generatedAt: "2026-05-01T13:00:00Z",
    });

    expect(viewModel.completeness).toBe("known");
    expect(viewModel.nodes.some((node) => node.isTarget)).toBe(true);
    expect(viewModel.edges.some((edge) => edge.direction === "unknown")).toBe(false);
    expect(viewModel.warnings).toEqual([]);
  });

  it("adds visible unknown links when backend graph and relationships are missing", () => {
    const target = normalizeTraceTarget(productsResponse.items?.[0] ?? {});
    const viewModel = buildTraceGraphViewModel({
      target,
      traceResponse: { product_id: target.id, trace_steps: [] },
      knowledgeGraph: normalizeKnowledgeGraphResponse({ nodes: [], edges: [] }),
    });

    expect(viewModel.completeness).toBe("missing-data");
    expect(viewModel.nodes.some((node) => node.type === "unknown" && node.isUnknown)).toBe(true);
    expect(viewModel.edges.some((edge) => edge.confidence === "missing-data")).toBe(true);
    expect(viewModel.warnings.join(" ")).toContain("unknown link marker");
  });

  it("materializes target relationships as directional graph edges", () => {
    const target = normalizeTraceTarget({
      ...(productsResponse.items?.[0] ?? {}),
      relationships: [
        {
          type: "produced_from",
          target_item: "farm-input-001",
          timestamp: "2026-05-01T08:00:00Z",
          transaction_id: "rel-1",
          metadata: { lot: "A" },
        },
      ],
    });
    const viewModel = buildTraceGraphViewModel({
      target,
      traceResponse: { product_id: target.id, trace_steps: [] },
      knowledgeGraph: normalizeKnowledgeGraphResponse({ nodes: [], edges: [] }),
    });

    expect(viewModel.edges).toContainEqual(
      expect.objectContaining({
        source: "batch-001",
        target: "farm-input-001",
        type: "produced_from",
        direction: "upstream",
        confidence: "known",
      }),
    );
    expect(viewModel.nodes).toContainEqual(
      expect.objectContaining({
        id: "farm-input-001",
        type: "unknown",
        confidence: "missing-data",
      }),
    );
    expect(viewModel.edges.some((edge) => edge.id === "batch-001--unknown-links")).toBe(false);
    expect(viewModel.warnings.join(" ")).not.toContain("unknown link marker");
  });

  it("converts view models back to renderer-compatible knowledge graphs and share payloads", () => {
    const target = normalizeTraceTarget(productsResponse.items?.[0] ?? {});
    const viewModel = buildTraceGraphViewModel({
      target,
      traceResponse,
      knowledgeGraph: normalizeKnowledgeGraphResponse(knowledgeGraphResponse),
    });
    const rendererGraph = traceGraphViewModelToKnowledgeGraph(viewModel);
    const sharePayload = createTraceGraphSharePayload(
      viewModel,
      { relationType: "all", depth: 2 },
      "dagre",
    );

    expect(rendererGraph.nodes.length).toBeGreaterThan(0);
    expect(rendererGraph.edges.length).toBeGreaterThan(0);
    expect(rendererGraph.nodes[0].properties).toHaveProperty("confidence");
    expect(sharePayload).toMatchObject({
      target_id: "batch-001",
      target_name: "Organic Mango Batch",
      layout: "dagre",
    });
  });
});
