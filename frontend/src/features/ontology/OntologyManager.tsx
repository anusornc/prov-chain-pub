import React, { useState, useEffect } from 'react';
import Card from '../../components/ui/Card';
import Button from '../../components/ui/Button';
import Badge from '../../components/ui/Badge';
import LoadingSpinner from '../../components/ui/LoadingSpinner';
import Alert from '../../components/ui/Alert';
import { sparqlAPI } from '../../services/api';

interface OntologyClass {
  id: string;
  label: string;
  description: string;
}

interface OntologyProperty {
  id: string;
  label: string;
  domain: string;
  range: string;
}


const OntologyManager: React.FC = () => {
  const [classes, setClasses] = useState<OntologyClass[]>([]);
  const [properties, setProperties] = useState<OntologyProperty[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadOntologyData();
  }, []);

  const loadOntologyData = async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      // Load ontology classes
      const classesQuery = `
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX owl: <http://www.w3.org/2002/07/owl#>
        PREFIX core: <http://provchain.org/core#>
        
        SELECT ?class ?label ?comment WHERE {
          ?class a owl:Class .
          OPTIONAL { ?class rdfs:label ?label }
          OPTIONAL { ?class rdfs:comment ?comment }
          FILTER(STRSTARTS(STR(?class), "http://provchain.org/core#"))
        }
        ORDER BY ?class
      `;
      
      const classesResponse: any = await sparqlAPI.query(classesQuery);
      const loadedClasses: OntologyClass[] = classesResponse.data.results.results.bindings.map((binding: any) => {
        const classUri = binding.class?.value || '';
        const classId = classUri.replace('http://provchain.org/core#', '');
        return {
          id: classId,
          label: binding.label?.value || classId,
          description: binding.comment?.value || 'No description available'
        };
      });
      
      // Load ontology properties
      const propertiesQuery = `
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX owl: <http://www.w3.org/2002/07/owl#>
        PREFIX core: <http://provchain.org/core#>
        
        SELECT ?property ?label ?domain ?range WHERE {
          ?property a owl:ObjectProperty .
          OPTIONAL { ?property rdfs:label ?label }
          OPTIONAL { ?property rdfs:domain ?domain }
          OPTIONAL { ?property rdfs:range ?range }
          FILTER(STRSTARTS(STR(?property), "http://provchain.org/core#"))
        }
        ORDER BY ?property
      `;
      
      const propertiesResponse = await sparqlAPI.query(propertiesQuery);
      const loadedProperties: OntologyProperty[] = propertiesResponse.data.results.results.bindings.map((binding: any) => ({
        id: binding.property.value.replace('http://provchain.org/core#', ''),
        label: binding.label?.value || binding.property.value.replace('http://provchain.org/core#', ''),
        domain: binding.domain?.value ? binding.domain.value.replace('http://provchain.org/core#', '') : 'Unknown',
        range: binding.range?.value ? binding.range.value.replace('http://provchain.org/core#', '') : 'Unknown'
      }));

      setClasses(loadedClasses);
      setProperties(loadedProperties);
    } catch (err: any) {
      setError('Failed to load ontology data: ' + (err.response?.data?.message || err.message || 'Please try again.'));
      console.error('Error loading ontology data:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const refreshData = () => {
    loadOntologyData();
  };

  if (isLoading) {
    return (
      <div className="feature-container">
        <LoadingSpinner size="lg" message="Loading Ontology..." />
      </div>
    );
  }

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">Ontology Management</h2>
          <p className="feature-description">
            Manage and explore the semantic ontology used for traceability
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="secondary" onClick={refreshData}>
            🔄 Refresh
          </Button>
          <Button variant="primary">
            ➕ Add New Class
          </Button>
        </div>
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

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title={`Classes (${classes.length})`}>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">ID</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Label</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Description</th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {classes.map((cls) => (
                  <tr key={cls.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant="primary">{cls.id}</Badge>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {cls.label}
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-500">
                      {cls.description}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        <Card title={`Properties (${properties.length})`}>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">ID</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Label</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Domain</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Range</th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {properties.map((prop) => (
                  <tr key={prop.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant="secondary">{prop.id}</Badge>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {prop.label}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant="default">{prop.domain}</Badge>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <Badge variant="default">{prop.range}</Badge>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      </div>

      <Card title="Ontology Statistics" className="mt-6">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-6 text-center">
            <div className="text-3xl mb-2">🏛️</div>
            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">{classes.length}</div>
            <div className="text-sm text-gray-600 dark:text-gray-300">Classes</div>
          </div>
          <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-6 text-center">
            <div className="text-3xl mb-2">🔗</div>
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">{properties.length}</div>
            <div className="text-sm text-gray-600 dark:text-gray-300">Properties</div>
          </div>
          <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-6 text-center">
            <div className="text-3xl mb-2">📊</div>
            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">{classes.length + properties.length}</div>
            <div className="text-sm text-gray-600 dark:text-gray-300">Total Elements</div>
          </div>
        </div>
      </Card>
    </div>
  );
};

export default OntologyManager;
