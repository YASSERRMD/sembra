import React, { useState, useRef, useEffect } from 'react';
import axios from 'axios';
import { Send, Bot, User as UserIcon, Loader2, Search as SearchIcon } from 'lucide-react';
import { motion } from 'framer-motion';
import clsx from 'clsx';

const Search = () => {
    const [query, setQuery] = useState('');
    const [messages, setMessages] = useState([]);
    const [loading, setLoading] = useState(false);
    const messagesEndRef = useRef(null);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    };

    useEffect(scrollToBottom, [messages]);

    const handleSearch = async (e) => {
        e.preventDefault();
        if (!query.trim()) return;

        const userMsg = { role: 'user', content: query };
        setMessages(prev => [...prev, userMsg]);
        setQuery('');
        setLoading(true);

        try {
            const res = await axios.post('http://localhost:3000/v1/ask', {
                query: userMsg.content
            });

            const aiMsg = {
                role: 'assistant',
                content: res.data.answer,
                snippets: res.data.context_snippets
            };
            setMessages(prev => [...prev, aiMsg]);
        } catch (err) {
            setMessages(prev => [...prev, { role: 'assistant', content: "Sorry, I encountered an error retrieving that information." }]);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="flex flex-col h-[calc(100vh-6rem)]">
            <div className="flex-none mb-6">
                <h2 className="text-2xl font-bold text-white">Semantic Search</h2>
                <p className="text-muted">Ask questions about your documents using RAG.</p>
            </div>

            <div className="flex-1 glass-panel rounded-2xl overflow-hidden flex flex-col">
                {/* Chat Area */}
                <div className="flex-1 overflow-y-auto p-6 space-y-6">
                    {messages.length === 0 && (
                        <div className="h-full flex flex-col items-center justify-center text-muted opacity-50">
                            <SearchIcon size={48} className="mb-4" />
                            <p>Ask a question to start searching...</p>
                        </div>
                    )}

                    {messages.map((msg, idx) => (
                        <motion.div
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            key={idx}
                            className={clsx(
                                "flex space-x-4 max-w-4xl",
                                msg.role === 'user' ? "ml-auto" : "mr-auto"
                            )}
                        >
                            <div className={clsx(
                                "w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 mt-1",
                                msg.role === 'user' ? "bg-primary text-white" : "bg-secondary text-white"
                            )}>
                                {msg.role === 'user' ? <UserIcon size={16} /> : <Bot size={16} />}
                            </div>
                            <div className="flex-1 space-y-2">
                                <div className={clsx(
                                    "p-4 rounded-2xl text-sm leading-relaxed",
                                    msg.role === 'user'
                                        ? "bg-primary/20 text-white rounded-tr-none"
                                        : "bg-surface border border-white/5 text-slate-200 rounded-tl-none shadow-lg"
                                )}>
                                    {msg.content}
                                </div>

                                {msg.snippets && msg.snippets.length > 0 && (
                                    <div className="grid gap-2 mt-2">
                                        {msg.snippets.map((snip, i) => (
                                            <div key={i} className="text-xs p-3 bg-black/40 rounded border border-white/5 text-muted hover:text-slate-300 transition-colors">
                                                <span className="font-mono text-primary/70 mb-1 block">Context Fragment {i + 1}</span>
                                                {snip.substring(0, 150)}...
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </div>
                        </motion.div>
                    ))}

                    {loading && (
                        <div className="flex space-x-4 max-w-4xl mr-auto">
                            <div className="w-8 h-8 rounded-lg bg-secondary text-white flex items-center justify-center flex-shrink-0 mt-1">
                                <Bot size={16} />
                            </div>
                            <div className="bg-surface border border-white/5 p-4 rounded-2xl rounded-tl-none text-muted flex items-center space-x-2">
                                <Loader2 size={16} className="animate-spin" />
                                <span className="text-xs">Processing query...</span>
                            </div>
                        </div>
                    )}
                    <div ref={messagesEndRef} />
                </div>

                {/* Input Area */}
                <div className="p-4 border-t border-white/5 bg-black/20 backdrop-blur-md">
                    <form onSubmit={handleSearch} className="relative max-w-4xl mx-auto">
                        <input
                            type="text"
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            placeholder="Ask about your documents..."
                            className="w-full bg-surface border border-white/10 text-white pl-4 pr-12 py-4 rounded-xl focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all shadow-inner"
                        />
                        <button
                            type="submit"
                            disabled={loading || !query.trim()}
                            className="absolute right-2 top-2 p-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            <Send size={20} />
                        </button>
                    </form>
                </div>
            </div>
        </div>
    );
};

export default Search;
