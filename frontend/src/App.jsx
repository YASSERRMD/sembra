import React, { useState, useEffect } from 'react'

export default function App() {
    const [activeTab, setActiveTab] = useState('dashboard')

    return (
        <>
            <div className="header">
                <h1>SEMBRA</h1>
                <div style={{ color: '#94a3b8' }}>v0.1.0</div>
            </div>

            <div className="tabs">
                <button className={`tab ${activeTab === 'dashboard' ? 'active' : ''}`} onClick={() => setActiveTab('dashboard')}>Dashboard</button>
                <button className={`tab ${activeTab === 'search' ? 'active' : ''}`} onClick={() => setActiveTab('search')}>Search</button>
                <button className={`tab ${activeTab === 'ingest' ? 'active' : ''}`} onClick={() => setActiveTab('ingest')}>Ingest</button>
                <button className={`tab ${activeTab === 'config' ? 'active' : ''}`} onClick={() => setActiveTab('config')}>Config</button>
            </div>

            <div className="content">
                {activeTab === 'dashboard' && <Dashboard />}
                {activeTab === 'search' && <Search />}
                {activeTab === 'ingest' && <Ingest />}
                {activeTab === 'config' && <Config />}
            </div>
        </>
    )
}

function Dashboard() {
    const [status, setStatus] = useState(null)
    const [error, setError] = useState(null)

    useEffect(() => {
        fetch('/api/v1/status')
            .then(res => {
                if (!res.ok) throw new Error('Failed to fetch status')
                return res.json()
            })
            .then(setStatus)
            .catch(err => setError(err.message))
    }, [])

    if (error) return <div className="card" style={{ color: 'var(--error)' }}>Error: {error}</div>
    if (!status) return <div className="card">Loading status...</div>

    return (
        <div className="card">
            <h2>System Overview</h2>
            <div className="stats-grid">
                <div className="stat-card">
                    <div className="stat-label">Total Chunks</div>
                    <div className="stat-value">{status.total_chunks}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Database Status</div>
                    <div className="stat-value" style={{ fontSize: '1.2rem' }}>
                        <span className={`status-badge ${status.components.database === 'Connected' ? 'status-connected' : 'status-error'}`}>
                            {status.components.database}
                        </span>
                    </div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Cache Status</div>
                    <div className="stat-value" style={{ fontSize: '1rem', color: '#94a3b8' }}>
                        {status.components.cache}
                    </div>
                </div>
            </div>
        </div>
    )
}

function Search() {
    const [query, setQuery] = useState('')
    const [results, setResults] = useState([])
    const [loading, setLoading] = useState(false)
    const [latency, setLatency] = useState(null)

    const handleSearch = async (e) => {
        e.preventDefault()
        if (!query.trim()) return

        setLoading(true)
        try {
            const res = await fetch('/api/v1/retrieve', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query, top_k: 5 })
            })

            if (res.ok) {
                const data = await res.json()
                setResults(data.results)
                setLatency(data.latency_ms)
            } else {
                console.error('Search failed')
            }
        } catch (err) {
            console.error(err)
        } finally {
            setLoading(false)
        }
    }

    return (
        <div className="card">
            <h2>Semantic Search</h2>
            <form onSubmit={handleSearch}>
                <div className="form-group">
                    <input
                        type="text"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="Search documents..."
                        autoFocus
                    />
                </div>
                <button className="primary" type="submit" disabled={loading}>
                    {loading ? 'Searching...' : 'Search'}
                </button>
            </form>

            {latency !== null && (
                <div className="result-header" style={{ marginTop: '2rem' }}>
                    <span>Found {results.length} results</span>
                    <span>Latency: {latency}ms</span>
                </div>
            )}

            <div className="results-list">
                {results.map((result) => (
                    <div key={result.chunk_id} className="result-item">
                        <div className="result-header">
                            <span>{result.document_id}</span>
                            <span className="score">Score: {(result.score * 100).toFixed(1)}%</span>
                        </div>
                        <div className="result-text">{result.text}</div>
                    </div>
                ))}

                {results.length === 0 && latency !== null && (
                    <div className="result-item" style={{ textAlign: 'center', color: '#94a3b8' }}>
                        No results found.
                    </div>
                )}
            </div>
        </div>
    )
}

