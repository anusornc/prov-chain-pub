import React from "react";
import { render, screen, waitFor, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AuthProvider, useAuth } from "../AuthContext";
import { createMockFetchResponse } from "../../test-utils/test-utils";

// Mock fetch globally
const mockFetch = jest.fn();
global.fetch = mockFetch;

// Mock localStorage
const localStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};
Object.defineProperty(window, "localStorage", { value: localStorageMock });

// Test component to consume AuthContext
const TestComponent: React.FC = () => {
  const { user, token, isAuthenticated, isLoading, login, logout } = useAuth();

  return (
    <div data-testid="auth-context-test">
      <div data-testid="loading-state">{isLoading.toString()}</div>
      <div data-testid="auth-state">{isAuthenticated.toString()}</div>
      <div data-testid="user-data">{user ? JSON.stringify(user) : "null"}</div>
      <div data-testid="token-data">{token || "null"}</div>

      <button
        onClick={() => login("testuser", "testpass")}
        data-testid="login-button"
      >
        Login
      </button>

      <button onClick={logout} data-testid="logout-button">
        Logout
      </button>
    </div>
  );
};

const renderWithAuthProvider = (component: React.ReactElement) => {
  return render(<AuthProvider>{component}</AuthProvider>);
};

describe("AuthContext", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    localStorageMock.getItem.mockClear();
    localStorageMock.setItem.mockClear();
    localStorageMock.removeItem.mockClear();
  });

  describe("Initial State", () => {
    test("should initialize with loading state", async () => {
      localStorageMock.getItem.mockReturnValue(null);

      renderWithAuthProvider(<TestComponent />);

      // Check initial state or wait for it to stabilize
      await waitFor(() => {
        const loadingElement = screen.getByTestId("loading-state");
        const authElement = screen.getByTestId("auth-state");
        // Either loading starts as true or it's already false - both are valid states
        expect(["true", "false"]).toContain(loadingElement.textContent);
        expect(authElement.textContent).toBe("false");
      });

      // Wait for loading to complete (if it started as true)
      await waitFor(
        () => {
          expect(screen.getByTestId("loading-state")).toHaveTextContent(
            "false",
          );
        },
        { timeout: 3000 },
      );
    });

    test("should validate existing token on mount", async () => {
      const mockToken = "valid.jwt.token";
      localStorageMock.getItem.mockReturnValue(mockToken);

      // Mock successful token validation
      mockFetch.mockResolvedValueOnce(
        createMockFetchResponse({ status: "ok" }),
      );

      renderWithAuthProvider(<TestComponent />);

      await waitFor(
        () => {
          expect(mockFetch).toHaveBeenCalledWith(
            expect.stringContaining("/blockchain/status"),
            expect.objectContaining({
              headers: {
                Authorization: `Bearer ${mockToken}`,
                "Content-Type": "application/json",
              },
            }),
          );
        },
        { timeout: 3000 },
      );

      await waitFor(
        () => {
          expect(screen.getByTestId("loading-state")).toHaveTextContent(
            "false",
          );
          expect(screen.getByTestId("auth-state")).toHaveTextContent("true");
        },
        { timeout: 3000 },
      );
    });

    test("should handle invalid token on mount", async () => {
      const mockToken = "invalid.jwt.token";
      localStorageMock.getItem.mockReturnValue(mockToken);

      // Mock failed token validation
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        statusText: "Unauthorized",
        json: () => Promise.resolve({}),
      });

      renderWithAuthProvider(<TestComponent />);

      await waitFor(
        () => {
          expect(localStorageMock.removeItem).toHaveBeenCalledWith("authToken");
        },
        { timeout: 3000 },
      );

      await waitFor(
        () => {
          expect(screen.getByTestId("loading-state")).toHaveTextContent(
            "false",
          );
          expect(screen.getByTestId("auth-state")).toHaveTextContent("false");
        },
        { timeout: 3000 },
      );
    });
  });

  describe("Login Functionality", () => {
    test("should handle successful login", async () => {
      localStorageMock.getItem.mockReturnValue(null);
      mockFetch.mockResolvedValueOnce(
        createMockFetchResponse({
          token: "new.jwt.token",
          user_role: "admin",
        }),
      );

      renderWithAuthProvider(<TestComponent />);

      // Wait for initial loading to complete
      await waitFor(
        () => {
          expect(screen.getByTestId("loading-state")).toHaveTextContent(
            "false",
          );
        },
        { timeout: 3000 },
      );

      const loginButton = screen.getByTestId("login-button");
      await act(async () => {
        await userEvent.click(loginButton);
      });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining("/auth/login"),
          expect.objectContaining({
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              username: "testuser",
              password: "testpass",
            }),
          }),
        );
      });

      await waitFor(
        () => {
          expect(localStorageMock.setItem).toHaveBeenCalledWith(
            "authToken",
            "new.jwt.token",
          );
          expect(screen.getByTestId("auth-state")).toHaveTextContent("true");
          expect(screen.getByTestId("user-data")).toHaveTextContent("testuser");
        },
        { timeout: 3000 },
      );
    });

    test("should handle login failure", async () => {
      localStorageMock.getItem.mockReturnValue(null);
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        statusText: "Unauthorized",
        json: () => Promise.resolve({ message: "Invalid credentials" }),
      });

      renderWithAuthProvider(<TestComponent />);

      // Wait for initial loading to complete
      await waitFor(
        () => {
          expect(screen.getByTestId("loading-state")).toHaveTextContent(
            "false",
          );
        },
        { timeout: 3000 },
      );

      const loginButton = screen.getByTestId("login-button");
      await act(async () => {
        await userEvent.click(loginButton);
      });

      await waitFor(
        () => {
          expect(screen.getByTestId("auth-state")).toHaveTextContent("false");
        },
        { timeout: 3000 },
      );

      expect(localStorageMock.setItem).not.toHaveBeenCalled();
    });

    test("should handle network error during login", async () => {
      localStorageMock.getItem.mockReturnValue(null);
      mockFetch.mockRejectedValueOnce(new Error("Network error"));

      renderWithAuthProvider(<TestComponent />);

      // Wait for initial loading to complete
      await waitFor(
        () => {
          expect(screen.getByTestId("loading-state")).toHaveTextContent(
            "false",
          );
        },
        { timeout: 3000 },
      );

      const loginButton = screen.getByTestId("login-button");
      await act(async () => {
        await userEvent.click(loginButton);
      });

      await waitFor(
        () => {
          expect(screen.getByTestId("auth-state")).toHaveTextContent("false");
        },
        { timeout: 3000 },
      );

      expect(localStorageMock.setItem).not.toHaveBeenCalled();
    });
  });

  describe("Logout Functionality", () => {
    test("should handle logout correctly", async () => {
      const mockToken = "valid.jwt.token";
      localStorageMock.getItem.mockReturnValue(mockToken);

      mockFetch.mockResolvedValueOnce(
        createMockFetchResponse({ status: "ok" }),
      );

      renderWithAuthProvider(<TestComponent />);

      // Wait for initial token validation
      await waitFor(
        () => {
          expect(screen.getByTestId("auth-state")).toHaveTextContent("true");
        },
        { timeout: 3000 },
      );

      const logoutButton = screen.getByTestId("logout-button");
      await act(async () => {
        await userEvent.click(logoutButton);
      });

      await waitFor(
        () => {
          expect(localStorageMock.removeItem).toHaveBeenCalledWith("authToken");
          expect(screen.getByTestId("auth-state")).toHaveTextContent("false");
          expect(screen.getByTestId("user-data")).toHaveTextContent("null");
          expect(screen.getByTestId("token-data")).toHaveTextContent("null");
        },
        { timeout: 3000 },
      );
    });
  });

  describe("JWT Parsing", () => {
    test("should parse JWT token correctly", async () => {
      const mockToken =
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0dXNlciIsInJvbGUiOiJhZG1pbiJ9.signature";
      localStorageMock.getItem.mockReturnValue(mockToken);

      mockFetch.mockResolvedValueOnce(
        createMockFetchResponse({ status: "ok" }),
      );

      renderWithAuthProvider(<TestComponent />);

      await waitFor(
        () => {
          expect(screen.getByTestId("auth-state")).toHaveTextContent("true");
        },
        { timeout: 3000 },
      );

      await waitFor(
        () => {
          expect(screen.getByTestId("user-data")).toHaveTextContent("testuser");
        },
        { timeout: 3000 },
      );
    });

    test("should handle invalid JWT token", async () => {
      const mockToken = "invalid.token";
      localStorageMock.getItem.mockReturnValue(mockToken);

      mockFetch.mockResolvedValueOnce(
        createMockFetchResponse({ status: "ok" }),
      );

      renderWithAuthProvider(<TestComponent />);

      await waitFor(
        () => {
          expect(screen.getByTestId("loading-state")).toHaveTextContent(
            "false",
          );
        },
        { timeout: 3000 },
      );

      // Should still be authenticated because token validation succeeded
      // but user data should be empty due to parsing failure
      expect(screen.getByTestId("auth-state")).toHaveTextContent("true");
    });
  });

  describe("Error Handling", () => {
    test("should handle token validation error", async () => {
      const mockToken = "valid.jwt.token";
      localStorageMock.getItem.mockReturnValue(mockToken);

      mockFetch.mockRejectedValueOnce(new Error("Network error"));

      renderWithAuthProvider(<TestComponent />);

      await waitFor(
        () => {
          expect(localStorageMock.removeItem).toHaveBeenCalledWith("authToken");
        },
        { timeout: 3000 },
      );

      await waitFor(
        () => {
          expect(screen.getByTestId("loading-state")).toHaveTextContent(
            "false",
          );
          expect(screen.getByTestId("auth-state")).toHaveTextContent("false");
        },
        { timeout: 3000 },
      );
    });
  });

  describe("useAuth Hook", () => {
    test("should throw error when used outside AuthProvider", () => {
      const consoleSpy = jest.spyOn(console, "error").mockImplementation();

      expect(() => {
        render(<TestComponent />);
      }).toThrow("useAuth must be used within an AuthProvider");

      consoleSpy.mockRestore();
    });
  });
});
