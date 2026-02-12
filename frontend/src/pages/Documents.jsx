import React, { useState, useCallback, useEffect } from 'react';
import { Upload as UploadIcon, FileText, X, CheckCircle, AlertCircle, Loader2, RefreshCw } from 'lucide-react';
import { uploadDocument, getDocuments } from '../lib/api';

export default function Documents() {
    const [files, setFiles] = useState([]);
    const [uploading, setUploading] = useState(false);
    const [results, setResults] = useState([]);
    const [dragActive, setDragActive] = useState(false);
    const [chunkSize, setChunkSize] = useState(512);
    const [chunkOverlap, setChunkOverlap] = useState(128);
    const [documents, setDocuments] = useState([]);
    const [loadingDocs, setLoadingDocs] = useState(true);

    const fetchDocuments = useCallback(async () => {
        setLoadingDocs(true);
        try {
            const res = await getDocuments();
            setDocuments(res.data || []);
        } catch {
            setDocuments([]);
        } finally {
            setLoadingDocs(false);
        }
    }, []);

    useEffect(() => {
        fetchDocuments();
    }, [fetchDocuments]);

    const handleDrag = useCallback((e) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.type === 'dragenter' || e.type === 'dragover') {
            setDragActive(true);
        } else if (e.type === 'dragleave') {
            setDragActive(false);
        }
    }, []);

    const handleDrop = useCallback((e) => {
        e.preventDefault();
        e.stopPropagation();
        setDragActive(false);
        if (e.dataTransfer.files && e.dataTransfer.files[0]) {
            setFiles((prev) => [...prev, ...Array.from(e.dataTransfer.files)]);
        }
    }, []);

    const handleFileSelect = (e) => {
        if (e.target.files) {
            setFiles((prev) => [...prev, ...Array.from(e.target.files)]);
        }
    };

    const removeFile = (index) => {
        setFiles((prev) => prev.filter((_, i) => i !== index));
    };

    const handleUpload = async () => {
        if (files.length === 0) return;
        setUploading(true);
        setResults([]);

        const uploadResults = [];
        for (const file of files) {
            try {
                const formData = new FormData();
                formData.append('file', file);
                formData.append('chunk_size', chunkSize.toString());
                formData.append('chunk_overlap', chunkOverlap.toString());

                const res = await uploadDocument(formData);
                uploadResults.push({
                    file: file.name,
                    success: true,
                    data: res.data,
                });
            } catch (err) {
                uploadResults.push({
                    file: file.name,
                    success: false,
                    error: err.response?.data?.error || err.message,
                });
            }
        }

        setResults(uploadResults);
        setUploading(false);
        setFiles([]);
        // Refresh document list after upload
        fetchDocuments();
    };

    return (
        <div className="p-8">
            {/* Header */}
            <div className="mb-8">
                <h1 className="text-3xl font-bold">Documents</h1>
                <p className="text-muted-foreground">Upload and manage your documents</p>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                {/* Upload Section */}
                <div className="lg:col-span-2">
                    <div className="card p-6">
                        <h2 className="text-lg font-semibold mb-4">Upload Documents</h2>

                        {/* Drag & Drop Zone */}
                        <div
                            onDragEnter={handleDrag}
                            onDragLeave={handleDrag}
                            onDragOver={handleDrag}
                            onDrop={handleDrop}
                            className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${dragActive
                                ? 'border-primary bg-primary/5'
                                : 'border-border hover:border-primary/50'
                                }`}
                        >
                            <UploadIcon className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
                            <p className="text-muted-foreground mb-2">
                                Drag and drop files here, or click to select
                            </p>
                            <input
                                type="file"
                                multiple
                                accept=".pdf,.docx,.txt,.md"
                                onChange={handleFileSelect}
                                className="hidden"
                                id="file-input"
                            />
                            <label
                                htmlFor="file-input"
                                className="btn btn-outline btn-sm cursor-pointer"
                            >
                                Select Files
                            </label>
                            <p className="text-xs text-muted-foreground mt-2">
                                Supports PDF, DOCX, TXT, MD
                            </p>
                        </div>

                        {/* File List */}
                        {files.length > 0 && (
                            <div className="mt-4 space-y-2">
                                <h3 className="text-sm font-medium">Selected Files</h3>
                                {files.map((file, index) => (
                                    <div
                                        key={index}
                                        className="flex items-center justify-between p-3 rounded-lg border border-border"
                                    >
                                        <div className="flex items-center gap-3">
                                            <FileText className="h-4 w-4 text-muted-foreground" />
                                            <span className="text-sm">{file.name}</span>
                                            <span className="text-xs text-muted-foreground">
                                                ({(file.size / 1024).toFixed(1)} KB)
                                            </span>
                                        </div>
                                        <button
                                            onClick={() => removeFile(index)}
                                            className="text-muted-foreground hover:text-destructive"
                                        >
                                            <X className="h-4 w-4" />
                                        </button>
                                    </div>
                                ))}
                            </div>
                        )}

                        {/* Upload Button */}
                        <button
                            onClick={handleUpload}
                            disabled={files.length === 0 || uploading}
                            className="btn btn-primary btn-md w-full mt-4"
                        >
                            {uploading ? (
                                <>
                                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                                    Uploading...
                                </>
                            ) : (
                                <>
                                    <UploadIcon className="h-4 w-4 mr-2" />
                                    Upload {files.length > 0 && `(${files.length})`}
                                </>
                            )}
                        </button>

                        {/* Results */}
                        {results.length > 0 && (
                            <div className="mt-4 space-y-2">
                                <h3 className="text-sm font-medium">Upload Results</h3>
                                {results.map((result, index) => (
                                    <div
                                        key={index}
                                        className={`flex items-center gap-3 p-3 rounded-lg ${result.success ? 'bg-green-500/10' : 'bg-destructive/10'
                                            }`}
                                    >
                                        {result.success ? (
                                            <CheckCircle className="h-4 w-4 text-green-500" />
                                        ) : (
                                            <AlertCircle className="h-4 w-4 text-destructive" />
                                        )}
                                        <div className="flex-1">
                                            <p className="text-sm font-medium">{result.file}</p>
                                            {result.success ? (
                                                <p className="text-xs text-muted-foreground">
                                                    {result.data.chunk_count} chunks created
                                                </p>
                                            ) : (
                                                <p className="text-xs text-destructive">{result.error}</p>
                                            )}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>

                    {/* Document Library */}
                    <div className="card p-6 mt-6">
                        <div className="flex items-center justify-between mb-4">
                            <h2 className="text-lg font-semibold">Document Library</h2>
                            <button onClick={fetchDocuments} className="btn btn-outline btn-sm">
                                <RefreshCw className={`h-4 w-4 mr-1 ${loadingDocs ? 'animate-spin' : ''}`} />
                                Refresh
                            </button>
                        </div>

                        {loadingDocs ? (
                            <div className="flex items-center justify-center py-8">
                                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                            </div>
                        ) : documents.length === 0 ? (
                            <div className="flex flex-col items-center justify-center py-8 text-center">
                                <FileText className="h-12 w-12 text-muted-foreground mb-3" />
                                <p className="text-muted-foreground text-sm">No documents uploaded yet</p>
                                <p className="text-xs text-muted-foreground mt-1">Upload a document above to get started</p>
                            </div>
                        ) : (
                            <div className="space-y-2">
                                {documents.map((doc, i) => (
                                    <div key={i} className="flex items-center justify-between p-4 rounded-lg border border-border">
                                        <div className="flex items-center gap-3">
                                            <FileText className="h-5 w-5 text-primary" />
                                            <div>
                                                <p className="text-sm font-medium">{doc.name}</p>
                                                <p className="text-xs text-muted-foreground">ID: {doc.document_id}</p>
                                            </div>
                                        </div>
                                        <span className="badge badge-secondary text-xs">
                                            {doc.chunk_count} chunks
                                        </span>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

                {/* Settings Sidebar */}
                <div className="card p-6 h-fit">
                    <h2 className="text-lg font-semibold mb-4">Chunking Settings</h2>

                    <div className="space-y-4">
                        <div>
                            <label className="label block mb-2">
                                Chunk Size: {chunkSize}
                            </label>
                            <input
                                type="range"
                                min="128"
                                max="2048"
                                step="64"
                                value={chunkSize}
                                onChange={(e) => setChunkSize(Number(e.target.value))}
                                className="w-full"
                            />
                            <div className="flex justify-between text-xs text-muted-foreground mt-1">
                                <span>128</span>
                                <span>2048</span>
                            </div>
                        </div>

                        <div>
                            <label className="label block mb-2">
                                Chunk Overlap: {chunkOverlap}
                            </label>
                            <input
                                type="range"
                                min="0"
                                max="512"
                                step="32"
                                value={chunkOverlap}
                                onChange={(e) => setChunkOverlap(Number(e.target.value))}
                                className="w-full"
                            />
                            <div className="flex justify-between text-xs text-muted-foreground mt-1">
                                <span>0</span>
                                <span>512</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
