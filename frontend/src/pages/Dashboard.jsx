import React, { useEffect, useState } from 'react';
import axios from 'axios';
import { Activity, Database, Server, Cpu, HardDrive, FileText } from 'lucide-react';
import { motion } from 'framer-motion';

const StatCard = ({ icon: Icon, label, value, subtext, color = "text-primary" }) => (
    <motion.div
        whileHover={{ y: -5 }}
        className="glass-card p-6 rounded-xl relative overflow-hidden"
    >
        <div className={`absolute top-0 right-0 p-4 opacity-10 ${color}`}>
            <Icon size={64} />
        </div>
        <div className="relative z-10">
            <div className={`p-3 rounded-lg w-fit mb-4 bg-white/5 ${color}`}>
                <Icon size={24} />
            </div>
            <h3 className="text-3xl font-bold text-white mb-1">{value}</h3>
            <p className="text-muted font-medium mb-2">{label}</p>
            {subtext && <p className="text-xs text-white/40">{subtext}</p>}
        </div>
    </motion.div>
);

const Dashboard = () => {
    const [stats, setStats] = useState(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchStats = async () => {
            try {
                // In real app, we fetch from /v1/status
                // For now, mock or fetch
                const res = await axios.get('http://localhost:3000/v1/status');
                setStats(res.data);
            } catch (err) {
                console.error("Failed to fetch stats", err);
            } finally {
                setLoading(false);
            }
        };
        fetchStats();
    }, []);

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-2xl font-bold text-white">System Overview</h2>
                    <p className="text-muted">Real-time infrastructure monitoring</p>
                </div>
                <div className="flex items-center space-x-2 text-xs bg-white/5 px-3 py-1 rounded-full border border-white/5">
                    <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
                    <span className="text-green-400 font-mono">SYSTEM ONLINE</span>
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <StatCard
                    icon={FileText}
                    label="Total Documents"
                    value={stats?.total_chunks || "0"}
                    subtext="Processed Chunks"
                    color="text-blue-500"
                />
                <StatCard
                    icon={Database}
                    label="Vector DB"
                    value={stats?.components?.barq_db || "Unknown"}
                    subtext="BarqDB Cluster"
                    color="text-purple-500"
                />
                <StatCard
                    icon={Activity}
                    label="AiMesh Queue"
                    value="Active"
                    subtext="Worker Swarm Ready"
                    color="text-rose-500"
                />
                <StatCard
                    icon={Cpu}
                    label="LLM Provider"
                    value="OpenAI"
                    subtext="GPT-4o Optimized"
                    color="text-emerald-500"
                />
            </div>

            {/* Recent Activity Placeholder */}
            <div className="glass-panel rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4">Recent Ingestion Activity</h3>
                <div className="space-y-4">
                    {[1, 2, 3].map((i) => (
                        <div key={i} className="flex items-center justify-between p-3 bg-white/5 rounded-lg border border-white/5">
                            <div className="flex items-center space-x-3">
                                <div className="w-8 h-8 rounded bg-primary/20 flex items-center justify-center text-primary">
                                    <FileText size={16} />
                                </div>
                                <div>
                                    <p className="text-sm font-medium text-white">contract_v{i}.pdf</p>
                                    <p className="text-xs text-muted">Processed {i * 15} mins ago</p>
                                </div>
                            </div>
                            <span className="text-xs font-mono text-green-400 bg-green-500/10 px-2 py-1 rounded">INDEXED</span>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export default Dashboard;
