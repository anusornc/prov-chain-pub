import React, { useState, useEffect } from 'react';
import './App.css';
import { OntologyManager } from './features/ontology/OntologyManager';
import { RDFTripleStore } from './features/rdf/RDFTripleStore';
import { KnowledgeGraph } from './features/knowledge-graph/KnowledgeGraph';
import { ProvenanceTracker } from './features/provenance/ProvenanceTracker';
import { TraceabilityQueries } from './features/queries/TraceabilityQueries';
import { DataManager } from './features/data/DataManager';
import { Header } from './components/Header';
import { TabNavigation } from './components/TabNavigation';
import { persistence } from './utils/persistence';

export type TabType = 'ontology' | 'rdf' | 'knowledge-graph' | 'provenance' | 'queries' | 'data';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('ontology');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Initialize with sample data if first time
        const hasData = await persistence.getItem('app-initialized');
        if (!hasData) {
          await initializeSampleData();
          await persistence.setItem('app-initialized', 'true');
        }
      } catch (error) {
        console.error('Error initializing app:', error);
      } finally {
        setIsLoading(false);
      }
    };

    initializeApp();
  }, []);

  const initializeSampleData = async () => {
    // Initialize sample ontology
    const sampleOntology = {
      classes: [
        { id: 'Product', label: 'Product', description: 'A manufactured product' },
        { id: 'Component', label: 'Component', description: 'A component of a product' },
        { id: 'Supplier', label: 'Supplier', description: 'A supplier organization' },
        { id: 'Process', label: 'Process', description: 'A manufacturing process' },
        { id: 'Location', label: 'Location', description: 'A geographical location' }
      ],
      properties: [
        { id: 'partOf', label: 'part of', domain: 'Component', range: 'Product' },
        { id: 'suppliedBy', label: 'supplied by', domain: 'Component', range: 'Supplier' },
        { id: 'locatedAt', label: 'located at', domain: 'Process', range: 'Location' },
        { id: 'processedBy', label: 'processed by', domain: 'Product', range: 'Process' }
      ]
    };

    // Initialize sample RDF triples
    const sampleTriples = [
      { subject: 'product:smartphone-001', predicate: 'rdf:type', object: 'Product' },
      { subject: 'component:battery-001', predicate: 'rdf:type', object: 'Component' },
      { subject: 'component:battery-001', predicate: 'partOf', object: 'product:smartphone-001' },
      { subject: 'supplier:acme-corp', predicate: 'rdf:type', object: 'Supplier' },
      { subject: 'component:battery-001', predicate: 'suppliedBy', object: 'supplier:acme-corp' },
      { subject: 'process:assembly-001', predicate: 'rdf:type', object: 'Process' },
      { subject: 'product:smartphone-001', predicate: 'processedBy', object: 'process:assembly-001' }
    ];

    await persistence.setItem('ontology', JSON.stringify(sampleOntology));
    await persistence.setItem('rdf-triples', JSON.stringify(sampleTriples));
  };

  if (isLoading) {
    return (
      <div className="app loading">
        <div className="loading-spinner">
          <div className="spinner"></div>
          <p>Initializing Semantic Traceability System...</p>
        </div>
      </div>
    );
  }

  const renderActiveTab = () => {
    switch (activeTab) {
      case 'ontology':
        return <OntologyManager />;
      case 'rdf':
        return <RDFTripleStore />;
      case 'knowledge-graph':
        return <KnowledgeGraph />;
      case 'provenance':
        return <ProvenanceTracker />;
      case 'queries':
        return <TraceabilityQueries />;
      case 'data':
        return <DataManager />;
      default:
        return <OntologyManager />;
    }
  };

  return (
    <div className="app">
      <Header />
      <TabNavigation activeTab={activeTab} onTabChange={setActiveTab} />
      <main className="main-content">
        {renderActiveTab()}
      </main>
    </div>
  );
}

export default App;