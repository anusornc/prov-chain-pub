import React, { useState } from 'react';
import { sparqlAPI } from '../../services/api';
import Card from '../../components/ui/Card';
import Button from '../../components/ui/Button';
import TextArea from '../../components/ui/TextArea';
import Alert from '../../components/ui/Alert';
import Badge from '../../components/ui/Badge';
import LoadingSpinner from '../../components/ui/LoadingSpinner';

const TraceabilityQueries: React.FC = () => {
  const [query, setQuery] = useState(`PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX core: <http://provchain.org/core#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?product ?process ?participant ?timestamp
WHERE {
  ?product rdf:type core:Product .
  ?process prov:used ?product .
  ?process prov:wasAssociatedWith ?participant .
  ?process prov:startedAtTime ?timestamp .
}
ORDER BY DESC(?timestamp)
LIMIT 10`);
  const [results, setResults] = useState<Record<string, { value: string; type: string }>[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const executeQuery = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await sparqlAPI.query(query);
      // Handle the SPARQL response structure correctly
      const bindings = response.data.results?.bindings || [];
      setResults(bindings);
    } catch (err: unknown) {
      const error = err as { response?: { data?: { message?: string } }; message?: string };
      setError(error.response?.data?.message || error.message || 'Failed to execute query');
    } finally {
      setLoading(false);
    }
  };

  const predefinedQueries = [
    {
      name: 'Product Traceability',
      query: `PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX core: <http://provchain.org/core#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?product ?process ?participant ?timestamp
WHERE {
  ?product rdf:type core:Product .
  ?process prov:used ?product .
  ?process prov:wasAssociatedWith ?participant .
  ?process prov:startedAtTime ?timestamp .
}
ORDER BY DESC(?timestamp)
LIMIT 10`
    },
    {
      name: 'Supplier Relationships',
      query: `PREFIX core: <http://provchain.org/core#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?supplier ?product ?component
WHERE {
  ?supplier rdf:type core:Supplier .
  ?component core:suppliedBy ?supplier .
  ?component core:partOf ?product .
}`
    },
    {
      name: 'Process Timeline',
      query: `PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX core: <http://provchain.org/core#>

SELECT ?process ?type ?startTime ?endTime ?participant
WHERE {
  ?process rdf:type core:Process .
  ?process prov:startedAtTime ?startTime .
  OPTIONAL { ?process prov:endedAtTime ?endTime }
  OPTIONAL { ?process prov:wasAssociatedWith ?participant }
}
ORDER BY DESC(?startTime)`
    }
  ];

  const loadPredefinedQuery = (predefinedQuery: string) => {
    setQuery(predefinedQuery);
  };

  return (
    <div className="feature-container">
      <div className="feature-header">
        <div>
          <h2 className="feature-title">Traceability Queries</h2>
          <p className="feature-description">
            Execute SPARQL queries to explore and analyze your semantic blockchain data
          </p>
        </div>
        <Button 
          variant="primary" 
          onClick={executeQuery}
          disabled={loading}
        >
          {loading ? 'Executing...' : '▶ Execute Query'}
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Query Editor">
          <div className="space-y-4">
            <TextArea
              label="SPARQL Query"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              rows={12}
              placeholder="Enter your SPARQL query here..."
              fullWidth
              className="font-mono text-sm"
            />
          </div>
        </Card>

        <Card title="Predefined Queries">
          <div className="space-y-2">
            {predefinedQueries.map((predefined, index) => (
              <Button
                key={index}
                variant="secondary"
                onClick={() => loadPredefinedQuery(predefined.query)}
                className="w-full justify-start font-medium"
              >
                {predefined.name}
              </Button>
            ))}
          </div>
        </Card>
      </div>

      {error && (
        <div className="mb-6">
          <Alert 
            variant="error" 
            title="Query Error"
            message={error}
            dismissible
            onClose={() => setError(null)}
          />
        </div>
      )}

      {loading && (
        <div className="my-8">
          <LoadingSpinner size="md" message="Executing query..." />
        </div>
      )}

      {results && results.length > 0 && !loading && (
        <Card title={`Query Results (${results.length} results)`}>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  {results[0] && Object.keys(results[0]).map((header) => (
                    <th key={header} className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      {header}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {results.map((row, index) => (
                  <tr key={index} className="hover:bg-gray-50">
                    {Object.values(row).map((cell: { value: string; type: string }, cellIndex) => (
                      <td key={cellIndex} className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        <div className="flex items-center">
                          <span>{cell?.value || ''}</span>
                          {cell?.type === 'uri' && (
                            <Badge variant="info" className="ml-2">URI</Badge>
                          )}
                          {cell?.type === 'literal' && (
                            <Badge variant="secondary" className="ml-2">Literal</Badge>
                          )}
                        </div>
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}

      {(!results || results.length === 0) && !loading && !error && (
        <Card>
          <div className="text-center py-8">
            <div className="text-4xl mb-4">🔍</div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">No Results Yet</h3>
            <p className="text-gray-600 dark:text-gray-300">
              Execute a query to see results here. Try one of the predefined queries or write your own SPARQL query.
            </p>
          </div>
        </Card>
      )}
    </div>
  );
};

export default TraceabilityQueries;
