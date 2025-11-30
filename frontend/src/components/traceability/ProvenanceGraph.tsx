import React, { useState, useEffect, useRef, useCallback } from "react";
import { useTraceability } from "../../hooks/useTraceability";
import cytoscape from "cytoscape";
import type { Core } from "cytoscape";
// @ts-expect-error - No type definitions available
import dagre from "cytoscape-dagre";
// @ts-expect-error - No type definitions available
import coseBilkent from "cytoscape-cose-bilkent";
import {
  Network,
  ZoomIn,
  ZoomOut,
  RotateCcw,
  Download,
  Filter,
  Layout,
  Settings,
  Maximize2,
  RefreshCw,
} from "lucide-react";
import Button from "../ui/Button";
import Card from "../ui/Card";
import Badge from "../ui/Badge";
import LoadingSpinner from "../ui/LoadingSpinner";
import Alert from "../ui/Alert";
import type {
  KnowledgeGraph,
  KnowledgeGraphNode,
  KnowledgeGraphEdge,
} from "../../types";

// Register Cytoscape extensions
cytoscape.use(dagre);
cytoscape.use(coseBilkent);

interface ProvenanceGraphProps {
  itemIds: string[];
  knowledgeGraph?: KnowledgeGraph;
  onNodeSelect?: (node: KnowledgeGraphNode) => void;
  onEdgeSelect?: (edge: KnowledgeGraphEdge) => void;
  onItemSelect?: (itemId: string) => void;
  className?: string;
}

interface CytoscapeStyleProperties {
  "background-color"?: string;
  "border-color"?: string;
  "border-width"?: number;
  color?: string;
  "text-valign"?: string;
  "text-halign"?: string;
  "font-size"?: string;
  "font-weight"?: string;
  "line-color"?: string;
  "target-arrow-color"?: string;
  "target-arrow-shape"?: string;
  "curve-style"?: string;
  width?: number | string;
  "text-rotation"?: string;
  "text-margin-y"?: number;
  [key: string]: unknown;
}

interface LayoutOptions {
  name: string;
  rankDir?: string;
  spacingFactor?: number;
  nodeSep?: number;
  rankSep?: number;
  animate?: boolean;
  animationDuration?: number;
  nodeRepulsion?: number;
  idealEdgeLength?: number;
  edgeElasticity?: number;
  nestingFactor?: number;
  radius?: number;
  [key: string]: unknown;
}

interface GraphConfig {
  width: number;
  height: number;
  layouts: LayoutOption[];
  nodeStyles: Record<string, CytoscapeStyleProperties>;
  edgeStyles: Record<string, CytoscapeStyleProperties>;
}

interface LayoutOption {
  name: string;
  label: string;
  icon: string;
  options: LayoutOptions;
}

interface GraphFilters {
  nodeTypes: Set<string>;
  edgeTypes: Set<string>;
  showLabels: boolean;
  showEdgeLabels: boolean;
  minNodeSize: number;
  maxNodeSize: number;
}

