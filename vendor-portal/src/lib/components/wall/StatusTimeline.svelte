<script lang="ts">
    import { CheckCircle } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';

    const i18n = getI18n();
</script>

<div class="progress-steps">
    <div class="step-item active">
        <div class="step-num"><CheckCircle size={16} /></div>
        <div class="step-label">
            <h4>{i18n.t.pendingWall.step1Title}</h4>
            <p>{i18n.t.pendingWall.step1Desc}</p>
        </div>
    </div>
    <div class="step-item active pulse-border">
        <div class="step-num spinner-small"></div>
        <div class="step-label">
            <h4>{i18n.t.pendingWall.step2Title}</h4>
            <p>{i18n.t.pendingWall.step2Desc}</p>
        </div>
    </div>
    <div class="step-item">
        <div class="step-num">3</div>
        <div class="step-label">
            <h4>{i18n.t.pendingWall.step3Title}</h4>
            <p>{i18n.t.pendingWall.step3Desc}</p>
        </div>
    </div>
</div>

<style>
    .progress-steps {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        background-color: #faf8f5;
        border-radius: 1rem;
        padding: 1.5rem;
        border: 1px solid rgba(0, 0, 0, 0.03);
    }

    .step-item {
        display: flex;
        gap: 1rem;
        align-items: flex-start;
        opacity: 0.45;
        transition: opacity 0.3s;
    }

    .step-item.active {
        opacity: 1;
    }

    .step-num {
        width: 1.75rem;
        height: 1.75rem;
        border-radius: 50%;
        background-color: #ffffff;
        border: 1px solid rgba(0, 0, 0, 0.1);
        color: #64748b;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.8rem;
        font-weight: 700;
        flex-shrink: 0;
        margin-top: 0.1rem;
    }

    .step-item.active .step-num {
        background-color: rgba(91, 33, 182, 0.06);
        border-color: var(--color-primary, #5b21b6);
        color: var(--color-primary, #5b21b6);
    }

    .step-label h4 {
        margin: 0 0 0.2rem 0;
        font-size: 0.92rem;
        font-weight: 700;
        color: var(--color-text-dark, #1e293b);
    }

    .step-label p {
        margin: 0;
        font-size: 0.825rem;
        color: #475569;
        line-height: 1.4;
    }

    .spinner-small {
        border: 2px solid rgba(91, 33, 182, 0.15);
        border-top-color: var(--color-primary, #5b21b6);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    .pulse-border {
        position: relative;
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }
</style>
