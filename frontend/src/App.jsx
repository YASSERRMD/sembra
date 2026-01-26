import React, { useState, useEffect } from 'react'

// --- SVG Icons ---
const DashboardIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <rect x="3" y="3" width="7" height="7"></rect>
        <rect x="14" y="3" width="7" height="7"></rect>
        <rect x="14" y="14" width="7" height="7"></rect>
        <rect x="3" y="14" width="7" height="7"></rect>
    </svg>
)
const SearchIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
    </svg>
)
const IngestIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
        <polyline points="17 8 12 3 7 8"></polyline>
        <line x1="12" y1="3" x2="12" y2="15"></line>
    </svg>
)
const ConfigIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="12" cy="12" r="3"></circle>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
    </svg>
)
const LogoIcon = () => (
    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect width="24" height="24" rx="6" fill="url(#paint0_linear)" />
        <path d="M7 12L10 15L17 8" stroke="white" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
        <defs>
            <linearGradient id="paint0_linear" x1="0" y1="0" x2="24" y2="24" gradientUnits="userSpaceOnUse">
                <stop stopColor="#3B82F6" />
                <stop offset="1" stopColor="#8B5CF6" />
            </linearGradient>
        </defs>
    </svg>
)

export default function App() {
    const [isAuthenticated, setIsAuthenticated] = useState(false)
    const [activeTab, setActiveTab] = useState('dashboard')

    if (!isAuthenticated) {
        return <LoginPage onLogin={() => setIsAuthenticated(true)} />
    }

    return (
        <div className="app-container">
            <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />
            <MainContent activeTab={activeTab} />
        </div>
    )
}

function LoginPage({ onLogin }) {
    const [loading, setLoading] = useState(false);

    const handleSubmit = (e) => {
        e.preventDefault();
        setLoading(true);
        setTimeout(() => {
            setLoading(false);
            onLogin();
        }, 800);
    }

    return (
        <div className="login-container">
            <div className="login-card">
                <div style={{ textAlign: 'center', marginBottom: '2rem' }}>
                    <div style={{ display: 'inline-block', marginBottom: '1rem' }}>
                        <LogoIcon />
                    </div>
                    <h1 style={{ margin: 0, fontSize: '1.75rem' }}>SEMBRA</h1>
                    <div style={{ color: '#94a3b8', marginTop: '0.5rem' }}>Enterprise Knowledge Engine</div>
                </div>
                <form onSubmit={handleSubmit}>
                    <div className="form-group">
                        <label>Email Address</label>
                        <input type="email" placeholder="admin@enterprise.com" required defaultValue="admin@enterprise.com" />
                    </div>
                    <div className="form-group">
                        <label>Password</label>
                        <input type="password" placeholder="••••••••" required defaultValue="password" />
                    </div>
                    <button className="primary" style={{ width: '100%', marginTop: '1rem' }}>
                        {loading ? 'Authenticating...' : 'Sign In'}
                    </button>
                    <div style={{ textAlign: 'center', marginTop: '1.5rem', fontSize: '0.875rem', color: '#64748b' }}>
                        Protected by Sembra Auth
                    </div>
                </form>
            </div>
        </div>
    )
}

