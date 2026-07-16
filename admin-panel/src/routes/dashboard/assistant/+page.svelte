<script lang="ts">
    import { enhance } from '$app/forms';
    import { 
        Sparkles, ShieldAlert, CheckCircle2, Clock, Mail, 
        MessageSquare, Calendar, Check, RotateCcw 
    } from 'lucide-svelte';

    let { data } = $props();

    // Local reactive state using Svelte 5 runes
    let inquiries = $derived(data.inquiries || []);
    let submittingIds = $state<string[]>([]);
    let errorMessage = $state('');

    // Compute stats reactively
    const totalCount = $derived(inquiries.length);
    const pendingCount = $derived(inquiries.filter((i: any) => i.status === 'pending').length);
    const resolvedCount = $derived(inquiries.filter((i: any) => i.status === 'resolved').length);

    function formatDate(dateStr: string): string {
        try {
            const date = new Date(dateStr);
            return date.toLocaleDateString(undefined, { 
                year: 'numeric', 
                month: 'short', 
                day: 'numeric',
                hour: '2-digit',
                minute: '2-digit'
            });
        } catch {
            return dateStr;
        }
    }
</script>

<svelte:head>
    <title>Afrah Planner Leads | ZafafWorld Admin</title>
</svelte:head>

<div class="assistant-container">
    <!-- Header banner -->
    <header class="assistant-banner">
        <div class="banner-glow"></div>
        <div class="banner-content">
            <span class="badge-premium">
                <Sparkles size={12} class="sparkle-icon" /> Planner Leads
            </span>
            <h1>Afrah Planner Management</h1>
            <p>Monitor platform-wide interactive planner leads, track client requests, and toggle resolution status.</p>
        </div>
    </header>

    <!-- Error notices -->
    {#if errorMessage}
        <div class="error-banner">
            <ShieldAlert size={18} class="error-icon" />
            <div class="error-text">{errorMessage}</div>
        </div>
    {/if}

    <!-- Metrics Cards -->
    <div class="metrics-grid">
        <div class="metric-card total">
            <div class="metric-info">
                <span class="metric-label">Total Leads</span>
                <span class="metric-value">{totalCount}</span>
            </div>
            <div class="metric-icon-wrap">
                <MessageSquare size={24} />
            </div>
        </div>

        <div class="metric-card pending">
            <div class="metric-info">
                <span class="metric-label">Pending Review</span>
                <span class="metric-value">{pendingCount}</span>
            </div>
            <div class="metric-icon-wrap">
                <Clock size={24} />
            </div>
        </div>

        <div class="metric-card resolved">
            <div class="metric-info">
                <span class="metric-label">Resolved Leads</span>
                <span class="metric-value">{resolvedCount}</span>
            </div>
            <div class="metric-icon-wrap">
                <CheckCircle2 size={24} />
            </div>
        </div>
    </div>

    <!-- Empty state -->
    {#if inquiries.length === 0}
        <div class="empty-state">
            <div class="empty-icon-wrapper">
                <Sparkles size={42} class="sparkle-icon-large" />
            </div>
            <h2>No Planner Leads Found</h2>
            <p>There are no inquiries recorded from the Afrah Interactive Planner yet.</p>
        </div>
    {:else}
        <!-- Inquiries Table Grid -->
        <div class="table-container">
            <table class="leads-table">
                <thead>
                    <tr>
                        <th>Client Email</th>
                        <th>Message Request</th>
                        <th>Submitted At</th>
                        <th>Status</th>
                        <th class="actions-header">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {#each inquiries as inquiry (inquiry.id)}
                        <tr class="lead-row" class:resolved={inquiry.status === 'resolved'}>
                            <td class="client-cell">
                                <div class="client-email-wrap">
                                    <Mail size={14} class="mail-icon" />
                                    <span>{inquiry.client_email}</span>
                                </div>
                            </td>
                            <td class="message-cell">
                                <p class="message-text">“{inquiry.message}”</p>
                            </td>
                            <td class="date-cell">
                                <div class="date-wrap">
                                    <Calendar size={13} class="calendar-icon" />
                                    <span>{formatDate(inquiry.created_at)}</span>
                                </div>
                            </td>
                            <td class="status-cell">
                                <span class="status-pill {inquiry.status}">
                                    <span class="status-dot"></span>
                                    <span>{inquiry.status}</span>
                                </span>
                            </td>
                            <td class="actions-cell">
                                {#if inquiry.status === 'pending'}
                                    <form 
                                        method="POST" 
                                        action="?/toggleStatus" 
                                        use:enhance={() => {
                                            submittingIds = [...submittingIds, inquiry.id];
                                            errorMessage = '';
                                            return async ({ result, update }) => {
                                                submittingIds = submittingIds.filter(id => id !== inquiry.id);
                                                if (result.type === 'success' && result.data?.success) {
                                                    // Status toggle handled by update
                                                } else if (result.type === 'failure') {
                                                    errorMessage = (result.data?.error as string) || 'Failed to update status';
                                                }
                                                update({ reset: false });
                                            };
                                        }}
                                    >
                                        <input type="hidden" name="id" value={inquiry.id} />
                                        <input type="hidden" name="status" value="resolved" />
                                        <button 
                                            type="submit" 
                                            class="btn-action resolve-btn"
                                            disabled={submittingIds.includes(inquiry.id)}
                                        >
                                            <Check size={14} />
                                            <span>Mark Resolved</span>
                                        </button>
                                    </form>
                                {:else}
                                    <form 
                                        method="POST" 
                                        action="?/toggleStatus" 
                                        use:enhance={() => {
                                            submittingIds = [...submittingIds, inquiry.id];
                                            errorMessage = '';
                                            return async ({ result, update }) => {
                                                submittingIds = submittingIds.filter(id => id !== inquiry.id);
                                                if (result.type === 'success' && result.data?.success) {
                                                    // Status toggle handled by update
                                                } else if (result.type === 'failure') {
                                                    errorMessage = (result.data?.error as string) || 'Failed to update status';
                                                }
                                                update({ reset: false });
                                            };
                                        }}
                                    >
                                        <input type="hidden" name="id" value={inquiry.id} />
                                        <input type="hidden" name="status" value="pending" />
                                        <button 
                                            type="submit" 
                                            class="btn-action reopen-btn"
                                            disabled={submittingIds.includes(inquiry.id)}
                                        >
                                            <RotateCcw size={14} />
                                            <span>Reopen Lead</span>
                                        </button>
                                    </form>
                                {/if}
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

<style>
    .assistant-container {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
        animation: fade-in 0.5s ease-out;
    }

    @keyframes fade-in {
        from { opacity: 0; transform: translateY(10px); }
        to { opacity: 1; transform: translateY(0); }
    }

    /* ─── BANNER ────────────────────────────────────────────────────────────── */
    .assistant-banner {
        position: relative;
        background: radial-gradient(circle at left, rgba(201, 169, 110, 0.05) 0%, transparent 60%);
        border: 1px solid rgba(255, 255, 255, 0.03);
        border-radius: 16px;
        padding: 2.5rem;
        overflow: hidden;
    }

    .banner-glow {
        position: absolute;
        width: 300px;
        height: 300px;
        background: radial-gradient(circle, rgba(201, 169, 110, 0.03) 0%, transparent 70%);
        top: -150px;
        left: -150px;
        pointer-events: none;
    }

    .banner-content h1 {
        margin: 0.5rem 0;
        font-size: 2.2rem;
        font-weight: 850;
        letter-spacing: -1px;
        color: #ffffff;
    }

    .banner-content p {
        margin: 0;
        font-size: 1rem;
        color: #64748b;
    }

    .badge-premium {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.65rem;
        font-weight: 850;
        text-transform: uppercase;
        letter-spacing: 1.2px;
        color: #C9A96E;
        background: rgba(201, 169, 110, 0.08);
        border: 1px solid rgba(201, 169, 110, 0.2);
        padding: 0.25rem 0.65rem;
        border-radius: 6px;
    }

    /* ─── ERROR BANNER ──────────────────────────────────────────────────────── */
    .error-banner {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 1.25rem 1.5rem;
        background: rgba(239, 68, 68, 0.08);
        border: 1px solid rgba(239, 68, 68, 0.25);
        border-radius: 12px;
        color: #fca5a5;
    }

    .error-text {
        font-size: 0.9rem;
        font-weight: 600;
    }

    /* ─── METRICS CARDS ─────────────────────────────────────────────────────── */
    .metrics-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1.5rem;
    }

    .metric-card {
        background: rgba(255, 255, 255, 0.01);
        border: 1px solid rgba(255, 255, 255, 0.03);
        border-radius: 14px;
        padding: 1.5rem;
        display: flex;
        justify-content: space-between;
        align-items: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
        transition: transform 0.2s ease, border-color 0.2s ease;
    }

    .metric-card:hover {
        transform: translateY(-2px);
        border-color: rgba(201, 169, 110, 0.15);
    }

    .metric-info {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .metric-label {
        font-size: 0.8rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: #64748b;
    }

    .metric-value {
        font-size: 1.8rem;
        font-weight: 800;
        color: #ffffff;
    }

    .metric-icon-wrap {
        width: 48px;
        height: 48px;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .total .metric-icon-wrap { background: rgba(99, 102, 241, 0.08); color: #818cf8; }
    .pending .metric-icon-wrap { background: rgba(245, 158, 11, 0.08); color: #fbbf24; }
    .resolved .metric-icon-wrap { background: rgba(16, 185, 129, 0.08); color: #34d399; }

    /* ─── EMPTY STATE ───────────────────────────────────────────────────────── */
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 6rem 2rem;
        background: rgba(10, 12, 26, 0.4);
        border: 1px dashed rgba(255, 255, 255, 0.05);
        border-radius: 16px;
        text-align: center;
    }

    .empty-icon-wrapper {
        width: 80px;
        height: 80px;
        border-radius: 50%;
        background: rgba(201, 169, 110, 0.04);
        border: 1px solid rgba(201, 169, 110, 0.15);
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: 1.5rem;
    }

    .sparkle-icon-large { color: #C9A96E; }

    .empty-state h2 {
        margin: 0;
        font-size: 1.5rem;
        color: #ffffff;
        font-weight: 750;
    }

    .empty-state p {
        margin: 0.5rem 0 0 0;
        font-size: 0.95rem;
        color: #64748b;
    }

    /* ─── TABLE GRID ────────────────────────────────────────────────────────── */
    .table-container {
        background: rgba(11, 15, 25, 0.65);
        backdrop-filter: blur(10px);
        -webkit-backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.04);
        border-radius: 16px;
        overflow: hidden;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.15);
    }

    .leads-table {
        width: 100%;
        border-collapse: collapse;
        text-align: left;
    }

    .leads-table th {
        padding: 1rem 1.5rem;
        background: rgba(255, 255, 255, 0.02);
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        font-size: 0.75rem;
        font-weight: 750;
        text-transform: uppercase;
        color: #64748b;
        letter-spacing: 0.5px;
    }

    .leads-table td {
        padding: 1.25rem 1.5rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.03);
        font-size: 0.85rem;
        color: #cbd5e1;
        vertical-align: middle;
        transition: background-color 0.2s ease;
    }

    .lead-row:hover td {
        background: rgba(255, 255, 255, 0.01);
    }

    .lead-row.resolved td {
        opacity: 0.65;
    }

    .client-email-wrap {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-weight: 700;
        color: #ffffff;
    }

    .mail-icon { color: rgba(201, 169, 110, 0.5); }

    .message-cell {
        max-width: 400px;
    }

    .message-text {
        margin: 0;
        line-height: 1.5;
        color: #94a3b8;
        display: -webkit-box;
        -webkit-line-clamp: 3;
        line-clamp: 3;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .date-wrap {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        color: #64748b;
    }

    .status-pill {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.7rem;
        font-weight: 750;
        text-transform: uppercase;
        letter-spacing: 0.3px;
        padding: 0.2rem 0.5rem;
        border-radius: 6px;
    }

    .status-pill.pending {
        background: rgba(245, 158, 11, 0.08);
        border: 1px solid rgba(245, 158, 11, 0.2);
        color: #fbbf24;
    }

    .status-pill.pending .status-dot {
        background: #fbbf24;
        box-shadow: 0 0 6px #fbbf24;
    }

    .status-pill.resolved {
        background: rgba(16, 185, 129, 0.08);
        border: 1px solid rgba(16, 185, 129, 0.2);
        color: #34d399;
    }

    .status-pill.resolved .status-dot {
        background: #10b981;
    }

    .status-dot {
        width: 5px;
        height: 5px;
        border-radius: 50%;
    }

    .actions-header {
        text-align: right;
    }

    .actions-cell {
        text-align: right;
    }

    .btn-action {
        border: none;
        border-radius: 8px;
        padding: 0.5rem 1rem;
        font-size: 0.75rem;
        font-weight: 750;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        transition: all 0.2s ease;
    }

    .resolve-btn {
        background: rgba(16, 185, 129, 0.08);
        border: 1px solid rgba(16, 185, 129, 0.25);
        color: #a7f3d0;
    }

    .resolve-btn:hover:not(:disabled) {
        background: #10b981;
        border-color: #10b981;
        color: #ffffff;
        box-shadow: 0 4px 10px rgba(16, 185, 129, 0.2);
    }

    .reopen-btn {
        background: rgba(99, 102, 241, 0.08);
        border: 1px solid rgba(99, 102, 241, 0.25);
        color: #c7d2fe;
    }

    .reopen-btn:hover:not(:disabled) {
        background: #6366f1;
        border-color: #6366f1;
        color: #ffffff;
        box-shadow: 0 4px 10px rgba(99, 102, 241, 0.2);
    }

    .btn-action:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    /* Responsive */
    @media (max-width: 1024px) {
        .metrics-grid {
            grid-template-columns: 1fr;
            gap: 1rem;
        }

        .leads-table th:nth-child(3),
        .leads-table td:nth-child(3) {
            display: none;
        }
    }

    @media (max-width: 768px) {
        .leads-table th:nth-child(1),
        .leads-table td:nth-child(1) {
            max-width: 150px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
    }
</style>
