import {
    LayoutDashboard, Users, User, Gift, Star, FileText, UserCog,
    MessageSquare, CheckSquare, Image, Package, Sparkles, Building2
} from 'lucide-svelte';

export function getNavItems(i18n: any, isOwner: boolean) {
    return [
        {
            id: 'dashboard',
            href: '/dashboard',
            label: i18n.t.nav.dashboard,
            icon: LayoutDashboard,
            exact: true
        },
        {
            id: 'products',
            href: '/dashboard/products',
            label: i18n.t.nav.listings,
            icon: Building2,
            match: ['/dashboard/products']
        },
        {
            id: 'vendor_inquiries',
            href: '/dashboard/inquiries',
            label: i18n.t.nav.inquiries,
            icon: MessageSquare,
            match: ['/dashboard/inquiries']
        },
        {
            id: 'vendor_inbox',
            href: '/dashboard/inbox',
            label: i18n.t.nav.inbox,
            icon: MessageSquare,
            match: ['/dashboard/inbox']
        },
        {
            id: 'couples',
            href: '/dashboard/couples',
            label: i18n.t.nav.couples,
            icon: Users,
            match: ['/dashboard/couples', '/dashboard/inquiries']
        },
        {
            id: 'offers',
            href: '/dashboard/offers',
            label: i18n.t.nav.offers,
            icon: Gift,
            match: ['/dashboard/offers', '/dashboard/packages']
        },
        {
            id: 'reviews',
            href: '/dashboard/reviews',
            label: i18n.t.nav.reviews,
            icon: Star
        },
        {
            id: 'pages',
            href: '/dashboard/pages',
            label: i18n.t.nav.pages,
            icon: FileText,
            match: ['/dashboard/pages', '/dashboard/settings']
        },
        {
            id: 'profile',
            href: '/dashboard/profile',
            label: i18n.locale === 'ar' ? 'الملف الشخصي' : 'Profile',
            icon: User,
            match: ['/dashboard/profile']
        },
        {
            id: 'gallery',
            href: '/dashboard/gallery',
            label: i18n.t.nav.gallery,
            icon: Image
        },
        ...(isOwner ? [
            {
                id: 'users',
                href: '/dashboard/users',
                label: i18n.t.nav.users,
                icon: UserCog
            }
        ] : []),
        {
            id: 'templates',
            href: '/dashboard/templates',
            label: i18n.t.nav.templates,
            icon: MessageSquare
        },
        {
            id: 'tasks',
            href: '/dashboard/tasks',
            label: i18n.t.nav.tasks,
            icon: CheckSquare
        },
        ...(isOwner ? [
            {
                id: 'pricing',
                href: '/dashboard/pricing',
                label: i18n.t.nav.pricing,
                icon: Package
            },
            {
                id: 'subscription',
                href: '/dashboard/subscription',
                label: i18n.locale === 'ar' ? 'الاشتراكات' : 'Subscriptions',
                icon: Sparkles
            }
        ] : []),
    ];
}

export function getBreadcrumbsAndTitle(i18n: any, currentRoute: string) {
    const t = i18n.t;
    const base = { label: t.layout.home, href: '/dashboard' };
    const map: Record<string, any> = {
        '/dashboard':                        { title: t.nav.dashboard,              crumbs: [base, { label: t.nav.dashboard }] },
        '/dashboard/couples':                { title: t.nav.couples,                crumbs: [base, { label: t.nav.couples }] },
        '/dashboard/inquiries':              { title: t.nav.inquiries,              crumbs: [base, { label: t.nav.inquiries }] },
        '/dashboard/inbox':                  { title: t.nav.inbox,                  crumbs: [base, { label: t.nav.inbox }] },
        '/dashboard/offers':                 { title: t.nav.offers,                 crumbs: [base, { label: t.nav.offers }] },
        '/dashboard/reviews':                { title: t.nav.reviews,                crumbs: [base, { label: t.nav.reviews }] },
        '/dashboard/gallery':                { title: t.nav.gallery,                crumbs: [base, { label: t.nav.gallery }] },
        '/dashboard/pages':                  { title: t.nav.pages,                  crumbs: [base, { label: t.nav.pages }] },
        '/dashboard/users':                  { title: t.nav.users,                  crumbs: [base, { label: t.nav.users }] },
        '/dashboard/profile':                { title: i18n.locale === 'ar' ? 'الملف الشخصي' : 'Profile', crumbs: [base, { label: i18n.locale === 'ar' ? 'الملف الشخصي' : 'Profile' }] },
        '/dashboard/templates':              { title: t.nav.templates,              crumbs: [base, { label: t.nav.templates }] },
        '/dashboard/tasks':                  { title: t.nav.tasks,                  crumbs: [base, { label: t.nav.tasks }] },
        '/dashboard/pricing':                { title: t.nav.pricing,                crumbs: [base, { label: t.nav.pricing }] },
        '/dashboard/products':               { title: t.nav.listings,               crumbs: [base, { label: t.nav.listings }] },
        '/dashboard/products/new':           { title: t.layout.addHall,              crumbs: [base, { label: t.nav.listings, href: '/dashboard/products' }, { label: t.layout.newWord }] },
        '/dashboard/products/[id]':          { title: t.layout.editHall,             crumbs: [base, { label: t.nav.listings, href: '/dashboard/products' }, { label: t.layout.editWord }] },
        '/dashboard/statistics/page':        { title: t.nav.statisticsPage,         crumbs: [base, { label: t.nav.statistics, href: '/dashboard/statistics/page' }, { label: t.nav.statisticsPage }] },
        '/dashboard/statistics/competitors': { title: t.nav.statisticsCompetitors,  crumbs: [base, { label: t.nav.statistics, href: '/dashboard/statistics/page' }, { label: t.nav.statisticsCompetitors }] },
    };
    return map[currentRoute] ?? { title: t.nav.dashboard, crumbs: [base] };
}

export function isNavItemActive(item: any, currentPath: string): boolean {
    if (item.exact) return currentPath === item.href;
    if (item.match) return item.match.some((p: string) => currentPath.startsWith(p));
    return currentPath.startsWith(item.href);
}
