<p align="center">
  <img src="docs/logo.png" alt="SEMBRA Logo" width="120" height="120">
</p>

<h1 align="center">SEMBRA</h1>

<p align="center">
  <strong>Enterprise Document Intelligence Platform</strong>
</p>

![Architecture](docs/architecture.png)

## Overview

SEMBRA is a high-performance document retrieval and intelligence platform that enables:

- **Document Ingestion**: Upload PDF, DOCX, TXT, Markdown files
- **Intelligent Chunking**: Configurable chunk size with overlap for context preservation
- **Semantic Search**: Hybrid vector + BM25 search with RRF fusion
- **Graph Context**: Document relationships and citation chains
- **LLM Synthesis**: AI-powered answers with source citations

## Architecture

| Component | Purpose | Port |
|-----------|---------|------|
| **Postgres** | Auth, Config, Tenants | 5432 |
| **BarqDB** | Vector embeddings storage | 8080 |
| **BarqGraphDB** | Document graph relationships | 8081 |
| **Celrix** | High-speed embedding cache | 6380 |
| **AiMesh** | Message queue for embedding pipeline | 9000 |
| **SEMBRA API** | REST API | 3000 |
| **SEMBRA UI** | Web interface | 80 |

## Quick Start

```bash
# Clone repository
git clone https://github.com/YASSERRMD/sembra.git
cd sembra

# Start all services
cd docker && docker-compose up -d

# Access
# API: http://localhost:3000
# UI:  http://localhost
```

## API Endpoints

### Authentication
```bash
POST /v1/login
{"email": "admin@enterprise.com", "password": "password"}
# Returns: {"token": "eyJ..."}
```

### Document Upload
```bash
POST /v1/upload
Content-Type: multipart/form-data
# files[]: PDF, DOCX, TXT, MD
# document_id: string
# document_name: string
```

### Configure Embedding
```bash
POST /v1/configure-embedding
{
  "provider": "openai|ollama|huggingface",
  "model": "text-embedding-3-small",
  "api_key": "sk-...",
  "batch_size": 32
}
```

### Retrieval
```bash
POST /v1/retrieve
{
  "query": "machine learning optimization",
  "top_k": 10,
  "include_graph": true
}
```

### LLM Synthesis
```bash
POST /v1/retrieve-with-llm
{
  "query": "How does gradient descent work?",
  "top_k": 5,
  "llm_provider": "openai",
  "llm_model": "gpt-4"
}
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Postgres connection | `postgresql://postgres:password@postgres:5432/sembra` |
| `BARQ_DB_URL` | BarqDB endpoint | `http://barq-db:8080` |
| `BARQ_GRAPHDB_URL` | BarqGraphDB endpoint | `http://barq-graphdb:8081` |
| `CELRIX_URL` | Celrix cache | `celrix:6380` |
| `AIMESH_URL` | AiMesh queue | `http://aimesh:9000` |
| `JWT_SECRET` | JWT signing key | Required |

## Development

### Backend (Rust)
```bash
cd backend
cargo build --release
cargo test
```

### Frontend (React)
```bash
cd frontend
npm install
npm run dev
```

### Docker Build
```bash
cd docker
docker-compose up --build -d
```

## Project Structure

```
sembra/
├── backend/
│   ├── sembra-api/      # REST API server
│   ├── sembra-core/     # Business logic, AiMesh consumer
│   ├── sembra-storage/  # Postgres + BarqDB clients
│   ├── sembra-graph/    # BarqGraphDB client
│   ├── sembra-cache/    # Celrix client
│   └── sembra-types/    # Shared types
├── frontend/            # React UI
├── docker/              # Docker configuration
└── docs/                # Documentation
```

## License

MIT License

---

**Built with BarqDB, BarqGraphDB, AiMesh, and Celrix**
