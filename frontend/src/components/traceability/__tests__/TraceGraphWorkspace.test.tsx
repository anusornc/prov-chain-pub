import React from "react";
import { act } from "react";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import TraceGraphWorkspace from "../TraceGraphWorkspace";
import { renderWithoutProviders } from "../../../test-utils/test-utils";
import { traceabilityService } from "../../../services/traceability";
import type { KnowledgeGraph, TraceabilityItem, TraceabilityResponse } from "../../../types";

jest.mock("../../../services/traceability", () => ({
  traceabilityService: {
    getItemTrace: jest.fn(),
    getKnowledgeGraph: jest.fn(),
  },
}));

jest.mock("../ProvenanceGraph", () => ({
  __esModule: true,
  default: ({
    knowledgeGraph,
    onNodeSelect,
    onEdgeSelect,
  }: {
    knowledgeGraph: KnowledgeGraph;
    onNodeSelect?: (node: KnowledgeGraph["nodes"][number]) => void;
    onEdgeSelect?: (edge: KnowledgeGraph["edges"][number]) => void;
  }) => (
    <div data-testid="mock-provenance-graph">
      <span>{knowledgeGraph.nodes.length} graph nodes</span>
      <span>{knowledgeGraph.edges.length} graph edges</span>
      <span>{knowledgeGraph.nodes.map((node) => node.label).join(", ")}</span>
      <button type="button" onClick={() => onNodeSelect?.(knowledgeGraph.nodes[0])}>
        select node
      </button>
      <button type="button" onClick={() => onEdgeSelect?.(knowledgeGraph.edges[0])}>
        select edge
      </button>
    </div>
  ),
}));

const target: TraceabilityItem = {
  id: "batch-001",
  name: "Organic Mango Batch",
  type: "batch",
  current_owner: "Farm Cooperative A",
  created_at: "2026-05-01T09:00:00Z",
  location: "Chiang Mai",
  properties: {},
  relationships: [],
};

const traceResponse: TraceabilityResponse = {
  item: target,
  trace_path: [
    {
      step_number: 1,
      timestamp: "2026-05-01T09:00:00Z",
      transaction_id: "event-1",
      action: "Harvested",
      participant: "Farm Cooperative A",
      location: "Chiang Mai",
      metadata: { status: "complete" },
    },
  ],
  knowledge_graph: {
    nodes: [],
    edges: [],
    metadata: {
      total_nodes: 0,
      total_edges: 0,
      node_types: {},
      edge_types: {},
      created_at: "2026-05-01T10:00:00Z",
      query_time_ms: 0,
    },
  },
  related_transactions: [],
};

const knowledgeGraph: KnowledgeGraph = {
  nodes: [
    {
      id: "batch-001",
      label: "Organic Mango Batch",
      type: "entity",
      properties: {},
    },
    {
      id: "farm-a",
      label: "Farm Cooperative A",
      type: "entity",
      properties: {},
    },
  ],
  edges: [
    {
      id: "farm-a--produced_by--batch-001--1",
      source: "farm-a",
      target: "batch-001",
      type: "produced_by",
      label: "produced by",
      properties: {},
    },
  ],
  metadata: {
    total_nodes: 2,
    total_edges: 1,
    node_types: { entity: 2 },
    edge_types: { produced_by: 1 },
    created_at: "2026-05-01T10:00:00Z",
    query_time_ms: 0,
  },
};

const secondTarget: TraceabilityItem = {
  id: "batch-002",
  name: "Dragon Fruit Batch",
  type: "batch",
  current_owner: "Packing House B",
  created_at: "2026-05-02T09:00:00Z",
  location: "Bangkok",
  properties: {},
  relationships: [],
};

const secondTraceResponse: TraceabilityResponse = {
  item: secondTarget,
  trace_path: [
    {
      step_number: 1,
      timestamp: "2026-05-02T09:00:00Z",
      transaction_id: "event-2",
      action: "Packed",
      participant: "Packing House B",
      location: "Bangkok",
      metadata: { status: "complete" },
    },
  ],
  knowledge_graph: {
    nodes: [],
    edges: [],
    metadata: {
      total_nodes: 0,
      total_edges: 0,
      node_types: {},
      edge_types: {},
      created_at: "2026-05-02T10:00:00Z",
      query_time_ms: 0,
    },
  },
  related_transactions: [],
};

const secondKnowledgeGraph: KnowledgeGraph = {
  nodes: [
    {
      id: "batch-002",
      label: "Dragon Fruit Batch",
      type: "entity",
      properties: {},
    },
    {
      id: "packing-house-b",
      label: "Packing House B",
      type: "entity",
      properties: {},
    },
  ],
  edges: [
    {
      id: "packing-house-b--packed--batch-002--1",
      source: "packing-house-b",
      target: "batch-002",
      type: "packed_by",
      label: "packed by",
      properties: {},
    },
  ],
  metadata: {
    total_nodes: 2,
    total_edges: 1,
    node_types: { entity: 2 },
    edge_types: { packed_by: 1 },
    created_at: "2026-05-02T10:00:00Z",
    query_time_ms: 0,
  },
};

