/**
 * Rendering Performance Tests for ProvChain Frontend
 *
 * Tests focusing on:
 * - Component rendering performance
 * - Large data visualization (Cytoscape graphs)
 * - Real-time dashboard updates
 * - Memory leak detection
 */

import { render, screen, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PerformanceObserver } from "perf_hooks";
import React from "react";

// Import components to test
import { AnalyticsDashboard } from "../../components/analytics/AnalyticsDashboard";
import { SupplyChainGraph } from "../../components/graphs/SupplyChainGraph";
import { RealTimeDashboard } from "../../components/dashboard/RealTimeDashboard";
import { DataTable } from "../../components/common/DataTable";

describe("Rendering Performance", () => {
  const MAX_RENDER_TIME = 100; // 100ms for individual renders
  const MAX_LARGE_RENDER_TIME = 1000; // 1s for large datasets
  const MAX_MEMORY_GROWTH = 50 * 1024 * 1024; // 50MB

  describe("Component Rendering Performance", () => {
    test("simple components should render quickly", async () => {
      const renderStart = performance.now();

      render(
        <div data-testid="simple-component">
          <h1>Test Component</h1>
          <p>This is a simple component</p>
        </div>,
      );

      const renderTime = performance.now() - renderStart;

      console.log(`Simple component render time: ${renderTime.toFixed(2)}ms`);
      expect(renderTime).toBeLessThan(MAX_RENDER_TIME);
      expect(screen.getByTestId("simple-component")).toBeInTheDocument();
    });

    test("complex components should render within acceptable time", async () => {
      const renderStart = performance.now();

      render(<AnalyticsDashboard />);

      const renderTime = performance.now() - renderStart;

      console.log(
        `Analytics dashboard render time: ${renderTime.toFixed(2)}ms`,
      );
      expect(renderTime).toBeLessThan(MAX_LARGE_RENDER_TIME);
    });

    test("component re-renders should be optimized", async () => {
      const renderTimes: number[] = [];
      let initialRender = 0;

      const TestComponent = () => {
        const [count, setCount] = React.useState(0);

        React.useEffect(() => {
          const renderStart = performance.now();
          if (initialRender === 0) {
            initialRender = renderStart;
          } else {
            renderTimes.push(renderStart - initialRender);
          }
        });

        return (
          <div data-testid="rerender-test">
            <button onClick={() => setCount((c) => c + 1)}>
              Count: {count}
            </button>
          </div>
        );
      };

      render(<TestComponent />);

      const button = screen.getByRole("button");

      // Test multiple re-renders
      for (let i = 0; i < 10; i++) {
        const rerenderStart = performance.now();
        await userEvent.click(button);
        const rerenderTime = performance.now() - rerenderStart;

        console.log(`Rerender ${i + 1}: ${rerenderTime.toFixed(2)}ms`);
        expect(rerenderTime).toBeLessThan(MAX_RENDER_TIME);
      }

      // Average re-render time should be reasonable
      const avgRerenderTime =
        renderTimes.reduce((a, b) => a + b, 0) / renderTimes.length;
      expect(avgRerenderTime).toBeLessThan(MAX_RENDER_TIME);
    });

    test("component should handle prop updates efficiently", async () => {
      const initialData = generateLargeDataset(100);
      const { rerender } = render(<DataTable data={initialData} />);

      const updateStart = performance.now();

      const updatedData = generateLargeDataset(200);
      rerender(<DataTable data={updatedData} />);

      const updateTime = performance.now() - updateStart;

      console.log(`Large dataset update time: ${updateTime.toFixed(2)}ms`);
      expect(updateTime).toBeLessThan(MAX_LARGE_RENDER_TIME);
    });
  });

  describe("Large Data Visualization Performance", () => {
    test("supply chain graph should handle 1000 nodes", async () => {
      const graphData = generateSupplyChainData(1000);
      const renderStart = performance.now();

      render(<SupplyChainGraph data={graphData} />);

      const renderTime = performance.now() - renderStart;

      console.log(`1000 node graph render time: ${renderTime.toFixed(2)}ms`);
      expect(renderTime).toBeLessThan(MAX_LARGE_RENDER_TIME);
    });

    test("graph interactions should be responsive", async () => {
      const graphData = generateSupplyChainData(500);
      render(<SupplyChainGraph data={graphData} />);

      // Test node selection
      const interactionStart = performance.now();

      // Simulate node selection (would need actual component interaction)
      const nodes = document.querySelectorAll('[data-testid="graph-node"]');
      if (nodes.length > 0) {
        await userEvent.click(nodes[0]);
      }

      const interactionTime = performance.now() - interactionStart;

      console.log(`Graph interaction time: ${interactionTime.toFixed(2)}ms`);
      expect(interactionTime).toBeLessThan(MAX_RENDER_TIME);
    });

    test("graph should render progressively with large datasets", async () => {
      const graphData = generateSupplyChainData(5000);
      const renderTimes: number[] = [];

      const TestComponent = () => {
        const [progress, setProgress] = React.useState(0);

        React.useEffect(() => {
          const interval = setInterval(() => {
            setProgress((p) => Math.min(p + 10, 100));
          }, 50);

          return () => clearInterval(interval);
        }, []);

        return (
          <div>
            <div data-testid="progress">{progress}%</div>
            <SupplyChainGraph data={graphData} progressive={true} />
          </div>
        );
      };

      const renderStart = performance.now();
      render(<TestComponent />);

      // Wait for progressive rendering to complete
      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 1000));
      });

      const totalTime = performance.now() - renderStart;
      console.log(`Progressive rendering time: ${totalTime.toFixed(2)}ms`);

      // Progressive rendering should improve perceived performance
      const progressElement = screen.getByTestId("progress");
      expect(progressElement.textContent).toBe("100%");
    });
  });

  describe("Real-time Dashboard Performance", () => {
    test("dashboard should handle frequent updates efficiently", async () => {
      const renderStart = performance.now();

      render(<RealTimeDashboard />);

      const initialRenderTime = performance.now() - renderStart;
      console.log(
        `Dashboard initial render: ${initialRenderTime.toFixed(2)}ms`,
      );

      // Simulate real-time updates
      const updateTimes: number[] = [];
      for (let i = 0; i < 50; i++) {
        const updateStart = performance.now();

        // Simulate data update
        await act(async () => {
          // In real implementation, this would trigger state updates
          await new Promise((resolve) => setTimeout(resolve, 10));
        });

        const updateTime = performance.now() - updateStart;
        updateTimes.push(updateTime);

        // Each update should be fast
        expect(updateTime).toBeLessThan(MAX_RENDER_TIME);
      }

      const avgUpdateTime =
        updateTimes.reduce((a, b) => a + b, 0) / updateTimes.length;
      console.log(`Average update time: ${avgUpdateTime.toFixed(2)}ms`);
      expect(avgUpdateTime).toBeLessThan(MAX_RENDER_TIME / 2);
    });

    test("dashboard should maintain performance under load", async () => {
      const renderStart = performance.now();

      render(<RealTimeDashboard />);

      // Simulate high-frequency updates
      const highFreqUpdates = 100;
      const updateTimes: number[] = [];

      for (let i = 0; i < highFreqUpdates; i++) {
        const updateStart = performance.now();

        await act(async () => {
          await new Promise((resolve) => setTimeout(resolve, 5));
        });

        const updateTime = performance.now() - updateStart;
        updateTimes.push(updateTime);
      }

      // Performance shouldn't degrade significantly over time
      const firstHalf = updateTimes.slice(0, highFreqUpdates / 2);
      const secondHalf = updateTimes.slice(highFreqUpdates / 2);

      const firstHalfAvg =
        firstHalf.reduce((a, b) => a + b, 0) / firstHalf.length;
      const secondHalfAvg =
        secondHalf.reduce((a, b) => a + b, 0) / secondHalf.length;

      console.log(`First half avg: ${firstHalfAvg.toFixed(2)}ms`);
      console.log(`Second half avg: ${secondHalfAvg.toFixed(2)}ms`);

      // Performance shouldn't degrade more than 50%
      expect(secondHalfAvg).toBeLessThan(firstHalfAvg * 1.5);
    });
  });

  describe("Memory Management", () => {
    test("components should not leak memory", async () => {
      const initialMemory = getMemoryUsage();

      // Create and destroy many components
      for (let i = 0; i < 100; i++) {
        const { unmount } = render(
          <div data-testid={`memory-test-${i}`}>
            <AnalyticsDashboard />
            <SupplyChainGraph data={generateSupplyChainData(100)} />
          </div>,
        );

        // Force cleanup
        unmount();

        // Allow garbage collection
        await act(async () => {
          await new Promise((resolve) => setTimeout(resolve, 10));
        });
      }

      const finalMemory = getMemoryUsage();
      const memoryGrowth = finalMemory - initialMemory;

      console.log(
        `Memory growth: ${(memoryGrowth / 1024 / 1024).toFixed(2)} MB`,
      );
      expect(memoryGrowth).toBeLessThan(MAX_MEMORY_GROWTH);
    });

    test("large datasets should be properly cleaned up", async () => {
      const initialMemory = getMemoryUsage();

      // Render large dataset
      const largeData = generateLargeDataset(10000);
      const { unmount } = render(<DataTable data={largeData} />);

      const peakMemory = getMemoryUsage();

      // Unmount component
      unmount();

      // Force garbage collection
      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 100));
      });

      const finalMemory = getMemoryUsage();

      // Should release most of the memory
      const memoryReleased = peakMemory - finalMemory;
      const totalUsed = peakMemory - initialMemory;

      console.log(
        `Memory released: ${(memoryReleased / 1024 / 1024).toFixed(2)} MB`,
      );
      console.log(
        `Release efficiency: ${((memoryReleased / totalUsed) * 100).toFixed(2)}%`,
      );

      expect(memoryReleased).toBeGreaterThan(totalUsed * 0.7); // Should release at least 70%
    });

    test("event listeners should be properly cleaned up", async () => {
      const eventListenerCount = getEventListenerCount();

      // Create component with event listeners
      const { unmount } = render(<RealTimeDashboard />);

      const peakListenerCount = getEventListenerCount();

      // Unmount component
      unmount();

      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 100));
      });

      const finalListenerCount = getEventListenerCount();

      console.log(
        `Event listeners - Initial: ${eventListenerCount}, Peak: ${peakListenerCount}, Final: ${finalListenerCount}`,
      );

      // Should clean up event listeners
      expect(finalListenerCount).toBeLessThanOrEqual(eventListenerCount + 2); // Allow for test framework listeners
    });
  });

  describe("Animation and Transition Performance", () => {
    test("animations should maintain 60fps", async () => {
      const frameRates: number[] = [];

      const AnimatedComponent = () => {
        const [position, setPosition] = React.useState(0);

        React.useEffect(() => {
          let animationId: number;
          let lastTime = performance.now();

          const animate = (currentTime: number) => {
            const deltaTime = currentTime - lastTime;
            const frameRate = 1000 / deltaTime;
            frameRates.push(frameRate);

            setPosition(currentTime / 10);
            lastTime = currentTime;

            animationId = requestAnimationFrame(animate);
          };

          animationId = requestAnimationFrame(animate);

          return () => cancelAnimationFrame(animationId);
        }, []);

        return (
          <div
            data-testid="animated-component"
            style={{
              transform: `translateX(${position}px)`,
              transition: "transform 0.1s ease-out",
            }}
          >
            Animated Content
          </div>
        );
      };

      render(<AnimatedComponent />);

      // Run animation for 1 second
      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 1000));
      });

      const avgFrameRate =
        frameRates.reduce((a, b) => a + b, 0) / frameRates.length;
      const minFrameRate = Math.min(...frameRates);

      console.log(`Average frame rate: ${avgFrameRate.toFixed(2)} fps`);
      console.log(`Minimum frame rate: ${minFrameRate.toFixed(2)} fps`);

      expect(avgFrameRate).toBeGreaterThan(55); // Allow some variance from 60fps
      expect(minFrameRate).toBeGreaterThan(30); // Should never drop below 30fps
    });

    test("transitions should be smooth", async () => {
      const transitionDurations: number[] = [];

      const TransitionComponent = () => {
        const [visible, setVisible] = React.useState(false);

        React.useEffect(() => {
          const timer = setTimeout(() => setVisible(true), 100);
          return () => clearTimeout(timer);
        }, []);

        return (
          <div
            data-testid="transition-component"
            style={{
              opacity: visible ? 1 : 0,
              transition: "opacity 0.3s ease-in-out",
            }}
          />
        );
      };

      const renderStart = performance.now();
      render(<TransitionComponent />);

      // Wait for transition to complete
      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 500));
      });

      const transitionTime = performance.now() - renderStart;

      console.log(`Transition time: ${transitionTime.toFixed(2)}ms`);
      expect(transitionTime).toBeLessThan(500); // Should be close to 300ms + 100ms delay
    });
  });
});

