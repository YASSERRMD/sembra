<p align="center">
  <img src="assets/logo.svg" alt="SEMBRA Logo" width="200">
</p>

<h1 align="center">SEMBRA</h1>

<p align="center">
  <strong>Semantic Search & Document Retrieval Engine</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#api">API</a> •
  <a href="#docker">Docker</a>
</p>

---

## Overview

SEMBRA is a high-performance document retrieval system that combines **vector similarity search**, **BM25 full-text search**, and **graph-based relationships** for intelligent semantic search. Built with Rust for maximum performance and reliability.

## Features

🔍 **Hybrid Search** - Combines vector embeddings with BM25 using Reciprocal Rank Fusion (RRF)

⚡ **High Performance** - Sub-100ms search latency with moka in-memory caching

🧠 **Vector Embeddings** - 768-dimensional vectors with pgvector HNSW indexing

📊 **Graph Relationships** - Document relationships via GraphDB with neighbor traversal

🔌 **REST API** - Clean axum-based API with `/health` and `/v1/retrieve` endpoints

🐳 **Docker Ready** - Production-ready Docker deployment with PostgreSQL and Redis

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      REST API (Axum)                        │
│                 GET /health  POST /v1/retrieve              │
└─────────────────────────┬───────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │  AiMesh  │   │  Celrix  │   │  Graph   │
    │ Consumer │   │  Cache   │   │    DB    │
    │ (Broker) │   │  (moka)  │   │ (nodes)  │
    └──────────┘   └──────────┘   └──────────┘
          │               │               │
          └───────────────┼───────────────┘
                          │
                          ▼
    ┌─────────────────────────────────────────────────────────┐
    │                      BarqDB                             │
    │              PostgreSQL + pgvector                      │
    │  ┌─────────────────┐    ┌─────────────────┐            │
    │  │  Vector Search  │    │   BM25 Search   │            │
    │  │  (HNSW Index)   │    │   (GIN Index)   │            │
    │  └─────────────────┘    └─────────────────┘            │
    │                    ▼                                    │
    │            Hybrid RRF Fusion                            │
    └─────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 15+ with pgvector extension
- Docker & Docker Compose (optional)

### Build

```bash
cd backend
cargo build --release
```

### Run

```bash
# Start the API server
cargo run -p sembra-api

# Server starts at http://localhost:3000
```

### Test

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p sembra-core
cargo test -p sembra-cache
cargo test -p sembra-storage
```

## API

### Health Check

```bash
curl http://localhost:3000/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_secs": 123
}
```

### Retrieve Documents

```bash
curl -X POST http://localhost:3000/v1/retrieve \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning concepts",
    "top_k": 10,
    "query_embedding": [0.1, 0.2, ...]
  }'
```

Response:
```json
{
  "results": [
    {"chunk_id": "chunk:123", "score": 0.95, "text": "..."},
    {"chunk_id": "chunk:456", "score": 0.89, "text": "..."}
  ],
  "latency_ms": 45
}
```

## Docker

### Start Services

```bash
cd docker
docker-compose up -d
```

This starts:
- **PostgreSQL** with pgvector on port 5432
- **Redis** cache on port 6379
- **SEMBRA API** on port 3000

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgresql://postgres:password@localhost:5432/sembra` | PostgreSQL connection |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection |
| `PORT` | `3000` | API server port |

## Project Structure

```
sembra/
├── backend/
│   ├── sembra-api/        # REST API server
│   ├── sembra-core/       # AiMesh message consumer
│   ├── sembra-cache/      # Celrix in-memory cache
│   ├── sembra-storage/    # BarqDB PostgreSQL layer
│   ├── sembra-graph/      # GraphDB relationships
│   └── sembra-types/      # Shared types
├── docker/
│   ├── docker-compose.yml
│   ├── Dockerfile.api
│   └── postgres-init.sql
├── tests/
│   └── integration_*.rs
└── assets/
    └── logo.svg
```

## Crates

| Crate | Description |
|-------|-------------|
| `sembra-api` | Axum REST API with health and retrieve endpoints |
| `sembra-core` | AiMesh consumer for message broker integration |
| `sembra-cache` | High-performance moka cache with TTL and hit rate tracking |
| `sembra-storage` | PostgreSQL storage with pgvector for embeddings |
| `sembra-graph` | Graph database for document relationships |

## Search Algorithms

### Vector Search
Uses pgvector's cosine distance with HNSW indexing for fast approximate nearest neighbor search on 768-dimensional embeddings.

### BM25 Search
PostgreSQL full-text search with `ts_rank_cd` scoring and GIN indexing.

### Hybrid RRF Fusion
Combines vector and BM25 results using Reciprocal Rank Fusion (k=60):

```
score = Σ 1/(k + rank) * weight
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  Built with 🦀 Rust
</p>
