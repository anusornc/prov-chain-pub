import { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { jest } from '@jest/globals';

// Create a test query client
const createTestQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      gcTime: 0,
      networkMode: 'offlineFirst',
    },
    mutations: {
      retry: false,
      networkMode: 'offlineFirst',
    },
  },
});

// Test wrapper for React Query
export const QueryWrapper = ({ children }: { children: ReactNode }) => {
  const queryClient = createTestQueryClient();

  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

// Mock useQuery hook
export const mockUseQuery = jest.fn();

// Mock useMutation hook
export const mockUseMutation = jest.fn();

// Helper to mock successful query
export const mockSuccessfulQuery = (data: any, isLoading = false) => {
  mockUseQuery.mockReturnValue({
    data,
    isLoading,
    isSuccess: true,
    isError: false,
    error: null,
    refetch: jest.fn(),
  });
};

// Helper to mock loading query
export const mockLoadingQuery = () => {
  mockUseQuery.mockReturnValue({
    data: undefined,
    isLoading: true,
    isSuccess: false,
    isError: false,
    error: null,
    refetch: jest.fn(),
  });
};

// Helper to mock error query
export const mockErrorQuery = (error: any) => {
  mockUseQuery.mockReturnValue({
    data: undefined,
    isLoading: false,
    isSuccess: false,
    isError: true,
    error,
    refetch: jest.fn(),
  });
};

// Helper to mock successful mutation
export const mockSuccessfulMutation = (data: any) => {
  mockUseMutation.mockReturnValue({
    data,
    isLoading: false,
    isSuccess: true,
    isError: false,
    error: null,
    mutate: jest.fn(),
    mutateAsync: jest.fn(() => Promise.resolve(data)),
  });
};

// Helper to mock loading mutation
export const mockLoadingMutation = () => {
  mockUseMutation.mockReturnValue({
    data: undefined,
    isLoading: true,
    isSuccess: false,
    isError: false,
    error: null,
    mutate: jest.fn(),
    mutateAsync: jest.fn(() => Promise.resolve(undefined)),
  });
};

// Helper to mock error mutation
export const mockErrorMutation = (error: any) => {
  mockUseMutation.mockReturnValue({
    data: undefined,
    isLoading: false,
    isSuccess: false,
    isError: true,
    error,
    mutate: jest.fn(),
    mutateAsync: jest.fn(() => Promise.reject(error)),
  });
};

export { createTestQueryClient };