import React, { useState } from 'react';
import axios from 'axios';
import { Save, Settings as SettingsIcon } from 'lucide-react';

const Settings = () => {
    const [gptKey, setGptKey] = useState('');
    const [provider, setProvider] = useState('openai');
    const [model, setModel] = useState('gpt-4o');
    const [saving, setSaving] = useState(false);
    const [msg, setMsg] = useState(null);

    const handleSave = async (e) => {
        e.preventDefault();
        setSaving(true);
        setMsg(null);

        try {
            await axios.post('http://localhost:3000/v1/configure-llm', {
                provider,
                model,
                api_key: provider === 'openai' ? gptKey : undefined
            });
            setMsg({ type: 'success', text: 'Configuration saved successfully!' });
        } catch (err) {
            setMsg({ type: 'error', text: 'Failed to save configuration.' });
        } finally {
            setSaving(false);
        }
    };

    return (
        <div className="max-w-3xl mx-auto">
            <div className="mb-8">
                <h2 className="text-2xl font-bold text-white">System Configuration</h2>
                <p className="text-muted">Manage LLM providers and model parameters.</p>
            </div>

            <div className="glass-panel p-8 rounded-2xl relative overflow-hidden">
                {/* Decorative background element */}
                <div className="absolute top-0 right-0 p-12 opacity-5 pointer-events-none">
                    <SettingsIcon size={200} className="text-white" />
                </div>

                <form onSubmit={handleSave} className="space-y-6 relative z-10">
                    <div className="space-y-2">
                        <label className="text-sm font-medium text-slate-300">LLM Provider</label>
                        <div className="grid grid-cols-2 gap-4">
                            <button
                                type="button"
                                onClick={() => setProvider('openai')}
                                className={`p-4 rounded-xl border transition-all text-left ${provider === 'openai'
                                        ? 'bg-primary/20 border-primary text-white shadow-[0_0_20px_rgba(109,40,217,0.2)]'
                                        : 'bg-surface border-white/5 text-muted hover:bg-white/5'
                                    }`}
                            >
                                <div className="font-bold">OpenAI</div>
                                <div className="text-xs opacity-70">Cloud-based, High Quality</div>
                            </button>
                            <button
                                type="button"
                                onClick={() => setProvider('ollama')}
                                className={`p-4 rounded-xl border transition-all text-left ${provider === 'ollama'
                                        ? 'bg-secondary/20 border-secondary text-white shadow-[0_0_20px_rgba(14,165,233,0.2)]'
                                        : 'bg-surface border-white/5 text-muted hover:bg-white/5'
                                    }`}
                            >
                                <div className="font-bold">Ollama</div>
                                <div className="text-xs opacity-70">Local, Privacy Focused</div>
                            </button>
                        </div>
                    </div>

                    <div className="space-y-2">
                        <label className="text-sm font-medium text-slate-300">Model Name</label>
                        <input
                            type="text"
                            value={model}
                            onChange={(e) => setModel(e.target.value)}
                            className="w-full bg-black/20 border border-white/10 rounded-lg p-3 text-white focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary transition-all"
                            placeholder="e.g. gpt-4o, llama3"
                        />
                    </div>

                    {provider === 'openai' && (
                        <div className="space-y-2">
                            <label className="text-sm font-medium text-slate-300">API Key</label>
                            <input
                                type="password"
                                value={gptKey}
                                onChange={(e) => setGptKey(e.target.value)}
                                className="w-full bg-black/20 border border-white/10 rounded-lg p-3 text-white focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary transition-all"
                                placeholder="sk-..."
                            />
                            <p className="text-xs text-muted">Your key is stored securely in the database.</p>
                        </div>
                    )}

                    <div className="pt-4">
                        <button
                            type="submit"
                            disabled={saving}
                            className="flex items-center space-x-2 px-6 py-3 bg-gradient-to-r from-primary to-accent hover:opacity-90 text-white rounded-lg font-bold shadow-lg shadow-primary/25 transition-all disabled:opacity-50"
                        >
                            <Save size={18} />
                            <span>{saving ? 'Saving...' : 'Save Configuration'}</span>
                        </button>
                    </div>

                    {msg && (
                        <div className={`p-4 rounded-lg border ${msg.type === 'success' ? 'bg-green-500/10 border-green-500/20 text-green-400' : 'bg-red-500/10 border-red-500/20 text-red-400'
                            }`}>
                            {msg.text}
                        </div>
                    )}
                </form>
            </div>
        </div>
    );
};

export default Settings;
