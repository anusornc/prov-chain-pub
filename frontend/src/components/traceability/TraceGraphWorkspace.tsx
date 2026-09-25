import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  AlertTriangle,
  Copy,
  Eye,
  Filter,
  GitBranch,
  Info,
  RefreshCw,
  X,
} from "lucide-react";
import { traceabilityService } from "../../services/traceability";
import {
  buildTraceGraphViewModel,
  createTraceGraphSharePayload,
  traceGraphViewModelToKnowledgeGraph,
} from "../../services/traceGraphAdapter";
import type {
  KnowledgeGraph,
  KnowledgeGraphEdge,
  KnowledgeGraphNode,
  TraceabilityItem,
} from "../../types";
import type {
  TraceGraphDirection,
  TraceGraphViewModel,
} from "../../types/traceGraph";
import Alert from "../ui/Alert";
import Badge from "../ui/Badge";
import Button from "../ui/Button";
import Card from "../ui/Card";
import LoadingSpinner from "../ui/LoadingSpinner";
import ProvenanceGraph from "./ProvenanceGraph";

interface TraceGraphWorkspaceProps {
  target: TraceabilityItem | null;
  onClearTarget?: () => void;
  className?: string;
}

interface WorkspaceFilters {
  relationType: string;
  entityType: string;
  direction: TraceGraphDirection | "all";
  depth: number | "all";
  showUnknownLinks: boolean;
}

const DEFAULT_FILTERS: WorkspaceFilters = {
  relationType: "all",
  entityType: "all",
  direction: "all",
  depth: "all",
  showUnknownLinks: true,
};

const copyToClipboard = async (text: string) => {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "absolute";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();
  document.execCommand("copy");
  document.body.removeChild(textarea);
};

const depthLimitedIds = (
  viewModel: TraceGraphViewModel,
  maxDepth: number | "all",
): Set<string> => {
  if (maxDepth === "all") {
    return new Set(viewModel.nodes.map((node) => node.id));
  }

  const visited = new Set<string>([viewModel.targetId]);
  let frontier = new Set<string>([viewModel.targetId]);

  for (let depth = 0; depth < maxDepth; depth += 1) {
    const next = new Set<string>();
    viewModel.edges.forEach((edge) => {
      if (frontier.has(edge.source) && !visited.has(edge.target)) {
        next.add(edge.target);
      }
      if (frontier.has(edge.target) && !visited.has(edge.source)) {
        next.add(edge.source);
      }
    });
    next.forEach((id) => visited.add(id));
    frontier = next;
  }

  return visited;
};

const applyWorkspaceFilters = (
  viewModel: TraceGraphViewModel,
  filters: WorkspaceFilters,
): KnowledgeGraph => {
  const visibleByDepth = depthLimitedIds(viewModel, filters.depth);
  const graph = traceGraphViewModelToKnowledgeGraph(viewModel);
  const nodes = graph.nodes.filter((node) => {
    const isUnknown = node.properties.isUnknown === true;
    return (
      visibleByDepth.has(node.id) &&
      (filters.entityType === "all" || node.type === filters.entityType) &&
      (filters.showUnknownLinks || !isUnknown)
    );
  });
  const nodeIds = new Set(nodes.map((node) => node.id));
  const edges = graph.edges.filter((edge) => {
    const direction = edge.properties.direction;
    const isUnknown = edge.properties.isUnknown === true;
    return (
      nodeIds.has(edge.source) &&
      nodeIds.has(edge.target) &&
      (filters.relationType === "all" || edge.type === filters.relationType) &&
      (filters.direction === "all" || direction === filters.direction) &&
      (filters.showUnknownLinks || !isUnknown)
    );
  });

  return {
    nodes,
    edges,
    metadata: {
      ...graph.metadata,
      total_nodes: nodes.length,
      total_edges: edges.length,
    },
  };
};

