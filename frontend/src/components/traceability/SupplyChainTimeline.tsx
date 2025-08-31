import React, { useState, useEffect, useRef } from 'react';
import { useTraceability } from '../../hooks/useTraceability';
import * as d3 from 'd3';
import { 
  Clock, 
  MapPin, 
  User, 
  ZoomIn,
  ZoomOut,
  RotateCcw,
  Download,
  Filter,
  Calendar
} from 'lucide-react';
import Button from '../ui/Button';
import Card from '../ui/Card';
import Badge from '../ui/Badge';
import LoadingSpinner from '../ui/LoadingSpinner';
import Alert from '../ui/Alert';
import type { TraceStep, TraceabilityResponse } from '../../types';

interface SupplyChainTimelineProps {
  itemId: string;
  traceData?: TraceabilityResponse;
  onStepSelect?: (step: TraceStep) => void;
  onParticipantSelect?: (participant: string) => void;
}

interface TimelineStep extends TraceStep {
  x_position: number;
  y_position: number;
  color: string;
  icon: string;
}

interface TimelineConfig {
  width: number;
  height: number;
  margin: { top: number; right: number; bottom: number; left: number };
  stepRadius: number;
  lineHeight: number;
}

const SupplyChainTimeline: React.FC<SupplyChainTimelineProps> = ({
  itemId,
  traceData,
  onStepSelect,
  onParticipantSelect
}) => {
  const { loadItemTrace, traceLoading, traceError, traceData: hookTraceData } = useTraceability();
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  
  const [selectedStep, setSelectedStep] = useState<TraceStep | null>(null);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [showFilters, setShowFilters] = useState(false);
  const [filterType, setFilterType] = useState<string>('all');
  const [filterParticipant, setFilterParticipant] = useState<string>('all');

  // Use provided traceData or hook's traceData
  const timelineData = traceData || hookTraceData;

  const config: TimelineConfig = {
    width: 1200,
    height: 600,
    margin: { top: 60, right: 60, bottom: 100, left: 60 },
    stepRadius: 12,
    lineHeight: 4
  };

  // Load trace data if not provided
  useEffect(() => {
    const loadTraceData = async () => {
      if (!traceData && !hookTraceData && itemId) {
        try {
          await loadItemTrace(itemId);
        } catch (err) {
          console.error('Failed to load trace data:', err);
        }
      }
    };

    loadTraceData();
  }, [itemId, traceData, hookTraceData, loadItemTrace]);

  // Process timeline steps with positioning and styling
  const processTimelineSteps = (steps: TraceStep[]): TimelineStep[] => {
    if (!steps.length) return [];

    // Filter steps based on current filters
    let filteredSteps = steps;
    if (filterType !== 'all') {
      filteredSteps = filteredSteps.filter(step => 
        step.action.toLowerCase().includes(filterType.toLowerCase())
      );
    }
    if (filterParticipant !== 'all') {
      filteredSteps = filteredSteps.filter(step => 
        step.participant === filterParticipant
      );
    }

    // Sort by timestamp
    filteredSteps.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());

    // Calculate positions
    const innerWidth = config.width - config.margin.left - config.margin.right;
    const innerHeight = config.height - config.margin.top - config.margin.bottom;
    
    return filteredSteps.map((step, index) => {
      const x = (index / Math.max(1, filteredSteps.length - 1)) * innerWidth;
      const y = innerHeight / 2;

      return {
        ...step,
        x_position: x,
        y_position: y,
        color: getStepColor(step.action),
        icon: getStepIcon(step.action)
      };
    });
  };

  const getStepColor = (action: string): string => {
    const actionLower = action.toLowerCase();
    if (actionLower.includes('production') || actionLower.includes('created')) return '#10b981';
    if (actionLower.includes('processing') || actionLower.includes('manufactured')) return '#3b82f6';
    if (actionLower.includes('transport') || actionLower.includes('shipped')) return '#f59e0b';
    if (actionLower.includes('quality') || actionLower.includes('tested')) return '#8b5cf6';
    if (actionLower.includes('transfer') || actionLower.includes('ownership')) return '#ef4444';
    if (actionLower.includes('environmental')) return '#059669';
    if (actionLower.includes('compliance')) return '#dc2626';
    return '#6b7280';
  };

  const getStepIcon = (action: string): string => {
    const actionLower = action.toLowerCase();
    if (actionLower.includes('production') || actionLower.includes('created')) return 'package';
    if (actionLower.includes('processing') || actionLower.includes('manufactured')) return 'settings';
    if (actionLower.includes('transport') || actionLower.includes('shipped')) return 'truck';
    if (actionLower.includes('quality') || actionLower.includes('tested')) return 'check-circle';
    if (actionLower.includes('transfer') || actionLower.includes('ownership')) return 'arrow-right';
    if (actionLower.includes('environmental')) return 'leaf';
    if (actionLower.includes('compliance')) return 'shield';
    return 'circle';
  };

  const renderTimeline = () => {
    if (!timelineData || !svgRef.current) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const timelineSteps = processTimelineSteps(timelineData.trace_path);
    if (!timelineSteps.length) return;

    // Create main group with zoom behavior
    const g = svg.append('g')
      .attr('transform', `translate(${config.margin.left}, ${config.margin.top}) scale(${zoomLevel})`);

    // Draw timeline line
    if (timelineSteps.length > 1) {
      const line = d3.line<TimelineStep>()
        .x(d => d.x_position)
        .y(d => d.y_position)
        .curve(d3.curveMonotoneX);

      g.append('path')
        .datum(timelineSteps)
        .attr('d', line)
        .attr('stroke', '#e5e7eb')
        .attr('stroke-width', config.lineHeight)
        .attr('fill', 'none')
        .attr('stroke-dasharray', '5,5');
    }

    // Draw step circles
    const stepGroups = g.selectAll('.step-group')
      .data(timelineSteps)
      .enter()
      .append('g')
      .attr('class', 'step-group')
      .attr('transform', d => `translate(${d.x_position}, ${d.y_position})`)
      .style('cursor', 'pointer')
      .on('click', (_event, d) => {
        setSelectedStep(d);
        if (onStepSelect) onStepSelect(d);
      })
      .on('mouseover', function(event, d) {
        d3.select(this).select('circle').attr('r', config.stepRadius * 1.2);
        
        // Show tooltip
        const tooltip = d3.select('body').append('div')
          .attr('class', 'timeline-tooltip')
          .style('position', 'absolute')
          .style('background', 'rgba(0, 0, 0, 0.8)')
          .style('color', 'white')
          .style('padding', '8px 12px')
          .style('border-radius', '4px')
          .style('font-size', '12px')
          .style('pointer-events', 'none')
          .style('z-index', '1000')
          .html(`
            <div><strong>${d.action}</strong></div>
            <div>Participant: ${d.participant}</div>
            <div>Time: ${new Date(d.timestamp).toLocaleString()}</div>
            ${d.location ? `<div>Location: ${d.location}</div>` : ''}
          `);

        tooltip
          .style('left', (event.pageX + 10) + 'px')
          .style('top', (event.pageY - 10) + 'px');
      })
      .on('mouseout', function() {
        d3.select(this).select('circle').attr('r', config.stepRadius);
        d3.selectAll('.timeline-tooltip').remove();
      });

    // Add circles
    stepGroups.append('circle')
      .attr('r', config.stepRadius)
      .attr('fill', d => d.color)
      .attr('stroke', '#ffffff')
      .attr('stroke-width', 3)
      .style('filter', 'drop-shadow(0 2px 4px rgba(0,0,0,0.1))');

    // Add step numbers
    stepGroups.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '0.35em')
      .attr('fill', 'white')
      .attr('font-size', '10px')
      .attr('font-weight', 'bold')
      .text(d => d.step_number);

    // Add labels below
    stepGroups.append('text')
      .attr('text-anchor', 'middle')
      .attr('y', config.stepRadius + 20)
      .attr('fill', '#374151')
      .attr('font-size', '12px')
      .attr('font-weight', '500')
      .text(d => d.action);

    // Add participant labels
    stepGroups.append('text')
      .attr('text-anchor', 'middle')
      .attr('y', config.stepRadius + 35)
      .attr('fill', '#6b7280')
      .attr('font-size', '10px')
      .text(d => d.participant);

    // Add timestamps
    stepGroups.append('text')
      .attr('text-anchor', 'middle')
      .attr('y', config.stepRadius + 50)
      .attr('fill', '#9ca3af')
      .attr('font-size', '9px')
      .text(d => new Date(d.timestamp).toLocaleDateString());

    // Add connecting arrows
    if (timelineSteps.length > 1) {
      for (let i = 0; i < timelineSteps.length - 1; i++) {
        const current = timelineSteps[i];
        const next = timelineSteps[i + 1];
        const midX = (current.x_position + next.x_position) / 2;

        g.append('polygon')
          .attr('points', `${midX-5},${current.y_position-3} ${midX+5},${current.y_position} ${midX-5},${current.y_position+3}`)
          .attr('fill', '#9ca3af')
          .style('opacity', 0.7);
      }
    }
  };

  // Get unique participants for filter
  const getUniqueParticipants = (): string[] => {
    if (!timelineData) return [];
    const participants = new Set(timelineData.trace_path.map(step => step.participant));
    return Array.from(participants).sort();
  };

  // Handle zoom controls
  const handleZoomIn = () => setZoomLevel(prev => Math.min(prev * 1.2, 3));
  const handleZoomOut = () => setZoomLevel(prev => Math.max(prev / 1.2, 0.5));
  const handleResetZoom = () => setZoomLevel(1);

  // Handle export
  const handleExport = () => {
    if (!svgRef.current) return;
    
    const svgData = new XMLSerializer().serializeToString(svgRef.current);
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    const img = new Image();
    
    img.onload = () => {
      canvas.width = img.width;
      canvas.height = img.height;
      ctx?.drawImage(img, 0, 0);
      
      const link = document.createElement('a');
      link.download = `supply-chain-timeline-${itemId}.png`;
      link.href = canvas.toDataURL();
      link.click();
    };
    
    img.src = 'data:image/svg+xml;base64,' + btoa(svgData);
  };

  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  useEffect(() => {
    renderTimeline();
  }, [timelineData, zoomLevel, filterType, filterParticipant]);

  if (traceLoading) {
    return (
      <div className="flex justify-center items-center h-96">
        <LoadingSpinner size="lg" message="Loading supply chain timeline..." />
      </div>
    );
  }

  if (traceError) {
    return (
      <Alert
        variant="error"
        message={traceError}
      />
    );
  }

  if (!timelineData) {
    return (
      <Card className="p-8 text-center">
        <Clock className="w-16 h-16 text-gray-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
          No Timeline Data
        </h3>
        <p className="text-gray-600 dark:text-gray-300">
          Timeline data will appear here once the item has trace steps.
        </p>
      </Card>
    );
  }

  const timelineSteps = processTimelineSteps(timelineData.trace_path);

  return (
    <div className="space-y-6">
      {/* Header with Controls */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Supply Chain Timeline
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mt-1">
            Interactive timeline showing the journey of {timelineData.item.name}
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
          
          <div className="flex items-center gap-1 border border-gray-300 dark:border-gray-600 rounded-md">
            <Button
              variant="outline"
              size="sm"
              onClick={handleZoomOut}
              disabled={zoomLevel <= 0.5}
            >
              <ZoomOut className="w-4 h-4" />
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleResetZoom}
            >
              <RotateCcw className="w-4 h-4" />
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleZoomIn}
              disabled={zoomLevel >= 3}
            >
              <ZoomIn className="w-4 h-4" />
            </Button>
          </div>
          
          <Button
            variant="outline"
            onClick={handleExport}
            className="flex items-center gap-2"
          >
            <Download className="w-4 h-4" />
            Export
          </Button>
        </div>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <Card className="p-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Action Type
              </label>
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                <option value="all">All Actions</option>
                <option value="production">Production</option>
                <option value="processing">Processing</option>
                <option value="transport">Transport</option>
                <option value="quality">Quality</option>
                <option value="transfer">Transfer</option>
                <option value="environmental">Environmental</option>
                <option value="compliance">Compliance</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Participant
              </label>
              <select
                value={filterParticipant}
                onChange={(e) => setFilterParticipant(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                <option value="all">All Participants</option>
                {getUniqueParticipants().map(participant => (
                  <option key={participant} value={participant}>
                    {participant}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </Card>
      )}

      {/* Timeline Visualization */}
      <Card className="p-6">
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Badge variant="info">
              {timelineSteps.length} steps
            </Badge>
            <Badge variant="default">
              Zoom: {Math.round(zoomLevel * 100)}%
            </Badge>
          </div>
          
          <div className="text-sm text-gray-600 dark:text-gray-300">
            {timelineSteps.length > 0 && (
              <>
                From {formatDate(timelineSteps[0].timestamp)} to{' '}
                {formatDate(timelineSteps[timelineSteps.length - 1].timestamp)}
              </>
            )}
          </div>
        </div>

        <div 
          ref={containerRef}
          className="w-full overflow-auto border border-gray-200 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800"
          style={{ height: config.height + 'px' }}
        >
          <svg
            ref={svgRef}
            width={config.width}
            height={config.height}
            className="w-full h-full"
          />
        </div>
      </Card>

      {/* Selected Step Details */}
      {selectedStep && (
        <Card className="p-6">
          <div className="flex items-start justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Step Details
            </h3>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setSelectedStep(null)}
            >
              ×
            </Button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center gap-3">
                <div 
                  className="w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-bold"
                  style={{ backgroundColor: getStepColor(selectedStep.action) }}
                >
                  {selectedStep.step_number}
                </div>
                <div>
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    {selectedStep.action}
                  </h4>
                  <p className="text-sm text-gray-600 dark:text-gray-300">
                    Step {selectedStep.step_number}
                  </p>
                </div>
              </div>

              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm">
                  <User className="w-4 h-4 text-gray-400" />
                  <span className="text-gray-600 dark:text-gray-300">Participant:</span>
                  <button
                    onClick={() => onParticipantSelect?.(selectedStep.participant)}
                    className="text-blue-600 dark:text-blue-400 hover:underline"
                  >
                    {selectedStep.participant}
                  </button>
                </div>

                <div className="flex items-center gap-2 text-sm">
                  <Calendar className="w-4 h-4 text-gray-400" />
                  <span className="text-gray-600 dark:text-gray-300">Timestamp:</span>
                  <span className="text-gray-900 dark:text-white">
                    {formatDate(selectedStep.timestamp)}
                  </span>
                </div>

                {selectedStep.location && (
                  <div className="flex items-center gap-2 text-sm">
                    <MapPin className="w-4 h-4 text-gray-400" />
                    <span className="text-gray-600 dark:text-gray-300">Location:</span>
                    <span className="text-gray-900 dark:text-white">
                      {selectedStep.location}
                    </span>
                  </div>
                )}
              </div>
            </div>

            <div>
              <h5 className="font-medium text-gray-900 dark:text-white mb-2">
                Metadata
              </h5>
              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
                <pre className="text-xs text-gray-600 dark:text-gray-300 whitespace-pre-wrap">
                  {JSON.stringify(selectedStep.metadata, null, 2)}
                </pre>
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Timeline Summary */}
      <Card className="p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Timeline Summary
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
              {timelineSteps.length}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-300">
              Total Steps
            </div>
          </div>
          
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {getUniqueParticipants().length}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-300">
              Participants
            </div>
          </div>
          
          <div className="text-center">
            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
              {timelineSteps.length > 0 ? Math.ceil(
                (new Date(timelineSteps[timelineSteps.length - 1].timestamp).getTime() - 
                 new Date(timelineSteps[0].timestamp).getTime()) / (1000 * 60 * 60 * 24)
              ) : 0}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-300">
              Days Duration
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
};

export default SupplyChainTimeline;
