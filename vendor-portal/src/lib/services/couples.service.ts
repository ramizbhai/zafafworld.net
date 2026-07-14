export const baselineDate = '2026-05-20';

export function getCountdown(weddingDateStr: string, locale: string): { days: number; text: string; isPast: boolean } {
    if (!weddingDateStr) return { days: 0, text: locale === 'ar' ? 'لا يوجد تاريخ' : 'No event date', isPast: false };
    
    const wDate = new Date(weddingDateStr);
    const baseline = new Date(baselineDate);
    
    const diffMs = wDate.getTime() - baseline.getTime();
    const days = Math.ceil(diffMs / (1000 * 60 * 60 * 24));
    
    if (days < 0) {
        const absDays = Math.abs(days);
        return { 
            days: absDays, 
            text: locale === 'ar' ? `منذ ${absDays} يوم` : `${absDays} days ago`, 
            isPast: true 
        };
    } else if (days === 0) {
        return { 
            days: 0, 
            text: locale === 'ar' ? 'اليوم!' : 'Happening today!', 
            isPast: false 
        };
    } else {
        return { 
            days, 
            text: locale === 'ar' ? `متبقي ${days} يوم` : `${days} days remaining`, 
            isPast: false 
        };
    }
}

export function isUrgentLead(weddingDateStr: string, locale: string): boolean {
    const countdown = getCountdown(weddingDateStr, locale);
    return !countdown.isPast && countdown.days > 0 && countdown.days <= 90;
}

export function getStatusLabel(status: string, i18n: any) {
    const t = i18n.t.couples;
    switch (status) {
        case 'new': return t.statusNew || 'New';
        case 'read': return i18n.locale === 'ar' ? 'مقروء' : 'Read';
        case 'done': return t.statusDone || 'Deal Closed';
        case 'expired': return t.statusExpired || 'Expired';
        case 'rejected': return t.statusRejected || 'Rejected';
        case 'negotiation': return t.statusNegot || 'Negotiation';
        case 'unreachable': return t.statusUnreach || 'Unreachable';
        case 'paid': return t.statusPaid || 'Paid';
        default: return status;
    }
}
