import type { SPARQLQuery, SPARQLResult } from "../types";
import { API_ENDPOINTS } from "../config/api";

const API_BASE_URL = API_ENDPOINTS.API;

export interface QueryTemplate {
  id: string;
  name: string;
  description: string;
  category: "traceability" | "provenance" | "analytics" | "compliance";
  query: string;
  parameters?: {
    name: string;
    type: "uri" | "literal" | "variable";
    description: string;
    required: boolean;
    default?: string;
  }[];
}

export interface QueryBuilderConfig {
  templates: QueryTemplate[];
  predicates: string[];
  classes: string[];
  namespaces: Record<string, string>;
}

export class SPARQLService {
  private static instance: SPARQLService;

  public static getInstance(): SPARQLService {
    if (!SPARQLService.instance) {
      SPARQLService.instance = new SPARQLService();
    }
    return SPARQLService.instance;
  }

  private getAuthHeaders(): HeadersInit {
    const token = localStorage.getItem("authToken");
    return {
      "Content-Type": "application/json",
      Accept: "application/sparql-results+json",
      ...(token && { Authorization: `Bearer ${token}` }),
    };
  }

  /**
   * Execute a SPARQL query
   */
  async executeQuery(query: string): Promise<SPARQLResult> {
    try {
      const response = await fetch(`${API_BASE_URL}/sparql/query`, {
        method: "POST",
        headers: this.getAuthHeaders(),
        body: JSON.stringify({ query }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(
          `SPARQL query failed: ${response.statusText} - ${errorText}`,
        );
      }

      return await response.json();
    } catch (error) {
      console.error("Error executing SPARQL query:", error);
      throw error;
    }
  }

  /**
   * Save a SPARQL query
   */
  async saveQuery(query: SPARQLQuery): Promise<SPARQLQuery> {
    try {
      const response = await fetch(`${API_BASE_URL}/sparql/queries`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(query),
      });

      if (!response.ok) {
        throw new Error(`Failed to save query: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Error saving SPARQL query:", error);
      throw error;
    }
  }

  /**
   * Get saved queries
   */
  async getSavedQueries(): Promise<SPARQLQuery[]> {
    try {
      const response = await fetch(`${API_BASE_URL}/sparql/queries`);

      if (!response.ok) {
        throw new Error(
          `Failed to fetch saved queries: ${response.statusText}`,
        );
      }

      return await response.json();
    } catch (error) {
      console.error("Error fetching saved queries:", error);
      throw error;
    }
  }

  /**
   * Delete a saved query
   */
  async deleteQuery(queryId: string): Promise<void> {
    try {
      const response = await fetch(
        `${API_BASE_URL}/sparql/queries/${queryId}`,
        {
          method: "DELETE",
        },
      );

      if (!response.ok) {
        throw new Error(`Failed to delete query: ${response.statusText}`);
      }
    } catch (error) {
      console.error("Error deleting SPARQL query:", error);
      throw error;
    }
  }

  /**
   * Toggle favorite status of a query
   */
  async toggleFavorite(queryId: string): Promise<SPARQLQuery> {
    try {
      const response = await fetch(
        `${API_BASE_URL}/sparql/queries/${queryId}/favorite`,
        {
          method: "POST",
        },
      );

      if (!response.ok) {
        throw new Error(`Failed to toggle favorite: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Error toggling favorite:", error);
      throw error;
    }
  }

  /**
   * Get query templates and configuration
   */
  async getQueryBuilderConfig(): Promise<QueryBuilderConfig> {
    try {
      const response = await fetch(`${API_BASE_URL}/sparql/config`);

      if (!response.ok) {
        // Return default config if endpoint not available
        return this.getDefaultConfig();
      }

      return await response.json();
    } catch (error) {
      console.warn("Using default SPARQL config:", error);
      return this.getDefaultConfig();
    }
  }

  /**
   * Validate SPARQL query syntax
   */
  async validateQuery(query: string): Promise<{
    is_valid: boolean;
    errors: string[];
    warnings: string[];
  }> {
    try {
      const response = await fetch(`${API_BASE_URL}/sparql/validate`, {
        method: "POST",
        headers: {
          "Content-Type": "application/sparql-query",
        },
        body: query,
      });

      if (!response.ok) {
        return {
          is_valid: false,
          errors: [`HTTP ${response.status}: ${response.statusText}`],
          warnings: [],
        };
      }

      return await response.json();
    } catch (error) {
      return {
        is_valid: false,
        errors: [
          error instanceof Error ? error.message : "Unknown validation error",
        ],
        warnings: [],
      };
    }
  }

  /**
   * Get default configuration for query builder
   */
  private getDefaultConfig(): QueryBuilderConfig {
    return {
      templates: [
        {
          id: "item-trace",
          name: "Item Trace Path",
          description: "Get the complete trace path for an item",
          category: "traceability",
          query: `PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT ?step ?activity ?agent ?timestamp ?location WHERE {
  ?item pc:hasId "{{ITEM_ID}}" .
  ?item prov:wasGeneratedBy* ?activity .
  ?activity prov:startedAtTime ?timestamp .
  OPTIONAL { ?activity prov:wasAssociatedWith ?agent }
  OPTIONAL { ?activity prov:atLocation ?location }
  BIND(ROW_NUMBER() OVER (ORDER BY ?timestamp) AS ?step)
}
ORDER BY ?timestamp`,
          parameters: [
            {
              name: "ITEM_ID",
              type: "literal",
              description: "The ID of the item to trace",
              required: true,
            },
          ],
        },
        {
          id: "provenance-chain",
          name: "Provenance Chain",
          description: "Get the provenance chain showing how items are derived",
          category: "provenance",
          query: `PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT ?source ?target ?relationship ?timestamp WHERE {
  ?target prov:wasDerivedFrom ?source .
  ?target prov:wasGeneratedBy ?activity .
  ?activity prov:startedAtTime ?timestamp .
  ?activity a ?relationship .
}
ORDER BY ?timestamp`,
        },
        {
          id: "participant-activities",
          name: "Participant Activities",
          description: "Get all activities performed by a specific participant",
          category: "analytics",
          query: `PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT ?activity ?type ?timestamp ?item WHERE {
  ?activity prov:wasAssociatedWith ?participant .
  ?participant pc:hasId "{{PARTICIPANT_ID}}" .
  ?activity a ?type .
  ?activity prov:startedAtTime ?timestamp .
  OPTIONAL { ?activity prov:generated ?item }
}
ORDER BY DESC(?timestamp)`,
          parameters: [
            {
              name: "PARTICIPANT_ID",
              type: "literal",
              description: "The ID of the participant",
              required: true,
            },
          ],
        },
        {
          id: "quality-compliance",
          name: "Quality Compliance Check",
          description: "Check quality compliance status for items",
          category: "compliance",
          query: `PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT ?item ?qualityCheck ?result ?timestamp WHERE {
  ?item a pc:Product .
  ?qualityCheck a pc:QualityActivity .
  ?qualityCheck prov:used ?item .
  ?qualityCheck pc:hasResult ?result .
  ?qualityCheck prov:startedAtTime ?timestamp .
}
ORDER BY DESC(?timestamp)`,
        },
        {
          id: "supply-chain-analytics",
          name: "Supply Chain Analytics",
          description: "Get comprehensive supply chain metrics",
          category: "analytics",
          query: `PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT 
  (COUNT(DISTINCT ?item) AS ?totalItems)
  (COUNT(DISTINCT ?participant) AS ?totalParticipants)
  (COUNT(DISTINCT ?location) AS ?totalLocations)
  (COUNT(DISTINCT ?activity) AS ?totalActivities)
WHERE {
  ?item a pc:Product .
  ?activity prov:generated ?item .
  ?activity prov:wasAssociatedWith ?participant .
  OPTIONAL { ?activity prov:atLocation ?location }
}`,
        },
        {
          id: "recent-transactions",
          name: "Recent Transactions",
          description: "Get recent blockchain transactions",
          category: "traceability",
          query: `PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT ?transaction ?type ?participant ?timestamp ?item WHERE {
  ?transaction a pc:Transaction .
  ?transaction pc:hasType ?type .
  ?transaction prov:wasAssociatedWith ?participant .
  ?transaction prov:startedAtTime ?timestamp .
  OPTIONAL { ?transaction prov:generated ?item }
}
ORDER BY DESC(?timestamp)
LIMIT 50`,
        },
      ],
      predicates: [
        "prov:wasGeneratedBy",
        "prov:wasDerivedFrom",
        "prov:used",
        "prov:wasAssociatedWith",
        "prov:startedAtTime",
        "prov:endedAtTime",
        "prov:atLocation",
        "pc:hasId",
        "pc:hasType",
        "pc:hasResult",
        "pc:hasOwner",
        "pc:hasLocation",
        "pc:hasQuality",
      ],
      classes: [
        "prov:Entity",
        "prov:Activity",
        "prov:Agent",
        "pc:Product",
        "pc:RawMaterial",
        "pc:Component",
        "pc:Batch",
        "pc:Shipment",
        "pc:ProductionActivity",
        "pc:ProcessingActivity",
        "pc:TransportActivity",
        "pc:QualityActivity",
        "pc:Producer",
        "pc:Manufacturer",
        "pc:LogisticsProvider",
        "pc:QualityLab",
        "pc:Auditor",
        "pc:Retailer",
      ],
      namespaces: {
        prov: "http://www.w3.org/ns/prov#",
        pc: "http://provchain.org/ontology#",
        xsd: "http://www.w3.org/2001/XMLSchema#",
        rdfs: "http://www.w3.org/2000/01/rdf-schema#",
        rdf: "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
      },
    };
  }

  /**
   * Format SPARQL results for display
   */
  formatResults(results: SPARQLResult): {
    headers: string[];
    rows: string[][];
    totalRows: number;
  } {
    const headers = results.head.vars;
    const rows = results.results.bindings.map((binding) =>
      headers.map((header) => {
        const value = binding[header];
        if (!value) return "";

        // Format different value types
        if (value.type === "uri") {
          // Extract local name from URI
          const localName = value.value.split(/[#/]/).pop() || value.value;
          return localName;
        } else if (value.type === "literal") {
          if (value.datatype === "http://www.w3.org/2001/XMLSchema#dateTime") {
            return new Date(value.value).toLocaleString();
          }
          return value.value;
        }
        return value.value;
      }),
    );

    return {
      headers,
      rows,
      totalRows: rows.length,
    };
  }
}

export const sparqlService = SPARQLService.getInstance();
