import os
import json
from typing import Dict, Any

HTML_TEMPLATE = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ZafafWorld - Security Threat Model & Compliance Dashboard</title>
    <!-- Tailwind CSS -->
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            darkMode: 'class',
            theme: {
                extend: {
                    colors: {
                        brand: {
                            50: '#f0f3ff',
                            100: '#e0e7ff',
                            500: '#6366f1',
                            600: '#4f46e5',
                            700: '#4338ca',
                            900: '#312e81',
                        }
                    }
                }
            }
        }
    </script>
    <!-- Lucide Icons -->
    <link href="https://cdn.jsdelivr.net/npm/lucide-static@0.321.0/font/lucide.min.css" rel="stylesheet">
    <!-- React & ReactDOM -->
    <script src="https://unpkg.com/react@18/umd/react.production.min.js" crossorigin></script>
    <script src="https://unpkg.com/react-dom@18/umd/react-dom.production.min.js" crossorigin></script>
    <!-- Babel for JSX compilation -->
    <script src="https://unpkg.com/@babel/standalone/babel.min.js"></script>
    <style>
        body {
            font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
        }
        .custom-scrollbar::-webkit-scrollbar {
            width: 6px;
            height: 6px;
        }
        .custom-scrollbar::-webkit-scrollbar-track {
            background: #1e1e24;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb {
            background: #4b5563;
            border-radius: 3px;
        }
    </style>
