import React, { useState, useEffect } from 'react'

function App() {
    const [query, setQuery] = useState('')
    const [results, setResults] = useState([])
    const [loading, setLoading] = useState(false)
    const [health, setHealth] = useState('checking')
    const [latency, setLatency] = useState(null)

    useEffect(() => {
        checkHealth()
        const interval = setInterval(checkHealth, 30000)
        return () => clearInterval(interval)
    }, [])

    const checkHealth = async () => {
        try {
            const res = await fetch('/api/health')
            if (res.ok) {
                const data = await res.json()
                setHealth(data.status === 'healthy' ? 'healthy' : 'degraded')
            } else {
                setHealth('error')
            }
        } catch {
            setHealth('error')
        }
    }

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
        <>
            <div className="status-bar">
                <span className={`status-indicator status-${health}`}></span>
                API Status: {health.toUpperCase()}
            </div>

            <h1>SEMBRA Search</h1>

            <div className="card">
                <form className="search-container" onSubmit={handleSearch}>
                    <input
                        type="text"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="Search documents..."
                        autoFocus
                    />
                    <button type="submit" disabled={loading}>
                        {loading ? 'Searching...' : 'Search'}
                    </button>
                </form>

                {latency !== null && (
                    <div className="result-header">
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

                    {results.length === 0 && !loading && latency !== null && (
                        <div className="result-item" style={{ textAlign: 'center', color: '#94a3b8' }}>
                            No results found. Try indexing some documents first.
                        </div>
                    )}
                </div>
            </div>
        </>
    )
}

export default App
