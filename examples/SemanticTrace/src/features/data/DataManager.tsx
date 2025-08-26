import React, { useState, useEffect } from 'react';
import './DataManager.css';
import { persistence } from '../../utils/persistence';

interface DataSummary {
  ontologyClasses: number;
  ontologyProperties: number;
  rdfTriples: number;
  provenanceEntities: number;
  provenanceRelations: number;
}

export const DataManager: React.FC = () => {
  const [dataSummary, setDataSummary] = useState<DataSummary>({
    ontologyClasses: 0,
    ontologyProperties: 0,
    rdfTriples: 0,
    provenanceEntities: 0,
    provenanceRelations: 0
  });
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  useEffect(() => {
    loadDataSummary();
  }, []);

  const loadDataSummary = async () => {
    try {
      const [ontologyData, triplesData, provenanceData] = await Promise.all([
        persistence.getItem('ontology'),
        persistence.getItem('rdf-triples'),
        persistence.getItem('provenance-data')
      ]);

      const ontology = ontologyData ? JSON.parse(ontologyData) : { classes: [], properties: [] };
      const triples = triplesData ? JSON.parse(triplesData) : [];
      const provenance = provenanceData ? JSON.parse(provenanceData) : { entities: [], relations: [] };

      setDataSummary({
        ontologyClasses: ontology.classes.length,
        ontologyProperties: ontology.properties.length,
        rdfTriples: triples.length,
        provenanceEntities: provenance.entities.length,
        provenanceRelations: provenance.relations.length
      });
    } catch (error) {
      console.error('Error loading data summary:', error);
    }
  };

  const showMessage = (type: 'success' | 'error', text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 5000);
  };

  const exportAllData = () => {
    setIsLoading(true);
    
    setTimeout(async () => {
      try {
        const [ontologyData, triplesData, provenanceData, queriesData] = await Promise.all([
          persistence.getItem('ontology'),
          persistence.getItem('rdf-triples'),
          persistence.getItem('provenance-data'),
          persistence.getItem('saved-queries')
        ]);

        const exportData = {
          timestamp: new Date().toISOString(),
          version: '1.0',
          ontology: ontologyData ? JSON.parse(ontologyData) : null,
          rdfTriples: triplesData ? JSON.parse(triplesData) : null,
          provenance: provenanceData ? JSON.parse(provenanceData) : null,
          savedQueries: queriesData ? JSON.parse(queriesData) : null
        };

        const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `traceability-data-${new Date().toISOString().split('T')[0]}.json`;
        a.click();
        URL.revokeObjectURL(url);

        showMessage('success', 'Data exported successfully!');
      } catch (error) {
        showMessage('error', 'Error exporting data. Please try again.');
      } finally {
        setIsLoading(false);
      }
    }, 1000);
  };

  const importData = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setIsLoading(true);
    
    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const content = e.target?.result as string;
        const data = JSON.parse(content);

        // Validate data structure
        if (!data.version || !data.timestamp) {
          throw new Error('Invalid data format');
        }

        // Import data
        if (data.ontology) {
          await persistence.setItem('ontology', JSON.stringify(data.ontology));
        }
        if (data.rdfTriples) {
          await persistence.setItem('rdf-triples', JSON.stringify(data.rdfTriples));
        }
        if (data.provenance) {
          await persistence.setItem('provenance-data', JSON.stringify(data.provenance));
        }
        if (data.savedQueries) {
          await persistence.setItem('saved-queries', JSON.stringify(data.savedQueries));
        }

        await loadDataSummary();
        showMessage('success', 'Data imported successfully!');
      } catch (error) {
        showMessage('error', 'Error importing data. Please check the file format.');
      } finally {
        setIsLoading(false);
        // Reset file input
        event.target.value = '';
      }
    };
    
    reader.readAsText(file);
  };

  const clearAllData = async () => {
    if (!window.confirm('Are you sure you want to clear all data? This action cannot be undone.')) {
      return;
    }

    setIsLoading(true);
    
    try {
      await Promise.all([
        persistence.removeItem('ontology'),
        persistence.removeItem('rdf-triples'),
        persistence.removeItem('provenance-data'),
        persistence.removeItem('saved-queries')
      ]);

      setDataSummary({
        ontologyClasses: 0,
        ontologyProperties: 0,
        rdfTriples: 0,
        provenanceEntities: 0,
        provenanceRelations: 0
      });

      showMessage('success', 'All data cleared successfully!');
    } catch (error) {
      showMessage('error', 'Error clearing data. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const generateSampleData = async () => {
    setIsLoading(true);
    
    try {
      // Generate comprehensive sample data
      const sampleOntology = {
        classes: [
          { id: 'Product', label: 'Product', description: 'A manufactured product' },
          { id: 'Component', label: 'Component', description: 'A component of a product' },
          { id: 'Supplier', label: 'Supplier', description: 'A supplier organization' },
          { id: 'Process', label: 'Process', description: 'A manufacturing process' },
          { id: 'Location', label: 'Location', description: 'A geographical location' },
          { id: 'Material', label: 'Material', description: 'Raw material' },
          { id: 'QualityCheck', label: 'Quality Check', description: 'Quality control process' }
        ],
        properties: [
          { id: 'partOf', label: 'part of', domain: 'Component', range: 'Product' },
          { id: 'suppliedBy', label: 'supplied by', domain: 'Component', range: 'Supplier' },
          { id: 'locatedAt', label: 'located at', domain: 'Process', range: 'Location' },
          { id: 'processedBy', label: 'processed by', domain: 'Product', range: 'Process' },
          { id: 'madeFrom', label: 'made from', domain: 'Component', range: 'Material' },
          { id: 'testedBy', label: 'tested by', domain: 'Product', range: 'QualityCheck' }
        ]
      };

      const sampleTriples = [
        // Products
        { subject: 'product:smartphone-001', predicate: 'rdf:type', object: 'Product' },
        { subject: 'product:smartphone-002', predicate: 'rdf:type', object: 'Product' },
        { subject: 'product:tablet-001', predicate: 'rdf:type', object: 'Product' },
        
        // Components
        { subject: 'component:battery-001', predicate: 'rdf:type', object: 'Component' },
        { subject: 'component:screen-001', predicate: 'rdf:type', object: 'Component' },
        { subject: 'component:processor-001', predicate: 'rdf:type', object: 'Component' },
        { subject: 'component:camera-001', predicate: 'rdf:type', object: 'Component' },
        
        // Part relationships
        { subject: 'component:battery-001', predicate: 'partOf', object: 'product:smartphone-001' },
        { subject: 'component:screen-001', predicate: 'partOf', object: 'product:smartphone-001' },
        { subject: 'component:processor-001', predicate: 'partOf', object: 'product:smartphone-001' },
        { subject: 'component:camera-001', predicate: 'partOf', object: 'product:smartphone-001' },
        
        // Suppliers
        { subject: 'supplier:acme-corp', predicate: 'rdf:type', object: 'Supplier' },
        { subject: 'supplier:tech-solutions', predicate: 'rdf:type', object: 'Supplier' },
        { subject: 'supplier:global-parts', predicate: 'rdf:type', object: 'Supplier' },
        
        // Supply relationships
        { subject: 'component:battery-001', predicate: 'suppliedBy', object: 'supplier:acme-corp' },
        { subject: 'component:screen-001', predicate: 'suppliedBy', object: 'supplier:tech-solutions' },
        { subject: 'component:processor-001', predicate: 'suppliedBy', object: 'supplier:global-parts' },
        
        // Processes
        { subject: 'process:assembly-001', predicate: 'rdf:type', object: 'Process' },
        { subject: 'process:testing-001', predicate: 'rdf:type', object: 'Process' },
        { subject: 'process:packaging-001', predicate: 'rdf:type', object: 'Process' },
        
        // Process relationships
        { subject: 'product:smartphone-001', predicate: 'processedBy', object: 'process:assembly-001' },
        { subject: 'product:smartphone-001', predicate: 'processedBy', object: 'process:testing-001' },
        
        // Locations
        { subject: 'location:factory-a', predicate: 'rdf:type', object: 'Location' },
        { subject: 'location:warehouse-b', predicate: 'rdf:type', object: 'Location' },
        { subject: 'process:assembly-001', predicate: 'locatedAt', object: 'location:factory-a' },
        { subject: 'process:testing-001', predicate: 'locatedAt', object: 'location:factory-a' }
      ];

      const sampleProvenance = {
        entities: [
          {
            id: 'entity:raw-lithium',
            label: 'Raw Lithium Batch #LI-001',
            type: 'Entity',
            attributes: { batchNumber: 'LI-001', purity: '99.9%', origin: 'Australia' },
            timestamp: '2024-01-10T08:00:00Z'
          },
          {
            id: 'entity:battery-cell',
            label: 'Battery Cell BC-2000',
            type: 'Entity',
            attributes: { cellType: 'Li-ion', capacity: '2500mAh', voltage: '3.7V' },
            timestamp: '2024-01-12T14:30:00Z'
          },
          {
            id: 'entity:final-battery',
            label: 'Assembled Battery Pack',
            type: 'Entity',
            attributes: { totalCapacity: '5000mAh', cells: '2', safety: 'UL certified' },
            timestamp: '2024-01-15T16:45:00Z'
          },
          {
            id: 'activity:cell-manufacturing',
            label: 'Battery Cell Manufacturing',
            type: 'Activity',
            attributes: { machine: 'CellMaker-3000', temperature: '25°C', humidity: '45%' },
            timestamp: '2024-01-12T10:00:00Z'
          },
          {
            id: 'activity:battery-assembly',
            label: 'Battery Pack Assembly',
            type: 'Activity',
            attributes: { assemblyLine: 'BAL-03', workstation: 'WS-07', duration: '30 minutes' },
            timestamp: '2024-01-15T14:00:00Z'
          },
          {
            id: 'activity:quality-test',
            label: 'Battery Quality Testing',
            type: 'Activity',
            attributes: { testType: 'Capacity & Safety', result: 'PASS', duration: '45 minutes' },
            timestamp: '2024-01-15T17:00:00Z'
          },
          {
            id: 'agent:technician-alice',
            label: 'Alice Johnson (Technician)',
            type: 'Agent',
            attributes: { employeeId: 'EMP101', certification: 'Battery Specialist', experience: '5 years' },
            timestamp: '2024-01-15T14:00:00Z'
          },
          {
            id: 'agent:qc-inspector-bob',
            label: 'Bob Wilson (QC Inspector)',
            type: 'Agent',
            attributes: { employeeId: 'EMP202', certification: 'Senior QC', experience: '8 years' },
            timestamp: '2024-01-15T17:00:00Z'
          }
        ],
        relations: [
          {
            id: 'rel-001',
            type: 'used',
            from: 'activity:cell-manufacturing',
            to: 'entity:raw-lithium',
            timestamp: '2024-01-12T10:00:00Z',
            attributes: { quantity: '2.5kg' }
          },
          {
            id: 'rel-002',
            type: 'wasGeneratedBy',
            from: 'entity:battery-cell',
            to: 'activity:cell-manufacturing',
            timestamp: '2024-01-12T14:30:00Z',
            attributes: { quantity: '2 cells' }
          },
          {
            id: 'rel-003',
            type: 'used',
            from: 'activity:battery-assembly',
            to: 'entity:battery-cell',
            timestamp: '2024-01-15T14:30:00Z',
            attributes: { quantity: '2 cells' }
          },
          {
            id: 'rel-004',
            type: 'wasGeneratedBy',
            from: 'entity:final-battery',
            to: 'activity:battery-assembly',
            timestamp: '2024-01-15T16:45:00Z',
            attributes: { serialNumber: 'BP-001234' }
          },
          {
            id: 'rel-005',
            type: 'wasAssociatedWith',
            from: 'activity:battery-assembly',
            to: 'agent:technician-alice',
            timestamp: '2024-01-15T14:00:00Z',
            attributes: { role: 'primary assembler' }
          },
          {
            id: 'rel-006',
            type: 'wasAssociatedWith',
            from: 'activity:quality-test',
            to: 'agent:qc-inspector-bob',
            timestamp: '2024-01-15T17:00:00Z',
            attributes: { role: 'quality inspector' }
          },
          {
            id: 'rel-007',
            type: 'wasInformedBy',
            from: 'activity:quality-test',
            to: 'activity:battery-assembly',
            timestamp: '2024-01-15T17:00:00Z',
            attributes: { testId: 'QT-001' }
          }
        ]
      };

      const sampleQueries = [
        {
          id: 'sample-1',
          name: 'Product Component Analysis',
          description: 'Analyze all components and their suppliers for products',
          category: 'Analysis',
          query: `SELECT ?product ?component ?supplier WHERE {
  ?component :partOf ?product .
  ?component :suppliedBy ?supplier .
  ?product rdf:type :Product .
}`
        },
        {
          id: 'sample-2',
          name: 'Full Supply Chain Trace',
          description: 'Complete supply chain traceability from raw materials to products',
          category: 'Supply Chain',
          query: `SELECT ?material ?component ?product ?supplier WHERE {
  ?component :madeFrom ?material .
  ?component :partOf ?product .
  ?component :suppliedBy ?supplier .
}`
        }
      ];

      // Save all sample data
      await Promise.all([
        persistence.setItem('ontology', JSON.stringify(sampleOntology)),
        persistence.setItem('rdf-triples', JSON.stringify(sampleTriples)),
        persistence.setItem('provenance-data', JSON.stringify(sampleProvenance)),
        persistence.setItem('saved-queries', JSON.stringify(sampleQueries))
      ]);

      await loadDataSummary();
      showMessage('success', 'Sample data generated successfully!');
    } catch (error) {
      showMessage('error', 'Error generating sample data. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">Data Manager</h2>
          <p className="feature-description">
            Import, export, and manage your traceability data across all system components
          </p>
        </div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <button 
            className="btn btn-secondary" 
            onClick={generateSampleData}
            disabled={isLoading}
          >
            {isLoading ? '⏳' : '🎲'} Generate Sample Data
          </button>
          <button 
            className="btn btn-primary" 
            onClick={exportAllData}
            disabled={isLoading}
          >
            {isLoading ? '⏳ Exporting...' : '📥 Export All Data'}
          </button>
        </div>
      </div>

      {message && (
        <div className={`notification notification-${message.type}`}>
          {message.text}
        </div>
      )}

      <div className="data-manager-grid">
        <div className="data-summary-section">
          <h3 className="section-title">Data Summary</h3>
          <div className="summary-cards">
            <div className="summary-card">
              <div className="card-icon">🏗️</div>
              <div className="card-content">
                <div className="card-value">{dataSummary.ontologyClasses}</div>
                <div className="card-label">Ontology Classes</div>
              </div>
            </div>
            <div className="summary-card">
              <div className="card-icon">🔗</div>
              <div className="card-content">
                <div className="card-value">{dataSummary.ontologyProperties}</div>
                <div className="card-label">Ontology Properties</div>
              </div>
            </div>
            <div className="summary-card">
              <div className="card-icon">🗄️</div>
              <div className="card-content">
                <div className="card-value">{dataSummary.rdfTriples}</div>
                <div className="card-label">RDF Triples</div>
              </div>
            </div>
            <div className="summary-card">
              <div className="card-icon">📦</div>
              <div className="card-content">
                <div className="card-value">{dataSummary.provenanceEntities}</div>
                <div className="card-label">Provenance Entities</div>
              </div>
            </div>
            <div className="summary-card">
              <div className="card-icon">🔄</div>
              <div className="card-content">
                <div className="card-value">{dataSummary.provenanceRelations}</div>
                <div className="card-label">Provenance Relations</div>
              </div>
            </div>
          </div>
        </div>

        <div className="data-operations-section">
          <h3 className="section-title">Data Operations</h3>
          <div className="operations-grid">
            <div className="operation-card">
              <div className="operation-header">
                <div className="operation-icon">📤</div>
                <h4>Export Data</h4>
              </div>
              <p>Export all system data including ontology, RDF triples, provenance information, and saved queries as a single JSON file.</p>
              <button 
                className="btn btn-primary full-width"
                onClick={exportAllData}
                disabled={isLoading}
              >
                {isLoading ? 'Exporting...' : 'Export All Data'}
              </button>
            </div>

            <div className="operation-card">
              <div className="operation-header">
                <div className="operation-icon">📥</div>
                <h4>Import Data</h4>
              </div>
              <p>Import data from a previously exported JSON file. This will merge with existing data.</p>
              <label className="btn btn-secondary full-width file-input-label">
                Choose File to Import
                <input
                  type="file"
                  accept=".json"
                  onChange={importData}
                  style={{ display: 'none' }}
                  disabled={isLoading}
                />
              </label>
            </div>

            <div className="operation-card">
              <div className="operation-header">
                <div className="operation-icon">🎲</div>
                <h4>Sample Data</h4>
              </div>
              <p>Generate comprehensive sample data to explore system capabilities and test different features.</p>
              <button 
                className="btn btn-secondary full-width"
                onClick={generateSampleData}
                disabled={isLoading}
              >
                {isLoading ? 'Generating...' : 'Generate Sample Data'}
              </button>
            </div>

            <div className="operation-card danger">
              <div className="operation-header">
                <div className="operation-icon">🗑️</div>
                <h4>Clear All Data</h4>
              </div>
              <p>Permanently delete all data from the system. This action cannot be undone.</p>
              <button 
                className="btn btn-danger full-width"
                onClick={clearAllData}
                disabled={isLoading}
              >
                {isLoading ? 'Clearing...' : 'Clear All Data'}
              </button>
            </div>
          </div>
        </div>

        <div className="data-formats-section">
          <h3 className="section-title">Supported Formats</h3>
          <div className="formats-grid">
            <div className="format-item">
              <div className="format-icon">🐢</div>
              <div className="format-content">
                <h4>Turtle (.ttl)</h4>
                <p>RDF serialization in Turtle format for ontologies and triples</p>
              </div>
            </div>
            <div className="format-item">
              <div className="format-icon">📄</div>
              <div className="format-content">
                <h4>RDF/XML (.rdf)</h4>
                <p>XML-based RDF serialization format</p>
              </div>
            </div>
            <div className="format-item">
              <div className="format-icon">📋</div>
              <div className="format-content">
                <h4>JSON-LD (.json)</h4>
                <p>JSON-based linked data format</p>
              </div>
            </div>
            <div className="format-item">
              <div className="format-icon">📊</div>
              <div className="format-content">
                <h4>System JSON</h4>
                <p>Complete system data backup format</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};