</head>
<body class="bg-gray-950 text-gray-100 min-h-screen flex flex-col font-sans selection:bg-brand-500 selection:text-white">

    <div id="root"></div>

    <script type="text/javascript">
        // Injecting the raw JSON payload
        const DATA = __DATA_PLACEHOLDER__;
    </script>

    <script type="text/babel">
        const { useState, useMemo } = React;

        function App() {
            const [activeTab, setActiveTab] = useState('dashboard');
            const [searchQuery, setSearchQuery] = useState('');
            const [severityFilter, setSeverityFilter] = useState('All');
            const [frameworkFilter, setFrameworkFilter] = useState('All');
            const [selectedComponent, setSelectedComponent] = useState(null);
            
            // Feedback form states
            const [feedbackTarget, setFeedbackTarget] = useState('');
            const [feedbackField, setFeedbackField] = useState('severity');
            const [feedbackOriginalVal, setFeedbackOriginalVal] = useState('');
            const [feedbackCorrectedVal, setFeedbackCorrectedVal] = useState('');
            const [feedbackReason, setFeedbackReason] = useState('');
            const [exportedFeedback, setExportedFeedback] = useState(null);

            // Compute statistics
            const stats = useMemo(() => {
                const totalThreats = DATA.threats.length;
                const totalIssues = DATA.operational_issues.length;
                const totalGaps = DATA.regulatory_gaps.length;

                const severityCounts = { Critical: 0, High: 0, Medium: 0, Low: 0 };
                DATA.threats.forEach(t => {
                    if (severityCounts[t.severity] !== undefined) {
                        severityCounts[t.severity]++;
                    }
                });

                const frameworkCounts = {};
                DATA.threats.forEach(t => {
                    frameworkCounts[t.framework] = (frameworkCounts[t.framework] || 0) + 1;
                });

                const componentThreats = {};
                DATA.threats.forEach(t => {
                    componentThreats[t.affected_component] = (componentThreats[t.affected_component] || 0) + 1;
                });

                return {
                    totalThreats,
                    totalIssues,
                    totalGaps,
                    severityCounts,
                    frameworkCounts,
                    componentThreats
                };
            }, []);

            // Filter threats
            const filteredThreats = useMemo(() => {
                return DATA.threats.filter(t => {
                    const matchesSearch = t.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                        t.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
                        t.affected_component.toLowerCase().includes(searchQuery.toLowerCase());
                    const matchesSeverity = severityFilter === 'All' || t.severity === severityFilter;
                    const matchesFramework = frameworkFilter === 'All' || t.framework === frameworkFilter;
                    const matchesComponent = !selectedComponent || t.affected_component === selectedComponent;
                    return matchesSearch && matchesSeverity && matchesFramework && matchesComponent;
                });
            }, [searchQuery, severityFilter, frameworkFilter, selectedComponent]);

            // Filter operational issues
            const filteredIssues = useMemo(() => {
                return DATA.operational_issues.filter(i => {
                    return i.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                        i.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
                        i.component.toLowerCase().includes(searchQuery.toLowerCase());
                });
            }, [searchQuery]);

            // Filter regulatory gaps
            const filteredGaps = useMemo(() => {
                return DATA.regulatory_gaps.filter(g => {
                    return g.regulation.toLowerCase().includes(searchQuery.toLowerCase()) ||
                        g.description.toLowerCase().includes(searchQuery.toLowerCase());
                });
            }, [searchQuery]);

            const generateFeedbackJSON = (e) => {
                e.preventDefault();
                const newFeedback = {
                    threat_title: feedbackTarget,
                    field: feedbackField,
                    original_value: feedbackOriginalVal,
                    corrected_value: feedbackCorrectedVal,
                    reason: feedbackReason
                };
                setExportedFeedback(JSON.stringify(newFeedback, null, 2));
            };

            return (
                <div className="flex flex-col min-h-screen">
                    {/* Header */}
                    <header className="border-b border-gray-800 bg-gray-900/50 backdrop-blur sticky top-0 z-50 px-6 py-4 flex flex-col md:flex-row md:items-center md:justify-between gap-4">
                        <div className="flex items-center gap-3">
                            <div className="bg-brand-600 p-2.5 rounded-xl shadow-lg shadow-brand-500/20">
                                <i className="lucide-shield-check text-2xl text-white"></i>
                            </div>
                            <div>
                                <h1 className="text-xl font-bold text-white tracking-wide">ZafafWorld</h1>
                                <p className="text-xs text-gray-400">Security Threat Model & Compliance Analysis</p>
                            </div>
                        </div>
                        <div className="flex items-center gap-2 overflow-x-auto pb-1 md:pb-0">
                            {['dashboard', 'threats', 'issues', 'gaps', 'topology', 'feedback'].map(tab => (
                                <button
                                    key={tab}
                                    onClick={() => setActiveTab(tab)}
                                    className={`px-4 py-2 rounded-lg text-sm font-medium transition-all ${
                                        activeTab === tab
                                            ? 'bg-brand-600 text-white shadow-md shadow-brand-500/10'
                                            : 'text-gray-400 hover:text-white hover:bg-gray-800/50'
                                    }`}
                                >
                                    {tab.charAt(0).toUpperCase() + tab.slice(1)}
                                </button>
                            ))}
                        </div>
                    </header>

                    {/* Main Content */}
                    <main className="flex-1 p-6 max-w-7xl w-full mx-auto">
                        
                        {/* DASHBOARD TAB */}
                        {activeTab === 'dashboard' && (
                            <div className="space-y-8">
                                {/* Stats Cards */}
                                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                                    <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 relative overflow-hidden group">
                                        <div className="absolute right-0 top-0 w-32 h-32 bg-red-500/10 rounded-full blur-3xl group-hover:bg-red-500/20 transition-all duration-500"></div>
                                        <div className="flex items-center justify-between mb-4">
                                            <span className="text-sm font-medium text-gray-400">Security Threats</span>
                                            <span className="p-2 bg-red-500/10 text-red-400 rounded-lg">
                                                <i className="lucide-alert-triangle"></i>
                                            </span>
                                        </div>
                                        <div className="text-3xl font-extrabold text-white">{stats.totalThreats}</div>
                                        <p className="text-xs text-gray-500 mt-2">Active security threats mapped via STRIDE/PASTA/OWASP</p>
                                    </div>
                                    <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 relative overflow-hidden group">
                                        <div className="absolute right-0 top-0 w-32 h-32 bg-yellow-500/10 rounded-full blur-3xl group-hover:bg-yellow-500/20 transition-all duration-500"></div>
                                        <div className="flex items-center justify-between mb-4">
                                            <span className="text-sm font-medium text-gray-400">Operational Issues</span>
                                            <span className="p-2 bg-yellow-500/10 text-yellow-400 rounded-lg">
                                                <i className="lucide-activity"></i>
                                            </span>
                                        </div>
                                        <div className="text-3xl font-extrabold text-white">{stats.totalIssues}</div>
                                        <p className="text-xs text-gray-500 mt-2">Risks related to architecture & system configuration</p>
                                    </div>
                                    <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 relative overflow-hidden group">
                                        <div className="absolute right-0 top-0 w-32 h-32 bg-blue-500/10 rounded-full blur-3xl group-hover:bg-blue-500/20 transition-all duration-500"></div>
                                        <div className="flex items-center justify-between mb-4">
                                            <span className="text-sm font-medium text-gray-400">Compliance Gaps</span>
                                            <span className="p-2 bg-blue-500/10 text-blue-400 rounded-lg">
                                                <i className="lucide-file-text"></i>
                                            </span>
                                        </div>
                                        <div className="text-3xl font-extrabold text-white">{stats.totalGaps}</div>
                                        <p className="text-xs text-gray-500 mt-2">Regulatory deviations (GDPR, HIPAA, etc.)</p>
                                    </div>
                                </div>

                                {/* Graphs section */}
                                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                    {/* Severity breakdown */}
                                    <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6">
                                        <h3 className="text-lg font-bold text-white mb-6">Threat Severity Breakdown</h3>
                                        <div className="flex flex-col sm:flex-row items-center gap-8">
                                            <div className="relative w-40 h-40 flex items-center justify-center">
                                                <svg viewBox="0 0 100 100" className="w-full h-full transform -rotate-90">
                                                    {/* Outer rings/arcs using SVG calculated offset depending on severity ratios */}
                                                    {(() => {
                                                        const total = stats.totalThreats || 1;
                                                        const criticalPct = (stats.severityCounts.Critical / total) * 100;
                                                        const highPct = (stats.severityCounts.High / total) * 100;
                                                        const mediumPct = (stats.severityCounts.Medium / total) * 100;
                                                        const lowPct = (stats.severityCounts.Low / total) * 100;

                                                        let currentOffset = 0;
                                                        return (
                                                            <>
                                                                {/* Base ring */}
                                                                <circle cx="50" cy="50" r="40" fill="transparent" stroke="#1f2937" strokeWidth="12" />
                                                                {/* Critical (Red) */}
                                                                {criticalPct > 0 && (
                                                                    <circle cx="50" cy="50" r="40" fill="transparent" stroke="#ef4444" strokeWidth="12"
                                                                        strokeDasharray={`${criticalPct * 2.51} 251.2`} strokeDashoffset={currentOffset}
                                                                    />
                                                                )}
                                                                {/* High (Orange) */}
                                                                {(() => { currentOffset -= (criticalPct * 2.51); return null; })()}
                                                                {highPct > 0 && (
                                                                    <circle cx="50" cy="50" r="40" fill="transparent" stroke="#f97316" strokeWidth="12"
                                                                        strokeDasharray={`${highPct * 2.51} 251.2`} strokeDashoffset={currentOffset}
                                                                    />
                                                                )}
                                                                {/* Medium (Yellow) */}
                                                                {(() => { currentOffset -= (highPct * 2.51); return null; })()}
                                                                {mediumPct > 0 && (
                                                                    <circle cx="50" cy="50" r="40" fill="transparent" stroke="#eab308" strokeWidth="12"
                                                                        strokeDasharray={`${mediumPct * 2.51} 251.2`} strokeDashoffset={currentOffset}
                                                                    />
                                                                )}
                                                                {/* Low (Blue) */}
                                                                {(() => { currentOffset -= (mediumPct * 2.51); return null; })()}
                                                                {lowPct > 0 && (
                                                                    <circle cx="50" cy="50" r="40" fill="transparent" stroke="#3b82f6" strokeWidth="12"
                                                                        strokeDasharray={`${lowPct * 2.51} 251.2`} strokeDashoffset={currentOffset}
                                                                    />
                                                                )}
                                                            </>
                                                        );
                                                    })()}
                                                </svg>
                                                <div className="absolute flex flex-col items-center">
                                                    <span className="text-3xl font-black text-white">{stats.totalThreats}</span>
                                                    <span className="text-xs text-gray-500 uppercase tracking-wider">Total</span>
                                                </div>
                                            </div>

                                            {/* Legend */}
                                            <div className="flex-1 space-y-4 w-full">
                                                {[
                                                    { label: 'Critical', val: stats.severityCounts.Critical, color: 'bg-red-500', text: 'text-red-400' },
                                                    { label: 'High', val: stats.severityCounts.High, color: 'bg-orange-500', text: 'text-orange-400' },
                                                    { label: 'Medium', val: stats.severityCounts.Medium, color: 'bg-yellow-500', text: 'text-yellow-400' },
                                                    { label: 'Low', val: stats.severityCounts.Low, color: 'bg-blue-500', text: 'text-blue-400' }
                                                ].map(item => (
                                                    <div key={item.label} className="flex items-center justify-between">
                                                        <div className="flex items-center gap-2">
                                                            <div className={`w-3 h-3 rounded-full ${item.color}`} />
                                                            <span className="font-medium text-sm text-gray-300">{item.label}</span>
                                                        </div>
                                                        <span className={`text-sm font-bold ${item.text}`}>{item.val}</span>
                                                    </div>
                                                ))}
                                            </div>
                                        </div>
                                    </div>

                                    {/* Framework Distribution */}
                                    <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6">
                                        <h3 className="text-lg font-bold text-white mb-6">Threats per Framework</h3>
                                        <div className="space-y-4">
                                            {Object.entries(stats.frameworkCounts).map(([fw, count]) => {
                                                const maxCount = Math.max(...Object.values(stats.frameworkCounts));
                                                const pct = (count / (maxCount || 1)) * 100;
                                                return (
                                                    <div key={fw}>
                                                        <div className="flex justify-between text-sm font-medium text-gray-300 mb-1">
                                                            <span>{fw}</span>
                                                            <span className="text-brand-400">{count}</span>
                                                        </div>
                                                        <div className="w-full bg-gray-800 h-2.5 rounded-full overflow-hidden">
                                                            <div className="bg-brand-500 h-full rounded-full" style={{ width: `${pct}%` }}></div>
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                </div>

                                {/* Metadata and Run info */}
                                <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6">
                                    <h3 className="text-md font-bold text-white mb-4">Run Information</h3>
                                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 text-sm">
                                        <div>
                                            <span className="text-gray-500 block">Target Location</span>
                                            <span className="font-mono text-gray-300 break-all">{DATA.metadata.target_path || 'Direct Codebase'}</span>
                                        </div>
                                        <div>
                                            <span className="text-gray-500 block">Analysis Time</span>
                                            <span className="text-gray-300">{new Date().toLocaleString()}</span>
                                        </div>
                                        <div>
                                            <span className="text-gray-500 block">Processed Chunks</span>
                                            <span className="text-gray-300">{DATA.metadata.total_chunks || 1} chunks parsed</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* THREATS TAB */}
                        {activeTab === 'threats' && (
                            <div className="space-y-6">
                                {/* Search and Filter Toolbar */}
                                <div className="flex flex-col md:flex-row gap-4 justify-between items-stretch">
                                    <div className="relative flex-1">
                                        <i className="lucide-search absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"></i>
                                        <input
                                            type="text"
                                            value={searchQuery}
                                            onChange={(e) => setSearchQuery(e.target.value)}
                                            placeholder="Search threats..."
                                            className="w-full pl-10 pr-4 py-2.5 bg-gray-900 border border-gray-800 rounded-xl focus:border-brand-500 focus:ring-1 focus:ring-brand-500 outline-none text-white text-sm"
                                        />
                                    </div>
                                    <div className="flex gap-4 overflow-x-auto">
                                        <select
                                            value={severityFilter}
                                            onChange={(e) => setSeverityFilter(e.target.value)}
                                            className="px-4 py-2.5 bg-gray-900 border border-gray-800 rounded-xl text-sm focus:border-brand-500 outline-none text-gray-300"
                                        >
                                            <option value="All">All Severities</option>
                                            <option value="Critical">Critical</option>
                                            <option value="High">High</option>
                                            <option value="Medium">Medium</option>
                                            <option value="Low">Low</option>
                                        </select>
                                        <select
                                            value={frameworkFilter}
                                            onChange={(e) => setFrameworkFilter(e.target.value)}
                                            className="px-4 py-2.5 bg-gray-900 border border-gray-800 rounded-xl text-sm focus:border-brand-500 outline-none text-gray-300"
                                        >
                                            <option value="All">All Frameworks</option>
                                            <option value="STRIDE">STRIDE</option>
                                            <option value="PASTA">PASTA</option>
                                            <option value="LINDDUN">LINDDUN</option>
                                            <option value="OWASP">OWASP</option>
                                        </select>
                                        {selectedComponent && (
                                            <button
                                                onClick={() => setSelectedComponent(null)}
                                                className="px-4 py-2.5 bg-brand-900/30 border border-brand-800 text-brand-400 rounded-xl text-sm flex items-center gap-2 hover:bg-brand-900/50"
                                            >
                                                <span>Component: {selectedComponent}</span>
                                                <i className="lucide-x text-xs"></i>
                                            </button>
                                        )}
                                    </div>
                                </div>

                                {/* List of Threats */}
                                <div className="space-y-4">
                                    {filteredThreats.length === 0 ? (
                                        <div className="text-center py-12 text-gray-500 border border-dashed border-gray-800 rounded-2xl">No threats match the current filters.</div>
                                    ) : (
                                        filteredThreats.map(threat => {
                                            const sevColors = {
                                                Critical: 'bg-red-500/10 text-red-500 border-red-500/20',
                                                High: 'bg-orange-500/10 text-orange-500 border-orange-500/20',
                                                Medium: 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20',
                                                Low: 'bg-blue-500/10 text-blue-500 border-blue-500/20'
                                            };
                                            return (
                                                <div key={threat.id} className="bg-gray-900 border border-gray-800 rounded-2xl p-6 transition-all hover:border-gray-700">
                                                    <div className="flex flex-wrap items-center justify-between gap-3 mb-4">
                                                        <div className="flex items-center gap-3">
                                                            <span className="font-bold text-brand-400 font-mono text-sm">{threat.id}</span>
                                                            <h4 className="text-lg font-bold text-white">{threat.title}</h4>
                                                        </div>
                                                        <div className="flex items-center gap-2">
                                                            <span className={`px-2.5 py-1 text-xs font-semibold rounded-full border ${sevColors[threat.severity] || 'bg-gray-500/10 text-gray-400'}`}>
                                                                {threat.severity} (Score: {threat.score})
                                                            </span>
                                                            <span className="px-2.5 py-1 text-xs font-semibold rounded-full border border-gray-800 bg-gray-800/30 text-gray-300">
                                                                {threat.framework} • {threat.category}
                                                            </span>
                                                        </div>
                                                    </div>

                                                    <p className="text-gray-300 text-sm mb-4 leading-relaxed">{threat.description}</p>

                                                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs border-t border-gray-800 pt-4 mt-4">
                                                        <div>
                                                            <span className="text-gray-500 uppercase tracking-wider block font-bold mb-1">Affected Component</span>
                                                            <span className="text-gray-200 font-semibold">{threat.affected_component}</span>
                                                        </div>
                                                        <div>
                                                            <span className="text-gray-500 uppercase tracking-wider block font-bold mb-1">Remediation</span>
                                                            <span className="text-emerald-400 font-semibold">{threat.remediation}</span>
                                                        </div>
                                                        {threat.cwe_id && (
                                                            <div>
                                                                <span className="text-gray-500 uppercase tracking-wider block font-bold mb-1">CWE ID</span>
                                                                <a href={`https://cwe.mitre.org/data/definitions/${threat.cwe_id.replace('CWE-', '')}.html`} target="_blank" rel="noreferrer" className="text-blue-400 hover:underline font-mono">
                                                                    {threat.cwe_id}
                                                                </a>
                                                            </div>
                                                        )}
                                                        {threat.mitre_attack_id && (
                                                            <div>
                                                                <span className="text-gray-500 uppercase tracking-wider block font-bold mb-1">MITRE ATT&CK Technique</span>
                                                                <a href={`https://attack.mitre.org/techniques/${threat.mitre_attack_id}`} target="_blank" rel="noreferrer" className="text-blue-400 hover:underline font-mono">
                                                                    {threat.mitre_attack_id}
                                                                </a>
                                                            </div>
                                                        )}
                                                    </div>

                                                    <div className="bg-gray-950 rounded-xl p-4 mt-4 border border-gray-800">
                                                        <span className="text-xs text-gray-500 font-bold block mb-1">Justification Analysis:</span>
                                                        <p className="text-xs text-gray-400 italic font-mono leading-relaxed">{threat.justification}</p>
                                                    </div>
                                                </div>
                                            );
                                        })
                                    )}
                                </div>
                            </div>
                        )}

                        {/* ISSUES TAB */}
                        {activeTab === 'issues' && (
                            <div className="space-y-6">
                                <div className="relative">
                                    <i className="lucide-search absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"></i>
                                    <input
                                        type="text"
                                        value={searchQuery}
                                        onChange={(e) => setSearchQuery(e.target.value)}
                                        placeholder="Search operational issues..."
                                        className="w-full pl-10 pr-4 py-2.5 bg-gray-900 border border-gray-800 rounded-xl focus:border-brand-500 focus:ring-1 focus:ring-brand-500 outline-none text-white text-sm"
                                    />
                                </div>

                                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                                    {filteredIssues.length === 0 ? (
                                        <div className="col-span-2 text-center py-12 text-gray-500 border border-dashed border-gray-800 rounded-2xl">No operational issues found.</div>
                                    ) : (
                                        filteredIssues.map(issue => {
                                            const issueColors = {
                                                High: 'border-red-500/30 bg-red-500/5',
                                                Medium: 'border-yellow-500/30 bg-yellow-500/5',
                                                Low: 'border-blue-500/30 bg-blue-500/5'
                                            };
                                            const badgeColors = {
                                                High: 'text-red-400 bg-red-950/55 border-red-500/20',
                                                Medium: 'text-yellow-400 bg-yellow-950/55 border-yellow-500/20',
                                                Low: 'text-blue-400 bg-blue-950/55 border-blue-500/20'
                                            };
                                            return (
                                                <div key={issue.id} className={`border rounded-2xl p-6 flex flex-col justify-between ${issueColors[issue.severity]}`}>
                                                    <div>
                                                        <div className="flex justify-between items-center mb-3">
                                                            <span className="font-bold text-brand-400 font-mono text-sm">{issue.id}</span>
                                                            <span className={`px-2.5 py-0.5 text-xs font-semibold rounded-full border ${badgeColors[issue.severity]}`}>
                                                                {issue.severity}
                                                            </span>
                                                        </div>
                                                        <h4 className="text-lg font-bold text-white mb-2">{issue.title}</h4>
                                                        <p className="text-gray-300 text-sm mb-4">{issue.description}</p>
                                                        <div className="text-xs space-y-2 mt-4 pt-4 border-t border-gray-800/50">
                                                            <div>
                                                                <span className="text-gray-500 font-bold block mb-0.5">Component</span>
                                                                <span className="text-gray-300 font-mono">{issue.component}</span>
                                                            </div>
                                                            <div>
                                                                <span className="text-gray-500 font-bold block mb-0.5">Operational Impact</span>
                                                                <span className="text-gray-300">{issue.impact}</span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div className="mt-6 pt-4 border-t border-gray-800/50">
                                                        <span className="text-xs text-gray-500 font-bold block mb-1">Recommendation</span>
                                                        <span className="text-emerald-400 text-sm font-semibold">{issue.recommendation}</span>
                                                    </div>
                                                </div>
                                            );
                                        })
                                    )}
                                </div>
                            </div>
                        )}

                        {/* REGULATORY GAPS TAB */}
                        {activeTab === 'gaps' && (
                            <div className="space-y-6">
                                <div className="relative">
                                    <i className="lucide-search absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"></i>
                                    <input
                                        type="text"
                                        value={searchQuery}
                                        onChange={(e) => setSearchQuery(e.target.value)}
                                        placeholder="Search compliance/regulatory requirements..."
                                        className="w-full pl-10 pr-4 py-2.5 bg-gray-900 border border-gray-800 rounded-xl focus:border-brand-500 focus:ring-1 focus:ring-brand-500 outline-none text-white text-sm"
                                    />
                                </div>

                                <div className="space-y-4">
                                    {filteredGaps.length === 0 ? (
                                        <div className="text-center py-12 text-gray-500 border border-dashed border-gray-800 rounded-2xl">No compliance gaps found.</div>
                                    ) : (
                                        filteredGaps.map(gap => (
                                            <div key={gap.id} className="bg-gray-900 border border-gray-800 rounded-2xl p-6 transition-all hover:border-gray-700">
                                                <div className="flex flex-wrap items-center justify-between gap-3 mb-4">
                                                    <div className="flex items-center gap-3">
                                                        <span className="font-bold text-brand-400 font-mono text-sm">{gap.id}</span>
                                                        <h4 className="text-lg font-bold text-white">{gap.regulation} - Clause: {gap.clause}</h4>
                                                    </div>
                                                    <span className="px-2.5 py-1 text-xs font-semibold rounded-full border border-red-500/20 bg-red-950/20 text-red-400">
                                                        Compliance Gap
                                                    </span>
                                                </div>
                                                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-4">
                                                    <div>
                                                        <h5 className="text-sm font-semibold text-gray-400 mb-1">Requirement Description</h5>
                                                        <p className="text-sm text-gray-300 leading-relaxed">{gap.description}</p>
                                                    </div>
                                                    <div>
                                                        <h5 className="text-sm font-semibold text-gray-400 mb-1">Gap Analysis (Findings)</h5>
                                                        <p className="text-sm text-gray-300 leading-relaxed">{gap.gap_analysis}</p>
                                                    </div>
                                                </div>
                                                <div className="mt-4 pt-4 border-t border-gray-800 text-xs">
                                                    <span className="text-gray-500 uppercase tracking-wider block font-bold mb-1">Remediation Action</span>
                                                    <span className="text-emerald-400 font-semibold text-sm">{gap.remediation}</span>
                                                </div>
                                            </div>
                                        ))
                                    )}
                                </div>
                            </div>
                        )}

                        {/* TOPOLOGY TAB */}
                        {activeTab === 'topology' && (
                            <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6">
                                <h3 className="text-lg font-bold text-white mb-2">Interactive System Architecture & Threat Topology</h3>
                                <p className="text-sm text-gray-400 mb-6">Click on a component in the graph to filter the associated threats listed below.</p>
                                
                                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                                    {/* SVG Interactive Topology Diagram */}
                                    <div className="lg:col-span-2 bg-gray-950 rounded-xl border border-gray-800 h-96 relative flex items-center justify-center overflow-hidden">
                                        <svg viewBox="0 0 600 400" className="w-full h-full">
                                            {/* Grid background pattern */}
                                            <defs>
                                                <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
                                                    <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#ffffff" strokeOpacity="0.03" strokeWidth="1"/>
                                                </pattern>
                                            </defs>
                                            <rect width="100%" height="100%" fill="url(#grid)" />

                                            {/* Connections / Edges */}
                                            {Object.keys(stats.componentThreats).map((comp, idx, arr) => {
                                                const spacing = 450 / Math.max(arr.length - 1, 1);
                                                const xPos = 300;
                                                const yPos = 50 + idx * spacing;
                                                return (
                                                    <g key={`edge-${comp}`}>
                                                        <line x1="80" y1="200" x2={xPos} y2={yPos} stroke="#4f46e5" strokeWidth="2" strokeOpacity="0.5" strokeDasharray="5,5" />
                                                        <line x1={xPos} y1={yPos} x2="520" y2="200" stroke="#f43f5e" strokeWidth="2" strokeOpacity="0.5" />
                                                    </g>
                                                );
                                            })}

                                            {/* Gateways / Central Nodes */}
                                            <circle cx="80" cy="200" r="25" fill="#312e81" stroke="#6366f1" strokeWidth="3" />
                                            <text x="80" y="205" fill="#fff" fontSize="10" fontWeight="bold" textAnchor="middle">API GW</text>

                                            <circle cx="520" cy="200" r="25" fill="#4c0519" stroke="#f43f5e" strokeWidth="3" />
                                            <text x="520" y="205" fill="#fff" fontSize="10" fontWeight="bold" textAnchor="middle">DB/S3</text>

                                            {/* Component Nodes */}
                                            {Object.keys(stats.componentThreats).map((comp, idx, arr) => {
                                                const spacing = 320 / Math.max(arr.length - 1, 1);
                                                const xPos = 300;
                                                const yPos = 40 + idx * spacing;
                                                const isSelected = selectedComponent === comp;
                                                const numThreats = stats.componentThreats[comp];

                                                return (
                                                    <g 
                                                        key={comp} 
                                                        className="cursor-pointer transition-all duration-300"
                                                        onClick={() => setSelectedComponent(isSelected ? null : comp)}
                                                    >
                                                        <rect 
                                                            x={xPos - 75} 
                                                            y={yPos - 20} 
                                                            width="150" 
                                                            height="40" 
                                                            rx="8" 
                                                            fill={isSelected ? '#4f46e5' : '#1f2937'} 
                                                            stroke={isSelected ? '#818cf8' : '#374151'} 
                                                            strokeWidth="2" 
                                                        />
                                                        <text 
                                                            x={xPos} 
                                                            y={yPos + 4} 
                                                            fill="#ffffff" 
                                                            fontSize="10" 
                                                            fontWeight="bold" 
                                                            textAnchor="middle"
                                                        >
                                                            {comp.length > 20 ? comp.slice(0, 17) + '...' : comp}
                                                        </text>
                                                        <circle cx={xPos + 75} cy={yPos - 15} r="10" fill="#ef4444" />
                                                        <text x={xPos + 75} y={yPos - 12} fill="#fff" fontSize="8" fontWeight="bold" textAnchor="middle">{numThreats}</text>
                                                    </g>
                                                );
                                            })}
                                        </svg>
                                    </div>

                                    {/* Sidebar filter feedback */}
                                    <div className="bg-gray-950 p-6 rounded-xl border border-gray-800">
                                        <h4 className="text-md font-bold text-white mb-4">Topology Details</h4>
                                        <div className="space-y-4">
                                            <div>
                                                <span className="text-gray-500 text-xs uppercase block">Active Filter</span>
                                                <span className="text-sm font-semibold text-white">
                                                    {selectedComponent ? `Showing threats in: ${selectedComponent}` : 'All Components'}
                                                </span>
                                            </div>
                                            <div>
                                                <span className="text-gray-500 text-xs uppercase block">Components count</span>
                                                <span className="text-sm font-semibold text-white">
                                                    {Object.keys(stats.componentThreats).length} registered components
                                                </span>
                                            </div>
                                            {selectedComponent && (
                                                <button
                                                    onClick={() => setSelectedComponent(null)}
                                                    className="w-full py-2 bg-gray-800 hover:bg-gray-700 text-white rounded-lg text-sm transition"
                                                >
                                                    Clear Filter
                                                </button>
                                            )}
                                        </div>
                                    </div>
                                </div>

                                <div className="mt-8">
                                    <h4 className="text-md font-bold text-white mb-4">Component Threats</h4>
                                    <div className="space-y-3">
                                        {filteredThreats.map(threat => (
                                            <div key={`topo-${threat.id}`} className="p-4 bg-gray-950 rounded-xl border border-gray-800 flex justify-between items-center">
                                                <div>
                                                    <span className="text-xs text-brand-400 font-mono block">{threat.id}</span>
                                                    <span className="font-bold text-sm text-gray-200">{threat.title}</span>
                                                </div>
                                                <div className="flex gap-2">
                                                    <span className="px-2 py-0.5 text-xs rounded border border-gray-800 text-gray-400">{threat.affected_component}</span>
                                                    <span className="px-2 py-0.5 text-xs bg-red-950/40 text-red-400 rounded border border-red-900/30">{threat.severity}</span>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* FEEDBACK TAB */}
                        {activeTab === 'feedback' && (
                            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                                <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6">
                                    <h3 className="text-lg font-bold text-white mb-4">Feedback Studio</h3>
                                    <p className="text-sm text-gray-400 mb-6">
                                        Submit a corrective override to fine-tune future model analysis runs. 
                                        Fill in the form to generate the required JSON structure for your <code>human_feedback.json</code> file.
                                    </p>

                                    <form onSubmit={generateFeedbackJSON} className="space-y-4">
                                        <div>
                                            <label className="block text-xs font-bold uppercase text-gray-400 mb-2">Select Target Threat</label>
                                            <select
                                                value={feedbackTarget}
                                                onChange={(e) => setFeedbackTarget(e.target.value)}
                                                required
                                                className="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl text-sm text-white focus:border-brand-500 outline-none"
                                            >
                                                <option value="">-- Choose a Threat --</option>
                                                {DATA.threats.map(t => (
                                                    <option key={t.id} value={t.title}>{t.title} ({t.id})</option>
                                                ))}
                                            </select>
                                        </div>

                                        <div className="grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="block text-xs font-bold uppercase text-gray-400 mb-2">Target Attribute</label>
                                                <select
                                                    value={feedbackField}
                                                    onChange={(e) => setFeedbackField(e.target.value)}
                                                    className="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl text-sm text-white focus:border-brand-500 outline-none"
                                                >
                                                    <option value="severity">Severity</option>
                                                    <option value="score">Score</option>
                                                    <option value="remediation">Remediation</option>
                                                </select>
                                            </div>
                                            <div>
                                                <label className="block text-xs font-bold uppercase text-gray-400 mb-2">Original Value</label>
                                                <input
                                                    type="text"
                                                    value={feedbackOriginalVal}
                                                    onChange={(e) => setFeedbackOriginalVal(e.target.value)}
                                                    required
                                                    placeholder="e.g. Critical"
                                                    className="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl text-sm text-white focus:border-brand-500 outline-none"
                                                />
                                            </div>
                                        </div>

                                        <div>
                                            <label className="block text-xs font-bold uppercase text-gray-400 mb-2">Corrected Value</label>
                                            <input
                                                type="text"
                                                value={feedbackCorrectedVal}
                                                onChange={(e) => setFeedbackCorrectedVal(e.target.value)}
                                                required
                                                placeholder="e.g. High"
                                                className="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl text-sm text-white focus:border-brand-500 outline-none"
                                            />
                                        </div>

                                        <div>
                                            <label className="block text-xs font-bold uppercase text-gray-400 mb-2">Justification / Reason</label>
                                            <textarea
                                                rows="3"
                                                value={feedbackReason}
                                                onChange={(e) => setFeedbackReason(e.target.value)}
                                                required
                                                placeholder="Explain why this value should change (e.g. mitigated by network architecture)..."
                                                className="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl text-sm text-white focus:border-brand-500 outline-none"
                                            ></textarea>
                                        </div>

                                        <button
                                            type="submit"
                                            className="w-full py-3 bg-brand-600 hover:bg-brand-700 active:bg-brand-800 text-white rounded-xl font-semibold text-sm transition"
                                        >
                                            Generate Override
                                        </button>
                                    </form>
                                </div>

                                <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 flex flex-col justify-between">
                                    <div>
                                        <h3 className="text-lg font-bold text-white mb-4">Generated Override Payload</h3>
                                        <p className="text-sm text-gray-400 mb-4">
                                            Append this JSON snippet into your local <code>human_feedback.json</code> file to train the model dynamically next time.
                                        </p>
                                        
                                        {exportedFeedback ? (
                                            <pre className="p-4 bg-gray-950 border border-gray-800 rounded-xl overflow-x-auto text-xs text-brand-400 font-mono select-all">
                                                {exportedFeedback}
                                            </pre>
                                        ) : (
                                            <div className="flex flex-col items-center justify-center py-20 border border-dashed border-gray-800 rounded-xl text-gray-500 text-sm">
                                                <span>Fill out the form to generate JSON</span>
                                            </div>
                                        )}
                                    </div>
                                    <div className="mt-6 pt-4 border-t border-gray-800/50">
                                        <p className="text-xs text-gray-500">
                                            * Note: In production, you would automate saving of this payload back to your file storage system.
                                        </p>
                                    </div>
                                </div>
                            </div>
                        )}
                    </main>

                    {/* Footer */}
                    <footer className="border-t border-gray-900 bg-gray-950 py-6 px-6 text-center text-xs text-gray-600">
                        &copy; 2026 ZafafWorld. Developed in Partnership with DeepMind.
                    </footer>
                </div>
            );
        }

        const root = ReactDOM.createRoot(document.getElementById('root'));
        root.render(<App />);
    </script>
</body>
</html>
"""

def generate_report(data: Dict[str, Any], output_path: str):
    """Renders the HTML report with embedded JSON data."""
    # Convert data dictionary to JSON string
    serialized_data = json.dumps(data, indent=4)
    # Inject into the HTML template
    html_content = HTML_TEMPLATE.replace("__DATA_PLACEHOLDER__", serialized_data)
    
    # Write to target destination
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html_content)
    print(f"[+] Interactive HTML report generated at: {output_path}")
