import React, { useState } from 'react';
import { Search as SearchIcon, Send, Loader2, FileText, ChevronDown, ChevronUp } from 'lucide-react';
import { ask } from '../lib/api';

function ContextSnippet({ text, index }) {
    const [expanded, setExpanded] = useState(false);
    const preview = text.slice(0, 150);
    const hasMore = text.length > 150;

    return (
        <div className="p-3 rounded-lg border border-border bg-muted/30">
            <div className="flex items-start gap-2">
                <span className="badge badge-secondary text-xs">{index + 1}</span>
                <div className="flex-1 min-w-0">
                    <p className="text-sm text-muted-foreground">
                        {expanded ? text : preview}
                        {hasMore && !expanded && '...'}
                    </p>
                    {hasMore && (
                        <button
                            onClick={() => setExpanded(!expanded)}
                            className="text-xs text-primary hover:underline mt-1 flex items-center gap-1"
                        >
                            {expanded ? (
                                <>
                                    <ChevronUp className="h-3 w-3" />
                                    Show less
                                </>
                            ) : (
                                <>
                                    <ChevronDown className="h-3 w-3" />
                                    Show more
                                </>
                            )}
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
}

function Message({ role, content, context }) {
    const isUser = role === 'user';

    return (
        <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
            <div
                className={`max-w-[80%] rounded-lg p-4 ${isUser
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-muted'
                    }`}
            >
                <p className="text-sm whitespace-pre-wrap">{content}</p>

                {/* Context snippets for assistant messages */}
                {!isUser && context && context.length > 0 && (
                    <div className="mt-4 space-y-2">
                        <p className="text-xs font-medium text-muted-foreground mb-2">
                            <FileText className="h-3 w-3 inline mr-1" />
                            Sources ({context.length})
                        </p>
                        {context.map((snippet, i) => (
                            <ContextSnippet key={i} text={snippet} index={i} />
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

export default function Search() {
    const [query, setQuery] = useState('');
    const [messages, setMessages] = useState([]);
    const [loading, setLoading] = useState(false);
    const [includeGraph, setIncludeGraph] = useState(false);

    const handleSubmit = async (e) => {
        e.preventDefault();
        if (!query.trim() || loading) return;

        const userMessage = { role: 'user', content: query };
        setMessages((prev) => [...prev, userMessage]);
        setQuery('');
        setLoading(true);

        try {
            const res = await ask(query, includeGraph);
            const assistantMessage = {
                role: 'assistant',
                content: res.data.answer,
                context: res.data.context_snippets,
            };
            setMessages((prev) => [...prev, assistantMessage]);
        } catch (err) {
            const errorMessage = {
                role: 'assistant',
                content: `Error: ${err.response?.data?.error || err.message || 'Failed to get response'}`,
                context: [],
            };
            setMessages((prev) => [...prev, errorMessage]);
        } finally {
            setLoading(false);
        }
    };

    const clearChat = () => {
        setMessages([]);
    };

    return (
        <div className="h-full flex flex-col">
            {/* Header */}
            <div className="p-6 border-b border-border">
                <div className="flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold">Search & Ask</h1>
                        <p className="text-muted-foreground text-sm">
                            Query your documents using natural language
                        </p>
                    </div>
                    {messages.length > 0 && (
                        <button onClick={clearChat} className="btn btn-outline btn-sm">
                            Clear Chat
                        </button>
                    )}
                </div>

                {/* Options */}
                <div className="flex items-center gap-4 mt-4">
                    <label className="flex items-center gap-2 text-sm">
                        <input
                            type="checkbox"
                            checked={includeGraph}
                            onChange={(e) => setIncludeGraph(e.target.checked)}
                            className="rounded border-border"
                        />
                        Include graph context
                    </label>
                </div>
            </div>

            {/* Messages Area */}
            <div className="flex-1 overflow-y-auto p-6 space-y-4">
                {messages.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-full text-center">
                        <div className="p-4 rounded-full bg-muted mb-4">
                            <SearchIcon className="h-8 w-8 text-muted-foreground" />
                        </div>
                        <h2 className="text-lg font-semibold mb-2">Ask a Question</h2>
                        <p className="text-muted-foreground text-sm max-w-md">
                            Enter a question below to search your uploaded documents using RAG
                            (Retrieval-Augmented Generation).
                        </p>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-2 mt-6 max-w-lg">
                            {[
                                'What are the main topics in my documents?',
                                'Summarize the key findings',
                                'What does the document say about...?',
                                'Find information about...',
                            ].map((suggestion, i) => (
                                <button
                                    key={i}
                                    onClick={() => setQuery(suggestion)}
                                    className="text-left p-3 rounded-lg border border-border hover:bg-accent hover:text-accent-foreground transition-colors text-sm"
                                >
                                    {suggestion}
                                </button>
                            ))}
                        </div>
                    </div>
                ) : (
                    messages.map((msg, i) => (
                        <Message key={i} role={msg.role} content={msg.content} context={msg.context} />
                    ))
                )}

                {/* Loading indicator */}
                {loading && (
                    <div className="flex justify-start">
                        <div className="bg-muted rounded-lg p-4 flex items-center gap-2">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            <span className="text-sm">Thinking...</span>
                        </div>
                    </div>
                )}
            </div>

            {/* Input Area */}
            <div className="p-6 border-t border-border">
                <form onSubmit={handleSubmit} className="flex gap-3">
                    <input
                        type="text"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="Ask a question about your documents..."
                        className="input flex-1"
                        disabled={loading}
                    />
                    <button
                        type="submit"
                        disabled={!query.trim() || loading}
                        className="btn btn-primary btn-icon"
                    >
                        {loading ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                            <Send className="h-4 w-4" />
                        )}
                    </button>
                </form>
            </div>
        </div>
    );
}
