import type { 
  TraceabilityItem, 
  TraceabilityResponse, 
  TraceStep, 
  SearchQuery, 
  SearchResults,
  KnowledgeGraph
} from '../types';

const API_BASE_URL = 'http://localhost:8080/api';

export class TraceabilityService {
  private static instance: TraceabilityService;

  public static getInstance(): TraceabilityService {
    if (!TraceabilityService.instance) {
      TraceabilityService.instance = new TraceabilityService();
    }
    return TraceabilityService.instance;
  }

  private getAuthHeaders(): HeadersInit {
    const token = localStorage.getItem('authToken');
    return {
      'Content-Type': 'application/json',
      ...(token && { 'Authorization': `Bearer ${token}` })
    };
  }

  /**
   * Get all traceability items with optional filtering
   */
  async getItems(query?: SearchQuery): Promise<SearchResults<TraceabilityItem>> {
    try {
      const params = new URLSearchParams();
      
      if (query) {
        if (query.query) params.append('q', query.query);
        if (query.page) params.append('page', query.page.toString());
        if (query.limit) params.append('limit', query.limit.toString());
        if (query.sort_by) params.append('sort_by', query.sort_by);
        if (query.sort_order) params.append('sort_order', query.sort_order);
        
        // Add filters
        if (query.filters.type) params.append('type', query.filters.type);
        if (query.filters.participant) params.append('participant', query.filters.participant);
        if (query.filters.location) params.append('location', query.filters.location);
        if (query.filters.status) params.append('status', query.filters.status);
        if (query.filters.dateRange) {
          params.append('start_date', query.filters.dateRange[0].toISOString());
          params.append('end_date', query.filters.dateRange[1].toISOString());
        }
      }

      const response = await fetch(`${API_BASE_URL}/products?${params}`, {
        headers: this.getAuthHeaders()
      });
      
      if (!response.ok) {
        throw new Error(`Failed to fetch items: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching traceability items:', error);
      throw error;
    }
  }

  /**
   * Get detailed information about a specific item including its trace path
   */
  async getItemTrace(itemId: string): Promise<TraceabilityResponse> {
    try {
      const response = await fetch(`${API_BASE_URL}/products/${itemId}/trace`, {
        headers: this.getAuthHeaders()
      });
      
      if (!response.ok) {
        throw new Error(`Failed to fetch item trace: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching item trace:', error);
      throw error;
    }
  }

  /**
   * Get a specific traceability item by ID
   */
  async getItem(itemId: string): Promise<TraceabilityItem> {
    try {
      const response = await fetch(`${API_BASE_URL}/products/${itemId}`, {
        headers: this.getAuthHeaders()
      });
      
      if (!response.ok) {
        throw new Error(`Failed to fetch item: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching item:', error);
      throw error;
    }
  }

  /**
   * Get the provenance chain for an item
   */
  async getProvenanceChain(itemId: string): Promise<TraceStep[]> {
    try {
      const response = await fetch(`${API_BASE_URL}/products/${itemId}/provenance`, {
        headers: this.getAuthHeaders()
      });
      
      if (!response.ok) {
        throw new Error(`Failed to fetch provenance chain: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching provenance chain:', error);
      throw error;
    }
  }

  /**
   * Get knowledge graph for an item or set of items
   */
  async getKnowledgeGraph(itemIds: string[]): Promise<KnowledgeGraph> {
    try {
      const params = new URLSearchParams();
      itemIds.forEach(id => params.append('item_id', id));

      const response = await fetch(`${API_BASE_URL}/knowledge-graph?${params}`, {
        headers: this.getAuthHeaders()
      });
      
      if (!response.ok) {
        throw new Error(`Failed to fetch knowledge graph: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching knowledge graph:', error);
      throw error;
    }
  }

  /**
   * Search items by various criteria
   */
  async searchItems(searchTerm: string, filters?: SearchQuery['filters']): Promise<SearchResults<TraceabilityItem>> {
    const query: SearchQuery = {
      query: searchTerm,
      filters: filters || {},
      page: 1,
      limit: 20
    };

    return this.getItems(query);
  }

  /**
   * Get items by type
   */
  async getItemsByType(type: string): Promise<TraceabilityItem[]> {
    try {
      const response = await fetch(`${API_BASE_URL}/products/by-type/${type}`);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch items by type: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching items by type:', error);
      throw error;
    }
  }

  /**
   * Get items by participant
   */
  async getItemsByParticipant(participantId: string): Promise<TraceabilityItem[]> {
    try {
      const response = await fetch(`${API_BASE_URL}/products/by-participant/${participantId}`);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch items by participant: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching items by participant:', error);
      throw error;
    }
  }

  /**
   * Get supply chain analytics for an item
   */
  async getSupplyChainAnalytics(itemId: string): Promise<{
    total_steps: number;
    total_participants: number;
    total_locations: number;
    duration_days: number;
    carbon_footprint?: number;
    quality_scores: number[];
    compliance_status: 'compliant' | 'non_compliant' | 'pending';
  }> {
    try {
      const response = await fetch(`${API_BASE_URL}/products/${itemId}/analytics`, {
        headers: this.getAuthHeaders()
      });
      
      if (!response.ok) {
        throw new Error(`Failed to fetch supply chain analytics: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching supply chain analytics:', error);
      throw error;
    }
  }

  /**
   * Get related items (items that share provenance or relationships)
   */
  async getRelatedItems(itemId: string): Promise<TraceabilityItem[]> {
    try {
      const response = await fetch(`${API_BASE_URL}/products/${itemId}/related`);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch related items: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error fetching related items:', error);
      throw error;
    }
  }

  /**
   * Validate item authenticity and integrity
   */
  async validateItem(itemId: string): Promise<{
    is_authentic: boolean;
    integrity_score: number;
    validation_details: {
      signature_valid: boolean;
      chain_intact: boolean;
      data_consistent: boolean;
      timestamp_valid: boolean;
    };
    validation_time_ms: number;
  }> {
    try {
      const response = await fetch(`${API_BASE_URL}/products/${itemId}/validate`);
      
      if (!response.ok) {
        throw new Error(`Failed to validate item: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error validating item:', error);
      throw error;
    }
  }
}

export const traceabilityService = TraceabilityService.getInstance();
