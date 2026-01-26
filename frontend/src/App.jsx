import React, { useState, useEffect } from 'react'

// --- Icons (Feather UI) ---
const Icons = {
    Dashboard: () => <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>,
    Search: () => <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>,
    Database: () => <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path><path d="M3 5v14c0 1.66 4 3 9 3s 9-1.34 9-3V5"></path></svg>,
    Settings: () => <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>,
    Help: () => <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>,
    Upload: () => <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>,
    CheckCircle: () => <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
}

export default function App() {
    const [isAuthenticated, setIsAuthenticated] = useState(false)
    const [activeTab, setActiveTab] = useState('dashboard')

    if (!isAuthenticated) return <LoginPage onLogin={() => setIsAuthenticated(true)} />

    return (
        <div className="app-shell">
            <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />
            <div className="viewport">
                <Header activeTab={activeTab} />
                <div className="content-area">
                    {activeTab === 'dashboard' && <Dashboard />}
                    {activeTab === 'search' && <Search />}
                    {activeTab === 'ingest' && <Ingest />}
                    {activeTab === 'config' && <Config />}
                </div>
            </div>
        </div>
    )
}

// --- Layout Components ---

function Sidebar({ activeTab, setActiveTab }) {
    const menu = [
        { id: 'dashboard', label: 'Overview', icon: Icons.Dashboard },
        { id: 'search', label: 'Knowledge Graph', icon: Icons.Search },
        { id: 'ingest', label: 'Data Sources', icon: Icons.Database },
        { id: 'config', label: 'Settings', icon: Icons.Settings },
    ]

    return (
        <div className="sidebar">
            <div className="brand-section">
                <img src="/logo.png" alt="Sembra" className="brand-logo" />
                <div className="brand-name">SEMBRA</div>
            </div>

            <div className="nav-group">
                {menu.map(item => (
                    <div
                        key={item.id}
                        className={`nav-item ${activeTab === item.id ? 'active' : ''}`}
                        onClick={() => setActiveTab(item.id)}
                    >
                        <item.icon />
                        <span>{item.label}</span>
                    </div>
                ))}
            </div>

            <div style={{ marginTop: 'auto' }}>
                <div className="panel" style={{ padding: '0.75rem', marginBottom: 0, display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <div style={{ width: 32, height: 32, borderRadius: '50%', background: 'var(--accent-primary)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#000', fontWeight: 'bold' }}>
                        AD
                    </div>
                    <div>
                        <div style={{ fontSize: '0.85rem', fontWeight: 600 }}>Administrator</div>
                        <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>Enterprise License</div>
                    </div>
                </div>
            </div>
        </div>
    )
}

function Header({ activeTab }) {
    const titles = {
        dashboard: 'System Overview',
        search: 'Semantic Search Engine',
        ingest: 'Data Ingestion Pipeline',
        config: 'Cluster Configuration'
    }

    return (
        <div className="header">
            <div className="breadcrumbs">
                <span>SEMBRA</span>
                <span style={{ opacity: 0.3 }}>/</span>
                <span className="current">{titles[activeTab]}</span>
            </div>
            <div style={{ display: 'flex', gap: '1rem' }}>
                <button className="btn" style={{ background: 'transparent', border: '1px solid var(--border-subtle)', color: 'var(--text-muted)' }}>
                    <Icons.Help /> Documentation
                </button>
            </div>
        </div>
    )
}

// --- Page Components ---

function Dashboard() {
    const [status, setStatus] = useState(null)
    const [loading, setLoading] = useState(true)

    useEffect(() => {
        fetch('/api/v1/status')
            .then(res => res.json())
            .then(data => {
                setStatus(data)
                setLoading(false)
            })
            .catch(err => {
                console.error(err)
                setLoading(false)
            })
    }, [])

    if (loading) return <div className="panel">Accessing telemetry...</div>
    if (!status) return <div className="panel">System Offline</div>

    return (
        <>
            <div className="stats-row">
                <div className="stat-box">
                    <div className="stat-label">Total Documents</div>
                    <div className="stat-value">{status.total_chunks || 0}</div>
                    <div className="badge success">Chunks Indexed</div>
                </div>
                <div className="stat-box">
                    <div className="stat-label">Vector Database</div>
                    <div className="stat-value" style={{ fontSize: '1.5rem', marginTop: '1rem' }}>
                        {status.components.database === 'Connected' ? 'ONLINE' : 'OFFLINE'}
                    </div>
                    <div style={{ marginTop: '0.5rem', color: 'var(--text-muted)', fontSize: '0.8rem' }}>
                        Latency: 12ms
                    </div>
                </div>
                <div className="stat-box">
                    <div className="stat-label">Processing Queue</div>
                    <div className="stat-value" style={{ fontSize: '1.5rem', marginTop: '1rem' }}>IDLE</div>
                    <div style={{ marginTop: '0.5rem', color: 'var(--text-muted)', fontSize: '0.8rem' }}>
                        0 jobs pending
                    </div>
                </div>
            </div>

            <div className="panel">
                <div className="panel-header">
                    <h3 className="panel-title">System Topology</h3>
                </div>
                <div style={{ display: 'flex', gap: '2rem', justifyContent: 'center', padding: '2rem 0' }}>
                    <ComponentStatus name="API Gateway" status="Active" icon="⚡" />
                    <Arrow />
                    <ComponentStatus name="AiMesh Queue" status="Active" icon="🔄" />
                    <Arrow />
                    <ComponentStatus name="Vector Store" status="Active" icon="💾" />
                </div>
            </div>
        </>
    )
}

const ComponentStatus = ({ name, status, icon }) => (
    <div style={{ textAlign: 'center' }}>
        <div style={{ width: 64, height: 64, borderRadius: '16px', background: 'var(--bg-surface)', border: '1px solid var(--border-subtle)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '2rem', margin: '0 auto 1rem auto' }}>
            {icon}
        </div>
        <div style={{ fontWeight: 600 }}>{name}</div>
        <div style={{ fontSize: '0.8rem', color: 'var(--accent-primary)', marginTop: '0.25rem' }}>{status}</div>
    </div>
)

const Arrow = () => (
    <div style={{ alignSelf: 'center', color: 'var(--text-muted)', fontSize: '1.5rem', opacity: 0.3 }}>→</div>
)

function Search() {
    const [query, setQuery] = useState('')
    const [results, setResults] = useState([])
    const [loading, setLoading] = useState(false)

    const search = async (e) => {
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
            }
        } finally {
            setLoading(false)
        }
    }

    return (
        <div style={{ maxWidth: '900px', margin: '0 auto' }}>
            <div className="panel" style={{ padding: '0', overflow: 'hidden' }}>
                <form onSubmit={search} className="search-container" style={{ border: 'none', borderRadius: 0 }}>
                    <Icons.Search style={{ opacity: 0.5 }} />
                    <input
                        className="search-input"
                        placeholder="Ask anything..."
                        value={query}
                        onChange={e => setQuery(e.target.value)}
                        autoFocus
                    />
                    <button type="submit" className="btn btn-primary" disabled={loading}>
                        {loading ? 'Analyzing...' : 'Search'}
                    </button>
                </form>
            </div>

            <div style={{ display: 'grid', gap: '1rem', marginTop: '2rem' }}>
                {results.map((r, i) => (
                    <div key={i} className="panel" style={{ marginBottom: 0, borderLeft: '4px solid var(--accent-primary)' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
                            <div className="badge neutral" style={{ fontSize: '0.7rem' }}>DOC: {r.document_id}</div>
                            <div style={{ color: 'var(--accent-primary)', fontWeight: 700, fontSize: '0.8rem' }}>
                                {Math.round(r.score * 100)}% RELEVANCE
                            </div>
                        </div>
                        <p style={{ lineHeight: 1.6, color: 'var(--text-main)', fontSize: '0.95rem' }}>
                            {r.text}
                        </p>
                    </div>
                ))}
            </div>
        </div>
    )
}

function Ingest() {
    const [text, setText] = useState('')
    const [docId, setDocId] = useState('')
    const [uploading, setUploading] = useState(false)
    const [status, setStatus] = useState(null)

    const handleSubmit = async (e) => {
        e.preventDefault()
        setUploading(true)
        setStatus(null)

        try {
            const res = await fetch('/api/v1/ingest', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text, document_id: docId || `doc_${Date.now()}` })
            })
            const data = await res.json()
            if (res.ok) {
                setStatus({ type: 'success', data })
                setText('')
            } else {
                setStatus({ type: 'error', error: data.error })
            }
        } catch (e) {
            setStatus({ type: 'error', error: e.message })
        } finally {
            setUploading(false)
        }
    }

    return (
        <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 2fr) minmax(0, 1fr)', gap: '2rem' }}>
            <div className="panel">
                <div className="panel-header">
                    <h3 className="panel-title">Ingest Document</h3>
                </div>
                <form onSubmit={handleSubmit}>
                    <div style={{ marginBottom: '1.5rem' }}>
                        <label className="stat-label" style={{ display: 'block', marginBottom: '0.5rem' }}>Document ID (Optional)</label>
                        <input className="input-control" value={docId} onChange={e => setDocId(e.target.value)} placeholder="fiscal_report_2026.pdf" />
                    </div>
                    <div style={{ marginBottom: '1.5rem' }}>
                        <label className="stat-label" style={{ display: 'block', marginBottom: '0.5rem' }}>Content</label>
                        <textarea
                            className="input-control"
                            style={{ minHeight: '300px', fontFamily: 'monospace', lineHeight: 1.5 }}
                            value={text}
                            onChange={e => setText(e.target.value)}
                            placeholder="Paste text content here..."
                        />
                    </div>
                    <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                        <button className="btn btn-primary" type="submit" disabled={uploading}>
                            {uploading ? 'Processing...' : 'Upload & Index'}
                        </button>
                    </div>
                </form>
            </div>

            <div>
                <div className="panel">
                    <div className="panel-header">
                        <h3 className="panel-title">Activity Log</h3>
                    </div>
                    {status ? (
                        <div style={{ background: status.type === 'success' ? 'rgba(16, 185, 129, 0.1)' : 'rgba(239, 68, 68, 0.1)', padding: '1rem', borderRadius: 'var(--radius-sm)', border: `1px solid ${status.type === 'success' ? 'rgba(16, 185, 129, 0.2)' : 'rgba(239, 68, 68, 0.2)'}` }}>
                            <div style={{ fontWeight: 600, color: status.type === 'success' ? '#34d399' : '#f87171', marginBottom: '0.5rem' }}>
                                {status.type === 'success' ? 'Ingestion Successful' : 'Ingestion Failed'}
                            </div>
                            {status.type === 'success' && (
                                <div style={{ fontSize: '0.85rem' }}>
                                    <div>ID: {status.data.document_id}</div>
                                    <div style={{ marginTop: '4px' }}>chunks: {status.data.chunk_count}</div>
                                </div>
                            )}
                        </div>
                    ) : (
                        <div style={{ color: 'var(--text-muted)', fontSize: '0.9rem', textAlign: 'center', padding: '2rem' }}>
                            No recent activity
                        </div>
                    )}
                </div>
            </div>
        </div>
    )
}

