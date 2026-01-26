-- SEMBRA PostgreSQL Initialization
-- Creates the required schema with pgvector extension

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS sembra_chunks (
    chunk_id VARCHAR(255) PRIMARY KEY,
    document_id VARCHAR(255) NOT NULL,
    text TEXT NOT NULL,
    embedding vector(768),
    metadata JSONB,
    created_at BIGINT
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_document ON sembra_chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_created ON sembra_chunks(created_at);

-- Full-text search index
CREATE INDEX IF NOT EXISTS idx_text_search ON sembra_chunks 
    USING GIN (to_tsvector('english', text));

-- Vector similarity search index (HNSW)
CREATE INDEX IF NOT EXISTS idx_embedding ON sembra_chunks 
    USING hnsw (embedding vector_cosine_ops);
