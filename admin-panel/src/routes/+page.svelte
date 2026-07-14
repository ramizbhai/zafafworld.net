<script lang="ts">
    import { enhance } from '$app/forms';
    import { ShieldAlert, Award, Radio, Calendar, MapPin, Tag, Check, ExternalLink, Loader2, Sparkles, Building } from 'lucide-svelte';

    // Prop definitions matching server payload and actions feedback
    interface Props {
        data: {
            vendors: Array<{
                id: string;
                name_ar: string;
                name_en: string;
                slug: string;
                category: string;
                status: string;
                created_at: string;
                city_name_en: string;
                city_name_ar: string;
            }>;
        };
        form: {
            success?: boolean;
            error?: string;
            message?: string;
        } | null;
    }

    let { data, form }: Props = $props();

    // Track loading operations per vendor ID to animate individual buttons
    let approvingId = $state('');

    // Prettify category string to readable tag labels
    function formatCategory(category: string): string {
        return category
            .split('-')
            .map(word => {
                if (word === 'and') return '&';
                if (word === 'up') return 'Up';
                return word.charAt(0).toUpperCase() + word.slice(1);
            })
            .join(' ');
    }

    // Formats ISO timestamp to human-friendly local string
    function formatDate(isoString: string): string {
        try {
            const date = new Date(isoString);
            return date.toLocaleDateString('en-US', {
                month: 'short',
                day: 'numeric',
                hour: '2-digit',
                minute: '2-digit'
            });
        } catch {
            return isoString;
        }
    }
</script>

