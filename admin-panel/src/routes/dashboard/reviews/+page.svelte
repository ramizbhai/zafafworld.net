<script lang="ts">
    import { enhance } from '$app/forms';
    import { resolveMediaUrl } from '$lib/shared/utils/media';
    import { MessageSquare, ShieldAlert, Sparkles, Calendar, Star, Check, Trash2, HelpCircle } from 'lucide-svelte';

    let { data } = $props();

    // Local reactive state using Svelte 5 runes
    let removingIds = $state<string[]>([]);
    let submittingIds = $state<string[]>([]);
    let errorMessage = $state('');

    // Keep data in sync with loader updates and apply optimistic removals
    let reviews = $derived(data.reviews.filter((r: any) => !removingIds.includes(r.id)));

    // Helper to format ratings
    function getStars(score: number): number[] {
        return Array.from({ length: 5 }, (_, i) => i + 1);
    }
</script>

<svelte:head>
    <title>Review Moderation Queue | ZafafWorld Admin</title>
</svelte:head>

<div class="moderation-container">
    <!-- Header title -->
    <header class="moderation-banner">
        <div class="banner-glow"></div>
        <div class="banner-content">
            <span class="badge-premium">
                <Sparkles size={12} class="sparkle-icon" /> Moderation Feed
            </span>
            <h1>Review Moderation Queue</h1>
            <p>Inspect client reviews, verify 3-axis quality scores, and publish or reject ratings.</p>
        </div>
    </header>

    <!-- Error notices -->
    {#if errorMessage}
        <div class="error-banner">
            <ShieldAlert size={18} class="error-icon" />
            <div class="error-text">{errorMessage}</div>
        </div>
    {/if}

    <!-- Empty state -->
    {#if reviews.length === 0}
        <div class="empty-state">
            <div class="empty-icon-wrapper">
                <MessageSquare size={42} class="message-icon" />
            </div>
            <h2>Feed Fully Moderated</h2>
            <p>Excellent! There are no pending reviews awaiting administrative verification.</p>
        </div>
    {:else}
        <!-- Curation Grid Layout -->
        <div class="reviews-feed">
            {#each reviews as review (review.id)}
                <div 
                    class="review-card" 
                    class:removing={removingIds.includes(review.id)}
                    class:submitting={submittingIds.includes(review.id)}
                >
                    <div class="card-edge"></div>
                    <div class="card-inner">
                        <!-- Top Row: Author Name, Wedding Date, Vendor Name -->
                        <div class="card-header-row">
                            <div class="author-block">
                                <span class="author-name">{review.author_name}</span>
                                <span class="wedding-tag">
                                    <Calendar size={12} class="tag-icon" />
                                    <span>Wedding: {review.wedding_date}</span>
                                </span>
                            </div>
                            <div class="vendor-block">
                                <span class="vendor-en">{review.vendor_name_en}</span>
                                <span class="vendor-ar" dir="rtl">{review.vendor_name_ar}</span>
                            </div>
                        </div>

                        <!-- Content Row: Comment & Scores -->
                        <div class="card-body-row">
                            <!-- Comment Text -->
                            <div class="comment-text-box">
                                <p class="comment-quote">“{review.comment}”</p>
                                
                                {#if review.attachments && review.attachments.length > 0}
                                    <div class="review-attachments">
                                        {#each review.attachments as img}
                                            <img src={resolveMediaUrl(img)} alt="Attachment" class="attachment-thumb" />
                                        {/each}
                                    </div>
                                {/if}
                            </div>

                            <!-- Granular 3-Axis Scores -->
                            <div class="scores-panel">
                                <div class="score-axis">
                                    <div class="axis-info">
                                        <span class="axis-label">Quality / الجودة</span>
                                        <span class="axis-value">{review.rating_quality}/5</span>
                                    </div>
                                    <div class="progress-bar">
                                        <div class="progress-fill gold-fill" style="width: {review.rating_quality * 20}%"></div>
                                    </div>
                                </div>

                                <div class="score-axis">
                                    <div class="axis-info">
                                        <span class="axis-label">Staff / الخدمة</span>
                                        <span class="axis-value">{review.rating_staff}/5</span>
                                    </div>
                                    <div class="progress-bar">
                                        <div class="progress-fill blue-fill" style="width: {review.rating_staff * 20}%"></div>
                                    </div>
                                </div>

                                <div class="score-axis">
                                    <div class="axis-info">
                                        <span class="axis-label">Communication / التواصل</span>
                                        <span class="axis-value">{review.rating_communication}/5</span>
                                    </div>
                                    <div class="progress-bar">
                                        <div class="progress-fill emerald-fill" style="width: {review.rating_communication * 20}%"></div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- Bottom Row: Moderation Actions -->
                        <div class="card-footer-row">
                            <div class="moderation-tag">
                                <span class="pulse-dot"></span>
                                <span>Awaiting Verification</span>
                            </div>

                            <div class="action-buttons">
                                <!-- Form for Approval -->
                                <form 
                                    method="POST" 
                                    action="?/updateStatus" 
                                    use:enhance={() => {
                                        submittingIds.push(review.id);
                                        errorMessage = '';
                                        return async ({ result, update }) => {
                                            submittingIds = submittingIds.filter(id => id !== review.id);
                                            if (result.type === 'success' && result.data?.success) {
                                                removingIds.push(review.id);
                                            } else if (result.type === 'failure') {
                                                errorMessage = (result.data?.error as string) || 'Failed to approve review';
                                            } else {
                                                errorMessage = 'Connection failure with backend service';
                                            }
                                            update({ reset: false });
                                        };
                                    }}
                                >
                                    <input type="hidden" name="id" value={review.id} />
                                    <input type="hidden" name="status" value="approved" />
                                    <button 
                                        type="submit" 
                                        class="btn-action btn-approve"
                                        disabled={submittingIds.includes(review.id)}
                                    >
                                        <Check size={14} />
                                        <span>Approve Review</span>
                                    </button>
                                </form>

                                <!-- Form for Rejection -->
                                <form 
                                    method="POST" 
                                    action="?/updateStatus" 
                                    use:enhance={() => {
                                        submittingIds.push(review.id);
                                        errorMessage = '';
                                        return async ({ result, update }) => {
                                            submittingIds = submittingIds.filter(id => id !== review.id);
                                            if (result.type === 'success' && result.data?.success) {
                                                removingIds.push(review.id);
                                            } else if (result.type === 'failure') {
                                                errorMessage = (result.data?.error as string) || 'Failed to reject review';
                                            } else {
                                                errorMessage = 'Connection failure with backend service';
                                            }
                                            update({ reset: false });
                                        };
                                    }}
                                >
                                    <input type="hidden" name="id" value={review.id} />
                                    <input type="hidden" name="status" value="rejected" />
                                    <button 
                                        type="submit" 
                                        class="btn-action btn-reject"
                                        disabled={submittingIds.includes(review.id)}
                                    >
                                        <Trash2 size={14} />
                                        <span>Reject/Spam</span>
                                    </button>
                                </form>
                            </div>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .moderation-container {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
        animation: fade-in 0.5s ease-out;
    }

    @keyframes fade-in {
        from { opacity: 0; transform: translateY(10px); }
        to { opacity: 1; transform: translateY(0); }
    }

    /* ─── BANNER COMPONENT ──────────────────────────────────────────────────── */
    .moderation-banner {
        position: relative;
        background: radial-gradient(circle at left, rgba(245, 158, 11, 0.05) 0%, transparent 60%);
        border: 1px solid rgba(255, 255, 255, 0.03);
        border-radius: 16px;
        padding: 2.5rem;
        overflow: hidden;
    }

    .banner-glow {
        position: absolute;
        width: 300px;
        height: 300px;
        background: radial-gradient(circle, rgba(245, 158, 11, 0.03) 0%, transparent 70%);
        top: -150px;
        left: -150px;
        pointer-events: none;
    }

    .banner-content h1 {
        margin: 0.5rem 0;
        font-size: 2.2rem;
        font-weight: 850;
        letter-spacing: -1px;
        color: var(--text-primary);
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
        color: var(--gold-deep);
        background: rgba(245, 158, 11, 0.08);
        border: 1px solid rgba(245, 158, 11, 0.2);
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
        color: #b91c1c;
    }

    .error-text {
        font-size: 0.9rem;
        font-weight: 600;
    }

    /* ─── EMPTY STATE ───────────────────────────────────────────────────────── */
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 6rem 2rem;
        background: var(--glass-sm);
        border: 1px dashed var(--glass-border);
        border-radius: 16px;
        text-align: center;
    }

    .empty-icon-wrapper {
        width: 80px;
        height: 80px;
        border-radius: 50%;
        background: var(--gold-subtle);
        border: 1px solid var(--gold-border);
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: 1.5rem;
        box-shadow: var(--shadow-md);
    }

    .empty-state h2 {
        margin: 0;
        font-size: 1.5rem;
        color: var(--text-primary);
        font-weight: 750;
    }

    .empty-state p {
        margin: 0.5rem 0 0 0;
        font-size: 0.95rem;
        color: var(--text-secondary);
        max-width: 400px;
    }

    /* ─── REVIEW CARDS FEED ─────────────────────────────────────────────────── */
    .reviews-feed {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .review-card {
        background: var(--glass-sm);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 1px solid var(--glass-border);
        border-radius: 16px;
        position: relative;
        overflow: hidden;
        transition: all 450ms cubic-bezier(0.16, 1, 0.3, 1);
        transform-origin: center center;
    }

    .review-card:hover {
        border-color: var(--glass-border-hover);
        box-shadow: var(--shadow-md);
    }

    /* Dismissal Animation */
    .review-card.removing {
        opacity: 0;
        transform: scale(0.95) translateY(-15px);
        max-height: 0;
        padding-top: 0;
        padding-bottom: 0;
        margin-top: -1.5rem;
        border-color: transparent;
        pointer-events: none;
    }

    .card-edge {
        position: absolute;
        left: 0;
        top: 0;
        bottom: 0;
        width: 3px;
        background: linear-gradient(180deg, #fbbf24 0%, transparent 100%);
        opacity: 0.4;
    }

    .card-inner {
        padding: 2rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    /* Top Row Layout */
    .card-header-row {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        border-bottom: 1px solid var(--glass-border);
        padding-bottom: 1.25rem;
        gap: 1.5rem;
    }

    .author-block {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .author-name {
        font-size: 1.2rem;
        font-weight: 750;
        color: var(--text-primary);
    }

    .wedding-tag {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.75rem;
        color: #64748b;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.04);
        padding: 0.2rem 0.6rem;
        border-radius: 6px;
        width: fit-content;
    }

    .vendor-block {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 0.15rem;
    }

    .vendor-en {
        font-size: 0.95rem;
        font-weight: 700;
        color: var(--text-secondary);
    }

    .vendor-ar {
        font-family: 'Cairo', sans-serif;
        font-size: 0.85rem;
        color: var(--gold-deep);
        font-weight: 600;
    }

    /* Content Row Layout */
    .card-body-row {
        display: grid;
        grid-template-columns: 1fr 320px;
        gap: 2.5rem;
        align-items: flex-start;
    }

    .comment-text-box {
        background: var(--bg-deep);
        border: 1px solid var(--glass-border);
        border-radius: 12px;
        padding: 1.25rem 1.5rem;
        min-height: 120px;
    }

    .comment-quote {
        margin: 0;
        font-size: 0.95rem;
        line-height: 1.6;
        color: var(--text-secondary);
        font-style: italic;
    }

    /* 3-Axis Scores styling */
    .scores-panel {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        background: rgba(255, 255, 255, 0.01);
        border: 1px solid rgba(255, 255, 255, 0.03);
        padding: 1.25rem;
        border-radius: 12px;
    }

    .score-axis {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .axis-info {
        display: flex;
        justify-content: space-between;
        font-size: 0.75rem;
        font-weight: 600;
    }

    .axis-label {
        color: #64748b;
    }

    .axis-value {
        color: var(--text-primary);
    }

    .progress-bar {
        height: 6px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 9999px;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        border-radius: 9999px;
    }

    .gold-fill { background: #fbbf24; }
    .blue-fill { background: #3b82f6; }
    .emerald-fill { background: #10b981; }

    /* Footer Row Layout */
    .card-footer-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-top: 1px solid var(--glass-border);
        padding-top: 1.25rem;
        margin-top: 0.5rem;
    }

    .moderation-tag {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--gold-deep);
        background: rgba(245, 158, 11, 0.05);
        border: 1px solid rgba(245, 158, 11, 0.15);
        padding: 0.3rem 0.75rem;
        border-radius: 6px;
    }

    .pulse-dot {
        width: 6px;
        height: 6px;
        background-color: var(--gold);
        border-radius: 50%;
        animation: blink 2s infinite ease-in-out;
        box-shadow: 0 0 8px var(--gold);
    }

    .action-buttons {
        display: flex;
        gap: 0.75rem;
    }

    .btn-action {
        border: none;
        border-radius: 8px;
        padding: 0.65rem 1.25rem;
        font-size: 0.8rem;
        font-weight: 750;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        transition: all 0.2s ease;
    }

    .btn-approve {
        background: rgba(16, 185, 129, 0.08);
        border: 1px solid rgba(16, 185, 129, 0.25);
        color: #047857;
    }

    .btn-approve:hover:not(:disabled) {
        background: #10b981;
        border-color: #10b981;
        color: #ffffff;
        box-shadow: 0 4px 12px rgba(16, 185, 129, 0.25);
        transform: translateY(-1px);
    }

    .btn-reject {
        background: rgba(239, 68, 68, 0.08);
        border: 1px solid rgba(239, 68, 68, 0.25);
        color: #b91c1c;
    }

    .btn-reject:hover:not(:disabled) {
        background: #ef4444;
        border-color: #ef4444;
        color: #ffffff;
        box-shadow: 0 4px 12px rgba(239, 68, 68, 0.25);
        transform: translateY(-1px);
    }

    .btn-action:active:not(:disabled) {
        transform: translateY(1px);
    }

    .btn-action:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    @keyframes blink {
        0%, 100% { opacity: 0.4; }
        50% { opacity: 1; }
    }

    /* Responsive Spacers */
    @media (max-width: 900px) {
        .card-body-row {
            grid-template-columns: 1fr;
            gap: 1.5rem;
        }

        .scores-panel {
            width: 100%;
        }
    }

    @media (max-width: 640px) {
        .card-header-row {
            flex-direction: column;
            align-items: stretch;
            gap: 1rem;
        }

        .vendor-block {
            align-items: flex-start;
        }

        .card-footer-row {
            flex-direction: column;
            align-items: stretch;
            gap: 1.25rem;
        }

        .action-buttons {
            flex-direction: column;
        }

        .btn-action {
            width: 100%;
            justify-content: center;
        }
    }

    .review-attachments {
        display: flex;
        gap: 0.75rem;
        margin-top: 1.25rem;
        flex-wrap: wrap;
    }

    .attachment-thumb {
        width: 80px;
        height: 80px;
        object-fit: cover;
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        transition: transform 0.2s ease, border-color 0.2s ease;
    }

    .attachment-thumb:hover {
        transform: scale(1.05);
        border-color: #fbbf24;
    }
</style>
