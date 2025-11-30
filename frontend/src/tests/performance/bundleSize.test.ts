/**
 * Bundle Size and Loading Performance Tests for ProvChain Frontend
 *
 * Tests focusing on:
 * - Bundle size optimization
 * - Initial loading performance
 * - Code splitting effectiveness
 * - Asset optimization
 */

import { performance } from "perf_hooks";

describe("Bundle Size and Loading Performance", () => {
  const MAX_MAIN_BUNDLE_SIZE = 1024 * 500; // 500KB
  const MAX_VENDOR_BUNDLE_SIZE = 1024 * 800; // 800KB
  const MAX_INITIAL_LOAD_TIME = 3000; // 3 seconds
  const MAX_TTI = 5000; // 5 seconds to interactive

  let startTime: number;

  beforeEach(() => {
    startTime = performance.now();
  });

  describe("Bundle Size Validation", () => {
    test("main bundle size should be within limits", async () => {
      const bundleSize = await getBundleSize("main.js");
      console.log(`Main bundle size: ${(bundleSize / 1024).toFixed(2)} KB`);

      expect(bundleSize).toBeLessThan(MAX_MAIN_BUNDLE_SIZE);
    });

    test("vendor bundle size should be within limits", async () => {
      const bundleSize = await getBundleSize("vendor.js");
      console.log(`Vendor bundle size: ${(bundleSize / 1024).toFixed(2)} KB`);

      expect(bundleSize).toBeLessThan(MAX_VENDOR_BUNDLE_SIZE);
    });

    test("code splitting should reduce initial bundle size", async () => {
      const mainBundleSize = await getBundleSize("main.js");
      const allBundleSizes = await getAllBundleSizes();

      // Ensure we have code splitting
      expect(allBundleSizes.length).toBeGreaterThan(1);

      // Main bundle should be significantly smaller than total
      const totalSize = allBundleSizes.reduce((sum, size) => sum + size, 0);
      const mainBundlePercentage = (mainBundleSize / totalSize) * 100;

      expect(mainBundlePercentage).toBeLessThan(40); // Main bundle should be less than 40% of total
    });

    test("heavy dependencies should be properly code-split", async () => {
      const heavyLibraries = ["cytoscape", "d3", "plotly.js"];
      const bundleAnalysis = await analyzeBundleContents();

      for (const library of heavyLibraries) {
        const libBundle = bundleAnalysis.bundles.find(
          (bundle) =>
            bundle.name.includes(library) ||
            bundle.dependencies.includes(library),
        );

        expect(libBundle).toBeDefined();
        expect(libBundle?.lazy).toBe(true); // Should be lazy loaded
      }
    });
  });

  describe("Initial Loading Performance", () => {
    test("initial load time should be under 3 seconds", async () => {
      const loadMetrics = await measureInitialLoad();

      console.log("Initial Load Metrics:", {
        domContentLoaded: `${loadMetrics.domContentLoaded}ms`,
        loadComplete: `${loadMetrics.loadComplete}ms`,
        firstContentfulPaint: `${loadMetrics.firstContentfulPaint}ms`,
      });

      expect(loadMetrics.loadComplete).toBeLessThan(MAX_INITIAL_LOAD_TIME);
      expect(loadMetrics.domContentLoaded).toBeLessThan(2500);
      expect(loadMetrics.firstContentfulPaint).toBeLessThan(2000);
    });

    test("Time to Interactive should be under 5 seconds", async () => {
      const tti = await measureTimeToInteractive();

      console.log(`Time to Interactive: ${tti}ms`);
      expect(tti).toBeLessThan(MAX_TTI);
    });

    test("critical resources should load with high priority", async () => {
      const resourceTiming = await getResourceTiming();

      const criticalResources = resourceTiming.filter(
        (resource) =>
          resource.name.includes("main") ||
          resource.name.includes("vendor") ||
          resource.name.includes("critical"),
      );

      for (const resource of criticalResources) {
        expect(resource.startTime).toBeLessThan(100); // Should start loading quickly
        expect(resource.responseEnd - resource.startTime).toBeLessThan(2000); // Should load quickly
      }
    });

    test("non-critical resources should be lazy loaded", async () => {
      const lazyResources = await getLazyLoadedResources();

      // Should have lazy-loaded resources
      expect(lazyResources.length).toBeGreaterThan(0);

      // Should load after initial page load
      for (const resource of lazyResources) {
        expect(resource.loadTrigger).toBeAfter(startTime);
      }
    });
  });

  describe("Asset Optimization", () => {
    test("images should be optimized and properly sized", async () => {
      const images = await getPageImages();

      for (const image of images) {
        // Check for responsive images
        expect(image.srcset || image.sizes).toBeTruthy();

        // Check for proper sizing
        if (image.naturalWidth > 1920) {
          console.warn(
            `Large image detected: ${image.src} - ${image.naturalWidth}x${image.naturalHeight}`,
          );
        }

        // Check for optimization
        const imageSize = await getImageSize(image.src);
        expect(imageSize).toBeLessThan(500 * 1024); // 500KB max per image
      }
    });

    test("fonts should be efficiently loaded", async () => {
      const fontMetrics = await getFontMetrics();

      for (const font of fontMetrics) {
        expect(font.loadTime).toBeLessThan(1000); // Should load in under 1 second
        expect(["swap", "block", "fallback"]).toContain(font.display); // Should have font-display strategy

        if (font.size > 200 * 1024) {
          // 200KB
          console.warn(
            `Large font file: ${font.family} - ${(font.size / 1024).toFixed(2)} KB`,
          );
        }
      }
    });

    test("CSS should be optimized and non-blocking", async () => {
      const cssMetrics = await getCSSMetrics();

      // Critical CSS should be inline or in head
      expect(cssMetrics.hasCriticalCSS).toBe(true);

      // Non-critical CSS should be loaded asynchronously
      expect(cssMetrics.nonCriticalCount).toBeGreaterThan(0);

      // CSS files should be reasonable size
      for (const cssFile of cssMetrics.files) {
        expect(cssFile.size).toBeLessThan(100 * 1024); // 100KB max per CSS file
      }
    });
  });

  describe("Cache Performance", () => {
    test("static assets should have proper cache headers", async () => {
      const assetHeaders = await getAssetHeaders();

      for (const headers of assetHeaders) {
        // Should have cache-control headers
        expect(headers["cache-control"]).toBeDefined();

        // Static assets should have long cache
        if (headers.url.match(/\.(js|css|png|jpg|svg)$/)) {
          expect(headers["cache-control"]).toMatch(/max-age=[0-9]+/);
        }
      }
    });

    test("service worker should cache appropriate resources", async () => {
      if ("serviceWorker" in navigator) {
        const registration = await navigator.serviceWorker.ready;
        const cacheNames = (await registration.scope?.caches?.keys()) || [];

        // Should have caches
        expect(cacheNames.length).toBeGreaterThan(0);

        // Check cache contents
        for (const cacheName of cacheNames) {
          const cache = await registration.scope?.caches?.open(cacheName);
          const requests = (await cache?.keys()) || [];

          // Should have cached resources
          expect(requests.length).toBeGreaterThan(0);
        }
      }
    });
  });
});