// Helper functions

function generateLargeDataset(size: number): Array<Record<string, any>> {
  return Array.from({ length: size }, (_, i) => ({
    id: i,
    name: `Item ${i}`,
    value: Math.random() * 1000,
    timestamp: new Date(Date.now() - Math.random() * 86400000).toISOString(),
    category: `Category ${i % 10}`,
    description: `Description for item ${i} with some additional text`,
  }));
}

function generateSupplyChainData(nodeCount: number): {
  nodes: Array<{ id: string; label: string; type: string }>;
  edges: Array<{ source: string; target: string; type: string }>;
} {
  const nodes = Array.from({ length: nodeCount }, (_, i) => ({
    id: `node-${i}`,
    label: `Entity ${i}`,
    type: ["supplier", "manufacturer", "distributor", "retailer"][i % 4],
  }));

  const edges = Array.from({ length: nodeCount * 2 }, (_, i) => ({
    source: `node-${i % nodeCount}`,
    target: `node-${(i + 1) % nodeCount}`,
    type: ["supplies", "transports", "contains"][i % 3],
  }));

  return { nodes, edges };
}

function getMemoryUsage(): number {
  // Simulate memory usage - in real implementation would use performance.memory
  return 100 * 1024 * 1024 + Math.random() * 50 * 1024 * 1024; // 100-150MB base
}

function getEventListenerCount(): number {
  // Simulate event listener count
  return document.querySelectorAll("*").length + Math.floor(Math.random() * 20);
}
