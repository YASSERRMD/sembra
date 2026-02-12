import React, { useEffect, useState } from 'react';
import { Activity, Database, FileText, Cpu, HardDrive, Zap, Network, Radio } from 'lucide-react';
import { getStatus, getHealth, getConfig } from '../lib/api';

function StatCard({ icon: Icon, title, value, status, description }) {
    return (
        <div className="card p-6">
            <div className="flex items-center justify-between mb-4">
                <div className="p-2 rounded-lg bg-primary/10">
                    <Icon className="h-5 w-5 text-primary" />
                </div>
                {status && (
                    <span className={`badge ${status === 'healthy' || status === 'Connected' ? 'badge-default' : 'badge-destructive'}`}>
                        {status}
                    </span>
                )}
            </div>
            <h3 className="text-2xl font-bold">{value}</h3>
            <p className="text-sm text-muted-foreground">{title}</p>
            {description && <p className="text-xs text-muted-foreground mt-1">{description}</p>}
        </div>
    );
}

function ComponentCard({ name, status }) {
    const isHealthy = status === 'Connected' || status === 'connected' || status === 'healthy';
    return (
        <div className="flex items-center justify-between p-4 rounded-lg border border-border">
            <div className="flex items-center gap-3">
                <div className={`w-2 h-2 rounded-full ${isHealthy ? 'bg-green-500' : 'bg-red-500'}`} />
                <span className="text-sm font-medium">{name}</span>
            </div>
            <span className={`text-xs ${isHealthy ? 'text-green-500' : 'text-red-500'}`}>
                {status}
            </span>
        </div>
    );
}

export default function Dashboard() {
    const [status, setStatus] = useState(null);
    const [health, setHealth] = useState(null);
    const [config, setConfig] = useState(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchData = async () => {
            try {
                const [statusRes, healthRes, configRes] = await Promise.all([
                    getStatus().catch(() => ({ data: { total_chunks: 0, components: {} } })),
                    getHealth().catch(() => ({ data: { status: 'unknown' } })),
                    getConfig().catch(() => ({ data: null })),
                ]);
                setStatus(statusRes.data);
                setHealth(healthRes.data);
                setConfig(configRes.data);
            } catch (err) {
                console.error('Failed to fetch status:', err);
            } finally {
                setLoading(false);
            }
        };
        fetchData();
        const interval = setInterval(fetchData, 10000);
        return () => clearInterval(interval);
    }, []);

    if (loading) {
        return (
            <div className="flex items-center justify-center h-full">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
        );
    }

    const components = status?.components || {};

    return (
        <div className="p-8">
            {/* Header */}
            <div className="mb-8">
                <h1 className="text-3xl font-bold">Dashboard</h1>
                <p className="text-muted-foreground">System overview and health status</p>
            </div>

            {/* Stats Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                <StatCard
                    icon={FileText}
                    title="Total Chunks"
                    value={status?.total_chunks || 0}
                    description="Documents indexed in BarqDB"
                />
                <StatCard
                    icon={Activity}
                    title="System Status"
                    value={health?.status || 'Unknown'}
                    status={health?.status}
                />
                <StatCard
                    icon={Cpu}
                    title="LLM Provider"
                    value={config?.llm?.provider || 'Not configured'}
                    description={config?.llm?.model || ''}
                />
                <StatCard
                    icon={Zap}
                    title="Embedding Provider"
                    value={config?.embedding?.provider || 'Not configured'}
                    description={config?.embedding?.model || ''}
                />
            </div>

            {/* Components Health */}
            <div className="card p-6">
                <h2 className="text-lg font-semibold mb-4">Infrastructure Health</h2>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <ComponentCard name="PostgreSQL" status={components.postgres || 'Unknown'} />
                    <ComponentCard name="BarqDB (Vector)" status={components.barq_db || 'Unknown'} />
                    <ComponentCard name="GraphDB" status={components.graph_db || 'Unknown'} />
                    <ComponentCard name="Cache (Celrix)" status={components.cache || 'Unknown'} />
                    <ComponentCard name="AiMesh" status={components.aimesh || 'Unknown'} />
                </div>
            </div>
        </div>
    );
}
