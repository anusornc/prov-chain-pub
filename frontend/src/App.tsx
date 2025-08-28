import { useState, useEffect } from 'react';
import './App.css';
import Header from './components/Header';
import OntologyManager from './features/ontology/OntologyManager';
import RDFTripleStore from './features/rdf/RDFTripleStore';
import KnowledgeGraph from './features/knowledge-graph/KnowledgeGraph';
import ProvenanceTracker from './features/provenance/ProvenanceTracker';
import TraceabilityQueries from './features/queries/TraceabilityQueries';
import { ThemeProvider } from './contexts/ThemeContext';
import LoadingSpinner from './components/ui/LoadingSpinner';

export type TabType = 'ontology' | 'rdf' | 'knowledge-graph' | 'provenance' | 'queries' | 'data';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('ontology');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Simulate initialization delay
        await new Promise(resolve => setTimeout(resolve, 1000));
      } catch (error) {
        console.error('Error initializing app:', error);
      } finally {
        setIsLoading(false);
      }
    };

    initializeApp();
  }, []);

  if (isLoading) {
    return (
      <ThemeProvider>
        <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center">
          <LoadingSpinner size="lg" message="Initializing ProvChain Explorer..." />
        </div>
      </ThemeProvider>
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
        return (
          <div className="feature-container">
            <div className="text-center py-12">
              <div className="text-6xl mb-4">📊</div>
              <h2 className="text-2xl font-bold text-gray-800 dark:text-white mb-2">Data Manager</h2>
              <p className="text-gray-600 dark:text-gray-300">Coming Soon</p>
            </div>
          </div>
        );
      default:
        return <OntologyManager />;
    }
  };

  return (
    <ThemeProvider>
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
        <Header activeTab={activeTab} onTabChange={setActiveTab} />
        <main className="main-content">
          {renderActiveTab()}
        </main>
      </div>
    </ThemeProvider>
  );
}

export default App;