function Sidebar({ activeTab, setActiveTab }) {
    const navItems = [
        { id: 'dashboard', label: 'Dashboard', icon: <DashboardIcon /> },
        { id: 'search', label: 'Search Engine', icon: <SearchIcon /> },
        { id: 'ingest', label: 'Data Ingestion', icon: <IngestIcon /> },
        { id: 'config', label: 'Configuration', icon: <ConfigIcon /> },
    ]

    return (
        <div className="sidebar">
            <div className="brand">
                <div style={{ width: 32, height: 32 }}><LogoIcon /></div>
                <div className="brand-text">SEMBRA</div>
            </div>
            <div className="nav-menu">
                {navItems.map(item => (
                    <div
                        key={item.id}
                        className={`nav-item ${activeTab === item.id ? 'active' : ''}`}
                        onClick={() => setActiveTab(item.id)}
                    >
                        {item.icon}
                        <span>{item.label}</span>
                    </div>
                ))}
            </div>

            <div style={{ marginTop: 'auto', paddingTop: '2rem', borderTop: '1px solid var(--border-color)' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                    <div className="avatar">AD</div>
                    <div style={{ fontSize: '0.875rem' }}>
                        <div style={{ fontWeight: 600, color: '#f8fafc' }}>Admin User</div>
                        <div style={{ color: '#64748b', fontSize: '0.75rem' }}>System Admin</div>
                    </div>
                </div>
            </div>
        </div>
    )
}

function MainContent({ activeTab }) {
    return (
        <div className="main-content">
            <div className="top-bar">
                <div className="page-title">
                    {activeTab === 'dashboard' && 'System Overview'}
                    {activeTab === 'search' && 'Semantic Search'}
                    {activeTab === 'ingest' && 'Ingestion Pipeline'}
                    {activeTab === 'config' && 'System Configuration'}
                </div>
                <div style={{ display: 'flex', gap: '1rem' }}>
                    <button style={{ background: 'transparent', border: '1px solid var(--border-color)', color: '#94a3b8', padding: '0.5rem 1rem', borderRadius: '0.5rem', cursor: 'pointer' }}>
                        Help
                    </button>
                </div>
            </div>
            <div className="content-scroll">
                {activeTab === 'dashboard' && <Dashboard />}
                {activeTab === 'search' && <Search />}
                {activeTab === 'ingest' && <Ingest />}
                {activeTab === 'config' && <Config />}
            </div>
        </div>
    )
}

function Dashboard() {
    const [status, setStatus] = useState(null)

    useEffect(() => {
        fetch('/api/v1/status')
            .then(res => res.json())
            .then(setStatus)
            .catch(console.error)
    }, [])

    if (!status) return <div className="card">Loading telemetry...</div>

    return (
        <>
            <div className="stats-grid">
                <div className="stat-card">
                    <div className="stat-label">Total Indexed Chunks</div>
                    <div className="stat-value">{status.total_chunks}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Service Health</div>
                    <div style={{ fontSize: '1.5rem', marginTop: '0.5rem' }}>
                        <span className={`status-badge ${status.components.database === 'Connected' ? 'status-connected' : 'status-error'}`}>
                            {status.components.database === 'Connected' ? 'Healthy' : 'Degraded'}
                        </span>
                    </div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Cache Level</div>
                    <div style={{ fontSize: '1.25rem', color: '#94a3b8', marginTop: '0.5rem' }}>
                        L1: {status.components.cache}
                    </div>
                </div>
            </div>

            <div className="card" style={{ minHeight: '300px', display: 'flex', alignItems: 'center', justifyContent: 'center', borderStyle: 'dashed' }}>
                <div style={{ textAlign: 'center', color: '#64748b' }}>
                    <h3>System Activity</h3>
                    <p>Real-time metrics visualization would appear here (Prometheus integration).</p>
                </div>
            </div>
        </>
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
            }
        } catch (err) {
            console.error(err)
        } finally {
            setLoading(false)
        }
    }

    return (
        <div style={{ maxWidth: '800px', margin: '0 auto' }}>
            <div className="card">
                <form onSubmit={handleSearch}>
                    <div style={{ position: 'relative' }}>
                        <input
                            type="text"
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            placeholder="Search across your enterprise data..."
                            autoFocus
                            style={{ paddingLeft: '3rem', fontSize: '1.1rem' }}
                        />
                        <div style={{ position: 'absolute', left: '1rem', top: '50%', transform: 'translateY(-50%)', color: '#64748b' }}>
                            <SearchIcon />
                        </div>
                    </div>
                </form>
            </div>

            {latency !== null && (
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '1rem', color: '#64748b', fontSize: '0.875rem', padding: '0 1rem' }}>
                    <span>Found {results.length} relevant documents</span>
                    <span>Latency: {latency}ms</span>
                </div>
            )}

            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                {results.map((result) => (
                    <div key={result.chunk_id} className="result-item">
                        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
                            <span style={{ color: '#3b82f6', fontWeight: 600, fontSize: '0.875rem' }}>{result.document_id}</span>
                            <span style={{ color: '#10b981', fontSize: '0.75rem', fontWeight: 600 }}>
                                {Math.round(result.score * 100)}% Match
                            </span>
                        </div>
                        <div style={{ color: '#cbd5e1', lineHeight: '1.6' }}>{result.text}</div>
                    </div>
                ))}
            </div>
        </div>
    )
}

