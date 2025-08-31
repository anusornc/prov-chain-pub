import React, { useState, useEffect, useRef } from 'react';
import cytoscape from 'cytoscape';
import Card from '../../components/ui/Card';
import Button from '../../components/ui/Button';
import Badge from '../../components/ui/Badge';
import Alert from '../../components/ui/Alert';
import LoadingSpinner from '../../components/ui/LoadingSpinner';
import { sparqlService } from '../../services/sparql';

interface GraphNode {
  id: string;
  label: string;
  type: 'Class' | 'Property' | 'Resource' | 'Literal';
}

interface GraphEdge {
  from: string;
  to: string;
  label: string;
}

type SparqlBindingTerm = {
  type: string;
  value: string;
  datatype?: string;
  'xml:lang'?: string;
};

type SparqlBinding = Record<string, SparqlBindingTerm>;

const KnowledgeGraph: React.FC = () => {
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const graphRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<cytoscape.Core | null>(null);

  useEffect(() => {
    loadGraphData();
    return () => {
      if (cyRef.current) {
        cyRef.current.destroy();
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function localName(iri: string): string {
    const parts = iri.split(/[#/]/);
    return parts[parts.length - 1] || iri;
  }

  function normalizeTerm(term: unknown): SparqlBindingTerm | undefined {
    if (!term) return undefined;
    if (typeof term === 'string') {
      return {
        type: term.startsWith('http') ? 'uri' : 'literal',
        value: term,
      };
    }
    return term as SparqlBindingTerm;
  }

  async function loadGraphData() {
    try {
      setIsLoading(true);
      setError(null);

      // Query across named graphs; include literal objects so the graph is not empty on literal-only data
      const query = `
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        SELECT ?subject ?predicate ?object ?subjectLabel ?objectLabel WHERE {
          GRAPH ?g {
            ?subject ?predicate ?object .
            FILTER(isIRI(?subject))
            OPTIONAL { ?subject rdfs:label ?subjectLabel }
            OPTIONAL { ?object rdfs:label ?objectLabel }
          }
        }
        LIMIT 300
      `;

      const response = await sparqlService.executeQuery(query);
      const bindings = (response.results?.bindings || []) as SparqlBinding[];

      const nodeSet = new Set<string>();
      const edgeList: GraphEdge[] = [];
      const nodeList: GraphNode[] = [];

      for (const raw of bindings) {
        const subject = normalizeTerm(raw.subject);
        const predicate = normalizeTerm(raw.predicate);
        const object = normalizeTerm(raw.object);
        const subjectLabelTerm = normalizeTerm(raw.subjectLabel);
        const objectLabelTerm = normalizeTerm(raw.objectLabel);

        if (!subject || !predicate || !object) continue;
        const subjId = subject.value;
        const predIri = predicate.value;

        // Determine object node id/type/label
        let objId: string;
        let objType: GraphNode['type'];
        let objLabel: string;

        if (object.type === 'uri') {
          objId = object.value;
          objType = 'Resource';
          objLabel = objectLabelTerm?.value || localName(objId);
        } else {
          // Create a stable id for literal nodes to avoid collisions
          const litKey = `${object.value}|${object.datatype || object['xml:lang'] || 'literal'}`;
          objId = `lit:${encodeURIComponent(litKey)}`;
          objType = 'Literal';
          objLabel = object.value;
        }

        // Subject node
        if (!nodeSet.has(subjId)) {
          nodeSet.add(subjId);
          nodeList.push({
            id: subjId,
            label: subjectLabelTerm?.value || (subject.type === 'uri' ? localName(subjId) : subjId),
            type: 'Resource',
          });
        }

        // Object node
        if (!nodeSet.has(objId)) {
          nodeSet.add(objId);
          nodeList.push({
            id: objId,
            label: objLabel,
            type: objType,
          });
        }

        // Edge
        edgeList.push({
          from: subjId,
          to: objId,
          label: localName(predIri),
        });
      }

      setNodes(nodeList);
      setEdges(edgeList);

      // Initialize or update Cytoscape graph
      initializeGraph(nodeList, edgeList);
    } catch (err: unknown) {
      const errorObj = err as { response?: { data?: { message?: string } }; message?: string };
      setError(
        'Failed to load graph data: ' +
          (errorObj.response?.data?.message || errorObj.message || 'Please try again.')
      );
      console.error('Error loading graph data:', err);

      // Fallback sample graph
      const sampleNodes: GraphNode[] = [
        { id: 'http://provchain.org/core#Product', label: 'Product', type: 'Class' },
        { id: 'http://provchain.org/core#Process', label: 'Process', type: 'Class' },
        { id: 'http://provchain.org/core#Participant', label: 'Participant', type: 'Class' },
        { id: 'http://provchain.org/core#Location', label: 'Location', type: 'Class' },
        { id: 'http://provchain.org/core#Transaction', label: 'Transaction', type: 'Class' },
        { id: 'lit:sample', label: 'Sample Literal', type: 'Literal' },
      ];
      const sampleEdges: GraphEdge[] = [
        { from: 'http://provchain.org/core#Product', to: 'http://provchain.org/core#Process', label: 'derivedFrom' },
        { from: 'http://provchain.org/core#Process', to: 'http://provchain.org/core#Participant', label: 'processedBy' },
        { from: 'http://provchain.org/core#Process', to: 'http://provchain.org/core#Location', label: 'locatedAt' },
        { from: 'http://provchain.org/core#Transaction', to: 'http://provchain.org/core#Product', label: 'involves' },
        { from: 'http://provchain.org/core#Transaction', to: 'lit:sample', label: 'hasNote' },
      ];

      setNodes(sampleNodes);
      setEdges(sampleEdges);
      initializeGraph(sampleNodes, sampleEdges);
    } finally {
      setIsLoading(false);
    }
  }

  function initializeGraph(nodeList: GraphNode[], edgeList: GraphEdge[]) {
    if (!graphRef.current) return;

    // Destroy existing graph if it exists
    if (cyRef.current) {
      cyRef.current.destroy();
    }

    cyRef.current = cytoscape({
      container: graphRef.current,
      elements: [
        ...nodeList.map((node) => ({
          data: {
            id: node.id,
            label: node.label,
            type: node.type,
          },
        })),
        ...edgeList.map((edge, index) => ({
          data: {
            id: `edge-${index}`,
            source: edge.from,
            target: edge.to,
            label: edge.label,
          },
        })),
      ],
      style: [
        {
          selector: 'node',
          style: {
            'background-color': '#667eea',
            label: 'data(label)',
            'text-valign': 'center',
            'text-halign': 'center',
            color: 'white',
            'font-size': '12px',
            width: '60px',
            height: '60px',
            'border-width': '2px',
            'border-color': '#fff',
          },
        },
        {
          selector: 'node[type="Class"]',
          style: {
            'background-color': '#48bb78',
            shape: 'rectangle',
          },
        },
        {
          selector: 'node[type="Property"]',
          style: {
            'background-color': '#ed8936',
            shape: 'diamond',
          },
        },
        {
          selector: 'node[type="Resource"]',
          style: {
            'background-color': '#4299e1',
            shape: 'ellipse',
          },
        },
        {
          selector: 'node[type="Literal"]',
          style: {
            'background-color': '#a0aec0',
            shape: 'round-rectangle',
            color: '#1a202c',
          },
        },
        {
          selector: 'edge',
          style: {
            width: 2,
            'line-color': '#a0aec0',
            'target-arrow-color': '#a0aec0',
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier',
            label: 'data(label)',
            'text-rotation': 'autorotate',
            color: '#4a5568',
            'font-size': '10px',
          },
        },
      ],
      layout: {
        name: 'cose',
        idealEdgeLength: 100,
        nodeOverlap: 20,
        refresh: 20,
        fit: true,
        padding: 30,
        randomize: false,
        componentSpacing: 100,
        nodeRepulsion: 400000,
        edgeElasticity: 100,
        nestingFactor: 5,
        gravity: 80,
        numIter: 1000,
        initialTemp: 200,
        coolingFactor: 0.95,
        minTemp: 1.0,
      },
    });

    // Add event listeners
    cyRef.current.on('tap', 'node', (event) => {
      const node = event.target;
      setSelectedNode(node.id());
    });

    cyRef.current.on('tap', (event) => {
      if (event.target === cyRef.current) {
        setSelectedNode(null);
      }
    });
  }

  function getNodeTypeVariant(
    type: string
  ): 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info' | 'default' {
    const variants: Record<string, 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info' | 'default'> = {
      Class: 'success',
      Property: 'warning',
      Resource: 'info',
      Literal: 'secondary',
    };
    return variants[type] || 'default';
  }

  const refreshGraph = () => {
    loadGraphData();
  };

  if (isLoading) {
    return (
      <div className="feature-container">
        <LoadingSpinner size="lg" message="Loading Knowledge Graph..." />
      </div>
    );
  }

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">Knowledge Graph</h2>
          <p className="feature-description">Visualize relationships and connections in your semantic data</p>
        </div>
        <Button variant="secondary" onClick={refreshGraph}>
          🔄 Refresh Graph
        </Button>
      </div>

      {error && (
        <div className="mb-6">
          <Alert variant="error" message={error} dismissible onClose={() => setError(null)} />
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Graph Visualization">
          <div
            ref={graphRef}
            className="h-[500px] bg-gray-50 dark:bg-gray-800 rounded-lg border-2 border-gray-200 dark:border-gray-700"
          />
        </Card>

        <Card title="Graph Statistics">
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4 text-center">
              <div className="text-2xl mb-1">🔢</div>
              <div className="text-xl font-bold text-blue-600 dark:text-blue-400">{nodes.length}</div>
              <div className="text-xs text-gray-600 dark:text-gray-300">Total Nodes</div>
            </div>
            <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4 text-center">
              <div className="text-2xl mb-1">🔗</div>
              <div className="text-xl font-bold text-green-600 dark:text-green-400">{edges.length}</div>
              <div className="text-xs text-gray-600 dark:text-gray-300">Total Edges</div>
            </div>
            <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4 text-center">
              <div className="text-2xl mb-1">🏷️</div>
              <div className="text-xl font-bold text-purple-600 dark:text-purple-400">
                {new Set(nodes.map((n) => n.type)).size}
              </div>
              <div className="text-xs text-gray-600 dark:text-gray-300">Node Types</div>
            </div>
            <div className="bg-yellow-50 dark:bg-yellow-900/20 rounded-lg p-4 text-center">
              <div className="text-2xl mb-1">📊</div>
              <div className="text-xl font-bold text-yellow-600 dark:text-yellow-400">
                {new Set(edges.map((e) => e.label)).size}
              </div>
              <div className="text-xs text-gray-600 dark:text-gray-300">Relationship Types</div>
            </div>
          </div>

          {selectedNode && (
            <div className="mt-6">
              <h4 className="text-lg font-medium text-gray-900 dark:text-white mb-3">Selected Node Details</h4>
              {(() => {
                const node = nodes.find((n) => n.id === selectedNode);
                if (!node) return null;

                return (
                  <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
                    <div className="mb-2">
                      <span className="font-medium text-gray-700 dark:text-gray-300">ID:</span>
                      <span className="font-mono text-sm ml-2">{node.id}</span>
                    </div>
                    <div className="mb-2">
                      <span className="font-medium text-gray-700 dark:text-gray-300">Label:</span> {node.label}
                    </div>
                    <div>
                      <span className="font-medium text-gray-700 dark:text-gray-300">Type:</span>
                      <Badge variant={getNodeTypeVariant(node.type)} className="ml-2">
                        {node.type}
                      </Badge>
                    </div>
                  </div>
                );
              })()}
            </div>
          )}
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
        <Card title="Nodes">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    ID
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Label
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Type
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {nodes.map((node) => (
                  <tr
                    key={node.id}
                    onClick={() => setSelectedNode(node.id)}
                    className={`cursor-pointer hover:bg-gray-50 ${selectedNode === node.id ? 'bg-blue-50' : ''}`}
                  >
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant="primary" title={node.id}>
                        {node.id.length > 30 ? node.id.substring(0, 30) + '...' : node.id}
                      </Badge>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{node.label}</td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant={getNodeTypeVariant(node.type)}>{node.type}</Badge>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        <Card title="Relationships">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    From
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    To
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Relationship
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {edges.map((edge, index) => (
                  <tr key={index} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant="primary" title={edge.from}>
                        {edge.from.length > 20 ? edge.from.substring(0, 20) + '...' : edge.from}
                      </Badge>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant="primary" title={edge.to}>
                        {edge.to.length > 20 ? edge.to.substring(0, 20) + '...' : edge.to}
                      </Badge>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant="success">{edge.label}</Badge>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      </div>

      {selectedNode && (
        <Card title={`Node Connections: ${selectedNode}`} className="mt-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h5 className="font-medium text-gray-900 dark:text-white mb-3">Incoming Connections</h5>
              <ul className="space-y-2">
                {edges
                  .filter((edge) => edge.to === selectedNode)
                  .map((edge, index) => (
                    <li key={index} className="flex items-center">
                      <Badge variant="primary" className="mr-2" title={edge.from}>
                        {edge.from.length > 20 ? edge.from.substring(0, 20) + '...' : edge.from}
                      </Badge>
                      <span className="mx-2">→</span>
                      <Badge variant="success">{edge.label}</Badge>
                    </li>
                  ))}
              </ul>
            </div>
            <div>
              <h5 className="font-medium text-gray-900 dark:text-white mb-3">Outgoing Connections</h5>
              <ul className="space-y-2">
                {edges
                  .filter((edge) => edge.from === selectedNode)
                  .map((edge, index) => (
                    <li key={index} className="flex items-center">
                      <Badge variant="success" className="mr-2">
                        {edge.label}
                      </Badge>
                      <span className="mx-2">→</span>
                      <Badge variant="primary" title={edge.to}>
                        {edge.to.length > 20 ? edge.to.substring(0, 20) + '...' : edge.to}
                      </Badge>
                    </li>
                  ))}
              </ul>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
};

export default KnowledgeGraph;