const ProvenanceGraph: React.FC<ProvenanceGraphProps> = ({
  itemIds,
  knowledgeGraph,
  onNodeSelect,
  onEdgeSelect,
  onItemSelect,
  className = "",
}) => {
  const {
    loadKnowledgeGraph,
    graphLoading,
    graphError,
    knowledgeGraph: hookKnowledgeGraph,
  } = useTraceability();
  const cyRef = useRef<Core | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const [selectedNode, setSelectedNode] = useState<KnowledgeGraphNode | null>(
    null,
  );
  const [selectedEdge, setSelectedEdge] = useState<KnowledgeGraphEdge | null>(
    null,
  );
  const [currentLayout, setCurrentLayout] = useState<string>("dagre");
  const [showFilters, setShowFilters] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [filters, setFilters] = useState<GraphFilters>({
    nodeTypes: new Set(),
    edgeTypes: new Set(),
    showLabels: true,
    showEdgeLabels: false,
    minNodeSize: 20,
    maxNodeSize: 80,
  });

  // Use provided knowledgeGraph or hook's knowledgeGraph
  const graphData = knowledgeGraph || hookKnowledgeGraph;

  const config: GraphConfig = {
    width: isFullscreen ? window.innerWidth : 1200,
    height: isFullscreen ? window.innerHeight : 700,
    layouts: [
      {
        name: "dagre",
        label: "Hierarchical",
        icon: "git-branch",
        options: {
          name: "dagre",
          rankDir: "TB",
          spacingFactor: 1.2,
          nodeSep: 50,
          rankSep: 100,
          animate: true,
          animationDuration: 500,
        },
      },
      {
        name: "cose-bilkent",
        label: "Force-Directed",
        icon: "target",
        options: {
          name: "cose-bilkent",
          animate: true,
          animationDuration: 1000,
          nodeRepulsion: 4500,
          idealEdgeLength: 100,
          edgeElasticity: 0.45,
          nestingFactor: 0.1,
        },
      },
      {
        name: "circle",
        label: "Circular",
        icon: "circle",
        options: {
          name: "circle",
          animate: true,
          animationDuration: 500,
          radius: 200,
          spacingFactor: 1.5,
        },
      },
      {
        name: "grid",
        label: "Grid",
        icon: "grid-3x3",
        options: {
          name: "grid",
          animate: true,
          animationDuration: 500,
          spacingFactor: 1.2,
        },
      },
    ],
    nodeStyles: {
      item: {
        "background-color": "#3b82f6",
        "border-color": "#1d4ed8",
        "border-width": 2,
        color: "#ffffff",
        "text-valign": "center",
        "text-halign": "center",
        "font-size": "12px",
        "font-weight": "bold",
      },
      participant: {
        "background-color": "#10b981",
        "border-color": "#047857",
        "border-width": 2,
        color: "#ffffff",
        "text-valign": "center",
        "text-halign": "center",
        "font-size": "12px",
        "font-weight": "bold",
      },
      location: {
        "background-color": "#f59e0b",
        "border-color": "#d97706",
        "border-width": 2,
        color: "#ffffff",
        "text-valign": "center",
        "text-halign": "center",
        "font-size": "12px",
        "font-weight": "bold",
      },
      process: {
        "background-color": "#8b5cf6",
        "border-color": "#7c3aed",
        "border-width": 2,
        color: "#ffffff",
        "text-valign": "center",
        "text-halign": "center",
        "font-size": "12px",
        "font-weight": "bold",
      },
    },
    edgeStyles: {
      default: {
        "line-color": "#6b7280",
        "target-arrow-color": "#6b7280",
        "target-arrow-shape": "triangle",
        "curve-style": "bezier",
        width: 2,
        "font-size": "10px",
        "text-rotation": "autorotate",
        "text-margin-y": -10,
      },
      selected: {
        "line-color": "#3b82f6",
        "target-arrow-color": "#3b82f6",
        width: 4,
      },
    },
  };

  // Load knowledge graph if not provided
  useEffect(() => {
    const loadGraphData = async () => {
      if (!knowledgeGraph && !hookKnowledgeGraph && itemIds.length > 0) {
        try {
          await loadKnowledgeGraph(itemIds);
        } catch (err) {
          console.error("Failed to load knowledge graph:", err);
        }
      }
    };

    loadGraphData();
  }, [itemIds, knowledgeGraph, hookKnowledgeGraph, loadKnowledgeGraph]);

  // Initialize filters when graph data changes
  useEffect(() => {
    if (graphData) {
      const nodeTypes = new Set(graphData.nodes.map((node) => node.type));
      const edgeTypes = new Set(graphData.edges.map((edge) => edge.type));

      setFilters((prev) => ({
        ...prev,
        nodeTypes: nodeTypes,
        edgeTypes: edgeTypes,
      }));
    }
  }, [graphData]);

  // Filter graph data based on current filters
  const getFilteredGraphData = useCallback((): {
    nodes: KnowledgeGraphNode[];
    edges: KnowledgeGraphEdge[];
  } => {
    if (!graphData) return { nodes: [], edges: [] };

    const filteredNodes = graphData.nodes.filter(
      (node) =>
        filters.nodeTypes.size === 0 || filters.nodeTypes.has(node.type),
    );

    const nodeIds = new Set(filteredNodes.map((node) => node.id));
    const filteredEdges = graphData.edges.filter(
      (edge) =>
        (filters.edgeTypes.size === 0 || filters.edgeTypes.has(edge.type)) &&
        nodeIds.has(edge.source) &&
        nodeIds.has(edge.target),
    );

    return { nodes: filteredNodes, edges: filteredEdges };
  }, [graphData, filters]);

  // Convert graph data to Cytoscape format
  const convertToCytoscapeFormat = useCallback(
    (nodes: KnowledgeGraphNode[], edges: KnowledgeGraphEdge[]) => {
      const cytoscapeNodes = nodes.map((node) => ({
        data: {
          id: node.id,
          label: filters.showLabels ? node.label : "",
          type: node.type,
          properties: node.properties,
          size: Math.max(
            filters.minNodeSize,
            Math.min(filters.maxNodeSize, node.size || 40),
          ),
        },
        position: node.x && node.y ? { x: node.x, y: node.y } : undefined,
      }));

      const cytoscapeEdges = edges.map((edge) => ({
        data: {
          id: edge.id,
          source: edge.source,
          target: edge.target,
          label: filters.showEdgeLabels ? edge.label : "",
          type: edge.type,
          properties: edge.properties,
          weight: edge.weight || 1,
        },
      }));

      return [...cytoscapeNodes, ...cytoscapeEdges];
    },
    [filters],
  );

  // Initialize Cytoscape
  const initializeCytoscape = useCallback(() => {
    if (!containerRef.current || !graphData) return;

    const { nodes, edges } = getFilteredGraphData();
    const elements = convertToCytoscapeFormat(nodes, edges);

    // Destroy existing instance
    if (cyRef.current) {
      cyRef.current.destroy();
    }

    // Create new Cytoscape instance
    const cy = cytoscape({
      container: containerRef.current,
      elements,
      style: [
        {
          selector: "node",
          style: {
            width: "data(size)",
            height: "data(size)",
            label: "data(label)",
            ...config.nodeStyles.item,
          },
        },
        {
          selector: 'node[type="item"]',
          style: config.nodeStyles.item as Record<string, unknown>,
        },
        {
          selector: 'node[type="participant"]',
          style: config.nodeStyles.participant as Record<string, unknown>,
        },
        {
          selector: 'node[type="location"]',
          style: config.nodeStyles.location as Record<string, unknown>,
        },
        {
          selector: 'node[type="process"]',
          style: config.nodeStyles.process as Record<string, unknown>,
        },
        {
          selector: "edge",
          style: {
            label: "data(label)",
            ...config.edgeStyles.default,
          },
        },
        {
          selector: "node:selected",
          style: {
            "border-width": 4,
            "border-color": "#3b82f6",
            "background-color": "#dbeafe",
          },
        },
        {
          selector: "edge:selected",
          style: config.edgeStyles.selected as Record<string, unknown>,
        },
      ],
      layout:
        config.layouts.find((l) => l.name === currentLayout)?.options ||
        config.layouts[0].options,
      wheelSensitivity: 0.2,
      minZoom: 0.1,
      maxZoom: 3,
    });

    // Event handlers
    cy.on("tap", "node", (event) => {
      const node = event.target;
      const nodeData = node.data();

      // Find the original node data
      const originalNode = nodes.find((n) => n.id === nodeData.id);
      if (originalNode) {
        setSelectedNode(originalNode);
        setSelectedEdge(null);
        if (onNodeSelect) onNodeSelect(originalNode);
        if (originalNode.type === "item" && onItemSelect) {
          onItemSelect(originalNode.id);
        }
      }
    });

    cy.on("tap", "edge", (event) => {
      const edge = event.target;
      const edgeData = edge.data();

      // Find the original edge data
      const originalEdge = edges.find((e) => e.id === edgeData.id);
      if (originalEdge) {
        setSelectedEdge(originalEdge);
        setSelectedNode(null);
        if (onEdgeSelect) onEdgeSelect(originalEdge);
      }
    });

    cy.on("tap", (event) => {
      if (event.target === cy) {
        setSelectedNode(null);
        setSelectedEdge(null);
      }
    });

    cyRef.current = cy;
  }, [
    graphData,
    currentLayout,
    filters,
    getFilteredGraphData,
    convertToCytoscapeFormat,
    config,
    onNodeSelect,
    onEdgeSelect,
    onItemSelect,
  ]);

  // Handle layout change
  const handleLayoutChange = (layoutName: string) => {
    setCurrentLayout(layoutName);
    if (cyRef.current) {
      const layoutOptions = config.layouts.find(
        (l) => l.name === layoutName,
      )?.options;
      if (layoutOptions) {
        cyRef.current.layout(layoutOptions).run();
      }
    }
  };

  // Handle zoom controls
  const handleZoomIn = () => cyRef.current?.zoom(cyRef.current.zoom() * 1.2);
  const handleZoomOut = () => cyRef.current?.zoom(cyRef.current.zoom() / 1.2);
  const handleResetZoom = () => cyRef.current?.fit();

  // Handle export
  const handleExport = () => {
    if (!cyRef.current) return;

    const png = cyRef.current.png({
      output: "blob",
      bg: "#ffffff",
      full: true,
      scale: 2,
    });

    const link = document.createElement("a");
    link.download = `provenance-graph-${itemIds.join("-")}.png`;
    link.href = URL.createObjectURL(png);
    link.click();
  };

  // Handle fullscreen toggle
  const handleFullscreenToggle = () => {
    setIsFullscreen(!isFullscreen);
    setTimeout(() => {
      if (cyRef.current) {
        cyRef.current.resize();
        cyRef.current.fit();
      }
    }, 100);
  };

  // Handle refresh
  const handleRefresh = () => {
    if (itemIds.length > 0) {
      loadKnowledgeGraph(itemIds);
    }
  };

  // Filter toggle functions
  const toggleNodeType = (nodeType: string) => {
    setFilters((prev) => {
      const newNodeTypes = new Set(prev.nodeTypes);
      if (newNodeTypes.has(nodeType)) {
        newNodeTypes.delete(nodeType);
      } else {
        newNodeTypes.add(nodeType);
      }
      return { ...prev, nodeTypes: newNodeTypes };
    });
  };

  const toggleEdgeType = (edgeType: string) => {
    setFilters((prev) => {
      const newEdgeTypes = new Set(prev.edgeTypes);
      if (newEdgeTypes.has(edgeType)) {
        newEdgeTypes.delete(edgeType);
      } else {
        newEdgeTypes.add(edgeType);
      }
      return { ...prev, edgeTypes: newEdgeTypes };
    });
  };

  // Get unique node and edge types
  const getUniqueNodeTypes = (): string[] => {
    if (!graphData) return [];
    return Array.from(new Set(graphData.nodes.map((node) => node.type))).sort();
  };

  const getUniqueEdgeTypes = (): string[] => {
    if (!graphData) return [];
    return Array.from(new Set(graphData.edges.map((edge) => edge.type))).sort();
  };

  // Initialize and update Cytoscape when data changes
  useEffect(() => {
    initializeCytoscape();
  }, [initializeCytoscape]);

  // Handle window resize for fullscreen
  useEffect(() => {
    const handleResize = () => {
      if (isFullscreen && cyRef.current) {
        cyRef.current.resize();
      }
    };

    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [isFullscreen]);

  if (graphLoading) {
    return (
      <div className="flex justify-center items-center h-96">
        <LoadingSpinner size="lg" message="Loading provenance graph..." />
      </div>
    );
  }

  if (graphError) {
    return <Alert variant="error" message={graphError} />;
  }

  if (!graphData) {
    return (
      <Card className="p-8 text-center">
        <Network className="w-16 h-16 text-gray-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
          No Graph Data
        </h3>
        <p className="text-gray-600 dark:text-gray-300">
          Knowledge graph will appear here once items are selected.
        </p>
      </Card>
    );
  }

  const { nodes, edges } = getFilteredGraphData();

  return (
    <div
      className={`space-y-6 ${isFullscreen ? "fixed inset-0 z-50 bg-white dark:bg-gray-900 p-6" : ""} ${className}`}
    >
      {/* Header with Controls */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Provenance Knowledge Graph
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mt-1">
            Interactive knowledge graph showing relationships and provenance
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            onClick={() => setShowFilters(!showFilters)}
            className="flex items-center gap-2"
          >
            <Filter className="w-4 h-4" />
            Filters
          </Button>

          <Button
            variant="outline"
            onClick={() => setShowSettings(!showSettings)}
            className="flex items-center gap-2"
          >
            <Settings className="w-4 h-4" />
            Settings
          </Button>

          <div className="flex items-center gap-1 border border-gray-300 dark:border-gray-600 rounded-md">
            <Button variant="outline" size="sm" onClick={handleZoomOut}>
              <ZoomOut className="w-4 h-4" />
            </Button>
            <Button variant="outline" size="sm" onClick={handleResetZoom}>
              <RotateCcw className="w-4 h-4" />
            </Button>
            <Button variant="outline" size="sm" onClick={handleZoomIn}>
              <ZoomIn className="w-4 h-4" />
            </Button>
          </div>

          <Button
            variant="outline"
            onClick={handleRefresh}
            className="flex items-center gap-2"
          >
            <RefreshCw className="w-4 h-4" />
            Refresh
          </Button>

          <Button
            variant="outline"
            onClick={handleExport}
            className="flex items-center gap-2"
          >
            <Download className="w-4 h-4" />
            Export
          </Button>

          <Button
            variant="outline"
            onClick={handleFullscreenToggle}
            className="flex items-center gap-2"
          >
            <Maximize2 className="w-4 h-4" />
            {isFullscreen ? "Exit" : "Fullscreen"}
          </Button>
        </div>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <Card className="p-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                Node Types
              </h4>
              <div className="space-y-2">
                {getUniqueNodeTypes().map((nodeType) => (
                  <label key={nodeType} className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={
                        filters.nodeTypes.size === 0 ||
                        filters.nodeTypes.has(nodeType)
                      }
                      onChange={() => toggleNodeType(nodeType)}
                      className="rounded border-gray-300 dark:border-gray-600"
                    />
                    <span className="text-sm text-gray-700 dark:text-gray-300 capitalize">
                      {nodeType}
                    </span>
                    <Badge variant="default" className="text-xs">
                      {
                        graphData.nodes.filter((n) => n.type === nodeType)
                          .length
                      }
                    </Badge>
                  </label>
                ))}
              </div>
            </div>

            <div>
              <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                Edge Types
              </h4>
              <div className="space-y-2">
                {getUniqueEdgeTypes().map((edgeType) => (
                  <label key={edgeType} className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={
                        filters.edgeTypes.size === 0 ||
                        filters.edgeTypes.has(edgeType)
                      }
                      onChange={() => toggleEdgeType(edgeType)}
                      className="rounded border-gray-300 dark:border-gray-600"
                    />
                    <span className="text-sm text-gray-700 dark:text-gray-300 capitalize">
                      {edgeType.replace(/_/g, " ")}
                    </span>
                    <Badge variant="default" className="text-xs">
                      {
                        graphData.edges.filter((e) => e.type === edgeType)
                          .length
                      }
                    </Badge>
                  </label>
                ))}
              </div>
            </div>
          </div>

          <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
            <div className="flex items-center gap-4">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={filters.showLabels}
                  onChange={(e) =>
                    setFilters((prev) => ({
                      ...prev,
                      showLabels: e.target.checked,
                    }))
                  }
                  className="rounded border-gray-300 dark:border-gray-600"
                />
                <span className="text-sm text-gray-700 dark:text-gray-300">
                  Show Node Labels
                </span>
              </label>

              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={filters.showEdgeLabels}
                  onChange={(e) =>
                    setFilters((prev) => ({
                      ...prev,
                      showEdgeLabels: e.target.checked,
                    }))
                  }
                  className="rounded border-gray-300 dark:border-gray-600"
                />
                <span className="text-sm text-gray-700 dark:text-gray-300">
                  Show Edge Labels
                </span>
              </label>
            </div>
          </div>
        </Card>
      )}

      {/* Settings Panel */}
      {showSettings && (
        <Card className="p-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                Layout Algorithm
              </h4>
              <div className="grid grid-cols-2 gap-2">
                {config.layouts.map((layout) => (
                  <Button
                    key={layout.name}
                    variant={
                      currentLayout === layout.name ? "primary" : "outline"
                    }
                    size="sm"
                    onClick={() => handleLayoutChange(layout.name)}
                    className="flex items-center gap-2"
                  >
                    <Layout className="w-4 h-4" />
                    {layout.label}
                  </Button>
                ))}
              </div>
            </div>

            <div>
              <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                Node Size Range
              </h4>
              <div className="space-y-3">
                <div>
                  <label className="block text-sm text-gray-700 dark:text-gray-300 mb-1">
                    Minimum Size: {filters.minNodeSize}px
                  </label>
                  <input
                    type="range"
                    min="10"
                    max="50"
                    value={filters.minNodeSize}
                    onChange={(e) =>
                      setFilters((prev) => ({
                        ...prev,
                        minNodeSize: parseInt(e.target.value),
                      }))
                    }
                    className="w-full"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-700 dark:text-gray-300 mb-1">
                    Maximum Size: {filters.maxNodeSize}px
                  </label>
                  <input
                    type="range"
                    min="50"
                    max="120"
                    value={filters.maxNodeSize}
                    onChange={(e) =>
                      setFilters((prev) => ({
                        ...prev,
                        maxNodeSize: parseInt(e.target.value),
                      }))
                    }
                    className="w-full"
                  />
                </div>
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Graph Visualization */}
      <Card className="p-6">
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Badge variant="info">{nodes.length} nodes</Badge>
            <Badge variant="default">{edges.length} edges</Badge>
            <Badge variant="default">
              Layout:{" "}
              {config.layouts.find((l) => l.name === currentLayout)?.label}
            </Badge>
          </div>

          <div className="text-sm text-gray-600 dark:text-gray-300">
            Click nodes and edges to explore relationships
          </div>
        </div>

        <div
          ref={containerRef}
          className="w-full border border-gray-200 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800"
          style={{
            height: isFullscreen ? "calc(100vh - 200px)" : `${config.height}px`,
            width: "100%",
          }}
        />
      </Card>

      {/* Selected Node Details */}
      {selectedNode && (
        <Card className="p-6">
          <div className="flex items-start justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Node Details
            </h3>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setSelectedNode(null)}
            >
              ×
            </Button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center gap-3">
                <div
                  className="w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-bold"
                  style={{
                    backgroundColor:
                      config.nodeStyles[selectedNode.type]?.[
                        "background-color"
                      ] || "#6b7280",
                  }}
                >
                  {selectedNode.type.charAt(0).toUpperCase()}
                </div>
                <div>
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    {selectedNode.label}
                  </h4>
                  <p className="text-sm text-gray-600 dark:text-gray-300 capitalize">
                    {selectedNode.type}
                  </p>
                </div>
              </div>

              <div className="space-y-2">
                <div className="text-sm">
                  <span className="text-gray-600 dark:text-gray-300">ID:</span>
                  <span className="text-gray-900 dark:text-white ml-2 font-mono">
                    {selectedNode.id}
                  </span>
                </div>
              </div>
            </div>

            <div>
              <h5 className="font-medium text-gray-900 dark:text-white mb-2">
                Properties
              </h5>
              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
                <pre className="text-xs text-gray-600 dark:text-gray-300 whitespace-pre-wrap">
                  {JSON.stringify(selectedNode.properties, null, 2)}
                </pre>
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Selected Edge Details */}
      {selectedEdge && (
        <Card className="p-6">
          <div className="flex items-start justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Edge Details
            </h3>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setSelectedEdge(null)}
            >
              ×
            </Button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <h4 className="font-medium text-gray-900 dark:text-white">
                  {selectedEdge.label}
                </h4>
                <p className="text-sm text-gray-600 dark:text-gray-300 capitalize">
                  {selectedEdge.type.replace(/_/g, " ")}
                </p>
              </div>

              <div className="space-y-2">
                <div className="text-sm">
                  <span className="text-gray-600 dark:text-gray-300">
                    Source:
                  </span>
                  <span className="text-gray-900 dark:text-white ml-2 font-mono">
                    {selectedEdge.source}
                  </span>
                </div>
                <div className="text-sm">
                  <span className="text-gray-600 dark:text-gray-300">
                    Target:
                  </span>
                  <span className="text-gray-900 dark:text-white ml-2 font-mono">
                    {selectedEdge.target}
                  </span>
                </div>
                {selectedEdge.weight && (
                  <div className="text-sm">
                    <span className="text-gray-600 dark:text-gray-300">
                      Weight:
                    </span>
                    <span className="text-gray-900 dark:text-white ml-2">
                      {selectedEdge.weight}
                    </span>
                  </div>
                )}
              </div>
            </div>

            <div>
              <h5 className="font-medium text-gray-900 dark:text-white mb-2">
                Properties
              </h5>
              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
                <pre className="text-xs text-gray-600 dark:text-gray-300 whitespace-pre-wrap">
                  {JSON.stringify(selectedEdge.properties, null, 2)}
                </pre>
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Graph Summary */}
      <Card className="p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Graph Summary
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
              {nodes.length}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-300">
              Nodes
            </div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {edges.length}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-300">
              Edges
            </div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
              {getUniqueNodeTypes().length}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-300">
              Node Types
            </div>
          </div>

          <div className="text-center">
            <div className="text-2xl font-bold text-orange-600 dark:text-orange-400">
              {getUniqueEdgeTypes().length}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-300">
              Edge Types
            </div>
          </div>
        </div>

        {graphData && (
          <div className="mt-6 pt-4 border-t border-gray-200 dark:border-gray-700">
            <div className="text-sm text-gray-600 dark:text-gray-300">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <span className="font-medium">Query Time:</span>{" "}
                  {graphData.metadata.query_time_ms}ms
                </div>
                <div>
                  <span className="font-medium">Created:</span>{" "}
                  {new Date(graphData.metadata.created_at).toLocaleString()}
                </div>
              </div>
            </div>
          </div>
        )}
      </Card>
    </div>
  );
};

export default ProvenanceGraph;
