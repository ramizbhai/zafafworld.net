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
}

export function trackPageView(url: string) {
    if (!browser || !window.gtag) return;
    
    window.gtag('config', GTAG_ID, {
        page_path: url,
    });
}