<div class="dashboard-workspace fade-in">
    <!-- Top-level Operational KPI Metrics -->
    <section class="metrics-grid">
        <div class="metric-card gold-border">
            <div class="metric-glow"></div>
            <div class="metric-icon-wrapper pending-icon">
                <ShieldAlert size={20} class="icon-core" />
            </div>
            <div class="metric-info">
                <span class="metric-label">Pending Approvals</span>
                <span class="metric-value">{data.vendors.length}</span>
                <span class="metric-comparison font-green">Action Required</span>
            </div>
        </div>

        <div class="metric-card">
            <div class="metric-icon-wrapper active-icon">
                <Building size={20} class="icon-core" />
            </div>
            <div class="metric-info">
                <span class="metric-label">Active Registry Partners</span>
                <span class="metric-value">---</span>
                <span class="metric-comparison">---</span>
            </div>
        </div>

        <div class="metric-card">
            <div class="metric-icon-wrapper health-icon">
                <Award size={20} class="icon-core" />
            </div>
            <div class="metric-info">
                <span class="metric-label">System Health Index</span>
                <span class="metric-value">---</span>
                <span class="metric-comparison">---</span>
            </div>
        </div>
    </section>

    <!-- Content area: Data grid container -->
    <section class="panel-section glass-panel">
        <div class="panel-header">
            <div class="panel-title-wrapper">
                <h2>Onboarding Registry Queue</h2>
                <p>Manage pending wedding brand registration applications, verify credentials, and approve live catalog placements.</p>
            </div>
            <div class="panel-actions">
                <span class="pulse-indicator">
                    <span class="dot pulse"></span> Live Queue Stream
                </span>
            </div>
        </div>

        <!-- Success/Error alert systems -->
        {#if form?.success}
            <div class="alert alert-success fade-in">
                <p>⚡ Success: {form.message}</p>
            </div>
        {/if}

        {#if form?.error}
            <div class="alert alert-danger fade-in">
                <p>⚠ Error: {form.error}</p>
            </div>
        {/if}

        {#if data.vendors.length === 0}
            <!-- Empty state illustration -->
            <div class="empty-state text-center">
                <Radio size={48} class="empty-icon" />
                <h3>Onboarding Queue Clear</h3>
                <p>There are no pending vendor onboarding applications left to verify. Outstanding records successfully mapped.</p>
            </div>
        {:else}
            <!-- Premium Table-grid layout -->
            <div class="table-responsive">
                <table class="admin-table">
                    <thead>
                        <tr>
                            <th>Wedding Brand / المنشأة</th>
                            <th>Category</th>
                            <th>Operational HQ</th>
                            <th>Submitted Date</th>
                            <th class="text-right">Administration Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each data.vendors as vendor (vendor.id)}
                            <tr class="table-row fade-in">
                                <td>
                                    <div class="brand-cell">
                                        <div class="brand-initial">{vendor.name_en.charAt(0)}</div>
                                        <div class="brand-names">
                                            <span class="brand-en">{vendor.name_en}</span>
                                            <span class="brand-ar" dir="rtl">{vendor.name_ar}</span>
                                            <span class="brand-slug">/{vendor.slug}</span>
                                        </div>
                                    </div>
                                </td>
                                <td>
                                    <span class="badge badge-category">
                                        <Tag size={12} class="badge-icon" /> {formatCategory(vendor.category)}
                                    </span>
                                </td>
                                <td>
                                    <span class="location-span">
                                        <MapPin size={14} class="location-icon" /> {vendor.city_name_en} ({vendor.city_name_ar})
                                    </span>
                                </td>
                                <td>
                                    <span class="date-span">
                                        <Calendar size={14} class="date-icon" /> {formatDate(vendor.created_at)}
                                    </span>
                                </td>
                                <td class="text-right">
                                    <div class="actions-cell">
                                        <a href={`http://localhost:5173/vendor/${vendor.slug}`} target="_blank" rel="noreferrer" class="btn btn-icon" title="Preview profile storefront">
                                            <ExternalLink size={16} />
                                        </a>

                                        <!-- Approve execution form -->
                                        <form method="POST" action="?/approve" use:enhance={() => {
                                            approvingId = vendor.id;
                                            return async ({ update }) => {
                                                approvingId = '';
                                                await update();
                                            };
                                        }}>
                                            <input type="hidden" name="id" value={vendor.id} />
                                            <button type="submit" class="btn btn-success" disabled={approvingId !== ''}>
                                                {#if approvingId === vendor.id}
                                                    <span class="spin"><Loader2 size={14} /></span> Activating
                                                {:else}
                                                    <Check size={14} /> Approve Brand
                                                {/if}
                                            </button>
                                        </form>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}
    </section>
</div>

<style>
    /* Styling components, glass structures, and fonts */
    .dashboard-workspace {
        display: flex;
        flex-direction: column;
        gap: 2rem;
        width: 100%;
    }

    /* KPI Metrics Row */
    .metrics-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1.5rem;
        width: 100%;
    }

    .metric-card {
        background: var(--glass-sm);
        border: 1px solid var(--glass-border);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border-radius: 1rem;
        padding: 1.5rem;
        display: flex;
        align-items: center;
        gap: 1.25rem;
        position: relative;
        overflow: hidden;
        box-shadow: var(--shadow-sm);
        transition: border-color var(--dur-base) var(--ease-smooth), box-shadow var(--dur-base) var(--ease-smooth);
    }
    .metric-card:hover {
        border-color: var(--glass-border-hover);
        box-shadow: var(--shadow-md);
    }

    .gold-border {
        border-color: rgba(245, 158, 11, 0.2);
    }

    .metric-glow {
        position: absolute;
        width: 150px;
        height: 150px;
        background: radial-gradient(circle, rgba(245, 158, 11, 0.06) 0%, transparent 70%);
        top: -50px;
        right: -50px;
        pointer-events: none;
    }

    .metric-icon-wrapper {
        width: 48px;
        height: 48px;
        border-radius: 0.75rem;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    :global(.icon-core) {
        transition: transform 0.2s ease;
    }

    .pending-icon {
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.15);
        color: #ef4444;
    }

    .active-icon {
        background: rgba(59, 130, 246, 0.1);
        border: 1px solid rgba(59, 130, 246, 0.15);
        color: #3b82f6;
    }

    .health-icon {
        background: rgba(16, 185, 129, 0.1);
        border: 1px solid rgba(16, 185, 129, 0.15);
        color: #10b981;
    }

    .metric-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .metric-label {
        font-size: 0.8rem;
        color: #64748b;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .metric-value {
        font-size: 1.75rem;
        font-weight: 800;
        color: var(--text-primary);
    }

    .metric-comparison {
        font-size: 0.75rem;
        color: #475569;
    }

    .font-green {
        color: #ef4444 !important;
        font-weight: 600;
    }

    /* Main glassmorphic data table panel */
    .glass-panel {
        background: var(--glass-md);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 1px solid var(--glass-border);
        border-radius: 1.25rem;
        padding: 2rem;
        box-shadow: var(--shadow-md);
    }

    .panel-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 2rem;
        border-bottom: 1px solid var(--glass-border);
        padding-bottom: 1.25rem;
    }

    .panel-title-wrapper h2 {
        font-size: 1.5rem;
        font-weight: 800;
        margin: 0 0 0.35rem 0;
        background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .panel-title-wrapper p {
        color: #64748b;
        font-size: 0.9rem;
        margin: 0;
        max-width: 800px;
    }

    .pulse-indicator {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: #fbbf24;
        font-weight: 600;
        font-size: 0.8rem;
        background: rgba(245, 158, 11, 0.1);
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        border: 1px solid rgba(245, 158, 11, 0.2);
    }

    .dot {
        width: 6px;
        height: 6px;
        background-color: #f59e0b;
        border-radius: 50%;
    }

    .pulse {
        animation: blink 2s infinite ease-in-out;
    }

    /* Core Admin Table Styling */
    .table-responsive {
        width: 100%;
        overflow-x: auto;
    }

    .admin-table {
        width: 100%;
        border-collapse: collapse;
        text-align: left;
    }

    .admin-table th {
        padding: 1rem 1.25rem;
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 1px;
        color: #475569;
        font-weight: 700;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .admin-table td {
        padding: 1.25rem;
        vertical-align: middle;
        border-bottom: 1px solid rgba(255, 255, 255, 0.03);
    }

    .table-row {
        transition: background 0.2s ease;
    }

    .table-row:hover {
        background: rgba(255, 255, 255, 0.01);
    }

    /* Brand Cell Component styling */
    .brand-cell {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .brand-initial {
        width: 40px;
        height: 40px;
        border-radius: 0.5rem;
        background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
        border: 1px solid rgba(255, 255, 255, 0.05);
        color: #fbbf24;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.1rem;
        font-weight: 800;
    }

    .brand-names {
        display: flex;
        flex-direction: column;
        gap: 0.1rem;
    }

    .brand-en {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--text-primary);
    }

    .brand-ar {
        font-family: 'Cairo', sans-serif;
        font-size: 0.8rem;
        color: var(--text-secondary);
    }

    .brand-slug {
        font-family: monospace;
        font-size: 0.75rem;
        color: #475569;
    }

    /* Category labels styling */
    .badge {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        font-size: 0.75rem;
        font-weight: 600;
    }

    .badge-category {
        background: rgba(148, 163, 184, 0.06);
        color: #cbd5e1;
        border: 1px solid rgba(148, 163, 184, 0.15);
    }

    :global(.badge-icon) {
        color: #64748b;
    }

    /* Operations location styling */
    .location-span {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.85rem;
        color: #cbd5e1;
    }

    :global(.location-icon) {
        color: #f59e0b;
    }

    /* Submitted operation date styling */
    .date-span {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.85rem;
        color: #64748b;
    }

    :global(.date-icon) {
        color: #475569;
    }

    /* Tabular Action controls */
    .actions-cell {
        display: flex;
        justify-content: flex-end;
        align-items: center;
        gap: 0.75rem;
    }

    /* Dashboard Button styling */
    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-size: 0.8rem;
        font-weight: 600;
        cursor: pointer;
        border: none;
        transition: all 0.2s ease;
    }

    .btn-success {
        background: linear-gradient(135deg, #10b981 0%, #059669 100%);
        color: #ffffff;
        box-shadow: 0 4px 10px rgba(16, 185, 129, 0.15);
    }

    .btn-success:hover:not(:disabled) {
        transform: translateY(-1px);
        box-shadow: 0 6px 14px rgba(16, 185, 129, 0.25);
    }

    .btn-icon {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        color: #94a3b8;
        padding: 0.5rem;
        text-decoration: none;
    }

    .btn-icon:hover {
        background: rgba(255, 255, 255, 0.08);
        color: #ffffff;
    }

    .btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    /* Alerts and console feedback structures */
    .alert {
        padding: 1rem 1.25rem;
        border-radius: 0.5rem;
        margin-bottom: 1.5rem;
        font-size: 0.9rem;
        font-weight: 500;
    }

    .alert-success {
        background: rgba(16, 185, 129, 0.08);
        border: 1px solid rgba(16, 185, 129, 0.20);
        color: #047857;
    }

    .alert-danger {
        background: rgba(239, 68, 68, 0.08);
        border: 1px solid rgba(239, 68, 68, 0.20);
        color: #b91c1c;
    }

    /* Empty queue styling */
    .empty-state {
        padding: 5rem 2rem;
        background: var(--glass-sm);
        border-radius: 0.75rem;
        border: 1px dashed var(--glass-border);
    }

    :global(.empty-icon) {
        color: var(--text-ghost);
        margin-bottom: 1.25rem;
    }

    .empty-state h3 {
        font-size: 1.2rem;
        font-weight: 700;
        margin: 0 0 0.5rem 0;
        color: var(--text-primary);
    }

    .empty-state p {
        color: #64748b;
        font-size: 0.9rem;
        margin: 0;
        max-width: 450px;
        margin-left: auto;
        margin-right: auto;
        line-height: 1.5;
    }

    .text-center {
        text-align: center;
    }

    .text-right {
        text-align: right;
    }

    /* Spinner rotation */
    .spin {
        display: inline-flex;
        animation: rotate 1s linear infinite;
    }

    @keyframes rotate {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }

    .fade-in {
        animation: fadeIn 0.4s ease-out forwards;
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(8px); }
        to { opacity: 1; transform: translateY(0); }
    }

    /* Responsive styling */
    @media (max-width: 1024px) {
        .metrics-grid {
            grid-template-columns: 1fr;
            gap: 1rem;
        }
        .admin-table th, .admin-table td {
            padding: 0.85rem;
        }
        .panel-header {
            flex-direction: column;
            gap: 1rem;
            align-items: stretch;
        }
    }
</style>
