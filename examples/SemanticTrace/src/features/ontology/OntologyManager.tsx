import React, { useState, useEffect } from 'react';
import './OntologyManager.css';
import { persistence } from '../../utils/persistence';

interface OntologyClass {
  id: string;
  label: string;
  description: string;
  superClass?: string;
}

interface OntologyProperty {
  id: string;
  label: string;
  domain: string;
  range: string;
  type?: 'ObjectProperty' | 'DatatypeProperty';
}

interface Ontology {
  classes: OntologyClass[];
  properties: OntologyProperty[];
}

export const OntologyManager: React.FC = () => {
  const [ontology, setOntology] = useState<Ontology>({ classes: [], properties: [] });
  const [showClassModal, setShowClassModal] = useState(false);
  const [showPropertyModal, setShowPropertyModal] = useState(false);
  const [editingClass, setEditingClass] = useState<OntologyClass | null>(null);
  const [editingProperty, setEditingProperty] = useState<OntologyProperty | null>(null);

  useEffect(() => {
    loadOntology();
  }, []);

  const loadOntology = async () => {
    try {
      const data = await persistence.getItem('ontology');
      if (data) {
        setOntology(JSON.parse(data));
      }
    } catch (error) {
      console.error('Error loading ontology:', error);
    }
  };

  const saveOntology = async (newOntology: Ontology) => {
    try {
      await persistence.setItem('ontology', JSON.stringify(newOntology));
      setOntology(newOntology);
    } catch (error) {
      console.error('Error saving ontology:', error);
    }
  };

  const addClass = (newClass: OntologyClass) => {
    const updatedOntology = {
      ...ontology,
      classes: [...ontology.classes, newClass]
    };
    saveOntology(updatedOntology);
  };

  const updateClass = (updatedClass: OntologyClass) => {
    const updatedOntology = {
      ...ontology,
      classes: ontology.classes.map(cls => 
        cls.id === updatedClass.id ? updatedClass : cls
      )
    };
    saveOntology(updatedOntology);
  };

  const removeClass = (classId: string) => {
    const updatedOntology = {
      ...ontology,
      classes: ontology.classes.filter(cls => cls.id !== classId),
      properties: ontology.properties.filter(prop => 
        prop.domain !== classId && prop.range !== classId
      )
    };
    saveOntology(updatedOntology);
  };

  const addProperty = (newProperty: OntologyProperty) => {
    const updatedOntology = {
      ...ontology,
      properties: [...ontology.properties, newProperty]
    };
    saveOntology(updatedOntology);
  };

  const updateProperty = (updatedProperty: OntologyProperty) => {
    const updatedOntology = {
      ...ontology,
      properties: ontology.properties.map(prop => 
        prop.id === updatedProperty.id ? updatedProperty : prop
      )
    };
    saveOntology(updatedOntology);
  };

  const removeProperty = (propertyId: string) => {
    const updatedOntology = {
      ...ontology,
      properties: ontology.properties.filter(prop => prop.id !== propertyId)
    };
    saveOntology(updatedOntology);
  };

  const exportOntology = () => {
    const rdfContent = generateRDFSchema(ontology);
    const blob = new Blob([rdfContent], { type: 'text/turtle' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'ontology.ttl';
    a.click();
    URL.revokeObjectURL(url);
  };

  const generateRDFSchema = (ontology: Ontology): string => {
    let rdf = `@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix : <http://example.org/traceability#> .

# Ontology Declaration
: a owl:Ontology ;
  rdfs:label "Traceability Ontology" ;
  rdfs:comment "An ontology for representing traceability information" .

# Classes
`;

    ontology.classes.forEach(cls => {
      rdf += `
:${cls.id} a owl:Class ;
  rdfs:label "${cls.label}" ;
  rdfs:comment "${cls.description}"`;
      if (cls.superClass) {
        rdf += ` ;
  rdfs:subClassOf :${cls.superClass}`;
      }
      rdf += ' .\n';
    });

    rdf += '\n# Properties\n';

    ontology.properties.forEach(prop => {
      const propertyType = prop.type === 'DatatypeProperty' ? 'owl:DatatypeProperty' : 'owl:ObjectProperty';
      rdf += `
:${prop.id} a ${propertyType} ;
  rdfs:label "${prop.label}" ;
  rdfs:domain :${prop.domain} ;
  rdfs:range :${prop.range} .
`;
    });

    return rdf;
  };

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">Ontology Manager</h2>
          <p className="feature-description">
            Define and manage OWL classes, properties, and hierarchies for your traceability domain
          </p>
        </div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <button className="btn btn-secondary" onClick={exportOntology}>
            📥 Export RDF/OWL
          </button>
          <button className="btn btn-primary" onClick={() => setShowClassModal(true)}>
            ➕ Add Class
          </button>
          <button className="btn btn-primary" onClick={() => setShowPropertyModal(true)}>
            🔗 Add Property
          </button>
        </div>
      </div>

      <div className="ontology-grid">
        <div className="ontology-section">
          <h3 className="section-title">Classes ({ontology.classes.length})</h3>
          <div className="class-list">
            {ontology.classes.map(cls => (
              <div key={cls.id} className="ontology-item">
                <div className="item-header">
                  <span className="item-id">{cls.id}</span>
                  <div className="item-actions">
                    <button 
                      className="btn-icon"
                      onClick={() => {
                        setEditingClass(cls);
                        setShowClassModal(true);
                      }}
                    >
                      ✏️
                    </button>
                    <button 
                      className="btn-icon btn-danger"
                      onClick={() => removeClass(cls.id)}
                    >
                      🗑️
                    </button>
                  </div>
                </div>
                <div className="item-label">{cls.label}</div>
                <div className="item-description">{cls.description}</div>
                {cls.superClass && (
                  <div className="item-meta">
                    <span className="badge">subClassOf: {cls.superClass}</span>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="ontology-section">
          <h3 className="section-title">Properties ({ontology.properties.length})</h3>
          <div className="property-list">
            {ontology.properties.map(prop => (
              <div key={prop.id} className="ontology-item">
                <div className="item-header">
                  <span className="item-id">{prop.id}</span>
                  <div className="item-actions">
                    <button 
                      className="btn-icon"
                      onClick={() => {
                        setEditingProperty(prop);
                        setShowPropertyModal(true);
                      }}
                    >
                      ✏️
                    </button>
                    <button 
                      className="btn-icon btn-danger"
                      onClick={() => removeProperty(prop.id)}
                    >
                      🗑️
                    </button>
                  </div>
                </div>
                <div className="item-label">{prop.label}</div>
                <div className="property-signature">
                  <span className="domain">{prop.domain}</span>
                  <span className="arrow">→</span>
                  <span className="range">{prop.range}</span>
                </div>
                <div className="item-meta">
                  <span className="badge badge-primary">
                    {prop.type || 'ObjectProperty'}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {showClassModal && (
        <ClassModal
          ontologyClass={editingClass}
          existingClasses={ontology.classes}
          onSave={(cls) => {
            if (editingClass) {
              updateClass(cls);
            } else {
              addClass(cls);
            }
            setShowClassModal(false);
            setEditingClass(null);
          }}
          onClose={() => {
            setShowClassModal(false);
            setEditingClass(null);
          }}
        />
      )}

      {showPropertyModal && (
        <PropertyModal
          ontologyProperty={editingProperty}
          availableClasses={ontology.classes}
          onSave={(prop) => {
            if (editingProperty) {
              updateProperty(prop);
            } else {
              addProperty(prop);
            }
            setShowPropertyModal(false);
            setEditingProperty(null);
          }}
          onClose={() => {
            setShowPropertyModal(false);
            setEditingProperty(null);
          }}
        />
      )}
    </div>
  );
};

interface ClassModalProps {
  ontologyClass?: OntologyClass | null;
  existingClasses: OntologyClass[];
  onSave: (cls: OntologyClass) => void;
  onClose: () => void;
}

const ClassModal: React.FC<ClassModalProps> = ({ ontologyClass, existingClasses, onSave, onClose }) => {
  const [formData, setFormData] = useState<OntologyClass>({
    id: '',
    label: '',
    description: '',
    superClass: ''
  });

  useEffect(() => {
    if (ontologyClass) {
      setFormData(ontologyClass);
    }
  }, [ontologyClass]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.id && formData.label) {
      onSave({
        ...formData,
        superClass: formData.superClass || undefined
      });
    }
  };

  return (
    <div className="modal-overlay">
      <div className="modal">
        <div className="modal-header">
          <h3 className="modal-title">
            {ontologyClass ? 'Edit Class' : 'Add New Class'}
          </h3>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="input-group">
            <label className="input-label">Class ID</label>
            <input
              type="text"
              className="input-field"
              value={formData.id}
              onChange={(e) => setFormData({ ...formData, id: e.target.value })}
              placeholder="e.g., Product"
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
            <label className="input-label">Description</label>
            <textarea
              className="textarea-field"
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="Describe this class..."
            />
          </div>
          <div className="input-group">
            <label className="input-label">Super Class (optional)</label>
            <select
              className="select-field"
              value={formData.superClass || ''}
              onChange={(e) => setFormData({ ...formData, superClass: e.target.value })}
            >
              <option value="">No super class</option>
              {existingClasses.filter(cls => cls.id !== formData.id).map(cls => (
                <option key={cls.id} value={cls.id}>{cls.label}</option>
              ))}
            </select>
          </div>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              {ontologyClass ? 'Update' : 'Create'} Class
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

interface PropertyModalProps {
  ontologyProperty?: OntologyProperty | null;
  availableClasses: OntologyClass[];
  onSave: (prop: OntologyProperty) => void;
  onClose: () => void;
}

const PropertyModal: React.FC<PropertyModalProps> = ({ ontologyProperty, availableClasses, onSave, onClose }) => {
  const [formData, setFormData] = useState<OntologyProperty>({
    id: '',
    label: '',
    domain: '',
    range: '',
    type: 'ObjectProperty'
  });

  useEffect(() => {
    if (ontologyProperty) {
      setFormData(ontologyProperty);
    }
  }, [ontologyProperty]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.id && formData.label && formData.domain && formData.range) {
      onSave(formData);
    }
  };

  return (
    <div className="modal-overlay">
      <div className="modal">
        <div className="modal-header">
          <h3 className="modal-title">
            {ontologyProperty ? 'Edit Property' : 'Add New Property'}
          </h3>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="input-group">
            <label className="input-label">Property ID</label>
            <input
              type="text"
              className="input-field"
              value={formData.id}
              onChange={(e) => setFormData({ ...formData, id: e.target.value })}
              placeholder="e.g., partOf"
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
          <div className="grid grid-2">
            <div className="input-group">
              <label className="input-label">Domain</label>
              <select
                className="select-field"
                value={formData.domain}
                onChange={(e) => setFormData({ ...formData, domain: e.target.value })}
                required
              >
                <option value="">Select domain class</option>
                {availableClasses.map(cls => (
                  <option key={cls.id} value={cls.id}>{cls.label}</option>
                ))}
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">Range</label>
              <select
                className="select-field"
                value={formData.range}
                onChange={(e) => setFormData({ ...formData, range: e.target.value })}
                required
              >
                <option value="">Select range class</option>
                {availableClasses.map(cls => (
                  <option key={cls.id} value={cls.id}>{cls.label}</option>
                ))}
                <option value="xsd:string">String</option>
                <option value="xsd:integer">Integer</option>
                <option value="xsd:dateTime">DateTime</option>
                <option value="xsd:boolean">Boolean</option>
              </select>
            </div>
          </div>
          <div className="input-group">
            <label className="input-label">Property Type</label>
            <select
              className="select-field"
              value={formData.type}
              onChange={(e) => setFormData({ ...formData, type: e.target.value as 'ObjectProperty' | 'DatatypeProperty' })}
            >
              <option value="ObjectProperty">Object Property</option>
              <option value="DatatypeProperty">Datatype Property</option>
            </select>
          </div>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              {ontologyProperty ? 'Update' : 'Create'} Property
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};