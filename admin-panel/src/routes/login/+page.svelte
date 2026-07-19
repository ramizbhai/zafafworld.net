<script lang="ts">
    import { enhance } from '$app/forms';
    import { ShieldCheck, Loader2 } from 'lucide-svelte';

    interface Props {
        form: {
            error?: string;
            values?: {
                email?: string;
            };
        } | null;
    }

    let { form }: Props = $props();

    // Svelte 5 states for live interactive feedbacks
    let email = $state('');
    let password = $state('');
    let isSubmitting = $state(false);

    // Sync state if form prop values update dynamically
    $effect(() => {
        if (form?.values?.email) {
            email = form.values.email;
        }
    });


</script>

<svelte:head>
    <title>ZafafWorld - Administration Console Secure Login</title>
    <link href="https://fonts.googleapis.com/css2?family=Cairo:wght@400;600;700;800&family=Outfit:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
</svelte:head>

<div class="login-wrapper">
    <div class="glow-sphere ambient-1"></div>
    <div class="glow-sphere ambient-2"></div>

    <div class="login-card glass-panel">
        <div class="gold-rim"></div>
        
        <header class="login-header">
            <div class="shield-badge">
                <ShieldCheck size={28} class="shield-icon" />
            </div>
            <h1>
                <span class="gold-text">ZAFAF</span> <span class="white-text">WORLD</span>
            </h1>
            <p class="subtitle">Secure Administrative Control Console</p>
        </header>

        {#if form?.error}
            <div class="alert alert-danger fade-in">
                <span class="alert-symbol">⚠️</span>
                <span class="alert-message">{form.error}</span>
            </div>
        {/if}

        <form method="POST" use:enhance={() => {
            isSubmitting = true;
            return async ({ result, update }) => {
                if (result.type === 'redirect') {
                    // Perform a full browser navigation so the Set-Cookie header
                    // from the action response is stored before the dashboard
                    // layout guard reads cookies.get('zafaf_admin_session').
                    // Using window.location.href guarantees the cookie is committed
                    // to the browser's cookie store before the new request fires.
                    window.location.href = result.location;
                } else {
                    isSubmitting = false;
                    await update();
                }
            };
        }} class="login-form">
            <div class="input-group">
                <label for="email">Administrative Email</label>
                <div class="field-container">
                    <input 
                        type="email" 
                        id="email" 
                        name="email" 
                        bind:value={email}
                        placeholder="admin@zafafworld.com"
                        required
                        disabled={isSubmitting}
                        autocomplete="email"
                    />
                </div>
            </div>

            <div class="input-group">
                <label for="password">Password Key</label>
                <div class="field-container">
                    <input 
                        type="password" 
                        id="password" 
                        name="password" 
                        bind:value={password}
                        placeholder="••••••••••••"
                        required
                        disabled={isSubmitting}
                        autocomplete="current-password"
                    />
                </div>
            </div>

            <button type="submit" class="btn btn-primary" disabled={isSubmitting}>
                {#if isSubmitting}
                    <span class="spin"><Loader2 size={16} /></span>
                    Establishing Secure Link...
                {:else}
                    Authenticate Admin Session
                {/if}
            </button>
        </form>

        <footer class="login-footer">
            <p>System activities are logged under strict corporate protocol.</p>
            <p class="copyright">&copy; 2026 ZafafWorld Co. All rights reserved.</p>
        </footer>
    </div>
</div>

<style>
    /* Premium Warm Ivory Light Theme Styling */
    :global(body) {
        background-color: var(--color-bg-warm, #faf8f5) !important;
        margin: 0;
        font-family: 'Outfit', 'Cairo', system-ui, sans-serif;
        color: var(--color-text-dark, #1e293b);
    }

    .login-wrapper {
        min-height: 100vh;
        width: 100vw;
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
        overflow: hidden;
        background: radial-gradient(circle at center, #ffffff 0%, var(--color-bg-warm, #faf8f5) 100%);
        padding: 1.5rem;
    }

    /* Soft ambient background glows using Royal Purple and Desert Gold */
    .glow-sphere {
        position: absolute;
        border-radius: 50%;
        filter: blur(140px);
        opacity: 0.08;
        pointer-events: none;
        z-index: 1;
    }

    .ambient-1 {
        width: 450px;
        height: 450px;
        background: #5b21b6; /* Purple */
        top: -10%;
        left: -10%;
    }

    .ambient-2 {
        width: 500px;
        height: 500px;
        background: #d97706; /* Gold */
        bottom: -15%;
        right: -10%;
    }

    /* Glassmorphic Panel Design but Light/Premium */
    .glass-panel {
        background: rgba(255, 255, 255, 0.85);
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
        border: 1px solid rgba(91, 33, 182, 0.08);
        border-radius: 1.5rem;
        padding: 3rem 2.5rem;
        width: 100%;
        max-width: 460px;
        box-shadow: 0 20px 50px -12px rgba(91, 33, 182, 0.08), 0 4px 12px -2px rgba(91, 33, 182, 0.04);
        position: relative;
        z-index: 10;
        overflow: hidden;
    }

    /* Sleek Purple Accent Rim */
    .gold-rim {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 4px;
        background: linear-gradient(90deg, transparent, #5b21b6, #d97706, transparent);
    }

    .login-header {
        text-align: center;
        margin-bottom: 2.25rem;
    }

    .shield-badge {
        width: 56px;
        height: 56px;
        border-radius: 1rem;
        background: rgba(91, 33, 182, 0.06);
        border: 1px solid rgba(91, 33, 182, 0.15);
        display: flex;
        align-items: center;
        justify-content: center;
        margin: 0 auto 1.25rem auto;
        color: #5b21b6;
        box-shadow: 0 8px 24px -6px rgba(91, 33, 182, 0.25);
    }

    h1 {
        margin: 0 0 0.5rem 0;
        font-size: 1.85rem;
        font-weight: 800;
        letter-spacing: 1px;
    }

    .gold-text {
        background: linear-gradient(135deg, #5b21b6 0%, #d97706 100%);
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .white-text {
        color: #1e293b;
    }

    .subtitle {
        color: #64748b;
        font-size: 0.9rem;
        font-weight: 500;
        margin: 0;
        letter-spacing: 0.5px;
    }

    /* Form Styles */
    .login-form {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .input-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .input-group label {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 1px;
        color: #475569;
        font-weight: 600;
        padding-left: 0.25rem;
    }

    .field-container {
        position: relative;
    }

    .field-container input {
        width: 100%;
        background: #ffffff;
        border: 1.5px solid #e2e8f0;
        border-radius: 0.75rem;
        padding: 0.85rem 1.25rem;
        color: #1e293b;
        font-size: 0.95rem;
        font-family: inherit;
        transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
        outline: none;
    }

    .field-container input::placeholder {
        color: #94a3b8;
    }

    .field-container input:focus {
        border-color: #5b21b6;
        background: #ffffff;
        box-shadow: 0 0 0 3px rgba(91, 33, 182, 0.15);
    }

    .field-container input:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    /* Buttons */
    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.75rem;
        padding: 0.9rem 1.5rem;
        border-radius: 0.75rem;
        font-size: 0.95rem;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
        border: none;
    }

    .btn-primary {
        background: linear-gradient(135deg, #5b21b6 0%, #4c1d95 100%);
        color: #ffffff;
        box-shadow: 0 10px 25px -5px rgba(91, 33, 182, 0.3);
    }

    .btn-primary:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 15px 30px -5px rgba(91, 33, 182, 0.45);
    }

    .btn-primary:active:not(:disabled) {
        transform: translateY(-1px);
    }

    .btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    /* Alerts */
    .alert {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.85rem 1.25rem;
        border-radius: 0.75rem;
        font-size: 0.85rem;
        line-height: 1.4;
        margin-bottom: 1.5rem;
        font-weight: 500;
    }

    .alert-danger {
        background: rgba(239, 68, 68, 0.08);
        border: 1px solid rgba(239, 68, 68, 0.2);
        color: #ef4444;
    }

    .alert-symbol {
        font-size: 1.1rem;
    }

    .alert-message {
        flex: 1;
    }

    /* Footer styling */
    .login-footer {
        text-align: center;
        margin-top: 2.5rem;
        border-top: 1px solid #e2e8f0;
        padding-top: 1.5rem;
    }

    .login-footer p {
        font-size: 0.75rem;
        color: #64748b;
        margin: 0 0 0.35rem 0;
        line-height: 1.4;
    }

    .login-footer .copyright {
        color: #94a3b8;
        font-weight: 500;
    }

    /* Animations */
    .spin {
        display: inline-flex;
        animation: rotate 1s linear infinite;
    }

    @keyframes rotate {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }

    .fade-in {
        animation: fadeIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(10px); }
        to { opacity: 1; transform: translateY(0); }
    }

    @media (max-width: 480px) {
        .glass-panel {
            padding: 2rem 1.5rem;
            border-radius: 1rem;
        }
    }
</style>