const TraceGraphWorkspace: React.FC<TraceGraphWorkspaceProps> = ({
  target,
  onClearTarget,
  className = "",
}) => {
  const [viewModel, setViewModel] = useState<TraceGraphViewModel | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<WorkspaceFilters>(DEFAULT_FILTERS);
  const [selectedNode, setSelectedNode] = useState<KnowledgeGraphNode | null>(
    null,
  );
  const [selectedEdge, setSelectedEdge] = useState<KnowledgeGraphEdge | null>(
    null,
  );
  const [shareStatus, setShareStatus] = useState<string | null>(null);
  const loadSequenceRef = useRef(0);

  const clearGraphState = useCallback(() => {
    setViewModel(null);
    setSelectedNode(null);
    setSelectedEdge(null);
    setShareStatus(null);
  }, []);

  const loadTraceGraph = useCallback(async () => {
    const requestId = loadSequenceRef.current + 1;
    loadSequenceRef.current = requestId;

    if (!target) {
      clearGraphState();
      setLoading(false);
      setError(null);
      return;
    }

    const requestedTarget = target;
    clearGraphState();
    setLoading(true);
    setError(null);

    try {
      const [traceResponse, knowledgeGraph] = await Promise.all([
        traceabilityService.getItemTrace(requestedTarget.id),
        traceabilityService.getKnowledgeGraph([requestedTarget.id]),
      ]);

      if (loadSequenceRef.current !== requestId) {
        return;
      }

      setViewModel(
        buildTraceGraphViewModel({
          target: requestedTarget,
          traceabilityResponse: traceResponse,
          knowledgeGraph,
        }),
      );
    } catch (err) {
      if (loadSequenceRef.current !== requestId) {
        return;
      }

      const message = err instanceof Error ? err.message : "Failed to load trace graph";
      setError(message);
      setViewModel(null);
    } finally {
      if (loadSequenceRef.current === requestId) {
        setLoading(false);
      }
    }
  }, [clearGraphState, target]);

  useEffect(() => {
    loadTraceGraph();
  }, [loadTraceGraph]);

  const relationTypes = useMemo(
    () => Array.from(new Set(viewModel?.edges.map((edge) => edge.type) ?? [])).sort(),
    [viewModel],
  );
  const entityTypes = useMemo(
    () => Array.from(new Set(viewModel?.nodes.map((node) => node.type) ?? [])).sort(),
    [viewModel],
  );
  const filteredGraph = useMemo(
    () => (viewModel ? applyWorkspaceFilters(viewModel, filters) : null),
    [viewModel, filters],
  );

  const handleShare = async () => {
    if (!viewModel) return;

    const payload = createTraceGraphSharePayload(
      viewModel,
      { ...filters },
      "dagre",
    );
    await copyToClipboard(JSON.stringify(payload, null, 2));
    setShareStatus("Copied graph view payload to clipboard");
  };

  const updateFilter = <K extends keyof WorkspaceFilters>(
    key: K,
    value: WorkspaceFilters[K],
  ) => {
    setFilters((current) => ({ ...current, [key]: value }));
  };

  if (!target) {
    return (
      <Card className={`border-dashed border-blue-200 bg-blue-50 ${className}`}>
        <div className="flex items-center gap-4 text-blue-900">
          <GitBranch className="h-8 w-8" />
          <div>
            <h2 className="text-xl font-semibold">Trace path graph</h2>
            <p className="text-sm">
              Select a search result to open a graph-first trace workspace with
              direction, inspection, filters, and unknown-link markers.
            </p>
          </div>
        </div>
      </Card>
    );
  }

  return (
    <section className={`space-y-4 ${className}`} aria-label="Trace path graph workspace">
      <Card>
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div className="space-y-2">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="primary">Trace path graph</Badge>
              {viewModel && (
                <Badge
                  variant={viewModel.completeness === "known" ? "success" : "warning"}
                >
                  {viewModel.completeness}
                </Badge>
              )}
            </div>
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
                {target.name}
              </h2>
              <p className="text-sm text-gray-600 dark:text-gray-300">
                {target.id} · owner {target.current_owner || "Unknown"}
              </p>
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              variant="outline"
              onClick={loadTraceGraph}
              disabled={loading}
              className="flex items-center gap-2"
            >
              <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
              Refresh graph
            </Button>
            <Button
              variant="outline"
              onClick={handleShare}
              disabled={!viewModel}
              className="flex items-center gap-2"
            >
              <Copy className="h-4 w-4" />
              Copy share view
            </Button>
            {onClearTarget && (
              <Button
                variant="outline"
                onClick={onClearTarget}
                className="flex items-center gap-2"
              >
                <X className="h-4 w-4" />
                Clear
              </Button>
            )}
          </div>
        </div>
      </Card>

      {error && <Alert variant="error" message={error} />}
      {shareStatus && <Alert variant="success" message={shareStatus} />}

      {loading && !viewModel ? (
        <Card>
          <div className="flex justify-center py-10">
            <LoadingSpinner size="lg" message="Building trace path graph..." />
          </div>
        </Card>
      ) : (
        viewModel &&
        filteredGraph && (
          <>
            <Card>
              <div className="mb-4 flex items-center gap-2">
                <Filter className="h-5 w-5 text-blue-600" />
                <h3 className="font-semibold text-gray-900 dark:text-white">
                  Graph filters
                </h3>
              </div>
              <div className="grid grid-cols-1 gap-4 md:grid-cols-5">
                <label className="space-y-1 text-sm text-gray-700 dark:text-gray-300">
                  <span>Relation</span>
                  <select
                    value={filters.relationType}
                    onChange={(event) => updateFilter("relationType", event.target.value)}
                    className="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                  >
                    <option value="all">All relations</option>
                    {relationTypes.map((type) => (
                      <option key={type} value={type}>
                        {type.replace(/_/g, " ")}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="space-y-1 text-sm text-gray-700 dark:text-gray-300">
                  <span>Entity</span>
                  <select
                    value={filters.entityType}
                    onChange={(event) => updateFilter("entityType", event.target.value)}
                    className="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                  >
                    <option value="all">All entities</option>
                    {entityTypes.map((type) => (
                      <option key={type} value={type}>
                        {type}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="space-y-1 text-sm text-gray-700 dark:text-gray-300">
                  <span>Direction</span>
                  <select
                    value={filters.direction}
                    onChange={(event) =>
                      updateFilter("direction", event.target.value as WorkspaceFilters["direction"])
                    }
                    className="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                  >
                    <option value="all">All directions</option>
                    <option value="upstream">Upstream</option>
                    <option value="downstream">Downstream</option>
                    <option value="related">Related</option>
                    <option value="unknown">Unknown</option>
                  </select>
                </label>
                <label className="space-y-1 text-sm text-gray-700 dark:text-gray-300">
                  <span>Depth</span>
                  <select
                    value={filters.depth}
                    onChange={(event) =>
                      updateFilter(
                        "depth",
                        event.target.value === "all" ? "all" : Number(event.target.value),
                      )
                    }
                    className="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                  >
                    <option value="all">All depths</option>
                    <option value="1">1 hop</option>
                    <option value="2">2 hops</option>
                    <option value="3">3 hops</option>
                  </select>
                </label>
                <label className="flex items-end gap-2 text-sm text-gray-700 dark:text-gray-300">
                  <input
                    type="checkbox"
                    checked={filters.showUnknownLinks}
                    onChange={(event) =>
                      updateFilter("showUnknownLinks", event.target.checked)
                    }
                    className="mb-3 rounded border-gray-300 dark:border-gray-600"
                  />
                  <span className="pb-2">Show unknown links</span>
                </label>
              </div>
            </Card>

            {viewModel.warnings.length > 0 && (
              <Card className="border-yellow-200 bg-yellow-50">
                <div className="flex items-start gap-3 text-yellow-900">
                  <AlertTriangle className="mt-1 h-5 w-5" />
                  <div>
                    <h3 className="font-semibold">Graph completeness warnings</h3>
                    <ul className="mt-2 list-disc space-y-1 pl-5 text-sm">
                      {viewModel.warnings.map((warning) => (
                        <li key={warning}>{warning}</li>
                      ))}
                    </ul>
                  </div>
                </div>
              </Card>
            )}

            <div className="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr)_320px]">
              <ProvenanceGraph
                itemIds={[target.id]}
                knowledgeGraph={filteredGraph}
                onNodeSelect={(node) => {
                  setSelectedNode(node);
                  setSelectedEdge(null);
                }}
                onEdgeSelect={(edge) => {
                  setSelectedEdge(edge);
                  setSelectedNode(null);
                }}
              />

              <Card>
                <div className="mb-4 flex items-center gap-2">
                  <Eye className="h-5 w-5 text-blue-600" />
                  <h3 className="font-semibold text-gray-900 dark:text-white">
                    Inspector
                  </h3>
                </div>
                {selectedNode ? (
                  <div className="space-y-3 text-sm">
                    <Badge variant="info">Node · {selectedNode.type}</Badge>
                    <h4 className="font-medium text-gray-900 dark:text-white">
                      {selectedNode.label}
                    </h4>
                    <p className="break-all font-mono text-xs text-gray-500">
                      {selectedNode.id}
                    </p>
                    <pre className="max-h-80 overflow-auto rounded bg-gray-50 p-3 text-xs text-gray-700 dark:bg-gray-800 dark:text-gray-200">
                      {JSON.stringify(selectedNode.properties, null, 2)}
                    </pre>
                  </div>
                ) : selectedEdge ? (
                  <div className="space-y-3 text-sm">
                    <Badge variant="info">Edge · {selectedEdge.type}</Badge>
                    <h4 className="font-medium text-gray-900 dark:text-white">
                      {selectedEdge.label}
                    </h4>
                    <p className="break-all font-mono text-xs text-gray-500">
                      {selectedEdge.source} → {selectedEdge.target}
                    </p>
                    <pre className="max-h-80 overflow-auto rounded bg-gray-50 p-3 text-xs text-gray-700 dark:bg-gray-800 dark:text-gray-200">
                      {JSON.stringify(selectedEdge.properties, null, 2)}
                    </pre>
                  </div>
                ) : (
                  <div className="flex items-start gap-2 text-sm text-gray-600 dark:text-gray-300">
                    <Info className="mt-0.5 h-4 w-4" />
                    Select a node or edge in the graph to inspect normalized
                    backend data, confidence, direction, and missing-data flags.
                  </div>
                )}
              </Card>
            </div>
          </>
        )
      )}
    </section>
  );
};

export default TraceGraphWorkspace;
