export function isUserOwner(user: any): boolean {
    if (!user || !user.scopes) return false;
    return user.scopes.includes('owner');
}

export function getUserName(user: any, locale: 'ar' | 'en'): string {
    if (locale === 'ar') {
        return user?.last_name || user?.first_name || 'شريك';
    }
    return user?.first_name || user?.last_name || 'Partner';
}

export function getUserInitial(user: any, locale: 'ar' | 'en'): string {
    if (locale === 'ar') {
        const name = user?.last_name ?? user?.first_name ?? 'ش';
        return name.charAt(0);
    }
    const name = user?.first_name ?? 'P';
    return name.charAt(0);
}

export function getVendorInitials(vendor: any, locale: 'ar' | 'en'): string {
    if (locale === 'ar') {
        return vendor?.name_ar?.charAt(0) ?? 'ز';
    }
    return vendor?.name_en?.charAt(0) ?? 'V';
}
