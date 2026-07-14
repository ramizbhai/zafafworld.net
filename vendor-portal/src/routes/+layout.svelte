<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { page, navigating } from "$app/stores";
    import { setI18nContext } from "$lib/i18n/i18n.svelte";
    import { uiStore } from "$lib/stores/ui.svelte";
    import Loading from "$lib/components/Loading.svelte";
    import "../app.css";

    let { data, children } = $props();

    // Call synchronously in script setup to initialize B2B i18n context
    const i18n = setI18nContext(untrack(() => data.locale as "ar" | "en") || "ar");

    // Sync state dynamically when route locale loads
    $effect(() => {
        if (data.locale) {
            i18n.locale = data.locale as "ar" | "en";
        }
    });

    // Derived value to check if active route is the cockpit dashboard
    let isDashboard = $derived($page.url.pathname.startsWith("/dashboard"));
</script>

<Loading show={!!$navigating || uiStore.globalLoading} />

<div
    class="app-container"
    class:dashboard-mode={isDashboard}
    dir={i18n.isRtl ? "rtl" : "ltr"}
>
    {#if !isDashboard}
        <!-- Sleek Glassmorphic Header -->
        <header class="navbar">
            <a href="/" class="logo-container">
                <img src="/logo.webp" alt="ZafafWorld" class="logo-image" />
                <div class="logo-text-stack">
                    <span class="logo-title"
                        >{i18n.locale === "ar" ? "زفاف" : "ZAFAF"}</span
                    >
                    <span class="logo-subtitle"
                        >{i18n.locale === "ar" ? "وورلد" : "WORLD"}</span
                    >
                </div>
                <span class="badge"
                    >{i18n.locale === "ar"
                        ? "بوابة الموردين"
                        : "Vendor Hub"}</span
                >
            </a>
            <div class="nav-links">
                <a
                    href="https://zafafworld.net"
                    target="_blank"
                    rel="noreferrer"
                    class="nav-link"
                >
                    {i18n.locale === "ar" ? "البوابة الرئيسية" : "Main Portal"}
                </a>
                <span class="divider">|</span>
                <button
                    type="button"
                    onclick={() => {
                        i18n.locale = i18n.locale === "ar" ? "en" : "ar";
                    }}
                    class="lang-btn"
                >
                    {i18n.locale === "ar" ? "English" : "العربية"}
                </button>
                <span class="divider">|</span>
                <span class="status-indicator">
                    <span class="dot pulse"></span>
                    {i18n.locale === "ar" ? "تسجيل آمن" : "Secure Registration"}
                </span>
            </div>
        </header>
    {/if}

    <!-- Main Content Area -->
    <main class="content-wrapper">
        {@render children()}
    </main>

    {#if !isDashboard}
        <!-- Sleek Minimal Footer -->
        <footer class="footer">
            <div class="footer-content">
                <p>
                    {i18n.locale === "ar"
                        ? "© 2026 زفاف وورلد. جميع الحقوق محفوظة. النظام المتكامل لإدارة خدمات وقاعات الأفراح."
                        : "© 2026 ZafafWorld. All rights reserved. Premium Wedding Registry Ecosystem."}
                </p>
                <div class="footer-links">
                    <a href="#privacy"
                        >{i18n.locale === "ar"
                            ? "سياسة الخصوصية"
                            : "Privacy"}</a
                    >
                    <a href="#terms"
                        >{i18n.locale === "ar"
                            ? "شروط الخدمة"
                            : "Terms of Service"}</a
                    >
                    <a href="#support"
                        >{i18n.locale === "ar"
                            ? "دعم الشركاء"
                            : "Partner Support"}</a
                    >
                </div>
            </div>
        </footer>
    {/if}
</div>

<style>
    /* Global Reset & Base Variables */
    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
        background-color: var(--color-bg-warm); /* Warm Ivory Background */
        color: var(--color-text-dark); /* Slate Charcoal text */
        font-family:
            "Outfit",
            "Cairo",
            system-ui,
            -apple-system,
            sans-serif;
        -webkit-font-smoothing: antialiased;
        overflow-x: hidden;
    }

    :global(*) {
        box-sizing: border-box;
    }

    /* Scrollbar Styling */
    :global(::-webkit-scrollbar) {
        width: 8px;
    }
    :global(::-webkit-scrollbar-track) {
        background: var(--color-bg-warm);
    }
    :global(::-webkit-scrollbar-thumb) {
        background: #cbd5e1;
        border-radius: 4px;
    }
    :global(::-webkit-scrollbar-thumb:hover) {
        background: #94a3b8;
    }

    /* CSS Custom Properties / Design Tokens */
    .app-container {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        background: radial-gradient(
            circle at 50% -20%,
            #ffffff 0%,
            var(--color-bg-warm) 70%
        );
        position: relative;
    }

    .app-container::before {
        content: "";
        position: absolute;
        width: 100%;
        height: 100%;
        top: 0;
        left: 0;
        background-image: radial-gradient(
            rgba(217, 119, 6, 0.03) 1px,
            transparent 1px
        );
        background-size: 24px 24px;
        pointer-events: none;
        z-index: 1;
    }

    /* Navbar styling */
    .navbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.25rem 2.5rem;
        background: rgba(250, 248, 245, 0.85);
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        border-bottom: 1px solid rgba(0, 0, 0, 0.05);
        position: sticky;
        top: 0;
        z-index: 100;
    }

    /* Logo styling aligned with client portal design */
    .logo-container {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        text-decoration: none;
    }

    .logo-image {
        height: 2.5rem;
        width: auto;
        object-fit: contain;
        flex-shrink: 0;
        transition: transform 0.3s ease;
    }

    .logo-container:hover .logo-image {
        transform: scale(1.05);
    }

    .logo-text-stack {
        display: flex;
        flex-direction: column;
        line-height: 1;
    }

    .logo-title {
        font-family: "Outfit", sans-serif;
        font-size: 1.1rem;
        font-weight: 800;
        color: var(--color-text-dark);
        letter-spacing: 0.5px;
    }

    .logo-subtitle {
        font-size: 0.5rem;
        color: var(--color-primary);
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 1.5px;
        margin-top: 0.15rem;
    }

    .badge {
        font-size: 0.65rem;
        background: rgba(217, 119, 6, 0.15);
        color: var(--color-secondary);
        border: 1px solid rgba(217, 119, 6, 0.3);
        padding: 0.15rem 0.5rem;
        border-radius: 9999px;
        margin-inline-start: 0.5rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    /* Language Switcher Styling */
    .lang-btn {
        background: none;
        border: none;
        color: #475569;
        font-family: inherit;
        font-size: 0.9rem;
        font-weight: 500;
        cursor: pointer;
        padding: 0;
        transition: color 0.2s ease;
    }

    .lang-btn:hover {
        color: var(--color-primary);
    }

    .nav-links {
        display: flex;
        align-items: center;
        gap: 1rem;
        font-size: 0.9rem;
    }

    .nav-link {
        color: #475569;
        text-decoration: none;
        transition: color 0.2s ease;
        font-weight: 500;
    }

    .nav-link:hover {
        color: var(--color-primary);
    }

    .divider {
        color: rgba(0, 0, 0, 0.1);
    }

    .status-indicator {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: #10b981;
        font-weight: 600;
        font-size: 0.85rem;
        background: rgba(16, 185, 129, 0.1);
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        border: 1px solid rgba(16, 185, 129, 0.2);
    }

    .dot {
        width: 6px;
        height: 6px;
        background-color: #10b981;
        border-radius: 50%;
    }

    .pulse {
        animation: blink 2s infinite ease-in-out;
    }

    @keyframes blink {
        0%,
        100% {
            opacity: 0.4;
        }
        50% {
            opacity: 1;
        }
    }

    /* Content Wrapper */
    .content-wrapper {
        flex: 1;
        display: flex;
        flex-direction: column;
        z-index: 10;
    }

    /* Footer styling */
    .footer {
        background: rgba(250, 248, 245, 0.95);
        border-top: 1px solid rgba(0, 0, 0, 0.05);
        padding: 1.5rem 2.5rem;
        text-align: center;
        font-size: 0.8rem;
        color: #64748b;
        z-index: 10;
    }

    .footer-content {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
        max-width: 1200px;
        margin: 0 auto;
        width: 100%;
    }

    .footer-content p {
        margin: 0;
    }

    .footer-links {
        display: flex;
        gap: 1.5rem;
    }

    .footer-links a {
        color: #475569;
        text-decoration: none;
        transition: color 0.2s ease;
    }

    .footer-links a:hover {
        color: #94a3b8;
    }

    /* Responsive adjustments */
    @media (max-width: 768px) {
        .navbar {
            padding: 1rem 1.5rem;
        }
        .footer {
            padding: 1.5rem;
        }
        .footer-content {
            flex-direction: column;
            text-align: center;
        }
        .footer-links {
            justify-content: center;
        }
    }
</style>
