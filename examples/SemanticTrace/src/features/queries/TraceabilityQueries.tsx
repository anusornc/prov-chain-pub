import React, { useState, useEffect } from 'react';
import './TraceabilityQueries.css';
import { persistence } from '../../utils/persistence';

interface QueryResult {
  bindings: Record<string, any>[];
  variables: string[];
}

interface SavedQuery {
  id: string;
  name: string;
  description: string;
  query: string;
  category: string;
}

export const TraceabilityQueries: React.FC = () => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<QueryResult | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const [savedQueries, setSavedQueries] = useState<SavedQuery[]>([]);
  const [showSaveModal, setShowSaveModal] = useState(false);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [rdfData, setRdfData] = useState<any>({ triples: [], ontology: {}, provenance: {} });

  const predefinedQueries: SavedQuery[] = [
    {
      id: 'find-products',
      name: 'Find All Products',
      description: 'List all products in the system',
      query: `SELECT ?product ?label WHERE {
  ?product rdf:type :Product .
  OPTIONAL { ?product rdfs:label ?label }
}`,
      category: 'Basic'
    },
    {
      id: 'trace-components',
      name: 'Trace Product Components',
      description: 'Find all components of a specific product',
      query: `SELECT ?component ?product WHERE {
  ?component :partOf ?product .
  ?product rdf:type :Product .
}`,
      category: 'Traceability'
    },
    {
      id: 'supplier-chain',
      name: 'Supplier Chain Analysis',
      description: 'Map the supplier chain for components',
      query: `SELECT ?component ?supplier ?product WHERE {
  ?component :partOf ?product .
  ?component :suppliedBy ?supplier .
  ?supplier rdf:type :Supplier .
}`,
      category: 'Supply Chain'
    },
    {
      id: 'process-lineage',
      name: 'Process Lineage',
      description: 'Trace which processes created which products',
      query: `SELECT ?product ?process ?location WHERE {
  ?product :processedBy ?process .
  ?process rdf:type :Process .
  OPTIONAL { ?process :locatedAt ?location }
}`,
      category: 'Process'
    },
    {
      id: 'provenance-chain',
      name: 'Provenance Chain',
      description: 'Full provenance chain using PROV-O',
      query: `SELECT ?entity ?activity ?agent WHERE {
  ?entity prov:wasGeneratedBy ?activity .
  ?activity prov:wasAssociatedWith ?agent .
}`,
      category: 'Provenance'
    },
    {
      id: 'quality-traceability',
      name: 'Quality Traceability',
      description: 'Find quality control activities and their results',
      query: `SELECT ?entity ?activity ?result WHERE {
  ?entity prov:wasGeneratedBy ?activity .
  ?activity rdf:type :QualityTest .
  ?activity :testResult ?result .
}`,
      category: 'Quality'
    }
  ];

  useEffect(() => {
    loadRDFData();
    loadSavedQueries();
    // Set default query
    setQuery(predefinedQueries[0].query);
  }, []);

  const loadRDFData = async () => {
    try {
      const [triplesData, ontologyData, provenanceData] = await Promise.all([
        persistence.getItem('rdf-triples'),
        persistence.getItem('ontology'),
        persistence.getItem('provenance-data')
      ]);

      setRdfData({
        triples: triplesData ? JSON.parse(triplesData) : [],
        ontology: ontologyData ? JSON.parse(ontologyData) : { classes: [], properties: [] },
        provenance: provenanceData ? JSON.parse(provenanceData) : { entities: [], relations: [] }
      });
    } catch (error) {
      console.error('Error loading RDF data:', error);
    }
  };

  const loadSavedQueries = async () => {
    try {
      const data = await persistence.getItem('saved-queries');
      if (data) {
        setSavedQueries(JSON.parse(data));
      }
    } catch (error) {
      console.error('Error loading saved queries:', error);
    }
  };

  const saveQuery = async (queryData: Omit<SavedQuery, 'id'>) => {
    const newQuery = {
      ...queryData,
      id: `query-${Date.now()}`
    };
    const updatedQueries = [...savedQueries, newQuery];
    await persistence.setItem('saved-queries', JSON.stringify(updatedQueries));
    setSavedQueries(updatedQueries);
  };

  const deleteQuery = async (queryId: string) => {
    const updatedQueries = savedQueries.filter(q => q.id !== queryId);
    await persistence.setItem('saved-queries', JSON.stringify(updatedQueries));
    setSavedQueries(updatedQueries);
  };

  const executeQuery = async () => {
    if (!query.trim()) return;

    setIsExecuting(true);
    try {
      // Simulate SPARQL-like query execution
      const result = simulateSPARQLQuery(query, rdfData);
      setResults(result);
    } catch (error) {
      console.error('Query execution error:', error);
      setResults({
        bindings: [],
        variables: []
      });
    } finally {
      setIsExecuting(false);
    }
  };

  const simulateSPARQLQuery = (sparqlQuery: string, data: any): QueryResult => {
    // Simple SPARQL simulator - in a real implementation, this would use a proper SPARQL engine
    const lines = sparqlQuery.toLowerCase().split('\n');
    const selectLine = lines.find(line => line.trim().startsWith('select'));
    
    if (!selectLine) {
      return { bindings: [], variables: [] };
    }

    // Extract variables from SELECT clause
    const variables = extractVariables(selectLine);
    
    // Simulate different query patterns
    let bindings: any[] = [];

    if (sparqlQuery.includes('rdf:type :Product')) {
      bindings = findProductInstances(data.triples);
    } else if (sparqlQuery.includes(':partOf')) {
      bindings = findPartOfRelations(data.triples);
    } else if (sparqlQuery.includes(':suppliedBy')) {
      bindings = findSupplierRelations(data.triples);
    } else if (sparqlQuery.includes(':processedBy')) {
      bindings = findProcessRelations(data.triples);
    } else if (sparqlQuery.includes('prov:wasGeneratedBy')) {
      bindings = findProvenanceRelations(data.provenance);
    } else if (sparqlQuery.includes(':QualityTest')) {
      bindings = findQualityTestActivities(data.provenance);
    } else {
      // Generic pattern matching
      bindings = performGenericQuery(sparqlQuery, data);
    }

    return { bindings, variables };
  };

  const extractVariables = (selectClause: string): string[] => {
    const matches = selectClause.match(/\?(\w+)/g);
    return matches ? matches.map(v => v.slice(1)) : [];
  };

  const findProductInstances = (triples: any[]): any[] => {
    return triples
      .filter(t => t.predicate === 'rdf:type' && t.object === 'Product')
      .map(t => ({
        product: t.subject,
        label: t.subject.split(':')[1] || t.subject
      }));
  };

  const findPartOfRelations = (triples: any[]): any[] => {
    return triples
      .filter(t => t.predicate === 'partOf')
      .map(t => ({
        component: t.subject,
        product: t.object
      }));
  };

  const findSupplierRelations = (triples: any[]): any[] => {
    const partOfTriples = triples.filter(t => t.predicate === 'partOf');
    const suppliedByTriples = triples.filter(t => t.predicate === 'suppliedBy');
    
    return partOfTriples.map(partOf => {
      const suppliedBy = suppliedByTriples.find(s => s.subject === partOf.subject);
      return {
        component: partOf.subject,
        supplier: suppliedBy?.object || 'Unknown',
        product: partOf.object
      };
    });
  };

  const findProcessRelations = (triples: any[]): any[] => {
    return triples
      .filter(t => t.predicate === 'processedBy')
      .map(t => {
        const locationTriple = triples.find(lt => 
          lt.subject === t.object && lt.predicate === 'locatedAt'
        );
        return {
          product: t.subject,
          process: t.object,
          location: locationTriple?.object || null
        };
      });
  };

  const findProvenanceRelations = (provenance: any): any[] => {
    const generateRelations = provenance.relations?.filter((r: any) => 
      r.type === 'wasGeneratedBy'
    ) || [];
    
    const associateRelations = provenance.relations?.filter((r: any) => 
      r.type === 'wasAssociatedWith'
    ) || [];

    return generateRelations.map((gen: any) => {
      const assoc = associateRelations.find((a: any) => a.from === gen.to);
      return {
        entity: gen.from,
        activity: gen.to,
        agent: assoc?.to || null
      };
    });
  };

  const findQualityTestActivities = (provenance: any): any[] => {
    const activities = provenance.entities?.filter((e: any) => 
      e.type === 'Activity' && e.label.toLowerCase().includes('quality')
    ) || [];

    return activities.map((activity: any) => ({
      entity: activity.id,
      activity: activity.label,
      result: activity.attributes.result || 'Unknown'
    }));
  };

  const performGenericQuery = (sparqlQuery: string, data: any): any[] => {
    // Fallback generic query - just return some sample data
    return [
      { subject: 'product:smartphone-001', predicate: 'rdf:type', object: 'Product' },
      { subject: 'component:battery-001', predicate: 'partOf', object: 'product:smartphone-001' }
    ];
  };

  const formatResultValue = (value: any): string => {
    if (value === null || value === undefined) {
      return '-';
    }
    return String(value);
  };

  const allQueries = [...predefinedQueries, ...savedQueries];
  const filteredQueries = selectedCategory === 'all' 
    ? allQueries 
    : allQueries.filter(q => q.category === selectedCategory);

  const categories = ['all', ...Array.from(new Set(allQueries.map(q => q.category)))];

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">Traceability Queries</h2>
          <p className="feature-description">
            Execute SPARQL-like queries to analyze traceability and provenance data
          </p>
        </div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <button className="btn btn-secondary" onClick={() => setShowSaveModal(true)}>
            💾 Save Query
          </button>
          <button 
            className="btn btn-primary" 
            onClick={executeQuery}
            disabled={isExecuting}
          >
            {isExecuting ? '⏳ Executing...' : '▶️ Execute Query'}
          </button>
        </div>
      </div>

      <div className="query-interface">
        <div className="query-sidebar">
          <div className="sidebar-section">
            <h3>Query Library</h3>
            <div className="category-filter">
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                className="category-select"
              >
                {categories.map(cat => (
                  <option key={cat} value={cat}>
                    {cat === 'all' ? 'All Categories' : cat}
                  </option>
                ))}
              </select>
            </div>
            <div className="query-list">
              {filteredQueries.map(savedQuery => (
                <div key={savedQuery.id} className="query-item">
                  <div className="query-header">
                    <span className="query-name">{savedQuery.name}</span>
                    <div className="query-actions">
                      <button
                        className="btn-icon"
                        onClick={() => setQuery(savedQuery.query)}
                        title="Load query"
                      >
                        📋
                      </button>
                      {!predefinedQueries.find(pq => pq.id === savedQuery.id) && (
                        <button
                          className="btn-icon btn-danger"
                          onClick={() => deleteQuery(savedQuery.id)}
                          title="Delete query"
                        >
                          🗑️
                        </button>
                      )}
                    </div>
                  </div>
                  <div className="query-description">{savedQuery.description}</div>
                  <div className="query-category">
                    <span className="badge">{savedQuery.category}</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="query-main">
          <div className="query-editor">
            <div className="editor-header">
              <h3>SPARQL Query Editor</h3>
              <div className="editor-stats">
                Lines: {query.split('\n').length} | 
                Characters: {query.length}
              </div>
            </div>
            <textarea
              className="query-textarea"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Enter your SPARQL query here..."
              rows={12}
            />
          </div>

          <div className="query-results">
            <div className="results-header">
              <h3>Query Results</h3>
              {results && (
                <div className="results-stats">
                  {results.bindings.length} result(s) | {results.variables.length} variable(s)
                </div>
              )}
            </div>
            
            {results ? (
              <div className="results-table-container">
                {results.bindings.length > 0 ? (
                  <table className="results-table">
                    <thead>
                      <tr>
                        {results.variables.map(variable => (
                          <th key={variable}>{variable}</th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {results.bindings.map((binding, index) => (
                        <tr key={index}>
                          {results.variables.map(variable => (
                            <td key={variable}>
                              <span className="result-value">
                                {formatResultValue(binding[variable])}
                              </span>
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                ) : (
                  <div className="no-results">
                    <div className="no-results-icon">🔍</div>
                    <p>No results found for this query</p>
                  </div>
                )}
              </div>
            ) : (
              <div className="no-query">
                <div className="no-query-icon">💡</div>
                <p>Execute a query to see results here</p>
              </div>
            )}
          </div>
        </div>
      </div>

      {showSaveModal && (
        <SaveQueryModal
          onSave={(queryData) => {
            saveQuery(queryData);
            setShowSaveModal(false);
          }}
          onClose={() => setShowSaveModal(false)}
          currentQuery={query}
        />
      )}
    </div>
  );
};

interface SaveQueryModalProps {
  onSave: (queryData: Omit<SavedQuery, 'id'>) => void;
  onClose: () => void;
  currentQuery: string;
}

const SaveQueryModal: React.FC<SaveQueryModalProps> = ({ onSave, onClose, currentQuery }) => {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    category: 'Custom',
    query: currentQuery
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.name && formData.query) {
      onSave(formData);
    }
  };

  return (
    <div className="modal-overlay">
      <div className="modal">
        <div className="modal-header">
          <h3 className="modal-title">Save Query</h3>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="input-group">
            <label className="input-label">Query Name</label>
            <input
              type="text"
              className="input-field"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder="Enter query name"
              required
            />
          </div>
          <div className="input-group">
            <label className="input-label">Description</label>
            <textarea
              className="textarea-field"
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="Describe what this query does..."
              rows={3}
            />
          </div>
          <div className="input-group">
            <label className="input-label">Category</label>
            <input
              type="text"
              className="input-field"
              value={formData.category}
              onChange={(e) => setFormData({ ...formData, category: e.target.value })}
              placeholder="e.g., Custom, Analysis, etc."
              required
            />
          </div>
          <div className="input-group">
            <label className="input-label">Query</label>
            <textarea
              className="textarea-field"
              value={formData.query}
              onChange={(e) => setFormData({ ...formData, query: e.target.value })}
              rows={8}
              required
            />
          </div>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              Save Query
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};