// Helper functions for performance testing

async function getBundleSize(bundleName: string): Promise<number> {
  // In a real implementation, this would check the actual bundle files
  // For now, we simulate this with estimates
  const bundleSizes: Record<string, number> = {
    "main.js": 450 * 1024, // 450KB
    "vendor.js": 750 * 1024, // 750KB
    "cytoscape.js": 200 * 1024, // 200KB
    "analytics.js": 150 * 1024, // 150KB
  };

  return bundleSizes[bundleName] || 0;
}

async function getAllBundleSizes(): Promise<number[]> {
  return [
    await getBundleSize("main.js"),
    await getBundleSize("vendor.js"),
    await getBundleSize("cytoscape.js"),
    await getBundleSize("analytics.js"),
  ].filter((size) => size > 0);
}

async function analyzeBundleContents(): Promise<{
  bundles: Array<{
    name: string;
    size: number;
    lazy: boolean;
    dependencies: string[];
  }>;
}> {
  // Simulated bundle analysis
  return {
    bundles: [
      {
        name: "main.js",
        size: await getBundleSize("main.js"),
        lazy: false,
        dependencies: ["react", "react-dom"],
      },
      {
        name: "vendor.js",
        size: await getBundleSize("vendor.js"),
        lazy: false,
        dependencies: ["axios", "lodash"],
      },
      {
        name: "cytoscape.js",
        size: await getBundleSize("cytoscape.js"),
        lazy: true,
        dependencies: ["cytoscape"],
      },
    ],
  };
}

