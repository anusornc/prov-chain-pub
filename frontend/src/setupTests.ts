import "@testing-library/jest-dom";
import { configure } from "@testing-library/react";
import { jest } from "@jest/globals";

// Configure React Testing Library
configure({
  testIdAttribute: "data-testid",
  asyncUtilTimeout: 5000,
});

// Mock IntersectionObserver for components that use it
const mockIntersectionObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));
(global as any).IntersectionObserver = mockIntersectionObserver;

// Mock ResizeObserver for responsive components
const mockResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));
(global as any).ResizeObserver = mockResizeObserver;

// Mock window.matchMedia
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: jest.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(), // deprecated
    removeListener: jest.fn(), // deprecated
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

// Mock window.scrollTo
Object.defineProperty(window, 'scrollTo', {
  value: jest.fn(),
  writable: true,
});

// Mock localStorage
const localStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};
Object.defineProperty(window, "localStorage", {
  value: localStorageMock,
});

// Mock sessionStorage
const sessionStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};
Object.defineProperty(window, "sessionStorage", {
  value: sessionStorageMock,
});

// Mock crypto.randomUUID
Object.defineProperty(global.crypto, "randomUUID", {
  value: jest.fn(() => "mock-uuid-1234-5678-9abc"),
});

// Mock URL.createObjectURL
Object.defineProperty(URL, "createObjectURL", {
  value: jest.fn(() => "mock-object-url"),
});

Object.defineProperty(URL, "revokeObjectURL", {
  value: jest.fn(),
});

// Suppress console warnings in tests unless debugging
const originalError = console.error;
beforeAll(() => {
  console.error = (...args: unknown[]) => {
    if (
      typeof args[0] === "string" &&
      (args[0].includes("Warning: ReactDOM.render is deprecated") ||
        args[0].includes("ReactDOMTestUtils.act is deprecated") ||
        args[0].includes(
          "The current testing environment is not configured to support act",
        ))
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});

// Global test cleanup
afterEach(() => {
  jest.clearAllTimers();
  jest.clearAllMocks();
});

// Export testing utilities for convenience
export * from "@testing-library/react";
export { default as userEvent } from "@testing-library/user-event";
