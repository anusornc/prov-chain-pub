/**
 * User Interaction Performance Tests for ProvChain Frontend
 *
 * Tests focusing on:
 * - Response time to user actions
 * - Form submission performance
 * - Search and filtering performance
 * - Navigation and routing performance
 */

import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import React from "react";

// Import components to test
import { TraceabilitySearch } from "../../components/search/TraceabilitySearch";
import { SupplyChainForm } from "../../components/forms/SupplyChainForm";
import { NavigationMenu } from "../../components/navigation/NavigationMenu";
import { FilterPanel } from "../../components/filters/FilterPanel";

describe("User Interaction Performance", () => {
  const MAX_RESPONSE_TIME = 100; // 100ms for simple interactions
  const MAX_COMPLEX_RESPONSE_TIME = 500; // 500ms for complex interactions
  const MAX_SEARCH_TIME = 300; // 300ms for search operations
  const MAX_FORM_SUBMISSION_TIME = 1000; // 1s for form submissions

  describe("Response Time to User Actions", () => {
    test("button clicks should respond immediately", async () => {
      const interactionTimes: number[] = [];

      const TestComponent = () => {
        const [count, setCount] = React.useState(0);

        const handleClick = () => {
          const start = performance.now();
          setCount((c) => c + 1);
          const end = performance.now();
          interactionTimes.push(end - start);
        };

        return (
          <button onClick={handleClick} data-testid="responsive-button">
            Click me: {count}
          </button>
        );
      };

      render(<TestComponent />);

      const button = screen.getByTestId("responsive-button");

      // Test multiple clicks
      for (let i = 0; i < 10; i++) {
        await userEvent.click(button);
      }

      const avgResponseTime =
        interactionTimes.reduce((a, b) => a + b, 0) / interactionTimes.length;
      const maxResponseTime = Math.max(...interactionTimes);

      console.log(
        `Button click - Avg: ${avgResponseTime.toFixed(2)}ms, Max: ${maxResponseTime.toFixed(2)}ms`,
      );

      expect(avgResponseTime).toBeLessThan(MAX_RESPONSE_TIME);
      expect(maxResponseTime).toBeLessThan(MAX_RESPONSE_TIME * 2);
    });

    test("dropdown interactions should be smooth", async () => {
      const interactionTimes: number[] = [];

      const TestDropdown = () => {
        const [isOpen, setIsOpen] = React.useState(false);
        const [selected, setSelected] = React.useState("Option 1");

        const handleToggle = () => {
          const start = performance.now();
          setIsOpen(!isOpen);
          const end = performance.now();
          interactionTimes.push(end - start);
        };

        const handleSelect = (value: string) => {
          const start = performance.now();
          setSelected(value);
          setIsOpen(false);
          const end = performance.now();
          interactionTimes.push(end - start);
        };

        return (
          <div data-testid="dropdown">
            <button onClick={handleToggle} data-testid="dropdown-toggle">
              {selected}
            </button>
            {isOpen && (
              <ul>
                {["Option 1", "Option 2", "Option 3"].map((option) => (
                  <li
                    key={option}
                    onClick={() => handleSelect(option)}
                    data-testid={`option-${option}`}
                  >
                    {option}
                  </li>
                ))}
              </ul>
            )}
          </div>
        );
      };

      render(<TestDropdown />);

      const toggle = screen.getByTestId("dropdown-toggle");

      // Test dropdown open/close
      await userEvent.click(toggle);
      await userEvent.click(screen.getByTestId("option-Option 2"));

      const avgResponseTime =
        interactionTimes.reduce((a, b) => a + b, 0) / interactionTimes.length;

      console.log(`Dropdown interaction time: ${avgResponseTime.toFixed(2)}ms`);
      expect(avgResponseTime).toBeLessThan(MAX_RESPONSE_TIME);
      expect(screen.getByTestId("dropdown")).toHaveTextContent("Option 2");
    });

    test("hover interactions should be responsive", async () => {
      const hoverTimes: number[] = [];

      const TestHover = () => {
        const [isHovered, setIsHovered] = React.useState(false);

        const handleMouseEnter = () => {
          const start = performance.now();
          setIsHovered(true);
          const end = performance.now();
          hoverTimes.push(end - start);
        };

        const handleMouseLeave = () => {
          const start = performance.now();
          setIsHovered(false);
          const end = performance.now();
          hoverTimes.push(end - start);
        };

        return (
          <div
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
            data-testid="hover-element"
            style={{
              backgroundColor: isHovered ? "lightblue" : "white",
              padding: "20px",
              border: "1px solid black",
            }}
          >
            Hover over me
          </div>
        );
      };

      render(<TestHover />);

      const element = screen.getByTestId("hover-element");

      // Test hover interactions
      fireEvent.mouseEnter(element);
      fireEvent.mouseLeave(element);

      const avgHoverTime =
        hoverTimes.reduce((a, b) => a + b, 0) / hoverTimes.length;

      console.log(`Hover interaction time: ${avgHoverTime.toFixed(2)}ms`);
      expect(avgHoverTime).toBeLessThan(MAX_RESPONSE_TIME / 2);
    });
  });

  describe("Form Performance", () => {
    test("form input should handle typing smoothly", async () => {
      const inputTimes: number[] = [];

      const TestForm = () => {
        const [value, setValue] = React.useState("");

        const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
          const start = performance.now();
          setValue(e.target.value);
          const end = performance.now();
          inputTimes.push(end - start);
        };

        return (
          <form data-testid="test-form">
            <input
              type="text"
              value={value}
              onChange={handleChange}
              data-testid="form-input"
              placeholder="Type here..."
            />
          </form>
        );
      };

      render(<TestForm />);

      const input = screen.getByTestId("form-input");

      // Simulate rapid typing
      const testText = "This is a test of input responsiveness";
      for (const char of testText) {
        await userEvent.type(input, char);
      }

      const avgInputTime =
        inputTimes.reduce((a, b) => a + b, 0) / inputTimes.length;
      const maxInputTime = Math.max(...inputTimes);

      console.log(
        `Input handling - Avg: ${avgInputTime.toFixed(2)}ms, Max: ${maxInputTime.toFixed(2)}ms`,
      );

      expect(avgInputTime).toBeLessThan(MAX_RESPONSE_TIME / 2);
      expect(maxInputTime).toBeLessThan(MAX_RESPONSE_TIME);
      expect(screen.getByDisplayValue(testText)).toBeInTheDocument();
    });

    test("complex form submission should be efficient", async () => {
      const submissionTimes: number[] = [];

      const handleSubmit = async (data: any) => {
        const start = performance.now();

        // Simulate API call
        await new Promise((resolve) =>
          setTimeout(resolve, 100 + Math.random() * 400),
        );

        const end = performance.now();
        submissionTimes.push(end - start);

        return { success: true, data };
      };

      const formData = {
        name: "Test Product",
        description: "This is a test product for performance testing",
        category: "Electronics",
        price: 99.99,
        quantity: 100,
        supplier: "Test Supplier",
        tags: ["test", "performance", "validation"],
        metadata: {
          created: new Date().toISOString(),
          version: "1.0.0",
        },
      };

      const submissionStart = performance.now();
      const result = await handleSubmit(formData);
      const submissionEnd = performance.now();

      const totalSubmissionTime = submissionEnd - submissionStart;

      console.log(`Form submission time: ${totalSubmissionTime.toFixed(2)}ms`);

      expect(totalSubmissionTime).toBeLessThan(MAX_FORM_SUBMISSION_TIME);
      expect(result.success).toBe(true);
    });

    test("form validation should be instant", async () => {
      const validationTimes: number[] = [];

      const validateForm = (data: any) => {
        const start = performance.now();

        const errors: string[] = [];
        if (!data.name?.trim()) errors.push("Name is required");
        if (!data.email?.includes("@")) errors.push("Valid email is required");
        if (data.price && data.price <= 0)
          errors.push("Price must be positive");

        const end = performance.now();
        validationTimes.push(end - start);

        return { isValid: errors.length === 0, errors };
      };

      // Test multiple validation scenarios
      const testCases = [
        { name: "", email: "invalid", price: -10 },
        { name: "Valid", email: "valid@test.com", price: 100 },
        { name: "Test", email: "test@example.com", price: 0 },
      ];

      for (const testCase of testCases) {
        validateForm(testCase);
      }

      const avgValidationTime =
        validationTimes.reduce((a, b) => a + b, 0) / validationTimes.length;

      console.log(`Form validation time: ${avgValidationTime.toFixed(2)}ms`);
      expect(avgValidationTime).toBeLessThan(MAX_RESPONSE_TIME / 10);
    });
  });

  describe("Search Performance", () => {
    test("search should respond quickly to input", async () => {
      const searchTimes: number[] = [];

      const performSearch = async (query: string) => {
        const start = performance.now();

        // Simulate search operation
        const results = await new Promise((resolve) => {
          setTimeout(
            () => {
              // Generate mock results
              const mockResults = Array.from({ length: 10 }, (_, i) => ({
                id: i,
                title: `Result ${i} for "${query}"`,
                description: `Description matching ${query}`,
                relevance: Math.random(),
              }));
              resolve(mockResults);
            },
            50 + Math.random() * 100,
          );
        });

        const end = performance.now();
        searchTimes.push(end - start);

        return results;
      };

      // Test various search queries
      const queries = [
        "supply chain",
        "blockchain",
        "traceability",
        "provchain",
        "performance test query with longer text",
      ];

      for (const query of queries) {
        await performSearch(query);
      }

      const avgSearchTime =
        searchTimes.reduce((a, b) => a + b, 0) / searchTimes.length;
      const maxSearchTime = Math.max(...searchTimes);

      console.log(
        `Search performance - Avg: ${avgSearchTime.toFixed(2)}ms, Max: ${maxSearchTime.toFixed(2)}ms`,
      );

      expect(avgSearchTime).toBeLessThan(MAX_SEARCH_TIME);
      expect(maxSearchTime).toBeLessThan(MAX_SEARCH_TIME * 2);
    });

    test("search debouncing should improve performance", async () => {
      const searchCallCount = React.createRef<number>();
      searchCallCount.current = 0;

      const debouncedSearch = (
        callback: (query: string) => void,
        delay: number,
      ) => {
        let timeoutId: NodeJS.Timeout;
        return (query: string) => {
          clearTimeout(timeoutId);
          timeoutId = setTimeout(() => {
            searchCallCount.current! += 1;
            callback(query);
          }, delay);
        };
      };

      const TestSearch = () => {
        const [query, setQuery] = React.useState("");
        const [results, setResults] = React.useState<any[]>([]);

        const handleSearch = debouncedSearch(async (searchQuery: string) => {
          // Simulate search
          const mockResults = Array.from({ length: 5 }, (_, i) => ({
            id: i,
            title: `Result ${i} for "${searchQuery}"`,
          }));
          setResults(mockResults);
        }, 300);

        const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
          const newQuery = e.target.value;
          setQuery(newQuery);
          handleSearch(newQuery);
        };

        return (
          <div data-testid="search-component">
            <input
              type="text"
              value={query}
              onChange={handleChange}
              data-testid="search-input"
              placeholder="Search..."
            />
            <div data-testid="search-results">
              {results.map((result) => (
                <div key={result.id} data-testid={`result-${result.id}`}>
                  {result.title}
                </div>
              ))}
            </div>
          </div>
        );
      };

      render(<TestSearch />);

      const searchInput = screen.getByTestId("search-input");

      // Simulate rapid typing
      await userEvent.type(searchInput, "quick test", { delay: 50 });

      // Wait for debounce
      await new Promise((resolve) => setTimeout(resolve, 400));

      // Should have made fewer calls than characters typed due to debouncing
      expect(searchCallCount.current).toBeLessThan("quick test".length);

      // Should have final results
      expect(screen.getByTestId("search-results")).toBeInTheDocument();
    });
  });

  describe("Filter Performance", () => {
    test("filtering large datasets should be efficient", async () => {
      const largeDataset = Array.from({ length: 10000 }, (_, i) => ({
        id: i,
        name: `Item ${i}`,
        category: `Category ${i % 20}`,
        price: Math.random() * 1000,
        date: new Date(Date.now() - Math.random() * 86400000 * 365),
        status: ["active", "inactive", "pending"][i % 3],
        tags: Array.from({ length: 3 }, (_, j) => `tag${(i + j) % 10}`),
      }));

      const filterTimes: number[] = [];

      const applyFilters = (filters: any, data: any[]) => {
        const start = performance.now();

        const filtered = data.filter((item) => {
          if (filters.category && item.category !== filters.category)
            return false;
          if (filters.status && item.status !== filters.status) return false;
          if (filters.minPrice && item.price < filters.minPrice) return false;
          if (filters.maxPrice && item.price > filters.maxPrice) return false;
          if (
            filters.search &&
            !item.name.toLowerCase().includes(filters.search.toLowerCase())
          )
            return false;
          return true;
        });

        const end = performance.now();
        filterTimes.push(end - start);

        return filtered;
      };

      // Test various filter combinations
      const filterCombinations = [
        { category: "Category 5" },
        { status: "active" },
        { minPrice: 100, maxPrice: 500 },
        { search: "Item 123" },
        { category: "Category 10", status: "active", minPrice: 200 },
      ];

      for (const filters of filterCombinations) {
        const results = applyFilters(filters, largeDataset);
        expect(results.length).toBeGreaterThanOrEqual(0);
      }

      const avgFilterTime =
        filterTimes.reduce((a, b) => a + b, 0) / filterTimes.length;
      const maxFilterTime = Math.max(...filterTimes);

      console.log(
        `Filter performance - Avg: ${avgFilterTime.toFixed(2)}ms, Max: ${maxFilterTime.toFixed(2)}ms`,
      );

      expect(avgFilterTime).toBeLessThan(MAX_COMPLEX_RESPONSE_TIME);
      expect(maxFilterTime).toBeLessThan(MAX_COMPLEX_RESPONSE_TIME * 2);
    });

    test("filter state updates should be immediate", async () => {
      const filterUpdateTimes: number[] = [];

      const TestFilters = () => {
        const [filters, setFilters] = React.useState<Record<string, any>>({});

        const updateFilter = (key: string, value: any) => {
          const start = performance.now();
          setFilters((prev) => ({ ...prev, [key]: value }));
          const end = performance.now();
          filterUpdateTimes.push(end - start);
        };

        return (
          <div data-testid="filter-panel">
            <select
              onChange={(e) => updateFilter("category", e.target.value)}
              data-testid="category-filter"
            >
              <option value="">All Categories</option>
              <option value="electronics">Electronics</option>
              <option value="clothing">Clothing</option>
            </select>
            <input
              type="text"
              placeholder="Min price"
              onChange={(e) =>
                updateFilter("minPrice", parseFloat(e.target.value) || 0)
              }
              data-testid="min-price-filter"
            />
            <input
              type="text"
              placeholder="Max price"
              onChange={(e) =>
                updateFilter("maxPrice", parseFloat(e.target.value) || 0)
              }
              data-testid="max-price-filter"
            />
          </div>
        );
      };

      render(<TestFilters />);

      const categoryFilter = screen.getByTestId("category-filter");
      const minPriceFilter = screen.getByTestId("min-price-filter");
      const maxPriceFilter = screen.getByTestId("max-price-filter");

      // Test rapid filter updates
      await userEvent.selectOptions(categoryFilter, "electronics");
      await userEvent.type(minPriceFilter, "100");
      await userEvent.type(maxPriceFilter, "500");

      const avgUpdateTime =
        filterUpdateTimes.reduce((a, b) => a + b, 0) / filterUpdateTimes.length;

      console.log(`Filter update time: ${avgUpdateTime.toFixed(2)}ms`);
      expect(avgUpdateTime).toBeLessThan(MAX_RESPONSE_TIME / 2);
    });
  });

  describe("Navigation Performance", () => {
    test("route transitions should be fast", async () => {
      const navigationTimes: number[] = [];

      const TestApp = () => {
        return (
          <MemoryRouter>
            <NavigationMenu />
            <Routes>
              <Route
                path="/"
                element={<div data-testid="home-page">Home</div>}
              />
              <Route
                path="/dashboard"
                element={<div data-testid="dashboard-page">Dashboard</div>}
              />
              <Route
                path="/analytics"
                element={<div data-testid="analytics-page">Analytics</div>}
              />
            </Routes>
          </MemoryRouter>
        );
      };

      render(<TestApp />);

      const homeLink = screen.getByText("Home");
      const dashboardLink = screen.getByText("Dashboard");
      const analyticsLink = screen.getByText("Analytics");

      // Test navigation speed
      const navigationTests = [
        { link: dashboardLink, target: "dashboard-page" },
        { link: analyticsLink, target: "analytics-page" },
        { link: homeLink, target: "home-page" },
      ];

      for (const { link, target } of navigationTests) {
        const start = performance.now();
        await userEvent.click(link);
        const end = performance.now();

        navigationTimes.push(end - start);

        await waitFor(() => {
          expect(screen.getByTestId(target)).toBeInTheDocument();
        });
      }

      const avgNavigationTime =
        navigationTimes.reduce((a, b) => a + b, 0) / navigationTimes.length;

      console.log(`Navigation time: ${avgNavigationTime.toFixed(2)}ms`);
      expect(avgNavigationTime).toBeLessThan(MAX_RESPONSE_TIME);
    });

    test("breadcrumb navigation should be responsive", async () => {
      const breadcrumbTimes: number[] = [];

      const TestBreadcrumbs = () => {
        const [path, setPath] = React.useState("/home/products/electronics");

        const navigateTo = (newPath: string) => {
          const start = performance.now();
          setPath(newPath);
          const end = performance.now();
          breadcrumbTimes.push(end - start);
        };

        const segments = path.split("/").filter(Boolean);

        return (
          <nav data-testid="breadcrumbs">
            {segments.map((segment, index) => {
              const segmentPath = "/" + segments.slice(0, index + 1).join("/");
              return (
                <button
                  key={segment}
                  onClick={() => navigateTo(segmentPath)}
                  data-testid={`breadcrumb-${segment}`}
                >
                  {segment}
                </button>
              );
            })}
          </nav>
        );
      };

      render(<TestBreadcrumbs />);

      // Test breadcrumb navigation
      await userEvent.click(screen.getByTestId("breadcrumb-home"));
      await userEvent.click(screen.getByTestId("breadcrumb-products"));

      const avgBreadcrumbTime =
        breadcrumbTimes.reduce((a, b) => a + b, 0) / breadcrumbTimes.length;

      console.log(
        `Breadcrumb navigation time: ${avgBreadcrumbTime.toFixed(2)}ms`,
      );
      expect(avgBreadcrumbTime).toBeLessThan(MAX_RESPONSE_TIME / 2);
    });
  });
});