function Config() {
    const [config, setConfig] = useState(null)
    const [loading, setLoading] = useState(true)

    useEffect(() => {
        fetch('/api/v1/config')
            .then(res => res.json())
            .then(data => {
                setConfig(data)
                setLoading(false)
            })
    }, [])

    // Simplistic config for MVP - just display current state
    if (loading) return <div className="panel">Loading configuration...</div>

    return (
        <div style={{ maxWidth: '800px' }}>
            <div className="panel">
                <div className="panel-header">
                    <h3 className="panel-title">Cluster Configuration</h3>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2rem' }}>
                    <div>
                        <label className="stat-label">Embedding Provider</label>
                        <div style={{ marginTop: '0.5rem', padding: '0.75rem', background: 'var(--bg-surface)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-sm)' }}>
                            {config && config.embedding_provider}
                        </div>
                    </div>
                    <div>
                        <label className="stat-label">Vector Dimensions</label>
                        <div style={{ marginTop: '0.5rem', padding: '0.75rem', background: 'var(--bg-surface)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-sm)' }}>
                            {config && config.vector_dim}
                        </div>
                    </div>
                </div>

                <div style={{ marginTop: '2rem', paddingTop: '2rem', borderTop: '1px solid var(--border-subtle)' }}>
                    <h4 style={{ margin: '0 0 1rem 0' }}>API Access</h4>
                    <div style={{ display: 'flex', gap: '1rem' }}>
                        <div style={{ flex: 1 }}>
                            <label className="stat-label">Endpoint URL</label>
                            <input className="input-control" value="http://localhost:3000/v1" readOnly style={{ marginTop: '0.5rem' }} />
                        </div>
                        <div style={{ flex: 1 }}>
                            <label className="stat-label">Documentation</label>
                            <button className="btn" style={{ width: '100%', marginTop: '0.5rem', justifyContent: 'center', background: 'var(--bg-surface)' }}>
                                View Swagger UI
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
}

function LoginPage({ onLogin }) {
    const [isLoading, setIsLoading] = useState(false)

    const handleLogin = (e) => {
        e.preventDefault()
        setIsLoading(true)
        setTimeout(() => {
            setIsLoading(false)
            onLogin()
        }, 1200)
    }

    return (
        <div className="login-screen">
            <div className="panel" style={{ width: '400px', textAlign: 'center', padding: '3rem 2rem' }}>
                <img src="/logo.png" style={{ width: 48, height: 48, marginBottom: '1.5rem' }} />
                <h1 style={{ fontSize: '1.5rem', fontWeight: 700, margin: '0 0 0.5rem 0' }}>SEMBRA</h1>
                <p style={{ color: 'var(--text-muted)', fontSize: '0.9rem', marginBottom: '2rem' }}>Enterprise Knowledge Engine</p>

                <form onSubmit={handleLogin}>
                    <div style={{ marginBottom: '1rem', textAlign: 'left' }}>
                        <label className="stat-label" style={{ marginBottom: '0.5rem', display: 'block' }}>Email</label>
                        <input className="input-control" placeholder="admin@enterprise.com" defaultValue="admin@enterprise.com" />
                    </div>
                    <div style={{ marginBottom: '2rem', textAlign: 'left' }}>
                        <label className="stat-label" style={{ marginBottom: '0.5rem', display: 'block' }}>Password</label>
                        <input className="input-control" type="password" placeholder="••••••••" defaultValue="password" />
                    </div>
                    <button className="btn btn-primary" style={{ width: '100%', justifyContent: 'center' }} disabled={isLoading}>
                        {isLoading ? 'Authenticating...' : 'Sign In'}
                    </button>
                </form>

                <div style={{ marginTop: '2rem', fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                    Protected by Sembra SSO
                </div>
            </div>
        </div>
    )
}
