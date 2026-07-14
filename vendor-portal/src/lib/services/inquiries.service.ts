export function getStatusLabel(status: string, i18n: any) {
    switch (status) {
        case 'pending': return i18n.locale === 'ar' ? 'قيد الانتظار' : 'Pending';
        case 'replied': return i18n.locale === 'ar' ? 'تم الرد' : 'Replied';
        case 'declined': return i18n.locale === 'ar' ? 'مرفوض' : 'Declined';
        default: return status;
    }
}

export function getStatusBadgeClass(status: string) {
    switch (status) {
        case 'pending': return 'badge-pending';
        case 'replied': return 'badge-replied';
        case 'declined': return 'badge-declined';
        default: return '';
    }
}

export function formatInquiryDate(dateStr: string, locale: string) {
    if (!dateStr) return '';
    try {
        return new Date(dateStr).toLocaleDateString(locale === 'ar' ? 'ar-EG' : 'en-US', {
            year: 'numeric',
            month: 'long',
            day: 'numeric'
        });
    } catch {
        return dateStr;
    }
}
