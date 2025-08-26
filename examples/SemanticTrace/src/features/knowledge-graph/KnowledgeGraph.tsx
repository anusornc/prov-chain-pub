import React, { useState, useEffect, useRef } from 'react';
import './KnowledgeGraph.css';
import { persistence } from '../../utils/persistence';

interface Node {
  id: string;
  label: string;
  type: string;
  x: number;
  y: number;
  fx?: number;
  fy?: number;
}

interface Edge {
  source: string;
  target: string;
  label: string;
  type: string;
}

interface GraphData {
  nodes: Node[];
  edges: Edge[];
}

export const KnowledgeGraph: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [graphData, setGraphData] = useState<GraphData>({ nodes: [], edges: [] });
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragNode, setDragNode] = useState<Node | null>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [showSettings, setShowSettings] = useState(false);
  const [layoutSettings, setLayoutSettings] = useState({
    nodeSize: 20,
    linkDistance: 100,
    repulsion: 30,
    showLabels: true,
    physics: true
  });

  useEffect(() => {
    loadGraphData();
  }, []);

  useEffect(() => {
    if (layoutSettings.physics) {
      const interval = setInterval(() => {
        simulatePhysics();
      }, 50);
      return () => clearInterval(interval);
    }
  }, [graphData, layoutSettings.physics]);

  useEffect(() => {
    drawGraph();
  }, [graphData, selectedNode, zoom, pan, layoutSettings]);

  const loadGraphData = async () => {
    try {
      const [triplesData, ontologyData] = await Promise.all([
        persistence.getItem('rdf-triples'),
        persistence.getItem('ontology')
      ]);

      const triples = triplesData ? JSON.parse(triplesData) : [];
      const ontology = ontologyData ? JSON.parse(ontologyData) : { classes: [], properties: [] };

      const nodes = new Map<string, Node>();
      const edges: Edge[] = [];

      // Add nodes from triples
      triples.forEach((triple: any) => {
        // Add subject node
        if (!nodes.has(triple.subject)) {
          nodes.set(triple.subject, {
            id: triple.subject,
            label: getNodeLabel(triple.subject),
            type: getNodeType(triple.subject, triples),
            x: Math.random() * 800,
            y: Math.random() * 600
          });
        }

        // Add object node if it's not a literal
        if (!triple.object.startsWith('"') && !triple.object.startsWith('xsd:')) {
          if (!nodes.has(triple.object)) {
            nodes.set(triple.object, {
              id: triple.object,
              label: getNodeLabel(triple.object),
              type: getNodeType(triple.object, triples),
              x: Math.random() * 800,
              y: Math.random() * 600
            });
          }

          // Add edge
          edges.push({
            source: triple.subject,
            target: triple.object,
            label: triple.predicate,
            type: 'relation'
          });
        }
      });

      setGraphData({
        nodes: Array.from(nodes.values()),
        edges
      });
    } catch (error) {
      console.error('Error loading graph data:', error);
    }
  };

  const getNodeLabel = (nodeId: string): string => {
    const parts = nodeId.split(':');
    return parts[parts.length - 1].replace(/-/g, ' ');
  };

  const getNodeType = (nodeId: string, triples: any[]): string => {
    const typeTriple = triples.find(t => 
      t.subject === nodeId && t.predicate === 'rdf:type'
    );
    return typeTriple ? typeTriple.object : 'Unknown';
  };

  const simulatePhysics = () => {
    setGraphData(prevData => {
      const newNodes = [...prevData.nodes];
      
      // Apply forces
      newNodes.forEach((node, i) => {
        if (node.fx !== undefined && node.fy !== undefined) return; // Skip fixed nodes
        
        let fx = 0, fy = 0;
        
        // Repulsion between nodes
        newNodes.forEach((other, j) => {
          if (i !== j) {
            const dx = node.x - other.x;
            const dy = node.y - other.y;
            const distance = Math.sqrt(dx * dx + dy * dy) || 1;
            const force = layoutSettings.repulsion / (distance * distance);
            fx += (dx / distance) * force;
            fy += (dy / distance) * force;
          }
        });
        
        // Attraction along edges
        prevData.edges.forEach(edge => {
          let other: Node | undefined;
          let isSource = false;
          
          if (edge.source === node.id) {
            other = newNodes.find(n => n.id === edge.target);
            isSource = true;
          } else if (edge.target === node.id) {
            other = newNodes.find(n => n.id === edge.source);
          }
          
          if (other) {
            const dx = other.x - node.x;
            const dy = other.y - node.y;
            const distance = Math.sqrt(dx * dx + dy * dy) || 1;
            const force = (distance - layoutSettings.linkDistance) * 0.01;
            fx += (dx / distance) * force;
            fy += (dy / distance) * force;
          }
        });
        
        // Apply center gravity
        const centerX = 400;
        const centerY = 300;
        fx += (centerX - node.x) * 0.001;
        fy += (centerY - node.y) * 0.001;
        
        // Update position with damping
        node.x += fx * 0.1;
        node.y += fy * 0.1;
        
        // Keep nodes within bounds
        node.x = Math.max(layoutSettings.nodeSize, Math.min(800 - layoutSettings.nodeSize, node.x));
        node.y = Math.max(layoutSettings.nodeSize, Math.min(600 - layoutSettings.nodeSize, node.y));
      });
      
      return { ...prevData, nodes: newNodes };
    });
  };

  const drawGraph = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Apply zoom and pan
    ctx.save();
    ctx.translate(pan.x, pan.y);
    ctx.scale(zoom, zoom);

    // Draw edges
    graphData.edges.forEach(edge => {
      const sourceNode = graphData.nodes.find(n => n.id === edge.source);
      const targetNode = graphData.nodes.find(n => n.id === edge.target);
      
      if (sourceNode && targetNode) {
        drawEdge(ctx, sourceNode, targetNode, edge);
      }
    });

    // Draw nodes
    graphData.nodes.forEach(node => {
      drawNode(ctx, node);
    });

    ctx.restore();
  };

  const drawNode = (ctx: CanvasRenderingContext2D, node: Node) => {
    const isSelected = selectedNode?.id === node.id;
    const radius = layoutSettings.nodeSize;
    
    // Node background
    ctx.beginPath();
    ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI);
    ctx.fillStyle = getNodeColor(node.type);
    ctx.fill();
    
    // Node border
    ctx.strokeStyle = isSelected ? '#667eea' : '#e2e8f0';
    ctx.lineWidth = isSelected ? 3 : 1;
    ctx.stroke();
    
    // Node label
    if (layoutSettings.showLabels) {
      ctx.fillStyle = '#2d3748';
      ctx.font = '12px -apple-system, BlinkMacSystemFont, sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(node.label, node.x, node.y + radius + 15);
    }
  };

  const drawEdge = (ctx: CanvasRenderingContext2D, source: Node, target: Node, edge: Edge) => {
    // Calculate edge position (offset from node centers)
    const dx = target.x - source.x;
    const dy = target.y - source.y;
    const distance = Math.sqrt(dx * dx + dy * dy);
    
    if (distance === 0) return;
    
    const unitX = dx / distance;
    const unitY = dy / distance;
    
    const sourceX = source.x + unitX * layoutSettings.nodeSize;
    const sourceY = source.y + unitY * layoutSettings.nodeSize;
    const targetX = target.x - unitX * layoutSettings.nodeSize;
    const targetY = target.y - unitY * layoutSettings.nodeSize;
    
    // Draw edge line
    ctx.beginPath();
    ctx.moveTo(sourceX, sourceY);
    ctx.lineTo(targetX, targetY);
    ctx.strokeStyle = '#cbd5e0';
    ctx.lineWidth = 2;
    ctx.stroke();
    
    // Draw arrow
    const arrowLength = 10;
    const arrowAngle = Math.PI / 6;
    const angle = Math.atan2(dy, dx);
    
    ctx.beginPath();
    ctx.moveTo(targetX, targetY);
    ctx.lineTo(
      targetX - arrowLength * Math.cos(angle - arrowAngle),
      targetY - arrowLength * Math.sin(angle - arrowAngle)
    );
    ctx.moveTo(targetX, targetY);
    ctx.lineTo(
      targetX - arrowLength * Math.cos(angle + arrowAngle),
      targetY - arrowLength * Math.sin(angle + arrowAngle)
    );
    ctx.strokeStyle = '#a0aec0';
    ctx.lineWidth = 2;
    ctx.stroke();
    
    // Draw edge label
    if (layoutSettings.showLabels) {
      const midX = (sourceX + targetX) / 2;
      const midY = (sourceY + targetY) / 2;
      
      ctx.fillStyle = '#718096';
      ctx.font = '10px -apple-system, BlinkMacSystemFont, sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(edge.label, midX, midY - 5);
    }
  };

  const getNodeColor = (type: string): string => {
    const colors: Record<string, string> = {
      'Product': '#4299e1',
      'Component': '#48bb78',
      'Supplier': '#ed8936',
      'Process': '#9f7aea',
      'Location': '#38b2ac',
      'Unknown': '#a0aec0'
    };
    return colors[type] || colors['Unknown'];
  };

  const handleCanvasClick = (event: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (event.clientX - rect.left - pan.x) / zoom;
    const y = (event.clientY - rect.top - pan.y) / zoom;

    // Find clicked node
    const clickedNode = graphData.nodes.find(node => {
      const dx = x - node.x;
      const dy = y - node.y;
      const distance = Math.sqrt(dx * dx + dy * dy);
      return distance <= layoutSettings.nodeSize;
    });

    setSelectedNode(clickedNode || null);
  };

  const handleMouseDown = (event: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (event.clientX - rect.left - pan.x) / zoom;
    const y = (event.clientY - rect.top - pan.y) / zoom;

    // Find node to drag
    const nodeToTrack = graphData.nodes.find(node => {
      const dx = x - node.x;
      const dy = y - node.y;
      const distance = Math.sqrt(dx * dx + dy * dy);
      return distance <= layoutSettings.nodeSize;
    });

    if (nodeToTrack) {
      setIsDragging(true);
      setDragNode(nodeToTrack);
      // Fix the node position during drag
      nodeToTrack.fx = nodeToTrack.x;
      nodeToTrack.fy = nodeToTrack.y;
    }
  };

  const handleMouseMove = (event: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isDragging || !dragNode) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (event.clientX - rect.left - pan.x) / zoom;
    const y = (event.clientY - rect.top - pan.y) / zoom;

    // Update drag node position
    setGraphData(prevData => ({
      ...prevData,
      nodes: prevData.nodes.map(node => 
        node.id === dragNode.id 
          ? { ...node, x, y, fx: x, fy: y }
          : node
      )
    }));
  };

  const handleMouseUp = () => {
    if (dragNode) {
      // Release the fixed position
      setGraphData(prevData => ({
        ...prevData,
        nodes: prevData.nodes.map(node => 
          node.id === dragNode.id 
            ? { ...node, fx: undefined, fy: undefined }
            : node
        )
      }));
    }
    setIsDragging(false);
    setDragNode(null);
  };

  const handleWheel = (event: React.WheelEvent<HTMLCanvasElement>) => {
    event.preventDefault();
    const zoomFactor = event.deltaY > 0 ? 0.9 : 1.1;
    setZoom(prevZoom => Math.max(0.1, Math.min(3, prevZoom * zoomFactor)));
  };

  const resetView = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };

  const exportGraph = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const link = document.createElement('a');
    link.download = 'knowledge-graph.png';
    link.href = canvas.toDataURL();
    link.click();
  };

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">Knowledge Graph Visualization</h2>
          <p className="feature-description">
            Interactive visualization of your RDF knowledge graph with physics-based layout
          </p>
        </div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <button className="btn btn-secondary" onClick={() => setShowSettings(!showSettings)}>
            ⚙️ Settings
          </button>
          <button className="btn btn-secondary" onClick={exportGraph}>
            📸 Export PNG
          </button>
          <button className="btn btn-secondary" onClick={resetView}>
            🎯 Reset View
          </button>
          <button className="btn btn-primary" onClick={loadGraphData}>
            🔄 Refresh
          </button>
        </div>
      </div>

      <div className="graph-container">
        <div className="graph-canvas-container">
          <canvas
            ref={canvasRef}
            width={800}
            height={600}
            className="graph-canvas"
            onClick={handleCanvasClick}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onMouseLeave={handleMouseUp}
            onWheel={handleWheel}
          />
          
          <div className="graph-controls">
            <div className="zoom-controls">
              <button 
                className="control-btn"
                onClick={() => setZoom(zoom * 1.2)}
              >
                🔍➕
              </button>
              <span className="zoom-level">{Math.round(zoom * 100)}%</span>
              <button 
                className="control-btn"
                onClick={() => setZoom(zoom * 0.8)}
              >
                🔍➖
              </button>
            </div>
          </div>

          <div className="graph-stats">
            <div className="stat">
              <span className="stat-label">Nodes:</span>
              <span className="stat-value">{graphData.nodes.length}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Edges:</span>
              <span className="stat-value">{graphData.edges.length}</span>
            </div>
          </div>
        </div>

        {selectedNode && (
          <div className="node-details">
            <h4>Node Details</h4>
            <div className="detail-item">
              <strong>ID:</strong> {selectedNode.id}
            </div>
            <div className="detail-item">
              <strong>Label:</strong> {selectedNode.label}
            </div>
            <div className="detail-item">
              <strong>Type:</strong> 
              <span className="badge" style={{ backgroundColor: getNodeColor(selectedNode.type) }}>
                {selectedNode.type}
              </span>
            </div>
            <div className="detail-item">
              <strong>Position:</strong> ({Math.round(selectedNode.x)}, {Math.round(selectedNode.y)})
            </div>
            <div className="detail-item">
              <strong>Connections:</strong> {graphData.edges.filter(e => 
                e.source === selectedNode.id || e.target === selectedNode.id
              ).length}
            </div>
          </div>
        )}
      </div>

      {showSettings && (
        <div className="modal-overlay">
          <div className="modal">
            <div className="modal-header">
              <h3 className="modal-title">Graph Settings</h3>
              <button className="close-btn" onClick={() => setShowSettings(false)}>×</button>
            </div>
            <div className="settings-form">
              <div className="input-group">
                <label className="input-label">Node Size</label>
                <input
                  type="range"
                  min="10"
                  max="40"
                  value={layoutSettings.nodeSize}
                  onChange={(e) => setLayoutSettings({
                    ...layoutSettings,
                    nodeSize: parseInt(e.target.value)
                  })}
                />
                <span>{layoutSettings.nodeSize}px</span>
              </div>
              <div className="input-group">
                <label className="input-label">Link Distance</label>
                <input
                  type="range"
                  min="50"
                  max="200"
                  value={layoutSettings.linkDistance}
                  onChange={(e) => setLayoutSettings({
                    ...layoutSettings,
                    linkDistance: parseInt(e.target.value)
                  })}
                />
                <span>{layoutSettings.linkDistance}px</span>
              </div>
              <div className="input-group">
                <label className="input-label">Repulsion Force</label>
                <input
                  type="range"
                  min="10"
                  max="100"
                  value={layoutSettings.repulsion}
                  onChange={(e) => setLayoutSettings({
                    ...layoutSettings,
                    repulsion: parseInt(e.target.value)
                  })}
                />
                <span>{layoutSettings.repulsion}</span>
              </div>
              <div className="input-group">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={layoutSettings.showLabels}
                    onChange={(e) => setLayoutSettings({
                      ...layoutSettings,
                      showLabels: e.target.checked
                    })}
                  />
                  Show Labels
                </label>
              </div>
              <div className="input-group">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={layoutSettings.physics}
                    onChange={(e) => setLayoutSettings({
                      ...layoutSettings,
                      physics: e.target.checked
                    })}
                  />
                  Physics Simulation
                </label>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};