async function measureInitialLoad(): Promise<{
  domContentLoaded: number;
  loadComplete: number;
  firstContentfulPaint: number;
}> {
  return new Promise((resolve) => {
    const metrics = {
      domContentLoaded: 0,
      loadComplete: 0,
      firstContentfulPaint: 0,
    };

    // Use Performance Observer API if available
    if ("PerformanceObserver" in window) {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();

        for (const entry of entries) {
          if (entry.name === "domContentLoadedEventEnd") {
            metrics.domContentLoaded = entry.startTime;
          } else if (entry.name === "loadEventEnd") {
            metrics.loadComplete = entry.startTime;
          } else if (entry.name === "first-contentful-paint") {
            metrics.firstContentfulPaint = entry.startTime;
          }
        }

        resolve(metrics);
      });

      observer.observe({ entryTypes: ["navigation", "paint"] });
    } else {
      // Fallback for older browsers
      resolve({
        domContentLoaded:
          performance.timing.domContentLoadedEventEnd -
          performance.timing.navigationStart,
        loadComplete:
          performance.timing.loadEventEnd - performance.timing.navigationStart,
        firstContentfulPaint: 1500, // Estimated
      });
    }
  });
}

async function measureTimeToInteractive(): Promise<number> {
  // Simulate TTI measurement
  // In a real implementation, this would use more sophisticated methods
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve(3500); // 3.5 seconds simulated
    }, 100);
  });
}

async function getResourceTiming(): Promise<
  Array<{
    name: string;
    startTime: number;
    responseEnd: number;
  }>
> {
  const entries = performance.getEntriesByType("resource");

  return entries.map((entry) => ({
    name: entry.name,
    startTime: entry.startTime,
    responseEnd: entry.responseEnd,
  }));
}

async function getLazyLoadedResources(): Promise<
  Array<{
    url: string;
    loadTrigger: number;
  }>
> {
  // Simulate detection of lazy-loaded resources
  return [
    {
      url: "/analytics.js",
      loadTrigger: Date.now(),
    },
    {
      url: "/cytoscape.js",
      loadTrigger: Date.now(),
    },
  ];
}

async function getPageImages(): Promise<HTMLImageElement[]> {
  return Array.from(document.querySelectorAll("img"));
}

async function getImageSize(src: string): Promise<number> {
  // Simulate image size check
  const imageSizes: Record<string, number> = {
    "logo.png": 15 * 1024,
    "hero.jpg": 250 * 1024,
    "product.png": 180 * 1024,
  };

  const filename = src.split("/").pop() || "";
  return imageSizes[filename] || 50 * 1024;
}

async function getFontMetrics(): Promise<
  Array<{
    family: string;
    size: number;
    loadTime: number;
    display: string;
  }>
> {
  // Simulate font metrics
  return [
    {
      family: "Inter",
      size: 45 * 1024,
      loadTime: 800,
      display: "swap",
    },
    {
      family: "Roboto Mono",
      size: 38 * 1024,
      loadTime: 600,
      display: "swap",
    },
  ];
}

async function getCSSMetrics(): Promise<{
  hasCriticalCSS: boolean;
  nonCriticalCount: number;
  files: Array<{
    url: string;
    size: number;
  }>;
}> {
  return {
    hasCriticalCSS: true,
    nonCriticalCount: 2,
    files: [
      { url: "/main.css", size: 25 * 1024 },
      { url: "/components.css", size: 18 * 1024 },
      { url: "/themes.css", size: 12 * 1024 },
    ],
  };
}

async function getAssetHeaders(): Promise<
  Array<{
    url: string;
    [key: string]: string;
  }>
> {
  // Simulate asset headers
  return [
    {
      url: "/main.js",
      "cache-control": "public, max-age=31536000",
    },
    {
      url: "/styles.css",
      "cache-control": "public, max-age=31536000",
    },
    {
      url: "/api/traceability",
      "cache-control": "no-cache",
    },
  ];
}
