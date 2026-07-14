export function getWaitTime(createdAtStr: string, i18n: any): string {
    const created = new Date(createdAtStr);
    const diffMs  = Date.now() - created.getTime();
    const diffHrs = Math.floor(diffMs / 3600000);
    
    if (diffHrs < 1) {
        return i18n.locale === 'ar' ? `${Math.max(1, Math.floor(diffMs / 60000))} د` : `${Math.max(1, Math.floor(diffMs / 60000))}m`;
    }
    if (diffHrs < 24) {
        return i18n.locale === 'ar' ? `${diffHrs} س` : `${diffHrs}h`;
    }
    return i18n.locale === 'ar' ? `${Math.floor(diffHrs / 24)} ي` : `${Math.floor(diffHrs / 24)}d`;
}

export function formatDate(dateStr: string, locale: 'ar' | 'en'): string {
    return new Date(dateStr).toLocaleDateString(
        locale === 'ar' ? 'ar-EG' : 'en-US',
        { month: 'short', day: 'numeric', year: 'numeric' }
    );
}

export function formatWeddingDate(dateStr: string | null | undefined, locale: 'ar' | 'en'): string {
    if (!dateStr) return locale === 'ar' ? 'لم يحدد' : 'TBD';
    return new Date(dateStr).toLocaleDateString(
        locale === 'ar' ? 'ar-EG' : 'en-US',
        { month: 'short', day: 'numeric', year: 'numeric' }
    );
}

export function getRatingPercent(score: number): number {
    return Math.max(0, (score / 5) * 100);
}

export function getInitials(name: string): string {
    if (!name) return '?';
    return name.split(' ').slice(0, 2).map(w => w[0]).join('').toUpperCase() || '?';
}

export function getWeeklyData(inquiries: any[], locale: 'ar' | 'en') {
    const labels = [];
    const counts = [];
    const now = new Date();
    
    const dayNamesAr = ['الأحد', 'الإثنين', 'الثلاثاء', 'الأربعاء', 'الخميس', 'الجمعة', 'السبت'];
    const dayNamesEn = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    
    for (let i = 6; i >= 0; i--) {
        const d = new Date(now.getTime());
        d.setDate(now.getDate() - i);
        
        const dayLabel = locale === 'ar' ? dayNamesAr[d.getDay()] : dayNamesEn[d.getDay()];
        labels.push(dayLabel);
        
        // Count inquiries created on this exact local calendar day
        const count = inquiries.filter((inq: any) => {
            if (!inq.created_at) return false;
            try {
                const inqDate = new Date(inq.created_at);
                return inqDate.getFullYear() === d.getFullYear() &&
                       inqDate.getMonth() === d.getMonth() &&
                       inqDate.getDate() === d.getDate();
            } catch (e) {
                return false;
            }
        }).length;
        
        counts.push(count);
    }
    
    return { labels, data: counts };
}
