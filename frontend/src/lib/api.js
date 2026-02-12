import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';

const api = axios.create({
    baseURL: API_BASE,
    headers: {
        'Content-Type': 'application/json',
    },
});

// LLM Configuration
export const configureLLM = (data) => api.post('/v1/configure-llm', data);

// Embedding Configuration
export const configureEmbedding = (data) => api.post('/v1/configure-embedding', data);

// Get current config (returns both LLM + Embedding)
export const getConfig = () => api.get('/v1/config');

// Upload document
export const uploadDocument = (formData) =>
    api.post('/v1/upload', formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
    });

// Get document list
export const getDocuments = () => api.get('/v1/documents');

// Ask (RAG)
export const ask = (query, includeGraph = false) =>
    api.post('/v1/ask', { query, include_graph: includeGraph });

// Get status
export const getStatus = () => api.get('/v1/status');

// Health check
export const getHealth = () => api.get('/health');

export default api;
