import React, { ReactElement, ReactNode } from "react";
import { render, RenderOptions, RenderResult } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter } from "react-router-dom";
import { ThemeProvider } from "../contexts/ThemeContext";
import { AuthProvider } from "../contexts/AuthContext";

// Create a test QueryClient instance
const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        cacheTime: 0, // React Query v4 uses cacheTime
      },
      mutations: {
        retry: false,
      },
    },
  });

// Mock theme for testing
const mockTheme = {
  colors: {
    primary: "#3b82f6",
    secondary: "#64748b",
    background: "#ffffff",
    surface: "#f8fafc",
    text: "#1e293b",
    error: "#ef4444",
    warning: "#f59e0b",
    success: "#10b981",
    info: "#3b82f6",
  },
  spacing: {
    xs: "0.25rem",
    sm: "0.5rem",
    md: "1rem",
    lg: "1.5rem",
    xl: "2rem",
  },
  borderRadius: {
    sm: "0.125rem",
    md: "0.375rem",
    lg: "0.5rem",
    full: "9999px",
  },
  typography: {
    fontFamily: "Inter, system-ui, sans-serif",
    fontSize: {
      xs: "0.75rem",
      sm: "0.875rem",
      md: "1rem",
      lg: "1.125rem",
      xl: "1.25rem",
    },
  },
};

// All-in-one wrapper with all providers
const AllTheProviders = ({
  children,
  queryClient,
}: {
  children: ReactNode;
  queryClient?: QueryClient;
}) => {
  const testQueryClient = queryClient || createTestQueryClient();

  return (
    <QueryClientProvider client={testQueryClient}>
      <BrowserRouter>
        <ThemeProvider>
          <AuthProvider>{children}</AuthProvider>
        </ThemeProvider>
      </BrowserRouter>
    </QueryClientProvider>
  );
};

// Custom render function with all providers
export const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, "wrapper"> & { queryClient?: QueryClient },
): RenderResult => {
  const { queryClient, ...renderOptions } = options || {};

  return render(ui, {
    wrapper: ({ children }) => (
      <AllTheProviders queryClient={queryClient}>{children}</AllTheProviders>
    ),
    ...renderOptions,
  });
};

// Export component separately to avoid react-refresh warning
export { AllTheProviders };

// Custom render function without providers (for isolated component testing)
export const renderWithoutProviders = (
  ui: ReactElement,
  options?: RenderOptions,
): RenderResult => {
  return render(ui, options);
};

// Mock user for testing
export const mockUser = {
  id: "test-user-123",
  username: "testuser",
  email: "test@example.com",
  role: "user",
  permissions: ["read:blocks", "read:traceability"],
  token: "mock-jwt-token-12345",
  refreshToken: "mock-refresh-token-67890",
};

// Mock blockchain data
export const mockBlockData = {
  id: "block-123",
  hash: "0xabc123...",
  previousHash: "0xdef456...",
  timestamp: "2025-08-27T10:00:00Z",
  transactions: [
    {
      id: "tx-123",
      type: "PRODUCTION",
      data: { batchId: "BATCH-001", quantity: 1000 },
    },
  ],
  merkleRoot: "0xroot123...",
  nonce: 12345,
};

// Mock traceability data
export const mockTraceabilityData = {
  batchId: "BATCH-001",
  product: "UHT Milk",
  traceability: [
    {
      step: "Production",
      timestamp: "2025-08-27T08:00:00Z",
      location: "Farm A",
      details: { temperature: 4.2, humidity: 65.0 },
    },
    {
      step: "Processing",
      timestamp: "2025-08-27T10:00:00Z",
      location: "Plant B",
      details: { processing: "UHT", duration: "2h" },
    },
    {
      step: "Transport",
      timestamp: "2025-08-27T14:00:00Z",
      location: "Warehouse C",
      details: { vehicle: "Truck-123", distance: "50km" },
    },
  ],
};

// Mock SPARQL query results
export const mockSparqlResults = {
  head: { vars: ["subject", "predicate", "object"] },
  results: {
    bindings: [
      {
        subject: { type: "uri", value: "http://provchain.org/batch/BATCH-001" },
        predicate: { type: "uri", value: "http://provchain.org/hasProduct" },
        object: { type: "literal", value: "UHT Milk" },
      },
      {
        subject: { type: "uri", value: "http://provchain.org/batch/BATCH-001" },
        predicate: { type: "uri", value: "http://provchain.org/hasQuantity" },
        object: {
          type: "literal",
          value: "1000",
          datatype: "http://www.w3.org/2001/XMLSchema#integer",
        },
      },
    ],
  },
};

// Helper functions for testing
export function createMockResponse<T>(data: T, status = 200) {
  return {
    data,
    status,
    statusText: "OK",
    headers: {},
    config: {
      headers: {},
    },
  };
}

// Helper function for testing fetch responses
export function createMockFetchResponse<T>(data: T, status = 200) {
  return Promise.resolve({
    ok: status >= 200 && status < 300,
    status,
    statusText: "OK",
    json: () => Promise.resolve(data),
    headers: new Headers(),
  });
}

export function createMockError(message: string, status = 500) {
  return {
    response: {
      data: { message },
      status,
      statusText: "Internal Server Error",
    },
    message,
  };
}

// Wait for component to update
export const waitForComponentUpdate = () =>
  new Promise((resolve) => setTimeout(resolve, 0));

// Mock window.alert and window.confirm
export const mockWindowAlert = jest.fn();
export const mockWindowConfirm = jest.fn(() => true);

// Mock navigation
export const mockNavigate = jest.fn();

// Re-export everything from React Testing Library
export {
  render,
  screen,
  fireEvent,
  waitFor,
  act,
  cleanup,
} from "@testing-library/react";

// Export userEvent for interactions
export { default as userEvent } from "@testing-library/user-event";

// Export mocked data
export { mockTheme, createTestQueryClient };
