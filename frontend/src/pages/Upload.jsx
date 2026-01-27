import React, { useState, useCallback } from 'react';
import axios from 'axios';
import { UploadCloud, File, CheckCircle, AlertCircle, X } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

const Upload = () => {
    const [dragActive, setDragActive] = useState(false);
    const [file, setFile] = useState(null);
    const [uploading, setUploading] = useState(false);
    const [status, setStatus] = useState(null); // success | error

    const handleDrag = (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.type === "dragenter" || e.type === "dragover") {
            setDragActive(true);
        } else if (e.type === "dragleave") {
            setDragActive(false);
        }
    };

    const handleDrop = (e) => {
        e.preventDefault();
        e.stopPropagation();
        setDragActive(false);
        if (e.dataTransfer.files && e.dataTransfer.files[0]) {
            setFile(e.dataTransfer.files[0]);
        }
    };

    const handleChange = (e) => {
        e.preventDefault();
        if (e.target.files && e.target.files[0]) {
            setFile(e.target.files[0]);
        }
    };

    const handleUpload = async () => {
        if (!file) return;
        setUploading(true);
        setStatus(null);

        const formData = new FormData();
        formData.append('file', file);
        formData.append('chunk_size', '512');
        formData.append('overlap', '50');

        try {
            await axios.post('http://localhost:3000/v1/upload', formData, {
                headers: { 'Content-Type': 'multipart/form-data' }
            });
            setStatus('success');
            setFile(null);
        } catch (err) {
            console.error(err);
            setStatus('error');
        } finally {
            setUploading(false);
        }
    };

    return (
        <div className="max-w-4xl mx-auto space-y-8">
            <div>
                <h2 className="text-2xl font-bold text-white">Ingest Documents</h2>
                <p className="text-muted">Upload PDF, DOCX, or TXT files for processing.</p>
            </div>

            <div
                className={`glass-panel border-2 border-dashed rounded-2xl p-12 text-center transition-all duration-300 relative
          ${dragActive ? 'border-primary bg-primary/5' : 'border-white/10 hover:border-primary/50'}
        `}
                onDragEnter={handleDrag}
                onDragLeave={handleDrag}
                onDragOver={handleDrag}
                onDrop={handleDrop}
            >
                <input
                    type="file"
                    className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                    onChange={handleChange}
                />

                <div className="flex flex-col items-center justify-center space-y-4 pointer-events-none">
                    <div className="w-16 h-16 bg-gradient-to-br from-primary to-secondary rounded-full flex items-center justify-center shadow-lg shadow-primary/20">
                        <UploadCloud size={32} className="text-white" />
                    </div>
                    <div>
                        <p className="text-lg font-medium text-white">
                            {file ? file.name : "Drag & drop your files here"}
                        </p>
                        <p className="text-sm text-muted mt-1">
                            Supports .pdf, .docx, .txt (Max 50MB)
                        </p>
                    </div>
                </div>
            </div>

            <AnimatePresence>
                {file && (
                    <motion.div
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -20 }}
                        className="glass-card p-4 rounded-xl flex items-center justify-between"
                    >
                        <div className="flex items-center space-x-4">
                            <div className="p-2 bg-white/5 rounded-lg">
                                <File size={24} className="text-secondary" />
                            </div>
                            <div>
                                <p className="text-white font-medium">{file.name}</p>
                                <p className="text-xs text-muted">{(file.size / 1024 / 1024).toFixed(2)} MB</p>
                            </div>
                        </div>
                        <div className="flex items-center space-x-3">
                            <button
                                onClick={() => setFile(null)}
                                className="p-2 hover:bg-white/10 rounded-lg text-muted hover:text-white"
                            >
                                <X size={20} />
                            </button>
                            <button
                                onClick={handleUpload}
                                disabled={uploading}
                                className={`px-4 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg font-medium transition-all
                            ${uploading ? 'opacity-50 cursor-not-allowed' : ''}
                        `}
                            >
                                {uploading ? 'Uploading...' : 'Start Processing'}
                            </button>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            {status === 'success' && (
                <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="p-4 bg-green-500/10 border border-green-500/20 text-green-400 rounded-xl flex items-center space-x-3">
                    <CheckCircle size={20} />
                    <span>File uploaded successfully! It is now being processed by the swarm.</span>
                </motion.div>
            )}

            {status === 'error' && (
                <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="p-4 bg-red-500/10 border border-red-500/20 text-red-400 rounded-xl flex items-center space-x-3">
                    <AlertCircle size={20} />
                    <span>Upload failed. Please check the file and try again.</span>
                </motion.div>
            )}
        </div>
    );
};

export default Upload;
