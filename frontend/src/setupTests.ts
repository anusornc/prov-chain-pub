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

// Keep console output unsuppressed by default. Tests that exercise expected
// negative paths should assert those messages with scoped spies instead of
// hiding unexpected errors globally.

// React 19 act-environment setup is loaded from setupActEnvironment.ts before
// React initializes.

// Suppress narrowly scoped React 19 / Jest environment compatibility warnings
// that come from the current test harness rather than application behavior.
// Unexpected console errors are still surfaced; expected application errors are
// asserted with local spies.
const originalConsoleError = console.error;
beforeAll(() => {
  console.error = (...args: unknown[]) => {
    if (typeof args[0] === "string") {
      const message = args[0];
      if (
        (message.includes("ReactDOMTestUtils.act") &&
          message.includes("deprecated")) ||
        message.includes(
          "The current testing environment is not configured to support act",
        )
      ) {
        return;
      }
    }

    originalConsoleError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalConsoleError;
});

// Global test cleanup
afterEach(() => {
  jest.clearAllTimers();
  jest.clearAllMocks();
});

// Export testing utilities for convenience
export * from "@testing-library/react";
export { default as userEvent } from "@testing-library/user-event";
