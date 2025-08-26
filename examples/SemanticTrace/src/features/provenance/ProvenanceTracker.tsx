import React, { useState, useEffect } from 'react';
import './ProvenanceTracker.css';
import { persistence } from '../../utils/persistence';

interface ProvEntity {
  id: string;
  label: string;
  type: 'Entity' | 'Activity' | 'Agent';
  attributes: Record<string, any>;
  timestamp: string;
}

interface ProvRelation {
  id: string;
  type: 'wasGeneratedBy' | 'used' | 'wasAssociatedWith' | 'wasDerivedFrom' | 'wasAttributedTo' | 'wasInformedBy';
  from: string;
  to: string;
  timestamp: string;
  attributes: Record<string, any>;
}

interface ProvenanceData {
  entities: ProvEntity[];
  relations: ProvRelation[];
}

export const ProvenanceTracker: React.FC = () => {
  const [provData, setProvData] = useState<ProvenanceData>({ entities: [], relations: [] });
  const [showEntityModal, setShowEntityModal] = useState(false);
  const [showRelationModal, setShowRelationModal] = useState(false);
  const [editingEntity, setEditingEntity] = useState<ProvEntity | null>(null);
  const [editingRelation, setEditingRelation] = useState<ProvRelation | null>(null);
  const [selectedEntity, setSelectedEntity] = useState<ProvEntity | null>(null);
  const [viewMode, setViewMode] = useState<'timeline' | 'graph' | 'table'>('timeline');

  useEffect(() => {
    loadProvenanceData();
  }, []);

  const loadProvenanceData = async () => {
    try {
      const data = await persistence.getItem('provenance-data');
      if (data) {
        setProvData(JSON.parse(data));
      } else {
        // Initialize with sample PROV-O data
        const sampleData = generateSampleProvenance();
        setProvData(sampleData);
        await persistence.setItem('provenance-data', JSON.stringify(sampleData));
      }
    } catch (error) {
      console.error('Error loading provenance data:', error);
    }
  };

  const generateSampleProvenance = (): ProvenanceData => {
    const entities: ProvEntity[] = [
      {
        id: 'entity:raw-materials',
        label: 'Raw Materials Batch #001',
        type: 'Entity',
        attributes: { batchNumber: '001', supplier: 'ACME Corp', quality: 'Grade A' },
        timestamp: '2024-01-15T08:00:00Z'
      },
      {
        id: 'entity:component-battery',
        label: 'Lithium Battery LB-2000',
        type: 'Entity',
        attributes: { partNumber: 'LB-2000', capacity: '5000mAh', voltage: '3.7V' },
        timestamp: '2024-01-15T14:30:00Z'
      },
      {
        id: 'entity:product-smartphone',
        label: 'Smartphone Model X',
        type: 'Entity',
        attributes: { model: 'Model X', serialNumber: 'SN001234', color: 'Black' },
        timestamp: '2024-01-16T16:45:00Z'
      }
    ];

    const activities: ProvEntity[] = [
      {
        id: 'activity:manufacturing',
        label: 'Battery Manufacturing Process',
        type: 'Activity',
        attributes: { processId: 'MFG-001', location: 'Factory A', temperature: '25°C' },
        timestamp: '2024-01-15T10:00:00Z'
      },
      {
        id: 'activity:assembly',
        label: 'Product Assembly',
        type: 'Activity',
        attributes: { assemblyLine: 'Line 3', operator: 'John Smith', duration: '45 minutes' },
        timestamp: '2024-01-16T09:00:00Z'
      },
      {
        id: 'activity:quality-test',
        label: 'Quality Control Testing',
        type: 'Activity',
        attributes: { testType: 'Functional', result: 'PASS', inspector: 'Jane Doe' },
        timestamp: '2024-01-16T15:00:00Z'
      }
    ];

    const agents: ProvEntity[] = [
      {
        id: 'agent:operator-john',
        label: 'John Smith (Operator)',
        type: 'Agent',
        attributes: { employeeId: 'EMP001', role: 'Assembly Technician', certification: 'Level 2' },
        timestamp: '2024-01-16T09:00:00Z'
      },
      {
        id: 'agent:inspector-jane',
        label: 'Jane Doe (QC Inspector)',
        type: 'Agent',
        attributes: { employeeId: 'EMP002', role: 'Quality Inspector', certification: 'Senior' },
        timestamp: '2024-01-16T15:00:00Z'
      }
    ];

    const relations: ProvRelation[] = [
      {
        id: 'rel-001',
        type: 'wasGeneratedBy',
        from: 'entity:component-battery',
        to: 'activity:manufacturing',
        timestamp: '2024-01-15T14:30:00Z',
        attributes: { machine: 'Battery Assembler 3000' }
      },
      {
        id: 'rel-002',
        type: 'used',
        from: 'activity:manufacturing',
        to: 'entity:raw-materials',
        timestamp: '2024-01-15T10:00:00Z',
        attributes: { quantity: '5kg' }
      },
      {
        id: 'rel-003',
        type: 'wasGeneratedBy',
        from: 'entity:product-smartphone',
        to: 'activity:assembly',
        timestamp: '2024-01-16T16:45:00Z',
        attributes: { workstation: 'Assembly Station 7' }
      },
      {
        id: 'rel-004',
        type: 'used',
        from: 'activity:assembly',
        to: 'entity:component-battery',
        timestamp: '2024-01-16T09:30:00Z',
        attributes: { installationMethod: 'Snap-fit' }
      },
      {
        id: 'rel-005',
        type: 'wasAssociatedWith',
        from: 'activity:assembly',
        to: 'agent:operator-john',
        timestamp: '2024-01-16T09:00:00Z',
        attributes: { role: 'primary assembler' }
      },
      {
        id: 'rel-006',
        type: 'wasAssociatedWith',
        from: 'activity:quality-test',
        to: 'agent:inspector-jane',
        timestamp: '2024-01-16T15:00:00Z',
        attributes: { role: 'quality inspector' }
      }
    ];

    return {
      entities: [...entities, ...activities, ...agents],
      relations
    };
  };

  const saveProvenanceData = async (newData: ProvenanceData) => {
    try {
      await persistence.setItem('provenance-data', JSON.stringify(newData));
      setProvData(newData);
    } catch (error) {
      console.error('Error saving provenance data:', error);
    }
  };

  const addEntity = (entity: ProvEntity) => {
    const newData = {
      ...provData,
      entities: [...provData.entities, { ...entity, id: `entity-${Date.now()}` }]
    };
    saveProvenanceData(newData);
  };

  const updateEntity = (updatedEntity: ProvEntity) => {
    const newData = {
      ...provData,
      entities: provData.entities.map(e => e.id === updatedEntity.id ? updatedEntity : e)
    };
    saveProvenanceData(newData);
  };

  const removeEntity = (entityId: string) => {
    const newData = {
      ...provData,
      entities: provData.entities.filter(e => e.id !== entityId),
      relations: provData.relations.filter(r => r.from !== entityId && r.to !== entityId)
    };
    saveProvenanceData(newData);
  };

  const addRelation = (relation: ProvRelation) => {
    const newData = {
      ...provData,
      relations: [...provData.relations, { ...relation, id: `rel-${Date.now()}` }]
    };
    saveProvenanceData(newData);
  };

  const removeRelation = (relationId: string) => {
    const newData = {
      ...provData,
      relations: provData.relations.filter(r => r.id !== relationId)
    };
    saveProvenanceData(newData);
  };

  const generateProvNTriples = (): string => {
    let ntriples = `# PROV-O Provenance Data in N-Triples format
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix : <http://example.org/provenance#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

`;

    // Add entities
    provData.entities.forEach(entity => {
      const entityType = entity.type === 'Entity' ? 'prov:Entity' : 
                        entity.type === 'Activity' ? 'prov:Activity' : 'prov:Agent';
      ntriples += `:${entity.id} a ${entityType} ;\n`;
      ntriples += `  rdfs:label "${entity.label}" ;\n`;
      ntriples += `  prov:generatedAtTime "${entity.timestamp}"^^xsd:dateTime .\n\n`;
    });

    // Add relations
    provData.relations.forEach(relation => {
      ntriples += `:${relation.from} prov:${relation.type} :${relation.to} .\n`;
    });

    return ntriples;
  };

  const exportProvenance = () => {
    const content = generateProvNTriples();
    const blob = new Blob([content], { type: 'text/turtle' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'provenance.ttl';
    a.click();
    URL.revokeObjectURL(url);
  };

  const getEntityTypeIcon = (type: string) => {
    switch (type) {
      case 'Entity': return '📦';
      case 'Activity': return '⚡';
      case 'Agent': return '👤';
      default: return '❓';
    }
  };

  const getRelationTypeLabel = (type: string) => {
    const labels: Record<string, string> = {
      'wasGeneratedBy': 'was generated by',
      'used': 'used',
      'wasAssociatedWith': 'was associated with',
      'wasDerivedFrom': 'was derived from',
      'wasAttributedTo': 'was attributed to',
      'wasInformedBy': 'was informed by'
    };
    return labels[type] || type;
  };

  const renderTimelineView = () => {
    const sortedEntities = [...provData.entities].sort(
      (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
    );

    return (
      <div className="timeline-container">
        <div className="timeline">
          {sortedEntities.map((entity, index) => (
            <div 
              key={entity.id} 
              className={`timeline-item ${entity.type.toLowerCase()}`}
              onClick={() => setSelectedEntity(entity)}
            >
              <div className="timeline-marker">
                <span className="timeline-icon">{getEntityTypeIcon(entity.type)}</span>
              </div>
              <div className="timeline-content">
                <div className="timeline-header">
                  <h4 className="timeline-title">{entity.label}</h4>
                  <span className="timeline-time">
                    {new Date(entity.timestamp).toLocaleString()}
                  </span>
                </div>
                <div className="timeline-type">{entity.type}</div>
                <div className="timeline-attributes">
                  {Object.entries(entity.attributes).slice(0, 2).map(([key, value]) => (
                    <span key={key} className="attribute-tag">
                      {key}: {value}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  };

  const renderTableView = () => {
    return (
      <div className="table-container">
        <div className="table-section">
          <h3>Entities</h3>
          <table className="prov-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Label</th>
                <th>Type</th>
                <th>Timestamp</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {provData.entities.map(entity => (
                <tr key={entity.id}>
                  <td>
                    <span className="entity-id">{entity.id}</span>
                  </td>
                  <td>{entity.label}</td>
                  <td>
                    <span className={`type-badge ${entity.type.toLowerCase()}`}>
                      {getEntityTypeIcon(entity.type)} {entity.type}
                    </span>
                  </td>
                  <td>{new Date(entity.timestamp).toLocaleString()}</td>
                  <td>
                    <div className="table-actions">
                      <button
                        className="btn-icon"
                        onClick={() => {
                          setEditingEntity(entity);
                          setShowEntityModal(true);
                        }}
                      >
                        ✏️
                      </button>
                      <button
                        className="btn-icon btn-danger"
                        onClick={() => removeEntity(entity.id)}
                      >
                        🗑️
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="table-section">
          <h3>Relations</h3>
          <table className="prov-table">
            <thead>
              <tr>
                <th>From</th>
                <th>Relation</th>
                <th>To</th>
                <th>Timestamp</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {provData.relations.map(relation => (
                <tr key={relation.id}>
                  <td>
                    <span className="entity-ref">{relation.from}</span>
                  </td>
                  <td>
                    <span className="relation-type">{getRelationTypeLabel(relation.type)}</span>
                  </td>
                  <td>
                    <span className="entity-ref">{relation.to}</span>
                  </td>
                  <td>{new Date(relation.timestamp).toLocaleString()}</td>
                  <td>
                    <div className="table-actions">
                      <button
                        className="btn-icon btn-danger"
                        onClick={() => removeRelation(relation.id)}
                      >
                        🗑️
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    );
  };

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">PROV-O Provenance Tracker</h2>
          <p className="feature-description">
            Track and visualize provenance information using the W3C PROV-O ontology standard
          </p>
        </div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <select 
            className="view-selector"
            value={viewMode}
            onChange={(e) => setViewMode(e.target.value as any)}
          >
            <option value="timeline">🕒 Timeline</option>
            <option value="table">📋 Table</option>
          </select>
          <button className="btn btn-secondary" onClick={exportProvenance}>
            📥 Export PROV-O
          </button>
          <button className="btn btn-primary" onClick={() => setShowEntityModal(true)}>
            ➕ Add Entity
          </button>
          <button className="btn btn-primary" onClick={() => setShowRelationModal(true)}>
            🔗 Add Relation
          </button>
        </div>
      </div>

      <div className="provenance-content">
        {viewMode === 'timeline' && renderTimelineView()}
        {viewMode === 'table' && renderTableView()}

        {selectedEntity && (
          <div className="entity-details-panel">
            <div className="panel-header">
              <h3>Entity Details</h3>
              <button className="close-btn" onClick={() => setSelectedEntity(null)}>×</button>
            </div>
            <div className="entity-info">
              <div className="info-item">
                <strong>ID:</strong> {selectedEntity.id}
              </div>
              <div className="info-item">
                <strong>Label:</strong> {selectedEntity.label}
              </div>
              <div className="info-item">
                <strong>Type:</strong> {selectedEntity.type}
              </div>
              <div className="info-item">
                <strong>Timestamp:</strong> {new Date(selectedEntity.timestamp).toLocaleString()}
              </div>
              <div className="attributes-section">
                <strong>Attributes:</strong>
                <div className="attributes-list">
                  {Object.entries(selectedEntity.attributes).map(([key, value]) => (
                    <div key={key} className="attribute-item">
                      <span className="attr-key">{key}:</span>
                      <span className="attr-value">{value}</span>
                    </div>
                  ))}
                </div>
              </div>
              <div className="relations-section">
                <strong>Related:</strong>
                <div className="related-items">
                  {provData.relations
                    .filter(r => r.from === selectedEntity.id || r.to === selectedEntity.id)
                    .map(relation => (
                      <div key={relation.id} className="relation-item">
                        {relation.from === selectedEntity.id 
                          ? `${getRelationTypeLabel(relation.type)} → ${relation.to}`
                          : `← ${getRelationTypeLabel(relation.type)} ${relation.from}`
                        }
                      </div>
                    ))
                  }
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {showEntityModal && (
        <EntityModal
          entity={editingEntity}
          onSave={(entity) => {
            if (editingEntity) {
              updateEntity(entity);
            } else {
              addEntity(entity);
            }
            setShowEntityModal(false);
            setEditingEntity(null);
          }}
          onClose={() => {
            setShowEntityModal(false);
            setEditingEntity(null);
          }}
        />
      )}

      {showRelationModal && (
        <RelationModal
          availableEntities={provData.entities}
          onSave={(relation) => {
            addRelation(relation);
            setShowRelationModal(false);
          }}
          onClose={() => setShowRelationModal(false)}
        />
      )}
    </div>
  );
};

interface EntityModalProps {
  entity?: ProvEntity | null;
  onSave: (entity: ProvEntity) => void;
  onClose: () => void;
}

const EntityModal: React.FC<EntityModalProps> = ({ entity, onSave, onClose }) => {
  const [formData, setFormData] = useState<ProvEntity>({
    id: '',
    label: '',
    type: 'Entity',
    attributes: {},
    timestamp: new Date().toISOString()
  });
  const [attributeKey, setAttributeKey] = useState('');
  const [attributeValue, setAttributeValue] = useState('');

  useEffect(() => {
    if (entity) {
      setFormData(entity);
    }
  }, [entity]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.id && formData.label) {
      onSave(formData);
    }
  };

  const addAttribute = () => {
    if (attributeKey && attributeValue) {
      setFormData({
        ...formData,
        attributes: {
          ...formData.attributes,
          [attributeKey]: attributeValue
        }
      });
      setAttributeKey('');
      setAttributeValue('');
    }
  };

  const removeAttribute = (key: string) => {
    const newAttributes = { ...formData.attributes };
    delete newAttributes[key];
    setFormData({ ...formData, attributes: newAttributes });
  };

  return (
    <div className="modal-overlay">
      <div className="modal">
        <div className="modal-header">
          <h3 className="modal-title">
            {entity ? 'Edit Entity' : 'Add New Entity'}
          </h3>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="input-group">
            <label className="input-label">Entity ID</label>
            <input
              type="text"
              className="input-field"
              value={formData.id}
              onChange={(e) => setFormData({ ...formData, id: e.target.value })}
              placeholder="e.g., entity:product-001"
              required
            />
          </div>
          <div className="input-group">
            <label className="input-label">Label</label>
            <input
              type="text"
              className="input-field"
              value={formData.label}
              onChange={(e) => setFormData({ ...formData, label: e.target.value })}
              placeholder="Human-readable label"
              required
            />
          </div>
          <div className="input-group">
            <label className="input-label">Type</label>
            <select
              className="select-field"
              value={formData.type}
              onChange={(e) => setFormData({ ...formData, type: e.target.value as any })}
            >
              <option value="Entity">Entity</option>
              <option value="Activity">Activity</option>
              <option value="Agent">Agent</option>
            </select>
          </div>
          <div className="input-group">
            <label className="input-label">Timestamp</label>
            <input
              type="datetime-local"
              className="input-field"
              value={formData.timestamp.slice(0, -1)}
              onChange={(e) => setFormData({ ...formData, timestamp: e.target.value + 'Z' })}
            />
          </div>
          
          <div className="attributes-section">
            <label className="input-label">Attributes</label>
            <div className="attribute-input">
              <input
                type="text"
                placeholder="Key"
                value={attributeKey}
                onChange={(e) => setAttributeKey(e.target.value)}
              />
              <input
                type="text"
                placeholder="Value"
                value={attributeValue}
                onChange={(e) => setAttributeValue(e.target.value)}
              />
              <button type="button" onClick={addAttribute}>Add</button>
            </div>
            <div className="attributes-list">
              {Object.entries(formData.attributes).map(([key, value]) => (
                <div key={key} className="attribute-item">
                  <span>{key}: {value}</span>
                  <button type="button" onClick={() => removeAttribute(key)}>×</button>
                </div>
              ))}
            </div>
          </div>

          <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              {entity ? 'Update' : 'Create'} Entity
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

interface RelationModalProps {
  availableEntities: ProvEntity[];
  onSave: (relation: ProvRelation) => void;
  onClose: () => void;
}

const RelationModal: React.FC<RelationModalProps> = ({ availableEntities, onSave, onClose }) => {
  const [formData, setFormData] = useState<ProvRelation>({
    id: '',
    type: 'wasGeneratedBy',
    from: '',
    to: '',
    timestamp: new Date().toISOString(),
    attributes: {}
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.from && formData.to && formData.type) {
      onSave(formData);
    }
  };

  const relationTypes = [
    'wasGeneratedBy',
    'used',
    'wasAssociatedWith',
    'wasDerivedFrom',
    'wasAttributedTo',
    'wasInformedBy'
  ];

  return (
    <div className="modal-overlay">
      <div className="modal">
        <div className="modal-header">
          <h3 className="modal-title">Add New Relation</h3>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="input-group">
            <label className="input-label">From Entity</label>
            <select
              className="select-field"
              value={formData.from}
              onChange={(e) => setFormData({ ...formData, from: e.target.value })}
              required
            >
              <option value="">Select entity</option>
              {availableEntities.map(entity => (
                <option key={entity.id} value={entity.id}>
                  {entity.label} ({entity.type})
                </option>
              ))}
            </select>
          </div>
          <div className="input-group">
            <label className="input-label">Relation Type</label>
            <select
              className="select-field"
              value={formData.type}
              onChange={(e) => setFormData({ ...formData, type: e.target.value as any })}
            >
              {relationTypes.map(type => (
                <option key={type} value={type}>{type}</option>
              ))}
            </select>
          </div>
          <div className="input-group">
            <label className="input-label">To Entity</label>
            <select
              className="select-field"
              value={formData.to}
              onChange={(e) => setFormData({ ...formData, to: e.target.value })}
              required
            >
              <option value="">Select entity</option>
              {availableEntities.map(entity => (
                <option key={entity.id} value={entity.id}>
                  {entity.label} ({entity.type})
                </option>
              ))}
            </select>
          </div>
          <div className="input-group">
            <label className="input-label">Timestamp</label>
            <input
              type="datetime-local"
              className="input-field"
              value={formData.timestamp.slice(0, -1)}
              onChange={(e) => setFormData({ ...formData, timestamp: e.target.value + 'Z' })}
            />
          </div>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              Create Relation
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};