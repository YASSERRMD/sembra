import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { LayoutDashboard, UploadCloud, Search, Settings, BrainCircuit } from 'lucide-react';
import { motion } from 'framer-motion';
import clsx from 'clsx';

const NavItem = ({ to, icon: Icon, label, active }) => (
    <Link to={to}>
        <div className={clsx(
            "flex items-center space-x-3 px-4 py-3 rounded-lg transition-all duration-200 group",
            active ? "bg-primary/10 text-primary border border-primary/20 shadow-[0_0_15px_rgba(109,40,217,0.3)]"
                : "text-muted hover:text-white hover:bg-white/5"
        )}>
            <Icon size={20} className={clsx("transition-transform group-hover:scale-110", active && "text-primary")} />
            <span className="font-medium">{label}</span>
            {active && (
                <motion.div
                    layoutId="active-indicator"
                    className="absolute left-0 w-1 h-8 bg-primary rounded-r-full"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                />
            )}
        </div>
    </Link>
);

const Layout = ({ children }) => {
    const location = useLocation();

    const navItems = [
        { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
        { to: '/upload', icon: UploadCloud, label: 'Ingest' },
        { to: '/search', icon: Search, label: 'Search' },
        { to: '/settings', icon: Settings, label: 'Settings' },
    ];

    return (
        <div className="flex h-screen bg-background overflow-hidden text-sm">
            {/* Sidebar */}
            <div className="w-64 bg-surface border-r border-white/5 flex flex-col z-20 shadow-2xl">
                <div className="p-6 flex items-center space-x-3 border-b border-white/5">
                    <div className="p-2 bg-gradient-to-br from-primary to-accent rounded-lg shadow-lg">
                        <BrainCircuit size={24} className="text-white" />
                    </div>
                    <div>
                        <h1 className="text-lg font-bold tracking-tight text-white glow-text">SEMBRA</h1>
                        <p className="text-xs text-muted">Enterprise Intelligence</p>
                    </div>
                </div>

                <nav className="flex-1 p-4 space-y-2 mt-4 relative">
                    {navItems.map((item) => (
                        <NavItem
                            key={item.to}
                            {...item}
                            active={location.pathname === item.to}
                        />
                    ))}
                </nav>

                <div className="p-4 border-t border-white/5">
                    <div className="glass-panel p-3 rounded-lg flex items-center space-x-3">
                        <div className="w-8 h-8 rounded-full bg-gradient-to-r from-secondary to-primary flex items-center justify-center text-xs font-bold text-white">
                            AD
                        </div>
                        <div className="flex-1 min-w-0">
                            <p className="text-white font-medium truncate">Administrator</p>
                            <p className="text-xs text-muted truncate">admin@enterprise.com</p>
                        </div>
                    </div>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex flex-col overflow-hidden relative">
                {/* Background Gradients */}
                <div className="absolute top-0 left-0 w-full h-full pointer-events-none z-0">
                    <div className="absolute -top-[20%] -left-[10%] w-[50%] h-[50%] bg-primary/5 rounded-full blur-[120px]" />
                    <div className="absolute top-[20%] right-[10%] w-[30%] h-[30%] bg-secondary/5 rounded-full blur-[100px]" />
                </div>

                <main className="flex-1 overflow-y-auto p-8 z-10 scroll-smooth">
                    <motion.div
                        key={location.pathname}
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ duration: 0.3 }}
                    >
                        {children}
                    </motion.div>
                </main>
            </div>
        </div>
    );
};

export default Layout;
