import axios from 'axios';

const API_BASE_URL = 'http://localhost:8080/api';

// Create axios instance with default config
const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add auth token to requests if available
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('authToken');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle auth errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('authToken');
      window.location.href = '/';
    }
    return Promise.reject(error);
  }
);

// Auth API
export const authAPI = {
  login: (credentials: { username: string; password: string }) =>
    axios.post('http://localhost:8080/auth/login', credentials),
};

// Blockchain API
export const blockchainAPI = {
  getStatus: () => api.get('/blockchain/status'),
  getBlocks: () => api.get('/blockchain/blocks'),
  getBlock: (index: number) => api.get(`/blockchain/blocks/${index}`),
  validate: () => api.get('/blockchain/validate'),
};

// Transaction API
export const transactionAPI = {
  getRecent: () => api.get('/transactions/recent'),
  create: (data: unknown) => api.post('/transactions/create', data),
  sign: (data: unknown) => api.post('/transactions/sign', data),
  submit: (data: unknown) => api.post('/transactions/submit', data),
};

// SPARQL API
export const sparqlAPI = {
  query: (query: string) => api.post('/sparql/query', { query }),
};

// Product API
export const productAPI = {
  getTrace: (productId: string) => api.get('/products/trace', { params: { productId } }),
  getEnhancedTrace: (productId: string) => api.get('/products/trace/enhanced', { params: { productId } }),
};

// RDF API
export const rdfAPI = {
  addTriple: (triple: { subject: string; predicate: string; object: string }) =>
    api.post('/blockchain/add-triple', triple),
};

// Wallet API
export const walletAPI = {
  register: (data: unknown) => api.post('/wallet/register', data),
};

export default api;
