import React, { useState, useEffect } from 'react';
import { useSPARQL } from '../../hooks/useSPARQL';
import { 
  Play, 
  Save, 
  Download, 
  Copy, 
  Trash2, 
  Star, 
  StarOff, 
  Code, 
  Table, 
  FileText, 
  Settings, 
  ChevronDown, 
  ChevronRight, 
  CheckCircle, 
  Clock, 
  Database,
  BookOpen,
  Zap,
  X
} from 'lucide-react';
import Button from '../ui/Button';
import Input from '../ui/Input';
import Card from '../ui/Card';
import Badge from '../ui/Badge';
import LoadingSpinner from '../ui/LoadingSpinner';
import Alert from '../ui/Alert';
import type { SPARQLQuery, SPARQLResult } from '../../types';
import type { QueryTemplate } from '../../services/sparql';

interface SPARQLQueryBuilderProps {
  onQueryExecute?: (query: SPARQLQuery, results: SPARQLResult) => void;
  onQuerySave?: (query: SPARQLQuery) => void;
  initialQuery?: SPARQLQuery;
  className?: string;
}

const SPARQLQueryBuilder: React.FC<SPARQLQueryBuilderProps> = ({
  onQueryExecute,
  onQuerySave,
  initialQuery,
  className = ''
}) => {
  const {
    executeQuery,
    currentQuery,
    setCurrentQuery,
    queryResult,
    executionTime,
    isExecuting,
    executionError,
    savedQueries,
    saveQuery,
    deleteQuery,
    toggleFavorite,
    savedQueriesLoading,
    savedQueriesError,
    templates,
    configLoading,
    validateQuery,
    validationResult,
    isValidating,
    loadTemplate,
    fillTemplate,
    formatResults,
    clearResults
  } = useSPARQL();

  // UI state
  const [activeTab, setActiveTab] = useState<'editor' | 'templates' | 'saved' | 'results'>('editor');
  const [showSettings, setShowSettings] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<QueryTemplate | null>(null);
  const [templateParameters, setTemplateParameters] = useState<Record<string, string>>({});
  const [queryName, setQueryName] = useState('');
  const [queryDescription, setQueryDescription] = useState('');
  const [showSaveDialog, setShowSaveDialog] = useState(false);
  const [resultsView, setResultsView] = useState<'table' | 'json'>('table');
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set(['traceability']));

  // Initialize with initial query if provided
  useEffect(() => {
    if (initialQuery && !currentQuery) {
      setCurrentQuery(initialQuery.query);
      setQueryName(initialQuery.name || '');
      setQueryDescription(initialQuery.description || '');
    }
  }, [initialQuery, currentQuery, setCurrentQuery]);

  const handleExecuteQuery = async () => {
    if (!currentQuery.trim()) return;

    try {
      const result = await executeQuery(currentQuery);
      
      if (onQueryExecute) {
        const queryObj: SPARQLQuery = {
          query: currentQuery,
          name: queryName,
          description: queryDescription,
          execution_time_ms: executionTime
        };
        onQueryExecute(queryObj, result);
      }

      setActiveTab('results');
    } catch (error) {
      console.error('Query execution failed:', error);
    }
  };

  const handleSaveQuery = async () => {
    if (!currentQuery.trim() || !queryName.trim()) return;

    try {
      const queryToSave: SPARQLQuery = {
        name: queryName,
        query: currentQuery,
        description: queryDescription,
        created_at: new Date().toISOString()
      };

      await saveQuery(queryToSave);
      
      if (onQuerySave) {
        onQuerySave(queryToSave);
      }

      setShowSaveDialog(false);
      setQueryName('');
      setQueryDescription('');
    } catch (error) {
      console.error('Failed to save query:', error);
    }
  };

  const handleLoadTemplate = (template: QueryTemplate) => {
    setSelectedTemplate(template);
    
    if (template.parameters && template.parameters.length > 0) {
      // Initialize parameters with defaults
      const params: Record<string, string> = {};
      template.parameters.forEach(param => {
        params[param.name] = param.default || '';
      });
      setTemplateParameters(params);
    } else {
      // Load template directly if no parameters
      loadTemplate(template.id);
      setSelectedTemplate(null);
      setActiveTab('editor');
    }
  };

  const handleApplyTemplate = () => {
    if (!selectedTemplate) return;

    const filledQuery = fillTemplate(selectedTemplate, templateParameters);
    setCurrentQuery(filledQuery);
    setSelectedTemplate(null);
    setTemplateParameters({});
    setActiveTab('editor');
  };

  const handleValidateQuery = async () => {
    if (!currentQuery.trim()) return;
    await validateQuery(currentQuery);
  };

  const handleCopyQuery = () => {
    navigator.clipboard.writeText(currentQuery);
  };

  const handleExportResults = (format: 'csv' | 'json') => {
    if (!queryResult) return;

    const formatted = formatResults(queryResult);
    let content = '';
    let filename = '';

    if (format === 'csv') {
      const csvRows = [
        formatted.headers.join(','),
        ...formatted.rows.map(row => row.map(cell => `"${cell}"`).join(','))
      ];
      content = csvRows.join('\n');
      filename = 'sparql-results.csv';
    } else {
      content = JSON.stringify(queryResult, null, 2);
      filename = 'sparql-results.json';
    }

    const blob = new Blob([content], { type: format === 'csv' ? 'text/csv' : 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };

  const toggleCategory = (category: string) => {
    const newExpanded = new Set(expandedCategories);
    if (newExpanded.has(category)) {
      newExpanded.delete(category);
    } else {
      newExpanded.add(category);
    }
    setExpandedCategories(newExpanded);
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'traceability': return <Database className="w-4 h-4" />;
      case 'provenance': return <BookOpen className="w-4 h-4" />;
      case 'analytics': return <Zap className="w-4 h-4" />;
      case 'compliance': return <CheckCircle className="w-4 h-4" />;
      default: return <Code className="w-4 h-4" />;
    }
  };

  const formatExecutionTime = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  const groupedTemplates = templates.reduce((acc, template) => {
    if (!acc[template.category]) {
      acc[template.category] = [];
    }
    acc[template.category].push(template);
    return acc;
  }, {} as Record<string, QueryTemplate[]>);

  const favoriteQueries = savedQueries.filter(q => q.is_favorite);
  const recentQueries = savedQueries.slice(0, 10);

  return (
    <div className={`min-h-screen bg-gray-50 dark:bg-gray-900 ${className}`}>
      <div className="max-w-7xl mx-auto p-6">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                SPARQL Query Builder
              </h1>
              <p className="text-gray-600 dark:text-gray-300 mt-1">
                Build and execute semantic queries on blockchain data
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                onClick={() => setShowSettings(!showSettings)}
                className="flex items-center gap-2"
              >
                <Settings className="w-4 h-4" />
                Settings
              </Button>
            </div>
          </div>

          {/* Tab Navigation */}
          <div className="flex space-x-1 bg-gray-100 dark:bg-gray-800 p-1 rounded-lg">
            {[
              { id: 'editor', label: 'Query Editor', icon: Code },
              { id: 'templates', label: 'Templates', icon: BookOpen },
              { id: 'saved', label: 'Saved Queries', icon: Star },
              { id: 'results', label: 'Results', icon: Table }
            ].map(({ id, label, icon: Icon }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id as 'editor' | 'templates' | 'saved' | 'results')}
                className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === id
                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
                    : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
                }`}
              >
                <Icon className="w-4 h-4" />
                {label}
                {id === 'results' && queryResult && (
                  <Badge variant="info" className="ml-1">
                    {formatResults(queryResult).totalRows}
                  </Badge>
                )}
              </button>
            ))}
          </div>
        </div>

        {/* Error Display */}
        {(executionError || savedQueriesError) && (
          <div className="mb-6">
            <Alert
              variant="error"
              message={executionError || savedQueriesError || 'An error occurred'}
            />
          </div>
        )}

        {/* Main Content */}
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
          {/* Main Panel */}
          <div className="lg:col-span-3">
            {activeTab === 'editor' && (
              <Card className="p-6">
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
                    Query Editor
                  </h2>
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      onClick={handleValidateQuery}
                      disabled={isValidating || !currentQuery.trim()}
                      className="flex items-center gap-2"
                    >
                      {isValidating ? <LoadingSpinner size="sm" /> : <CheckCircle className="w-4 h-4" />}
                      Validate
                    </Button>
                    <Button
                      variant="outline"
                      onClick={handleCopyQuery}
                      disabled={!currentQuery.trim()}
                      className="flex items-center gap-2"
                    >
                      <Copy className="w-4 h-4" />
                      Copy
                    </Button>
                    <Button
                      onClick={() => setShowSaveDialog(true)}
                      disabled={!currentQuery.trim()}
                      className="flex items-center gap-2"
                    >
                      <Save className="w-4 h-4" />
                      Save
                    </Button>
                    <Button
                      onClick={handleExecuteQuery}
                      disabled={isExecuting || !currentQuery.trim()}
                      className="flex items-center gap-2"
                    >
                      {isExecuting ? <LoadingSpinner size="sm" /> : <Play className="w-4 h-4" />}
                      Execute
                    </Button>
                  </div>
                </div>

                {/* Query Editor */}
                <div className="mb-4">
                  <textarea
                    value={currentQuery}
                    onChange={(e) => setCurrentQuery(e.target.value)}
                    placeholder="Enter your SPARQL query here..."
                    className="w-full h-64 p-4 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white font-mono text-sm resize-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                </div>

                {/* Validation Results */}
                {validationResult && (
                  <div className="mb-4">
                    {validationResult.is_valid ? (
                      <Alert
                        variant="success"
                        message="Query syntax is valid"
                      />
                    ) : (
                      <Alert
                        variant="error"
                        message={`Validation failed: ${validationResult.errors.join(', ')}`}
                      />
                    )}
                    {validationResult.warnings.length > 0 && (
                      <Alert
                        variant="warning"
                        message={`Warnings: ${validationResult.warnings.join(', ')}`}
                      />
                    )}
                  </div>
                )}

                {/* Execution Status */}
                {isExecuting && (
                  <div className="flex items-center gap-2 text-blue-600 dark:text-blue-400">
                    <LoadingSpinner size="sm" />
                    <span>Executing query...</span>
                  </div>
                )}

                {executionTime > 0 && (
                  <div className="flex items-center gap-2 text-green-600 dark:text-green-400">
                    <Clock className="w-4 h-4" />
                    <span>Executed in {formatExecutionTime(executionTime)}</span>
                  </div>
                )}
              </Card>
            )}

            {activeTab === 'templates' && (
              <Card className="p-6">
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
                    Query Templates
                  </h2>
                  {configLoading && <LoadingSpinner size="sm" />}
                </div>

                {Object.entries(groupedTemplates).map(([category, categoryTemplates]) => (
                  <div key={category} className="mb-6">
                    <button
                      onClick={() => toggleCategory(category)}
                      className="flex items-center gap-2 w-full text-left p-2 hover:bg-gray-50 dark:hover:bg-gray-800 rounded-md"
                    >
                      {expandedCategories.has(category) ? (
                        <ChevronDown className="w-4 h-4" />
                      ) : (
                        <ChevronRight className="w-4 h-4" />
                      )}
                      {getCategoryIcon(category)}
                      <span className="font-medium text-gray-900 dark:text-white capitalize">
                        {category}
                      </span>
                      <Badge variant="default" className="ml-auto">
                        {categoryTemplates.length}
                      </Badge>
                    </button>

                    {expandedCategories.has(category) && (
                      <div className="ml-6 mt-2 space-y-2">
                        {categoryTemplates.map((template) => (
                          <div
                            key={template.id}
                            className="p-4 border border-gray-200 dark:border-gray-700 rounded-md hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer"
                            onClick={() => handleLoadTemplate(template)}
                          >
                            <div className="flex items-start justify-between">
                              <div>
                                <h3 className="font-medium text-gray-900 dark:text-white">
                                  {template.name}
                                </h3>
                                <p className="text-sm text-gray-600 dark:text-gray-300 mt-1">
                                  {template.description}
                                </p>
                                {template.parameters && template.parameters.length > 0 && (
                                  <div className="flex items-center gap-1 mt-2">
                                    <Settings className="w-3 h-3 text-gray-400" />
                                    <span className="text-xs text-gray-500">
                                      {template.parameters.length} parameters
                                    </span>
                                  </div>
                                )}
                              </div>
                              <Button variant="outline" size="sm">
                                Use Template
                              </Button>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </Card>
            )}

            {activeTab === 'saved' && (
              <Card className="p-6">
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
                    Saved Queries
                  </h2>
                  {savedQueriesLoading && <LoadingSpinner size="sm" />}
                </div>

                {favoriteQueries.length > 0 && (
                  <div className="mb-6">
                    <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3 flex items-center gap-2">
                      <Star className="w-4 h-4 text-yellow-500" />
                      Favorites
                    </h3>
                    <div className="space-y-2">
                      {favoriteQueries.map((query) => (
                        <div
                          key={query.id}
                          className="p-4 border border-gray-200 dark:border-gray-700 rounded-md hover:bg-gray-50 dark:hover:bg-gray-800"
                        >
                          <div className="flex items-start justify-between">
                            <div className="flex-1">
                              <h4 className="font-medium text-gray-900 dark:text-white">
                                {query.name}
                              </h4>
                              {query.description && (
                                <p className="text-sm text-gray-600 dark:text-gray-300 mt-1">
                                  {query.description}
                                </p>
                              )}
                              <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
                                {query.created_at && (
                                  <span>Created {new Date(query.created_at).toLocaleDateString()}</span>
                                )}
                                {query.execution_time_ms && (
                                  <span>Last run: {formatExecutionTime(query.execution_time_ms)}</span>
                                )}
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => query.id && toggleFavorite(query.id)}
                              >
                                {query.is_favorite ? (
                                  <Star className="w-4 h-4 text-yellow-500 fill-current" />
                                ) : (
                                  <StarOff className="w-4 h-4" />
                                )}
                              </Button>
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => setCurrentQuery(query.query)}
                              >
                                Load
                              </Button>
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => query.id && deleteQuery(query.id)}
                                className="text-red-600 hover:text-red-700"
                              >
                                <Trash2 className="w-4 h-4" />
                              </Button>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div>
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                    Recent Queries
                  </h3>
                  <div className="space-y-2">
                    {recentQueries.map((query) => (
                      <div
                        key={query.id}
                        className="p-4 border border-gray-200 dark:border-gray-700 rounded-md hover:bg-gray-50 dark:hover:bg-gray-800"
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <h4 className="font-medium text-gray-900 dark:text-white">
                              {query.name}
                            </h4>
                            {query.description && (
                              <p className="text-sm text-gray-600 dark:text-gray-300 mt-1">
                                {query.description}
                              </p>
                            )}
                            <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
                              {query.created_at && (
                                <span>Created {new Date(query.created_at).toLocaleDateString()}</span>
                              )}
                              {query.execution_time_ms && (
                                <span>Last run: {formatExecutionTime(query.execution_time_ms)}</span>
                              )}
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => query.id && toggleFavorite(query.id)}
                            >
                              {query.is_favorite ? (
                                <Star className="w-4 h-4 text-yellow-500 fill-current" />
                              ) : (
                                <StarOff className="w-4 h-4" />
                              )}
                            </Button>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => setCurrentQuery(query.query)}
                            >
                              Load
                            </Button>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => query.id && deleteQuery(query.id)}
                              className="text-red-600 hover:text-red-700"
                            >
                              <Trash2 className="w-4 h-4" />
                            </Button>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </Card>
            )}

            {activeTab === 'results' && (
              <Card className="p-6">
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
                    Query Results
                  </h2>
                  {queryResult && (
                    <div className="flex items-center gap-2">
                      <div className="flex items-center gap-1 bg-gray-100 dark:bg-gray-800 rounded-md p-1">
                        {[
                          { id: 'table', icon: Table, label: 'Table' },
                          { id: 'json', icon: Code, label: 'JSON' }
                        ].map(({ id, icon: Icon, label }) => (
                          <button
                            key={id}
                            onClick={() => setResultsView(id as 'table' | 'json')}
                            className={`flex items-center gap-1 px-3 py-1 rounded text-sm font-medium transition-colors ${
                              resultsView === id
                                ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
                                : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
                            }`}
                          >
                            <Icon className="w-3 h-3" />
                            {label}
                          </button>
                        ))}
                      </div>
                      <Button
                        variant="outline"
                        onClick={() => handleExportResults('csv')}
                        className="flex items-center gap-2"
                      >
                        <Download className="w-4 h-4" />
                        CSV
                      </Button>
                      <Button
                        variant="outline"
                        onClick={() => handleExportResults('json')}
                        className="flex items-center gap-2"
                      >
                        <Download className="w-4 h-4" />
                        JSON
                      </Button>
                    </div>
                  )}
                </div>

                {queryResult ? (
                  <div>
                    {resultsView === 'table' && (
                      <div className="overflow-x-auto">
                        <table className="w-full border-collapse border border-gray-300 dark:border-gray-600">
                          <thead>
                            <tr className="bg-gray-50 dark:bg-gray-800">
                              {formatResults(queryResult).headers.map((header) => (
                                <th
                                  key={header}
                                  className="border border-gray-300 dark:border-gray-600 px-4 py-2 text-left font-medium text-gray-900 dark:text-white"
                                >
                                  {header}
                                </th>
                              ))}
                            </tr>
                          </thead>
                          <tbody>
                            {formatResults(queryResult).rows.map((row, index) => (
                              <tr key={index} className="hover:bg-gray-50 dark:hover:bg-gray-800">
                                {row.map((cell, cellIndex) => (
                                  <td
                                    key={cellIndex}
                                    className="border border-gray-300 dark:border-gray-600 px-4 py-2 text-gray-900 dark:text-white"
                                  >
                                    {cell}
                                  </td>
                                ))}
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </div>
                    )}

                    {resultsView === 'json' && (
                      <pre className="bg-gray-100 dark:bg-gray-800 p-4 rounded-md overflow-x-auto text-sm">
                        <code className="text-gray-900 dark:text-white">
                          {JSON.stringify(queryResult, null, 2)}
                        </code>
                      </pre>
                    )}

                    <div className="mt-4 flex items-center justify-between text-sm text-gray-600 dark:text-gray-300">
                      <span>
                        {formatResults(queryResult).totalRows} results
                        {executionTime > 0 && ` • Executed in ${formatExecutionTime(executionTime)}`}
                      </span>
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-12">
                    <Table className="w-16 h-16 text-gray-400 mx-auto mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                      No Results
                    </h3>
                    <p className="text-gray-600 dark:text-gray-300">
                      Execute a query to see results here
                    </p>
                  </div>
                )}
              </Card>
            )}
          </div>

          {/* Sidebar */}
          <div className="lg:col-span-1">
            {/* Template Parameters Dialog */}
            {selectedTemplate && (
              <Card className="p-4 mb-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="font-medium text-gray-900 dark:text-white">
                    Template Parameters
                  </h3>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setSelectedTemplate(null)}
                  >
                    <X className="w-4 h-4" />
                  </Button>
                </div>
                
                <div className="space-y-3">
                  {selectedTemplate.parameters?.map((param) => (
                    <div key={param.name}>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        {param.name}
                        {param.required && <span className="text-red-500 ml-1">*</span>}
                      </label>
                      <Input
                        type="text"
                        value={templateParameters[param.name] || ''}
                        onChange={(e) => setTemplateParameters(prev => ({
                          ...prev,
                          [param.name]: e.target.value
                        }))}
                        placeholder={param.description}
                      />
                    </div>
                  ))}
                  
                  <Button
                    onClick={handleApplyTemplate}
                    disabled={selectedTemplate.parameters?.some(p => p.required && !templateParameters[p.name])}
                    className="w-full"
                  >
                    Apply Template
                  </Button>
                </div>
              </Card>
            )}

            {/* Save Query Dialog */}
            {showSaveDialog && (
              <Card className="p-4 mb-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="font-medium text-gray-900 dark:text-white">
                    Save Query
                  </h3>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowSaveDialog(false)}
                  >
                    <X className="w-4 h-4" />
                  </Button>
                </div>
                
                <div className="space-y-3">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Query Name *
                    </label>
                    <Input
                      type="text"
                      value={queryName}
                      onChange={(e) => setQueryName(e.target.value)}
                      placeholder="Enter query name..."
                    />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Description
                    </label>
                    <textarea
                      value={queryDescription}
                      onChange={(e) => setQueryDescription(e.target.value)}
                      placeholder="Enter query description..."
                      className="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm resize-none"
                      rows={3}
                    />
                  </div>
                  
                  <div className="flex gap-2">
                    <Button
                      onClick={handleSaveQuery}
                      disabled={!queryName.trim() || !currentQuery.trim()}
                      className="flex-1"
                    >
                      Save Query
                    </Button>
                    <Button
                      variant="outline"
                      onClick={() => setShowSaveDialog(false)}
                    >
                      Cancel
                    </Button>
                  </div>
                </div>
              </Card>
            )}

            {/* Quick Actions */}
            <Card className="p-4">
              <h3 className="font-medium text-gray-900 dark:text-white mb-3">
                Quick Actions
              </h3>
              <div className="space-y-2">
                <Button
                  variant="outline"
                  onClick={() => setCurrentQuery('')}
                  className="w-full justify-start"
                >
                  <FileText className="w-4 h-4 mr-2" />
                  New Query
                </Button>
                <Button
                  variant="outline"
                  onClick={clearResults}
                  disabled={!queryResult}
                  className="w-full justify-start"
                >
                  <X className="w-4 h-4 mr-2" />
                  Clear Results
                </Button>
                <Button
                  variant="outline"
                  onClick={() => setActiveTab('templates')}
                  className="w-full justify-start"
                >
                  <BookOpen className="w-4 h-4 mr-2" />
                  Browse Templates
                </Button>
              </div>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SPARQLQueryBuilder;
