import { browser } from "$app/environment";

export const GTAG_ID = 'G-0C41X6YESG';

declare global {
    interface Window {
        dataLayer: any[];
        gtag: (...args: any[]) => void;
    }
}

export function initAnalytics() {
    if (!browser) return;
    
    window.dataLayer = window.dataLayer || [];
    window.gtag = function gtag() {
        window.dataLayer.push(arguments);
    };
    
    window.gtag('js', new Date());
    
    // Set send_page_view to false because we handle it manually for SPA routing
    window.gtag('config', GTAG_ID, {
        send_page_view: false
    });

    // Dynamically inject the google analytics script when browser is idle
    const injectScript = () => {
        // Prevent duplicate insertion
        if (document.querySelector(`script[src*="googletagmanager.com/gtag/js"]`)) return;
        const script = document.createElement('script');
        script.async = true;
        script.src = `https://www.googletagmanager.com/gtag/js?id=${GTAG_ID}`;
        document.head.appendChild(script);
    };

    if ('requestIdleCallback' in window) {
        (window as any).requestIdleCallback(() => injectScript(), { timeout: 3000 });
    } else {
        setTimeout(injectScript, 2000);
    }
}

export function trackPageView(url: string) {
    if (!browser || !window.gtag) return;
    
    window.gtag('config', GTAG_ID, {
        page_path: url,
    });
}
