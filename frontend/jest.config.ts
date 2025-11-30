/** @type {import('jest').Config} */
export default {
  // Test environment
  testEnvironment: 'jsdom',

  // Setup files
  setupFilesAfterEnv: ['<rootDir>/src/setupTests.ts'],

  // Module file extensions
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json'],

  // Test file patterns
  testMatch: [
    '<rootDir>/src/**/__tests__/**/*.{ts,tsx}',
    '<rootDir>/src/**/*.{test,spec}.{ts,tsx}',
    '<rootDir>/src/**/*.{test,spec}.{js,jsx}'
  ],

  // Module name mapping for CSS and asset files
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
    '\\.(css|less|scss|sass)$': 'identity-obj-proxy',
    '\\.(jpg|jpeg|png|gif|eot|otf|webp|svg|ttf|woff|woff2|mp4|webm|wav|mp3|m4a|aac|oga)$':
      '<rootDir>/src/test-utils/__mocks__/fileMock.js'
  },

  // Transform configuration
  transform: {
    '^.+\\.(ts|tsx)$': ['ts-jest', {
      tsconfig: 'tsconfig.jest.json'
    }],
    '^.+\\.(js|jsx)$': 'babel-jest',
  },

  // Transform ignore patterns
  transformIgnorePatterns: [
    'node_modules/(?!(.*\\.mjs$|cytoscape|react-dom|@types/cytoscape|d3|@types/d3|react-virtualized|@types/react-virtualized))'
  ],

  // Coverage configuration
  collectCoverage: false, // Disabled for initial setup
  collectCoverageFrom: [
    'src/**/*.{ts,tsx}',
    '!src/**/*.d.ts',
    '!src/test-utils/**/*',
    '!src/**/__tests__/**/*',
    '!src/**/index.ts',
    '!src/main.tsx',
    '!src/vite-env.d.ts',
    '!src/config/**/*',
    '!src/components/traceability/**/*'
  ],
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html', 'json'],
  coverageThreshold: {
    global: {
      branches: 10,
      functions: 10,
      lines: 10,
      statements: 10
    }
  },

  // Mock configurations
  clearMocks: true,
  restoreMocks: true,

  // Performance
  maxWorkers: '50%',

  // Verbose output
  verbose: false,

  // Error handling
  errorOnDeprecated: false,

  // Global variables
  globals: {
    'ts-jest': {
      tsconfig: 'tsconfig.app.json'
    }
  },

  // Test timeout
  testTimeout: 10000,

  // Browser mocks
  setupFiles: ['jest-canvas-mock'],

  // Extensions
  extensionsToTreatAsEsm: ['.ts', '.tsx'],

  // Module paths
  modulePathIgnorePatterns: ['<rootDir>/dist/'],

  // Silent mode for cleaner output
  silent: false,

  // Projects configuration
  projects: [
    {
      displayName: 'Unit Tests',
      testMatch: ['<rootDir>/src/**/__tests__/**/*.{ts,tsx}'],
      setupFilesAfterEnv: ['<rootDir>/src/setupTests.ts'],
      testEnvironment: 'jsdom',
      transform: {
        '^.+\\.(ts|tsx)$': ['ts-jest', { tsconfig: 'tsconfig.jest.json' }]
      },
      moduleNameMapper: {
        '\\.(css|less|scss|sass)$': 'identity-obj-proxy',
        '\\.(jpg|jpeg|png|gif|eot|otf|webp|svg|ttf|woff|woff2|mp4|webm|wav|mp3|m4a|aac|oga)$':
          '<rootDir>/src/test-utils/__mocks__/fileMock.js'
      }
    }
  ]
};