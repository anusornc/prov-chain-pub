import { jest } from "@jest/globals";

// Mock axios implementation
const mockAxios = {
  create: jest.fn(() => mockAxios),
  get: jest.fn(),
  post: jest.fn(),
  put: jest.fn(),
  delete: jest.fn(),
  patch: jest.fn(),
  request: jest.fn(),
  interceptors: {
    request: {
      use: jest.fn(),
      eject: jest.fn(),
    },
    response: {
      use: jest.fn(),
      eject: jest.fn(),
    },
  },
  defaults: {
    headers: {
      common: {},
      get: {},
      post: {},
      put: {},
      delete: {},
      patch: {},
    },
    baseURL: "",
    timeout: 10000,
  },
};

// Helper functions to set mock responses
export const mockAxiosResponse = <T = unknown>(data: T, status = 200) => {
  return Promise.resolve({
    data,
    status,
    statusText: "OK",
    headers: {},
    config: {},
  });
};

export const mockAxiosError = (message: string, status = 500) => {
  return Promise.reject({
    response: {
      data: { message },
      status,
      statusText: "Error",
    },
    message,
  });
};

// Helper to reset all mocks
export const resetAxiosMocks = () => {
  mockAxios.get.mockClear();
  mockAxios.post.mockClear();
  mockAxios.put.mockClear();
  mockAxios.delete.mockClear();
  mockAxios.patch.mockClear();
  mockAxios.request.mockClear();
};

export default mockAxios;
