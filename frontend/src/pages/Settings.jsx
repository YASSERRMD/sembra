import React, { useState, useEffect } from 'react';
import { Save, Loader2, CheckCircle, AlertCircle, Key, Globe, Cpu, Bot, Sparkles } from 'lucide-react';
import { configureLLM, configureEmbedding, getConfig } from '../lib/api';

const LLM_PROVIDERS = [
    { id: 'openai', name: 'OpenAI', requiresKey: true, models: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo'] },
    { id: 'anthropic', name: 'Anthropic', requiresKey: true, models: ['claude-3-5-sonnet-20241022', 'claude-3-opus-20240229', 'claude-3-haiku-20240307'] },
    { id: 'groq', name: 'Groq', requiresKey: true, models: ['llama-3.3-70b-versatile', 'llama-3.1-8b-instant', 'mixtral-8x7b-32768'] },
    { id: 'cohere', name: 'Cohere', requiresKey: true, models: ['command-r-plus', 'command-r', 'command'] },
    { id: 'ollama', name: 'Ollama', requiresKey: false, models: ['llama3', 'llama3:70b', 'mistral', 'mixtral', 'codellama', 'phi3'] },
];

const EMBEDDING_PROVIDERS = [
    { id: 'openai', name: 'OpenAI', requiresKey: true, models: ['text-embedding-3-small', 'text-embedding-3-large', 'text-embedding-ada-002'] },
    { id: 'cohere', name: 'Cohere', requiresKey: true, models: ['embed-english-v3.0', 'embed-multilingual-v3.0', 'embed-english-light-v3.0'] },
    { id: 'ollama', name: 'Ollama', requiresKey: false, models: ['nomic-embed-text', 'mxbai-embed-large', 'all-minilm'] },
];

function ProviderCard({ title, icon: Icon, providers, type, currentConfig, onSave }) {
    const [provider, setProvider] = useState(currentConfig?.provider || providers[0].id);
    const [model, setModel] = useState(currentConfig?.model || providers[0].models[0]);
    const [apiKey, setApiKey] = useState('');
    const [baseUrl, setBaseUrl] = useState('');
    const [saving, setSaving] = useState(false);
    const [result, setResult] = useState(null);

    const selectedProvider = providers.find(p => p.id === provider);

    useEffect(() => {
        setModel(selectedProvider?.models[0] || '');
        setBaseUrl(provider === 'ollama' ? 'http://localhost:11434' : '');
    }, [provider]);

    const handleSave = async () => {
        setSaving(true);
        setResult(null);
        try {
            const payload = {
                provider,
                model,
                ...(apiKey && { api_key: apiKey }),
                ...(baseUrl && { base_url: baseUrl }),
            };

            if (type === 'llm') {
                await configureLLM(payload);
            } else {
                await configureEmbedding(payload);
            }

            setResult({ success: true, message: 'Configuration saved!' });
            onSave?.();
        } catch (err) {
            setResult({ success: false, message: err.response?.data || err.message });
        } finally {
            setSaving(false);
        }
    };

    return (
        <div className="card p-6">
            <div className="flex items-center gap-3 mb-6">
                <div className="p-2 rounded-lg bg-primary/10">
                    <Icon className="h-5 w-5 text-primary" />
                </div>
                <h2 className="text-lg font-semibold">{title}</h2>
            </div>

            <div className="space-y-4">
                {/* Provider Select */}
                <div>
                    <label className="label block mb-2">Provider</label>
                    <select
                        value={provider}
                        onChange={(e) => setProvider(e.target.value)}
                        className="select"
                    >
                        {providers.map((p) => (
                            <option key={p.id} value={p.id}>{p.name}</option>
                        ))}
                    </select>
                </div>

                {/* Model Select */}
                <div>
                    <label className="label block mb-2">Model</label>
                    <select
                        value={model}
                        onChange={(e) => setModel(e.target.value)}
                        className="select"
                    >
                        {selectedProvider?.models.map((m) => (
                            <option key={m} value={m}>{m}</option>
                        ))}
                    </select>
                    <p className="text-xs text-muted-foreground mt-1">
                        Or enter a custom model name below
                    </p>
                    <input
                        type="text"
                        value={model}
                        onChange={(e) => setModel(e.target.value)}
                        placeholder="Custom model name..."
                        className="input mt-2"
                    />
                </div>

                {/* API Key */}
                {selectedProvider?.requiresKey && (
                    <div>
                        <label className="label block mb-2">
                            <Key className="h-3 w-3 inline mr-1" />
                            API Key
                        </label>
                        <input
                            type="password"
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder="sk-..."
                            className="input"
                        />
                        <p className="text-xs text-muted-foreground mt-1">
                            Leave empty to use environment variable
                        </p>
                    </div>
                )}

                {/* Base URL */}
                <div>
                    <label className="label block mb-2">
                        <Globe className="h-3 w-3 inline mr-1" />
                        Base URL (Optional)
                    </label>
                    <input
                        type="text"
                        value={baseUrl}
                        onChange={(e) => setBaseUrl(e.target.value)}
                        placeholder={provider === 'ollama' ? 'http://localhost:11434' : 'Custom endpoint URL...'}
                        className="input"
                    />
                    <p className="text-xs text-muted-foreground mt-1">
                        For custom endpoints or self-hosted models
                    </p>
                </div>

                {/* Save Button */}
                <button
                    onClick={handleSave}
                    disabled={saving}
                    className="btn btn-primary btn-md w-full"
                >
                    {saving ? (
                        <>
                            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            Saving...
                        </>
                    ) : (
                        <>
                            <Save className="h-4 w-4 mr-2" />
                            Save Configuration
                        </>
                    )}
                </button>

                {/* Result */}
                {result && (
                    <div
                        className={`flex items-center gap-2 p-3 rounded-lg ${result.success ? 'bg-green-500/10 text-green-500' : 'bg-destructive/10 text-destructive'
                            }`}
                    >
                        {result.success ? (
                            <CheckCircle className="h-4 w-4" />
                        ) : (
                            <AlertCircle className="h-4 w-4" />
                        )}
                        <span className="text-sm">{result.message}</span>
                    </div>
                )}
            </div>
        </div>
    );
}

export default function Settings() {
    const [currentConfig, setCurrentConfig] = useState(null);

    useEffect(() => {
        getConfig()
            .then((res) => setCurrentConfig(res.data))
            .catch(() => { });
    }, []);

    const refetchConfig = () => {
        getConfig()
            .then((res) => setCurrentConfig(res.data))
            .catch(() => { });
    };

    return (
        <div className="p-8">
            {/* Header */}
            <div className="mb-8">
                <h1 className="text-3xl font-bold">Settings</h1>
                <p className="text-muted-foreground">Configure LLM and Embedding providers</p>
            </div>

            {/* Provider Cards */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <ProviderCard
                    title="LLM Provider"
                    icon={Bot}
                    providers={LLM_PROVIDERS}
                    type="llm"
                    currentConfig={currentConfig}
                    onSave={refetchConfig}
                />
                <ProviderCard
                    title="Embedding Provider"
                    icon={Sparkles}
                    providers={EMBEDDING_PROVIDERS}
                    type="embedding"
                    currentConfig={currentConfig}
                    onSave={refetchConfig}
                />
            </div>

            {/* Current Config Info */}
            {currentConfig && (
                <div className="mt-6 card p-6">
                    <h2 className="text-lg font-semibold mb-4">Current Configuration</h2>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                        <div>
                            <span className="text-muted-foreground">Status:</span>
                            <p className="font-medium">{currentConfig.status}</p>
                        </div>
                        <div>
                            <span className="text-muted-foreground">Provider:</span>
                            <p className="font-medium">{currentConfig.provider}</p>
                        </div>
                        <div>
                            <span className="text-muted-foreground">Model:</span>
                            <p className="font-medium">{currentConfig.model}</p>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
