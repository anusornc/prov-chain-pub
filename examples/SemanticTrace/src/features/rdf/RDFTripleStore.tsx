import React, { useState, useEffect } from 'react';
import './RDFTripleStore.css';
import { persistence } from '../../utils/persistence';

interface RDFTriple {
  subject: string;
  predicate: string;
  object: string;
  id?: string;
}

export const RDFTripleStore: React.FC = () => {
  const [triples, setTriples] = useState<RDFTriple[]>([]);
  const [showAddModal, setShowAddModal] = useState(false);
  const [editingTriple, setEditingTriple] = useState<RDFTriple | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'subject' | 'predicate' | 'object'>('all');
  const [ontology, setOntology] = useState<any>({ classes: [], properties: [] });

  useEffect(() => {
    loadTriples();
    loadOntology();
  }, []);

  const loadTriples = async () => {
    try {
      const data = await persistence.getItem('rdf-triples');
      if (data) {
        const triplesData = JSON.parse(data);
        const triplesWithIds = triplesData.map((triple: RDFTriple, index: number) => ({
          ...triple,
          id: triple.id || `triple-${index}`
        }));
        setTriples(triplesWithIds);
      }
    } catch (error) {
      console.error('Error loading triples:', error);
    }
  };

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

  const saveTriples = async (newTriples: RDFTriple[]) => {
    try {
      await persistence.setItem('rdf-triples', JSON.stringify(newTriples));
      setTriples(newTriples);
    } catch (error) {
      console.error('Error saving triples:', error);
    }
  };

  const addTriple = (newTriple: RDFTriple) => {
    const tripleWithId = {
      ...newTriple,
      id: `triple-${Date.now()}`
    };
    const updatedTriples = [...triples, tripleWithId];
    saveTriples(updatedTriples);
  };

  const updateTriple = (updatedTriple: RDFTriple) => {
    const updatedTriples = triples.map(triple =>
      triple.id === updatedTriple.id ? updatedTriple : triple
    );
    saveTriples(updatedTriples);
  };

  const removeTriple = (tripleId: string) => {
    const updatedTriples = triples.filter(triple => triple.id !== tripleId);
    saveTriples(updatedTriples);
  };

  const filteredTriples = triples.filter(triple => {
    if (!searchQuery) return true;
    
    const query = searchQuery.toLowerCase();
    switch (filterType) {
      case 'subject':
        return triple.subject.toLowerCase().includes(query);
      case 'predicate':
        return triple.predicate.toLowerCase().includes(query);
      case 'object':
        return triple.object.toLowerCase().includes(query);
      default:
        return (
          triple.subject.toLowerCase().includes(query) ||
          triple.predicate.toLowerCase().includes(query) ||
          triple.object.toLowerCase().includes(query)
        );
    }
  });

  const exportTriples = (format: 'turtle' | 'rdfxml' | 'jsonld') => {
    let content = '';
    
    switch (format) {
      case 'turtle':
        content = generateTurtle();
        break;
      case 'rdfxml':
        content = generateRDFXML();
        break;
      case 'jsonld':
        content = generateJSONLD();
        break;
    }
    
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `triples.${format === 'turtle' ? 'ttl' : format === 'rdfxml' ? 'rdf' : 'json'}`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const generateTurtle = (): string => {
    let turtle = `@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix : <http://example.org/traceability#> .

`;

    triples.forEach(triple => {
      const subject = formatResource(triple.subject);
      const predicate = formatResource(triple.predicate);
      const object = formatResource(triple.object);
      turtle += `${subject} ${predicate} ${object} .\n`;
    });

    return turtle;
  };

  const generateRDFXML = (): string => {
    let rdf = `<?xml version="1.0" encoding="UTF-8"?>
<rdf:RDF
  xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
  xmlns="http://example.org/traceability#">

`;

    const subjectGroups = triples.reduce((acc, triple) => {
      if (!acc[triple.subject]) {
        acc[triple.subject] = [];
      }
      acc[triple.subject].push(triple);
      return acc;
    }, {} as Record<string, RDFTriple[]>);

    Object.entries(subjectGroups).forEach(([subject, subjectTriples]) => {
      rdf += `  <rdf:Description rdf:about="${subject}">\n`;
      subjectTriples.forEach(triple => {
        const predicate = triple.predicate.replace(':', '_');
        rdf += `    <${predicate}>${triple.object}</${predicate}>\n`;
      });
      rdf += `  </rdf:Description>\n\n`;
    });

    rdf += `</rdf:RDF>`;
    return rdf;
  };

  const generateJSONLD = (): string => {
    const context = {
      "@context": {
        "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
        "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
        "": "http://example.org/traceability#"
      }
    };

    const graph = triples.map(triple => ({
      "@id": triple.subject,
      [triple.predicate]: triple.object
    }));

    return JSON.stringify({ ...context, "@graph": graph }, null, 2);
  };

  const formatResource = (resource: string): string => {
    if (resource.startsWith('http://') || resource.startsWith('https://')) {
      return `<${resource}>`;
    }
    if (resource.includes(':')) {
      return resource;
    }
    if (resource.startsWith('"') && resource.endsWith('"')) {
      return resource;
    }
    return `:${resource}`;
  };

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">RDF Triple Store</h2>
          <p className="feature-description">
            Manage RDF triples representing your knowledge base using subject-predicate-object statements
          </p>
        </div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <select 
            className="export-select"
            onChange={(e) => {
              if (e.target.value) {
                exportTriples(e.target.value as 'turtle' | 'rdfxml' | 'jsonld');
                e.target.value = '';
              }
            }}
          >
            <option value="">📥 Export As...</option>
            <option value="turtle">Turtle (.ttl)</option>
            <option value="rdfxml">RDF/XML (.rdf)</option>
            <option value="jsonld">JSON-LD (.json)</option>
          </select>
          <button className="btn btn-primary" onClick={() => setShowAddModal(true)}>
            ➕ Add Triple
          </button>
        </div>
      </div>

      <div className="rdf-controls">
        <div className="search-controls">
          <div className="search-group">
            <input
              type="text"
              className="search-input"
              placeholder="Search triples..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            <select
              className="filter-select"
              value={filterType}
              onChange={(e) => setFilterType(e.target.value as any)}
            >
              <option value="all">All fields</option>
              <option value="subject">Subject</option>
              <option value="predicate">Predicate</option>
              <option value="object">Object</option>
            </select>
          </div>
        </div>
        
        <div className="stats">
          <span className="stat-item">
            <strong>{filteredTriples.length}</strong> triples
          </span>
          <span className="stat-item">
            <strong>{new Set(triples.map(t => t.subject)).size}</strong> subjects
          </span>
          <span className="stat-item">
            <strong>{new Set(triples.map(t => t.predicate)).size}</strong> predicates
          </span>
        </div>
      </div>

      <div className="triples-table-container">
        <table className="triples-table">
          <thead>
            <tr>
              <th>Subject</th>
              <th>Predicate</th>
              <th>Object</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {filteredTriples.map(triple => (
              <tr key={triple.id}>
                <td>
                  <span className="resource-subject">{triple.subject}</span>
                </td>
                <td>
                  <span className="resource-predicate">{triple.predicate}</span>
                </td>
                <td>
                  <span className="resource-object">{triple.object}</span>
                </td>
                <td>
                  <div className="table-actions">
                    <button
                      className="btn-icon"
                      onClick={() => {
                        setEditingTriple(triple);
                        setShowAddModal(true);
                      }}
                    >
                      ✏️
                    </button>
                    <button
                      className="btn-icon btn-danger"
                      onClick={() => removeTriple(triple.id!)}
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

      {showAddModal && (
        <TripleModal
          triple={editingTriple}
          ontology={ontology}
          onSave={(triple) => {
            if (editingTriple) {
              updateTriple({ ...triple, id: editingTriple.id });
            } else {
              addTriple(triple);
            }
            setShowAddModal(false);
            setEditingTriple(null);
          }}
          onClose={() => {
            setShowAddModal(false);
            setEditingTriple(null);
          }}
        />
      )}
    </div>
  );
};

interface TripleModalProps {
  triple?: RDFTriple | null;
  ontology: any;
  onSave: (triple: RDFTriple) => void;
  onClose: () => void;
}

const TripleModal: React.FC<TripleModalProps> = ({ triple, ontology, onSave, onClose }) => {
  const [formData, setFormData] = useState<RDFTriple>({
    subject: '',
    predicate: '',
    object: ''
  });

  useEffect(() => {
    if (triple) {
      setFormData(triple);
    }
  }, [triple]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.subject && formData.predicate && formData.object) {
      onSave(formData);
    }
  };

  const commonPredicates = [
    'rdf:type',
    'rdfs:label',
    'rdfs:comment',
    'rdfs:subClassOf',
    'rdfs:domain',
    'rdfs:range',
    ...ontology.properties.map((p: any) => p.id)
  ];

  const commonObjects = [
    ...ontology.classes.map((c: any) => c.id),
    'xsd:string',
    'xsd:integer',
    'xsd:dateTime',
    'xsd:boolean'
  ];

  return (
    <div className="modal-overlay">
      <div className="modal">
        <div className="modal-header">
          <h3 className="modal-title">
            {triple ? 'Edit Triple' : 'Add New Triple'}
          </h3>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="input-group">
            <label className="input-label">Subject</label>
            <input
              type="text"
              className="input-field"
              value={formData.subject}
              onChange={(e) => setFormData({ ...formData, subject: e.target.value })}
              placeholder="e.g., product:smartphone-001"
              required
            />
          </div>
          <div className="input-group">
            <label className="input-label">Predicate</label>
            <input
              type="text"
              className="input-field"
              list="predicates"
              value={formData.predicate}
              onChange={(e) => setFormData({ ...formData, predicate: e.target.value })}
              placeholder="e.g., rdf:type"
              required
            />
            <datalist id="predicates">
              {commonPredicates.map(pred => (
                <option key={pred} value={pred} />
              ))}
            </datalist>
          </div>
          <div className="input-group">
            <label className="input-label">Object</label>
            <input
              type="text"
              className="input-field"
              list="objects"
              value={formData.object}
              onChange={(e) => setFormData({ ...formData, object: e.target.value })}
              placeholder="e.g., Product"
              required
            />
            <datalist id="objects">
              {commonObjects.map(obj => (
                <option key={obj} value={obj} />
              ))}
            </datalist>
          </div>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              {triple ? 'Update' : 'Add'} Triple
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};