type Deferred<T> = {
  promise: Promise<T>;
  resolve: (value: T) => void;
};

const createDeferred = <T,>(): Deferred<T> => {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((promiseResolve) => {
    resolve = promiseResolve;
  });
  return { promise, resolve };
};

const mockedTraceabilityService = jest.mocked(traceabilityService);
let clipboardWriteText: jest.Mock;

const renderWorkspace = async (targetOverride: TraceabilityItem | null) => {
  await act(async () => {
    renderWithoutProviders(<TraceGraphWorkspace target={targetOverride} />);
  });
};

const waitForLoadedGraph = async () => {
  await waitFor(() => {
    expect(mockedTraceabilityService.getItemTrace).toHaveBeenCalledWith("batch-001");
    expect(mockedTraceabilityService.getKnowledgeGraph).toHaveBeenCalledWith(["batch-001"]);
  });
  await waitFor(() => {
    expect(screen.queryByText(/Building trace path graph/i)).not.toBeInTheDocument();
  });
  return screen.findByTestId("mock-provenance-graph");
};

describe("TraceGraphWorkspace", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    clipboardWriteText = jest.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: clipboardWriteText,
      },
    });
    mockedTraceabilityService.getItemTrace.mockResolvedValue(traceResponse);
    mockedTraceabilityService.getKnowledgeGraph.mockResolvedValue(knowledgeGraph);
  });

  it("prompts the user to select a target before loading graph data", () => {
    renderWithoutProviders(<TraceGraphWorkspace target={null} />);

    expect(screen.getByText("Trace path graph")).toBeInTheDocument();
    expect(screen.getByText(/Select a search result/i)).toBeInTheDocument();
    expect(mockedTraceabilityService.getItemTrace).not.toHaveBeenCalled();
  });

  it("loads trace and knowledge graph data for the selected target", async () => {
    await renderWorkspace(target);

    expect(await screen.findByText("Organic Mango Batch")).toBeInTheDocument();
    expect(await waitForLoadedGraph()).toBeInTheDocument();
    expect(screen.getByText("3 graph nodes")).toBeInTheDocument();
    expect(screen.getByText("2 graph edges")).toBeInTheDocument();
  });

  it("copies a frontend-only share view payload", async () => {
    const user = userEvent.setup();
    await renderWorkspace(target);

    await waitForLoadedGraph();
    await user.click(screen.getByRole("button", { name: /copy share view/i }));

    expect(
      await screen.findByText("Copied graph view payload to clipboard"),
    ).toBeInTheDocument();
  });


  it("keeps late responses from replacing the graph for a newer target", async () => {
    const firstTrace = createDeferred<TraceabilityResponse>();
    const firstGraph = createDeferred<KnowledgeGraph>();
    const secondTrace = createDeferred<TraceabilityResponse>();
    const secondGraph = createDeferred<KnowledgeGraph>();

    mockedTraceabilityService.getItemTrace.mockImplementation((itemId: string) => {
      return itemId === "batch-001" ? firstTrace.promise : secondTrace.promise;
    });
    mockedTraceabilityService.getKnowledgeGraph.mockImplementation((itemIds: string[]) => {
      return itemIds[0] === "batch-001" ? firstGraph.promise : secondGraph.promise;
    });

    const { rerender } = renderWithoutProviders(
      <TraceGraphWorkspace target={target} />,
    );

    rerender(<TraceGraphWorkspace target={secondTarget} />);

    await act(async () => {
      secondTrace.resolve(secondTraceResponse);
      secondGraph.resolve(secondKnowledgeGraph);
    });

    expect(await screen.findByText("Dragon Fruit Batch")).toBeInTheDocument();
    expect((await screen.findAllByText(/Packing House B/)).length).toBeGreaterThan(0);

    await act(async () => {
      firstTrace.resolve(traceResponse);
      firstGraph.resolve(knowledgeGraph);
    });

    await waitFor(() => {
      expect(screen.queryByText(/Farm Cooperative A/)).not.toBeInTheDocument();
    });
    expect(screen.getByText("Dragon Fruit Batch")).toBeInTheDocument();
  });

  it("keeps inspector data available for node and edge selection", async () => {
    const user = userEvent.setup();
    await renderWorkspace(target);

    await waitForLoadedGraph();
    await user.click(screen.getByRole("button", { name: /select node/i }));
    expect(screen.getByText(/Node ·/i)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /select edge/i }));
    expect(screen.getByText(/Edge ·/i)).toBeInTheDocument();
    expect(screen.getByText(/farm-a → batch-001/i)).toBeInTheDocument();
  });
});
