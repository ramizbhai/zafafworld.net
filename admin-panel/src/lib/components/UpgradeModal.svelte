<script lang="ts">
    import { ShieldAlert, ArrowRight, X } from 'lucide-svelte';

    export let isOpen = false;
    export let onClose: () => void = () => { isOpen = false; };
</script>

{#if isOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="modal-overlay" onclick={onClose} role="button" tabindex="0">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div class="modal-content glass-effect" onclick={(e) => e.stopPropagation()} role="document">
            <button class="close-btn" onclick={onClose} aria-label="Close modal">
                <X size={20} />
            </button>
            <div class="modal-header">
                <div class="icon-wrapper">
                    <ShieldAlert size={36} color="#ef4444" />
                </div>
                <h2>Limit Reached</h2>
                <p>You have reached your subscription quota limit.</p>
            </div>
            
            <div class="modal-body">
                <div class="quota-warning">
                    <p>Upgrade to a higher tier to add more products, photos, and features to your profile.</p>
                </div>
            </div>

            <div class="modal-footer">
                <button class="btn-cancel" onclick={onClose}>Maybe Later</button>
                <a href="/dashboard/billing/upgrade" class="btn-upgrade">
                    Upgrade Now <ArrowRight size={16} />
                </a>
            </div>
        </div>
    </div>
{/if}

<style>
    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(15, 23, 42, 0.7);
        backdrop-filter: blur(8px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 99999;
        animation: fadeIn 0.25s ease-out;
    }

    .glass-effect {
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(20px);
        border: 1px solid rgba(255, 255, 255, 0.5);
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25), 0 0 0 1px rgba(255, 255, 255, 0.2) inset;
    }

    .modal-content {
        position: relative;
        width: 90%;
        max-width: 420px;
        border-radius: 24px;
        padding: 32px;
        text-align: center;
        animation: slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .close-btn {
        position: absolute;
        top: 16px;
        right: 16px;
        background: rgba(0,0,0,0.05);
        border: none;
        width: 32px;
        height: 32px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s;
        color: #64748b;
    }

    .close-btn:hover {
        background: rgba(0,0,0,0.1);
        color: #0f172a;
        transform: rotate(90deg);
    }

    .modal-header {
        margin-bottom: 24px;
    }

    .icon-wrapper {
        width: 72px;
        height: 72px;
        background: rgba(239, 68, 68, 0.1);
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        margin: 0 auto 20px;
        border: 2px solid rgba(239, 68, 68, 0.2);
    }

    .modal-header h2 {
        font-size: 1.5rem;
        font-weight: 800;
        color: #0f172a;
        margin: 0 0 8px;
        font-family: 'Outfit', 'Cairo', sans-serif;
    }

    .modal-header p {
        color: #64748b;
        font-size: 0.95rem;
        margin: 0;
    }

    .quota-warning {
        background: linear-gradient(135deg, rgba(99, 102, 241, 0.05) 0%, rgba(168, 85, 247, 0.05) 100%);
        padding: 16px;
        border-radius: 12px;
        border: 1px dashed rgba(99, 102, 241, 0.3);
        margin-bottom: 28px;
    }

    .quota-warning p {
        color: #4338ca;
        font-size: 0.9rem;
        font-weight: 500;
        margin: 0;
        line-height: 1.5;
    }

    .modal-footer {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .btn-upgrade {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        width: 100%;
        padding: 14px;
        border-radius: 12px;
        background: linear-gradient(135deg, #6366f1 0%, #a855f7 100%);
        color: white;
        font-weight: 700;
        font-size: 1rem;
        text-decoration: none;
        border: none;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
    }

    .btn-upgrade:hover {
        transform: translateY(-2px);
        box-shadow: 0 6px 16px rgba(99, 102, 241, 0.4);
    }

    .btn-upgrade:active {
        transform: translateY(0);
    }

    .btn-cancel {
        width: 100%;
        padding: 14px;
        border-radius: 12px;
        background: transparent;
        color: #64748b;
        font-weight: 600;
        font-size: 1rem;
        border: 2px solid transparent;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-cancel:hover {
        background: rgba(100, 116, 139, 0.05);
        color: #0f172a;
    }

    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    @keyframes slideUp {
        from { opacity: 0; transform: translateY(20px) scale(0.95); }
        to { opacity: 1; transform: translateY(0) scale(1); }
    }
</style>
