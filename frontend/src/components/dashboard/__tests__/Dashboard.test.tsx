import React from "react";
import { act, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Dashboard from "../Dashboard";
import { customRender } from "../../../test-utils/test-utils";

// Mock hooks
const mockUseBlockchain = jest.fn();
const mockUseWebSocket = jest.fn();

jest.mock("../../../hooks/useBlockchain", () => ({
  __esModule: true,
  default: () => mockUseBlockchain(),
}));

jest.mock("../../../hooks/useWebSocket", () => ({
  __esModule: true,
  default: () => mockUseWebSocket(),
}));

// Mock fetch for recent activity
const mockFetch = jest.fn();
global.fetch = mockFetch;

const renderDashboard = async () => {
  const result = customRender(<Dashboard />, { withAuth: false });
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
    await Promise.resolve();
  });
  return result;
};

// Mock localStorage
const localStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};
Object.defineProperty(window, "localStorage", { value: localStorageMock });

describe("Dashboard Component", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    localStorageMock.getItem.mockReturnValue("mock-auth-token");
    mockUseBlockchain.mockReturnValue({
      metrics: {
        total_blocks: 1234,
        total_transactions: 5678,
        total_items: 9012,
        active_participants: 34,
        avg_block_time: 5.2,
        transactions_per_second: 12.5,
        network_hash_rate: 1500000000,
        network_status: "healthy",
      },
      networkHealth: {
        status: "healthy",
        uptime: 86400,
        peer_count: 8,
        sync_status: "synced",
        last_block_age: 30,
      },
      loading: false,
      error: null,
      refresh: jest.fn(),
    });
    mockUseWebSocket.mockReturnValue({
      isConnected: true,
    });

    // Mock successful recent activity fetch
    mockFetch.mockResolvedValue({
      ok: true,
      json: jest.fn().mockResolvedValue({
        transactions: [
          {
            id: "tx_1",
            type: "RDF_Data",
            timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
            data: { subject: "test-batch-001" },
          },
          {
            id: "tx_2",
            type: "Block_Creation",
            timestamp: new Date(Date.now() - 10 * 60 * 1000).toISOString(),
            data: { subject: "block-567" },
          },
        ],
      }),
    });
  });

  describe("Component Rendering", () => {
    test("should render dashboard header correctly", async () => {
      await renderDashboard();

      expect(
        screen.getByText("Blockchain Explorer Dashboard"),
      ).toBeInTheDocument();
      expect(
        screen.getByText(
          "Real-time overview of the ProvChain network and traceability system",
        ),
      ).toBeInTheDocument();
    });

    test("should display connection status", async () => {
      await renderDashboard();

      expect(screen.getByText("Connected")).toBeInTheDocument();
      const statusIndicator =
        screen.getByText("Connected").previousElementSibling;
      expect(statusIndicator).toHaveClass("bg-green-400");
    });

    test("should display all metric cards", async () => {
      await renderDashboard();

      // Core metrics
      expect(screen.getByText("Total Blocks")).toBeInTheDocument();
      expect(screen.getByText("1,234")).toBeInTheDocument();
      expect(screen.getByText("Total Transactions")).toBeInTheDocument();
      expect(screen.getByText("5,678")).toBeInTheDocument();
      expect(screen.getByText("Traced Items")).toBeInTheDocument();
      expect(screen.getByText("9,012")).toBeInTheDocument();
      expect(screen.getByText("Active Participants")).toBeInTheDocument();
      expect(screen.getByText("34")).toBeInTheDocument();

      // Performance metrics
      expect(screen.getByText("Avg Block Time")).toBeInTheDocument();
      expect(screen.getByText("5.2s")).toBeInTheDocument();
      expect(screen.getByText("TPS")).toBeInTheDocument();
      expect(screen.getByText("12.5")).toBeInTheDocument();
      expect(screen.getByText("Network Hash Rate")).toBeInTheDocument();
      expect(screen.getByText("1500.0M")).toBeInTheDocument();
      expect(screen.getAllByText("Network Status").length).toBeGreaterThan(0);
      expect(screen.getAllByText("healthy").length).toBeGreaterThan(0);
    });

    test("should display network status section", async () => {
      await renderDashboard();

      expect(screen.getAllByText("Network Status").length).toBeGreaterThan(0);
      expect(screen.getAllByText("healthy").length).toBeGreaterThan(0);
      expect(screen.getByText("24h 0m")).toBeInTheDocument(); // 86400 seconds
      expect(screen.getByText("8")).toBeInTheDocument(); // peer count
      expect(screen.getByText("synced")).toBeInTheDocument();
    });

    test("should display recent activity section", async () => {
      await renderDashboard();

      expect(screen.getByText("Recent Activity")).toBeInTheDocument();
      await waitFor(() => {
        expect(
          screen.getByText("RDF data added - test-batch-001"),
        ).toBeInTheDocument();
      });
    });
  });

  describe("Loading States", () => {
    test("should show loading state when metrics are loading", async () => {
      // Mock loading state
      mockUseBlockchain.mockReturnValue({
        metrics: null,
        networkHealth: null,
        loading: true,
        error: null,
        refresh: jest.fn(),
      });

      await renderDashboard();

      // Check for loading skeletons
      const loadingElements = document.querySelectorAll(".animate-pulse");
      expect(loadingElements.length).toBeGreaterThan(0);
    });
  });

  describe("Error Handling", () => {
    test("should display error message when there is an error", async () => {
      // Mock error state
      mockUseBlockchain.mockReturnValue({
        metrics: null,
        networkHealth: null,
        loading: false,
        error: "Network connection failed",
        refresh: jest.fn(),
      });

      await renderDashboard();

      expect(
        screen.getByText(
          "Error loading dashboard data: Network connection failed",
        ),
      ).toBeInTheDocument();
      expect(screen.getAllByText("Network Error").length).toBeGreaterThan(0);
    });
  });

  describe("Data Refresh", () => {
    test("should call refresh when refresh button is clicked", async () => {
      const mockRefresh = jest.fn();
      mockUseBlockchain.mockReturnValue({
        metrics: {
          total_blocks: 1234,
          total_transactions: 5678,
          total_items: 9012,
          active_participants: 34,
          avg_block_time: 5.2,
          transactions_per_second: 12.5,
          network_hash_rate: 1500000000,
          network_status: "healthy",
        },
        networkHealth: {
          status: "healthy",
          uptime: 86400,
          peer_count: 8,
          sync_status: "synced",
          last_block_age: 30,
        },
        loading: false,
        error: null,
        refresh: mockRefresh,
      });

      const user = userEvent.setup();
      await renderDashboard();

      const refreshButton = screen.getByText("Refresh");
      await user.click(refreshButton);

      expect(mockRefresh).toHaveBeenCalled();
    });

    test("should disable refresh button while loading", async () => {
      mockUseBlockchain.mockReturnValue({
        metrics: null,
        networkHealth: null,
        loading: true,
        error: null,
        refresh: jest.fn(),
      });

      await renderDashboard();

      const refreshButton = screen.getByText("Refresh");
      expect(refreshButton).toBeDisabled();
    });
  });

  describe("Recent Activity", () => {
    test("should handle recent activity fetch success", async () => {
      await renderDashboard();

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining("/transactions/recent"),
          expect.objectContaining({
            headers: {
              Authorization: "Bearer mock-auth-token",
              "Content-Type": "application/json",
            },
          }),
        );
      });

      await waitFor(() => {
        expect(
          screen.getByText("RDF data added - test-batch-001"),
        ).toBeInTheDocument();
        expect(screen.getByText("Block created - block-567")).toBeInTheDocument();
      });
    });

    test("should handle recent activity fetch failure with fallback", async () => {
      const consoleErrorSpy = jest
        .spyOn(console, "error")
        .mockImplementation(() => undefined);
      // Mock failed fetch
      mockFetch.mockRejectedValueOnce(new Error("Network error"));

      await renderDashboard();

      await waitFor(() => {
        expect(
          screen.getByText("System active - 1234 blocks processed"),
        ).toBeInTheDocument();
      });
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Error fetching recent activity:",
        expect.any(Error),
      );
      consoleErrorSpy.mockRestore();
    });

    test("should handle recent activity fetch with non-OK response", async () => {
      // Mock non-OK response
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
      });

      await renderDashboard();

      await waitFor(() => {
        expect(
          screen.getByText("Latest block #1233 created"),
        ).toBeInTheDocument();
      });
    });

    test("should format timestamps correctly", async () => {
      const timestamp = new Date("2025-08-27T10:30:00Z").toISOString();
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: jest.fn().mockResolvedValue({
          transactions: [
            {
              id: "tx_1",
              type: "RDF_Data",
              timestamp,
              data: { subject: "test-batch-001" },
            },
          ],
        }),
      });

      await renderDashboard();

      await waitFor(() => {
        const timeElement = screen.getByText(/\d{1,2}:\d{2}:\d{2} [AP]M/); // Time format
        expect(timeElement).toBeInTheDocument();
      });
    });
  });

  describe("Accessibility", () => {
    test("should have proper heading hierarchy", async () => {
      await renderDashboard();

      const mainHeading = screen.getByRole("heading", { level: 1 });
      expect(mainHeading).toHaveTextContent("Blockchain Explorer Dashboard");

      const subHeadings = screen.getAllByRole("heading", { level: 3 });
      expect(subHeadings.length).toBeGreaterThan(0);
    });

    test("should support keyboard navigation", async () => {
      const user = userEvent.setup();
      await renderDashboard();

      const refreshButton = screen.getByRole("button", { name: /refresh/i });
      await user.tab();

      expect(refreshButton).toHaveFocus();
    });

    test("should have proper ARIA labels", async () => {
      await renderDashboard();

      // Check for proper button labels
      const refreshButton = screen.getByRole("button", { name: /refresh/i });
      expect(refreshButton).toBeInTheDocument();
    });
  });

  describe("Responsive Design", () => {
    test("should render correctly on mobile viewports", async () => {
      // Mock mobile viewport
      Object.defineProperty(window, "innerWidth", {
        writable: true,
        configurable: true,
        value: 375,
      });

      await renderDashboard();

      // Should still render all main components
      expect(
        screen.getByText("Blockchain Explorer Dashboard"),
      ).toBeInTheDocument();
      expect(screen.getByText("Total Blocks")).toBeInTheDocument();
      expect(screen.getAllByText("Network Status").length).toBeGreaterThan(0);
      expect(screen.getByText("Recent Activity")).toBeInTheDocument();
    });
  });

  describe("Data Display", () => {
    test("should format large numbers correctly", async () => {
      await renderDashboard();

      expect(screen.getByText("1,234")).toBeInTheDocument(); // Thousands separator
      expect(screen.getByText("5,678")).toBeInTheDocument();
      expect(screen.getByText("9,012")).toBeInTheDocument();
    });

    test("should format hash rate correctly", async () => {
      await renderDashboard();

      expect(screen.getByText("1500.0M")).toBeInTheDocument(); // Convert to millions
    });

    test("should format uptime correctly", async () => {
      await renderDashboard();

      expect(screen.getByText("24h 0m")).toBeInTheDocument(); // 86400 seconds
    });

    test("should handle zero values gracefully", async () => {
      mockUseBlockchain.mockReturnValue({
        metrics: {
          total_blocks: 0,
          total_transactions: 0,
          total_items: 0,
          active_participants: 0,
          avg_block_time: 0,
          transactions_per_second: 0,
          network_hash_rate: 0,
          network_status: "offline",
        },
        networkHealth: {
          status: "offline",
          uptime: 0,
          peer_count: 0,
          sync_status: "syncing",
          last_block_age: 0,
        },
        loading: false,
        error: null,
        refresh: jest.fn(),
      });

      await renderDashboard();

      expect(screen.getAllByText("0").length).toBeGreaterThan(0);
      expect(screen.getByText("0s")).toBeInTheDocument();
      expect(screen.getAllByText("offline").length).toBeGreaterThan(0);
    });
  });
});
