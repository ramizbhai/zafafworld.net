export function stripHtml(html: string): string {
    if (!html) return "";
    return html.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim();
}

export function getTargetListingName(listingId: string, products: any[], locale: string) {
    if (!listingId) return "";
    const matched = products.find((p: any) => p.id === listingId);
    if (!matched) return "";
    return locale === "ar" ? matched.title_ar || matched.title_en : matched.title_en || matched.title_ar;
}

export function getStatusBadgeClass(status: string) {
    switch (status) {
        case "Active":    return "badge-active";
        case "Pending":   return "badge-pending";
        case "Scheduled": return "badge-scheduled";
        case "Paused":    return "badge-paused";
        case "Rejected":  return "badge-rejected";
        case "Cancelled": return "badge-cancelled";
        default:          return "badge-expired";
    }
}

export function getStatusLabel(status: string, locale: string) {
    const arMap: Record<string, string> = {
        Active: "نشط الآن",
        Pending: "قيد المراجعة",
        Scheduled: "مجدول",
        Paused: "موقوف مؤقتاً",
        Rejected: "مرفوض",
        Cancelled: "ملغي",
        Expired: "منتهي الصلاحية",
    };
    return locale === "ar" ? (arMap[status] ?? status) : status;
}

export function formatDate(dateStr: string, locale: string) {
    if (!dateStr) return "";
    return new Date(dateStr).toLocaleDateString(
        locale === "ar" ? "ar-EG" : "en-US",
        { month: "short", day: "numeric", year: "numeric" }
    );
}

export function focusTrap(node: HTMLElement) {
    const focusableElements = node.querySelectorAll(
        'a[href], area[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), button:not([disabled]), iframe, object, embed, [tabindex="0"], [contenteditable]'
    );
    const firstElement = focusableElements[0] as HTMLElement;
    const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

    function handleTab(e: KeyboardEvent) {
        if (e.key !== 'Tab') return;
        if (e.shiftKey) {
            if (document.activeElement === firstElement) { lastElement.focus(); e.preventDefault(); }
        } else {
            if (document.activeElement === lastElement) { firstElement.focus(); e.preventDefault(); }
        }
    }

    setTimeout(() => { if (firstElement) firstElement.focus(); }, 50);
    node.addEventListener('keydown', handleTab);
    return { destroy() { node.removeEventListener('keydown', handleTab); } };
}
