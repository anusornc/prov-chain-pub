import React, { useState, useEffect } from 'react';
import Card from '../../components/ui/Card';
import Button from '../../components/ui/Button';
import Input from '../../components/ui/Input';
import Badge from '../../components/ui/Badge';
import Alert from '../../components/ui/Alert';
import LoadingSpinner from '../../components/ui/LoadingSpinner';
import { sparqlAPI, rdfAPI } from '../../services/api';

interface RDFTriple {
  subject: string;
  predicate: string;
  object: string;
}

const RDFTripleStore: React.FC = () => {
  const [triples, setTriples] = useState<RDFTriple[]>([]);
  const [newTriple, setNewTriple] = useState({ subject: '', predicate: '', object: '' });
  const [isLoading, setIsLoading] = useState(false);
  const [isDataLoading, setIsDataLoading] = useState(true);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadRealData();
  }, []);

  const loadRealData = async () => {
    try {
      setIsDataLoading(true);
      setError(null);
      
      // Load real RDF triples from the SPARQL endpoint
      const query = `
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX core: <http://provchain.org/core#>
        
        SELECT ?subject ?predicate ?object WHERE {
          ?subject ?predicate ?object .
        }
        LIMIT 50
      `;
      
      const response: any = await sparqlAPI.query(query);
      const loadedTriples: RDFTriple[] = response.data.results.results.bindings.map((binding: any) => ({
        subject: binding.subject.value,
        predicate: binding.predicate.value,
        object: binding.object.value
      }));
      
      setTriples(loadedTriples);
    } catch (err: any) {
      setError('Failed to load RDF data: ' + (err.response?.data?.message || err.message || 'Please try again.'));
      console.error('Error loading RDF data:', err);
      
      // Fallback to sample data
      const sampleTriples: RDFTriple[] = [
        { subject: 'http://provchain.org/core#Product', predicate: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#type', object: 'http://www.w3.org/2002/07/owl#Class' },
        { subject: 'http://provchain.org/core#Process', predicate: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#type', object: 'http://www.w3.org/2002/07/owl#Class' },
        { subject: 'http://provchain.org/core#Supplier', predicate: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#type', object: 'http://www.w3.org/2002/07/owl#Class' },
        { subject: 'http://provchain.org/core#derivedFrom', predicate: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#type', object: 'http://www.w3.org/2002/07/owl#ObjectProperty' },
        { subject: 'http://provchain.org/core#Certificate', predicate: 'http://www.w3.org/2000/01/rdf-schema#label', object: 'Certificate' }
      ];
      setTriples(sampleTriples);
    } finally {
      setIsDataLoading(false);
    }
  };

  const handleAddTriple = async () => {
    if (!newTriple.subject || !newTriple.predicate || !newTriple.object) {
      setMessage({ type: 'error', text: 'Please fill in all fields' });
      return;
    }

    setIsLoading(true);
    setMessage(null);
    try {
      // Call the real API to add the triple
      await rdfAPI.addTriple(newTriple);
      
      // Add to local state and refresh data
      setTriples(prev => [...prev, newTriple]);
      setNewTriple({ subject: '', predicate: '', object: '' });
      setMessage({ type: 'success', text: 'Triple added successfully!' });
      
      // Clear message after 3 seconds
      setTimeout(() => setMessage(null), 3000);
    } catch (err: any) {
      const errorMessage = 'Error adding triple: ' + (err.response?.data?.message || err.message || 'Please try again.');
      setMessage({ type: 'error', text: errorMessage });
      console.error('Error adding triple:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputChange = (field: keyof typeof newTriple, value: string) => {
    setNewTriple(prev => ({ ...prev, [field]: value }));
    // Clear error message when user starts typing
    if (message?.type === 'error') {
      setMessage(null);
    }
  };

  const refreshData = () => {
    loadRealData();
  };

  if (isDataLoading) {
    return (
      <div className="feature-container">
        <LoadingSpinner size="lg" message="Loading RDF Data..." />
      </div>
    );
  }

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">RDF Triple Store</h2>
          <p className="feature-description">
            Manage and query RDF triples in the semantic blockchain
          </p>
        </div>
        <Button variant="secondary" onClick={refreshData}>
          🔄 Refresh Data
        </Button>
      </div>

      {error && (
        <div className="mb-6">
          <Alert 
            variant="error" 
            message={error}
            dismissible
            onClose={() => setError(null)}
          />
        </div>
      )}

      {message && (
        <div className="mb-6">
          <Alert 
            variant={message.type === 'success' ? 'success' : 'error'}
            message={message.text}
            dismissible
            onClose={() => setMessage(null)}
          />
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Add New Triple">
          <div className="space-y-4">
            <Input
              label="Subject"
              value={newTriple.subject}
              onChange={(e) => handleInputChange('subject', e.target.value)}
              placeholder="e.g., http://example.org/product1"
              fullWidth
            />
            <Input
              label="Predicate"
              value={newTriple.predicate}
              onChange={(e) => handleInputChange('predicate', e.target.value)}
              placeholder="e.g., http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
              fullWidth
            />
            <Input
              label="Object"
              value={newTriple.object}
              onChange={(e) => handleInputChange('object', e.target.value)}
              placeholder="e.g., http://provchain.org/core#Product"
              fullWidth
            />
            <Button
              variant="primary"
              onClick={handleAddTriple}
              disabled={isLoading}
              fullWidth
            >
              {isLoading ? 'Adding...' : '➕ Add Triple'}
            </Button>
          </div>
        </Card>

        <Card title="Statistics">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4 text-center">
              <div className="text-2xl mb-1">🔢</div>
              <div className="text-xl font-bold text-blue-600 dark:text-blue-400">{triples.length}</div>
              <div className="text-xs text-gray-600 dark:text-gray-300">Total Triples</div>
            </div>
            <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4 text-center">
              <div className="text-2xl mb-1">🏷️</div>
              <div className="text-xl font-bold text-green-600 dark:text-green-400">{new Set(triples.map(t => t.predicate)).size}</div>
              <div className="text-xs text-gray-600 dark:text-gray-300">Unique Predicates</div>
            </div>
            <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4 text-center">
              <div className="text-2xl mb-1">🎯</div>
              <div className="text-xl font-bold text-purple-600 dark:text-purple-400">{new Set(triples.map(t => t.subject)).size}</div>
              <div className="text-xs text-gray-600 dark:text-gray-300">Unique Subjects</div>
            </div>
          </div>
        </Card>
      </div>

      <Card title={`RDF Triples (${triples.length})`} className="mt-6">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Subject</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Predicate</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Object</th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {triples.map((triple, index) => (
                <tr key={index} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <Badge variant="primary" title={triple.subject}>
                      {triple.subject.length > 50 ? triple.subject.substring(0, 50) + '...' : triple.subject}
                    </Badge>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <Badge variant="secondary" title={triple.predicate}>
                      {triple.predicate.length > 40 ? triple.predicate.substring(0, 40) + '...' : triple.predicate}
                    </Badge>
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-900" title={triple.object}>
                    {triple.object.length > 60 ? triple.object.substring(0, 60) + '...' : triple.object}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
};

export default RDFTripleStore;
