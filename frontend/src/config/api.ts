export const API_BASE_URL =
  (global as typeof globalThis & { __VITE_API_BASE_URL__?: string })
    .__VITE_API_BASE_URL__ || "http://localhost:8080";

export const API_ENDPOINTS = {
  BASE: API_BASE_URL,
  API: `${API_BASE_URL}/api`,
  AUTH: `${API_BASE_URL}/auth`,
  WS: API_BASE_URL.replace("http", "ws"),
} as const;