function Ingest() {
    const [text, setText] = useState('')
    const [docId, setDocId] = useState('')
    const [loading, setLoading] = useState(false)
    const [result, setResult] = useState(null)
    const [error, setError] = useState(null)

    const handleIngest = async (e) => {
        e.preventDefault()
        setLoading(true)
        setError(null)
        setResult(null)

        try {
            const document_id = docId.trim() || 'doc_' + Date.now();
            const res = await fetch('/api/v1/ingest', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text, document_id })
            })

            if (!res.ok) throw new Error('Ingestion failed');

            const data = await res.json()
            setResult(data)
            setText('')
            setDocId('')
        } catch (err) {
            setError(err.message)
        } finally {
            setLoading(false)
        }
    }

    return (
        <div className="card">
            <h2>Ingest Document</h2>
            <div style={{ color: '#94a3b8', marginBottom: '1rem' }}>
                Paste text to chunk, embed, and index into the database.
            </div>
            <form onSubmit={handleIngest}>
                <div className="form-group">
                    <label>Document ID (Optional)</label>
                    <input value={docId} onChange={e => setDocId(e.target.value)} placeholder="e.g. policy-v1" />
                </div>
                <div className="form-group">
                    <label>Content</label>
                    <textarea value={text} onChange={e => setText(e.target.value)} placeholder="Paste text here..." />
                </div>
                <button className="primary" type="submit" disabled={loading || !text}>
                    {loading ? 'Chunking & Indexing...' : 'Ingest Document'}
                </button>
            </form>

            {error && (
                <div style={{ marginTop: '1rem', padding: '1rem', background: 'rgba(239,68,68,0.1)', color: 'var(--error)', borderRadius: '0.5rem' }}>
                    {error}
                </div>
            )}

            {result && (
                <div style={{ marginTop: '1rem', padding: '1rem', background: 'rgba(74,222,128,0.1)', borderRadius: '0.5rem', color: '#f0fdf4' }}>
                    ✅ Successfully ingested <strong>{result.chunk_count}</strong> chunks for Document <strong>{result.document_id}</strong>.
                </div>
            )}
        </div>
    )
}

function Config() {
    const [config, setConfig] = useState(null)
    const [provider, setProvider] = useState('')
    const [apiKey, setApiKey] = useState('')
    const [saving, setSaving] = useState(false)
    const [msg, setMsg] = useState(null)

    useEffect(() => {
        fetch('/api/v1/config')
            .then(res => res.json())
            .then(data => {
                setConfig(data)
                setProvider(data.embedding_provider)
            })
            .catch(console.error)
    }, [])

    const handleSave = async () => {
        setSaving(true)
        setMsg(null)
        try {
            const res = await fetch('/api/v1/config', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    embedding_provider: provider,
                    openai_api_key: apiKey || undefined
                })
            })
            if (res.ok) {
                const data = await res.json()
                setConfig(data)
                setMsg({ type: 'success', text: 'Configuration saved!' })
            } else {
                throw new Error('Failed to save')
            }
        } catch (e) {
            setMsg({ type: 'error', text: e.message })
        } finally {
            setSaving(false)
        }
    }

    if (!config) return <div className="card">Loading config...</div>

    return (
        <div className="card">
            <h2>Configuration</h2>
            <div className="form-group">
                <label>Embedding Provider</label>
                <select value={provider} onChange={e => setProvider(e.target.value)} style={{ width: '100%', padding: '0.75rem', borderRadius: '0.5rem', background: '#0f172a', color: 'white', border: '1px solid #334155' }}>
                    <option value="mock">Mock (Random)</option>
                    <option value="openai">OpenAI</option>
                </select>
            </div>

            {provider === 'openai' && (
                <div className="form-group">
                    <label>OpenAI API Key</label>
                    <input
                        type="password"
                        value={apiKey}
                        onChange={e => setApiKey(e.target.value)}
                        placeholder="sk-..."
                    />
                </div>
            )}

            <div className="form-group">
                <label>Vector Dimensions</label>
                <input value={config.vector_dim} disabled />
            </div>

            <button className="primary" onClick={handleSave} disabled={saving}>
                {saving ? 'Saving...' : 'Save Configuration'}
            </button>

            {msg && (
                <div style={{ marginTop: '1rem', color: msg.type === 'success' ? '#4ade80' : '#ef4444' }}>
                    {msg.text}
                </div>
            )}
        </div>
    )
}
