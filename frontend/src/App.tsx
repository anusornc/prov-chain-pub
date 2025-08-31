import { useState, useEffect } from 'react';
import './App.css';
import Navigation from './components/layout/Navigation';
import Dashboard from './components/dashboard/Dashboard';
import BlockExplorer from './components/explorer/BlockExplorer';
import BlockDetails from './components/explorer/BlockDetails';
import { ThemeProvider } from './contexts/ThemeContext';
import LoadingSpinner from './components/ui/LoadingSpinner';
import type { Block, Transaction } from './types';

// Import existing components for backward compatibility
import OntologyManager from './features/ontology/OntologyManager';
import RDFTripleStore from './features/rdf/RDFTripleStore';
import KnowledgeGraph from './features/knowledge-graph/KnowledgeGraph';
import ProvenanceTracker from './features/provenance/ProvenanceTracker';
import TraceabilityQueries from './features/queries/TraceabilityQueries';

// Import new Phase 3 traceability components
import TraceabilityExplorer from './components/traceability/TraceabilityExplorer';
import SPARQLQueryBuilder from './components/traceability/SPARQLQueryBuilder';

export type TabType = 
  | 'dashboard'
  | 'explorer' 
  | 'blocks'
  | 'transactions'
  | 'search'
  | 'traceability'
  | 'items'
  | 'knowledge-graph'
  | 'timeline'
  | 'participants'
  | 'analytics'
  | 'semantic'
  | 'sparql'
  | 'query-builder'
  | 'ontology'
  | 'rdf'
  | 'provenance'
  | 'queries';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('dashboard');
  const [isLoading, setIsLoading] = useState(true);
  const [selectedBlock, setSelectedBlock] = useState<Block | null>(null);
  const [, setSelectedTransaction] = useState<Transaction | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Check if we have an auth token, if not, auto-login for development
        const token = localStorage.getItem('authToken');
        if (!token) {
          console.log('No auth token found, attempting auto-login for development...');
          try {
            const response = await fetch('http://localhost:8080/auth/login', {
              method: 'POST',
              headers: {
                'Content-Type': 'application/json',
              },
              body: JSON.stringify({
                username: 'admin',
                password: 'admin123'
              })
            });
            
            if (response.ok) {
              const authData = await response.json();
              localStorage.setItem('authToken', authData.token);
              console.log('Auto-login successful');
            } else {
              console.warn('Auto-login failed:', response.statusText);
            }
          } catch (authError) {
            console.warn('Auto-login error:', authError);
          }
        }
        
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

  const handleBlockSelect = (block: Block) => {
    setSelectedBlock(block);
  };

  const handleBlockBack = () => {
    setSelectedBlock(null);
  };

  const handleTransactionSelect = (transaction: Transaction) => {
    setSelectedTransaction(transaction);
  };

  const renderActiveTab = () => {
    switch (activeTab) {
      case 'dashboard':
        return <Dashboard />;
      
      // Block Explorer tabs
      case 'explorer':
      case 'blocks':
        if (selectedBlock) {
          return (
            <BlockDetails
              block={selectedBlock}
              onBack={handleBlockBack}
              onTransactionSelect={handleTransactionSelect}
            />
          );
        }
        return <BlockExplorer onBlockSelect={handleBlockSelect} />;
      
      case 'transactions':
        return (
          <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
            <div className="max-w-7xl mx-auto">
              <div className="text-center py-12">
                <div className="text-6xl mb-4">📊</div>
                <h2 className="text-2xl font-bold text-gray-800 dark:text-white mb-2">Transaction Explorer</h2>
                <p className="text-gray-600 dark:text-gray-300">Advanced transaction analysis coming soon</p>
              </div>
            </div>
          </div>
        );
      
      case 'search':
        return (
          <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
            <div className="max-w-7xl mx-auto">
              <div className="text-center py-12">
                <div className="text-6xl mb-4">🔍</div>
                <h2 className="text-2xl font-bold text-gray-800 dark:text-white mb-2">Advanced Search</h2>
                <p className="text-gray-600 dark:text-gray-300">Powerful search capabilities coming soon</p>
              </div>
            </div>
          </div>
        );
      
      // Traceability tabs
      case 'traceability':
      case 'items':
        return <TraceabilityExplorer />;
      
      case 'knowledge-graph':
        return <KnowledgeGraph />;
      
      case 'timeline':
        return (
          <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
            <div className="max-w-7xl mx-auto">
              <div className="text-center py-12">
                <div className="text-6xl mb-4">⏱️</div>
                <h2 className="text-2xl font-bold text-gray-800 dark:text-white mb-2">Traceability Timeline</h2>
                <p className="text-gray-600 dark:text-gray-300">Interactive timeline visualization coming soon</p>
              </div>
            </div>
          </div>
        );
      
      // Other tabs
      case 'participants':
        return (
          <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
            <div className="max-w-7xl mx-auto">
              <div className="text-center py-12">
                <div className="text-6xl mb-4">👥</div>
                <h2 className="text-2xl font-bold text-gray-800 dark:text-white mb-2">Participants</h2>
                <p className="text-gray-600 dark:text-gray-300">Participant management coming soon</p>
              </div>
            </div>
          </div>
        );
      
      case 'analytics':
        return (
          <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
            <div className="max-w-7xl mx-auto">
              <div className="text-center py-12">
                <div className="text-6xl mb-4">📈</div>
                <h2 className="text-2xl font-bold text-gray-800 dark:text-white mb-2">Analytics</h2>
                <p className="text-gray-600 dark:text-gray-300">Advanced analytics dashboard coming soon</p>
              </div>
            </div>
          </div>
        );
      
      // Semantic tabs
      case 'semantic':
      case 'sparql':
        return <TraceabilityQueries />;
      
      case 'query-builder':
        return <SPARQLQueryBuilder />;
      
      case 'ontology':
        return <OntologyManager />;
      
      // Legacy tabs for backward compatibility
      case 'rdf':
        return <RDFTripleStore />;
      
      case 'provenance':
        return <ProvenanceTracker />;
      
      case 'queries':
        return <TraceabilityQueries />;
      
      default:
        return <Dashboard />;
    }
  };

  const handleTabChange = (tab: string) => {
    setActiveTab(tab as TabType);
    // Reset selected items when changing tabs
    setSelectedBlock(null);
    setSelectedTransaction(null);
  };

  return (
    <ThemeProvider>
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex">
        {/* Navigation Sidebar */}
        <Navigation activeTab={activeTab} onTabChange={handleTabChange} />
        
        {/* Main Content */}
        <main className="flex-1 lg:ml-0">
          <div className="lg:pl-64">
            {renderActiveTab()}
          </div>
        </main>
      </div>
    </ThemeProvider>
  );
}

export default App;