function Ingest() {
    const [text, setText] = useState('')
    const [docId, setDocId] = useState('')
    const [loading, setLoading] = useState(false)
    const [result, setResult] = useState(null)

    const handleIngest = async (e) => {
        e.preventDefault()
        setLoading(true)
        setResult(null)

        try {
            const document_id = docId.trim() || 'doc_' + Date.now();
            const res = await fetch('/api/v1/ingest', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text, document_id })
            })

            if (res.ok) {
                const data = await res.json()
                setResult(data)
                setText('')
            }
        } catch (err) {
            console.error(err)
        } finally {
            setLoading(false)
        }
    }

    return (
        <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '2rem' }}>
            <div className="card">
                <h2>Ingest Content</h2>
                <form onSubmit={handleIngest}>
                    <div className="form-group">
                        <label>Document Identifier</label>
                        <input value={docId} onChange={e => setDocId(e.target.value)} placeholder="e.g. quarterly_report_q1_2026.pdf" />
                    </div>
                    <div className="form-group">
                        <label>Raw Text Content</label>
                        <textarea
                            value={text}
                            onChange={e => setText(e.target.value)}
                            placeholder="Paste document content here..."
                            style={{ minHeight: '300px', fontFamily: 'monospace', fontSize: '0.875rem' }}
                        />
                    </div>
                    <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                        <button className="primary" type="submit" disabled={loading || !text}>
                            {loading ? 'Processing Pipeline...' : 'Start Ingestion'}
                        </button>
                    </div>
                </form>
            </div>

            <div>
                <div className="card">
                    <h2>Pipeline Status</h2>
                    {loading ? (
                        <div style={{ textAlign: 'center', padding: '2rem', color: '#94a3b8' }}>
                            <div className="spinner" style={{ marginBottom: '1rem' }}>⚙️</div>
                            Processing...
                        </div>
                    ) : result ? (
                        <div style={{ background: 'rgba(16, 185, 129, 0.1)', border: '1px solid rgba(16, 185, 129, 0.2)', padding: '1rem', borderRadius: '0.75rem' }}>
                            <div style={{ fontWeight: 600, color: '#34d399', marginBottom: '0.5rem' }}>Success</div>
                            <div style={{ fontSize: '0.875rem' }}>
                                Document <strong>{result.document_id}</strong> processed.
                                <br />
                                Chunks: <strong>{result.chunk_count}</strong>
                            </div>
                        </div>
                    ) : (
                        <div style={{ color: '#64748b', fontSize: '0.875rem' }}>
                            Ready to process. Waiting for input...
                        </div>
                    )}
                </div>
            </div>
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
                setMsg({ type: 'success', text: 'Settings applied successfully' })
            }
        } catch (e) {
            setMsg({ type: 'error', text: e.message })
        } finally {
            setSaving(false)
        }
    }

    if (!config) return <div className="card">Loading...</div>

    return (
        <div style={{ maxWidth: '600px' }}>
            <div className="card">
                <h2>System Configuration</h2>
                <div className="form-group">
                    <label>Embedding Provider</label>
                    <select value={provider} onChange={e => setProvider(e.target.value)}>
                        <option value="mock">Dev (Mock)</option>
                        <option value="openai">OpenAI (Production)</option>
                    </select>
                    <div style={{ fontSize: '0.75rem', color: '#64748b', marginTop: '0.5rem' }}>
                        Select the provider for vector embeddings.
                    </div>
                </div>

                {provider === 'openai' && (
                    <div className="form-group">
                        <label>OpenAI Secret Key</label>
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
                    <input value={config.vector_dim} disabled style={{ background: 'rgba(0,0,0,0.2)', cursor: 'not-allowed' }} />
                </div>

                <div style={{ borderTop: '1px solid var(--border-color)', margin: '2rem 0' }}></div>

                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div>
                        {msg && (
                            <div style={{ color: msg.type === 'success' ? '#34d399' : '#f87171', fontSize: '0.875rem', fontWeight: 500 }}>
                                {msg.text}
                            </div>
                        )}
                    </div>
                    <button className="primary" onClick={handleSave} disabled={saving}>
                        {saving ? 'Saving...' : 'Save Configuration'}
                    </button>
                </div>
            </div>
        </div>
    )
}
