import { writable, derived } from 'svelte/store';

// ─── Language Types ────────────────────────────────────────────────────────
export type Lang = 'ar' | 'en';

// ─── Translation Dictionary ───────────────────────────────────────────────
const translations: Record<Lang, Record<string, string>> = {
  ar: {
    // Navigation
    'nav.dashboard':      'لوحة التحكم',
    'nav.vendors':        'الموردون',
    'nav.users':          'المستخدمون',
    'nav.bookings':       'الحجوزات',
    'nav.payments':       'المدفوعات',
    'nav.finance':        'المالية',
    'nav.commissions':    'العمولات',
    'nav.support':        'الدعم الفني',
    'nav.notifications':  'الإشعارات',
    'nav.roles':          'الصلاحيات',
    'nav.audit':          'سجل المراجعة',
    'nav.moderation':     'الإشراف',
    'nav.messages':       'مركز الرسائل',
    'nav.listings':       'القوائم',
    'nav.cms':            'إدارة المحتوى',
    'nav.marketing':      'التسويق',
    'nav.categories':     'التصنيفات',
    'nav.monitoring':     'مراقبة النظام',
    'nav.settings':       'الإعدادات',
    'nav.analytics':      'التحليلات',
    'nav.reviews':        'التقييمات',
    'nav.subscriptions':  'إدارة الاشتراكات',
    'nav.discover':       'الاستكشاف والمدونات',
    'nav.blog':           'إدارة المدونة',
    'nav.assistant':      'مساعد الأفراح / Afrah Assistant',

    // Sections
    'section.executive':  'وحدة التحكم التنفيذية',
    'section.moderation': 'إجراءات الإشراف',
    'section.users_mgmt': 'إدارة المستخدمين',
    'section.commerce':   'التجارة والمالية',
    'section.support':    'الدعم وإدارة علاقات العملاء',
    'section.content':    'المحتوى والتسويق',
    'section.system':     'النظام',

    // Header
    'header.search_placeholder': 'البحث في لوحة التحكم...',
    'header.live':               'جلسة النظام نشطة',
    'header.db_live':            'قاعدة البيانات نشطة',
    'header.logout':             'تسجيل الخروج',
    'header.notifications':      'الإشعارات',
    'header.lang_switch':        'English',

    // Dashboard
    'dash.title':                'لوحة التحكم الرئيسية',
    'dash.subtitle':             'نظرة عامة في الوقت الفعلي على جميع عمليات المنصة',
    'dash.total_revenue':        'إجمالي الإيرادات',
    'dash.active_vendors':       'الموردون النشطون',
    'dash.total_bookings':       'إجمالي الحجوزات',
    'dash.total_users':          'المستخدمون',
    'dash.pending_approvals':    'موافقات معلقة',
    'dash.support_queue':        'قائمة انتظار الدعم',
    'dash.total_inquiries':      'إجمالي الاستفسارات',
    'dash.active_subscriptions': 'الاشتراكات النشطة',
    'dash.pending_reviews':      'التقييمات المعلقة',
    'dash.vs_last_month':        'مقارنة بالشهر الماضي',
    'dash.action_required':      'تتطلب إجراءً فورياً',
    'dash.market_matrix':        'مصفوفة الأسواق العربية',
    'dash.market_matrix_sub':    'كثافة التسجيل والأداء المالي عبر الأسواق',
    'dash.moderation_queue':     'قائمة انتظار الإشراف',
    'dash.moderation_queue_sub': 'العناصر ذات الأولوية العالية التي تنتظر مراجعة المشرف',
    'dash.recent_activity':      'النشاط الأخير',
    'dash.revenue_trend':        'اتجاه الإيرادات',

    // Common
    'common.review':     'مراجعة',
    'common.approve':    'موافقة',
    'common.reject':     'رفض',
    'common.search':     'البحث...',
    'common.filter':     'فلترة',
    'common.export':     'تصدير',
    'common.add':        'إضافة',
    'common.edit':       'تعديل',
    'common.delete':     'حذف',
    'common.view':       'عرض',
    'common.save':       'حفظ',
    'common.cancel':     'إلغاء',
    'common.confirm':    'تأكيد',
    'common.loading':    'جاري التحميل...',
    'common.no_data':    'لا توجد بيانات',
    'common.all':        'الكل',
    'common.active':     'نشط',
    'common.inactive':   'غير نشط',
    'common.pending':    'معلق',
    'common.approved':   'موافق عليه',
    'common.rejected':   'مرفوض',
    'common.critical':   'حرج',
    'common.vendor':     'مورد',
    'common.user':       'مستخدم',
    'common.amount':     'المبلغ',
    'common.date':       'التاريخ',
    'common.status':     'الحالة',
    'common.actions':    'الإجراءات',
    'common.name':       'الاسم',
    'common.email':      'البريد الإلكتروني',
    'common.phone':      'رقم الهاتف',
    'common.city':       'المدينة',
    'common.category':   'الفئة',
    'common.today':      'اليوم',
    'common.this_week':  'هذا الأسبوع',
    'common.this_month': 'هذا الشهر',
    'common.prev':       'السابق',
    'common.next':       'التالي',
    'common.of':         'من',
    'common.rows':       'سطور',

    // Profile
    'profile.super_admin': 'مدير النظام الرئيسي',
    'profile.admin':       'مدير',
    'profile.moderator':   'مشرف',
  },

  en: {
    // Navigation
    'nav.dashboard':      'Dashboard',
    'nav.vendors':        'Vendors',
    'nav.users':          'Users',
    'nav.bookings':       'Bookings',
    'nav.payments':       'Payments',
    'nav.finance':        'Finance',
    'nav.commissions':    'Commissions',
    'nav.support':        'Support',
    'nav.notifications':  'Notifications',
    'nav.roles':          'Roles & Permissions',
    'nav.audit':          'Audit Log',
    'nav.moderation':     'Moderation',
    'nav.messages':       'Messages Center',
    'nav.listings':       'Listings',
    'nav.cms':            'Content Management',
    'nav.marketing':      'Marketing',
    'nav.categories':     'Categories',
    'nav.monitoring':     'System Monitor',
    'nav.settings':       'Settings',
    'nav.analytics':      'Analytics',
    'nav.reviews':        'Reviews',
    'nav.subscriptions':  'Subscriptions Management',
    'nav.discover':       'Discover & Blogs',
    'nav.blog':           'Blog Management',
    'nav.assistant':      'Afrah Assistant / مساعد الأفراح',

    // Sections
    'section.executive':  'Executive Console',
    'section.moderation': 'Moderation Actions',
    'section.users_mgmt': 'User Management',
    'section.commerce':   'Commerce & Finance',
    'section.support':    'Support & CRM',
    'section.content':    'Content & Marketing',
    'section.system':     'System',

    // Header
    'header.search_placeholder': 'Search control panel...',
    'header.live':               'System Session Live',
    'header.db_live':            'Database Active',
    'header.logout':             'Sign Out',
    'header.notifications':      'Notifications',
    'header.lang_switch':        'العربية',

    // Dashboard
    'dash.title':                'Executive Dashboard',
    'dash.subtitle':             'Real-time overview of all platform operations',
    'dash.total_revenue':        'Total Revenue',
    'dash.active_vendors':       'Active Vendors',
    'dash.total_bookings':       'Total Bookings',
    'dash.total_users':          'Total Users',
    'dash.pending_approvals':    'Pending Approvals',
    'dash.support_queue':        'Support Queue',
    'dash.total_inquiries':      'Total Inquiries',
    'dash.active_subscriptions': 'Active Subscriptions',
    'dash.pending_reviews':      'Pending Reviews',
    'dash.vs_last_month':        'vs last month',
    'dash.action_required':      'Requires immediate action',
    'dash.market_matrix':        'Cross-Border Market Matrix',
    'dash.market_matrix_sub':    'Jurisdictional registration densities and financial performance',
    'dash.moderation_queue':     'Moderation Queue',
    'dash.moderation_queue_sub': 'High priority items awaiting administrator review',
    'dash.recent_activity':      'Recent Activity',
    'dash.revenue_trend':        'Revenue Trend',

    // Common
    'common.review':     'Review',
    'common.approve':    'Approve',
    'common.reject':     'Reject',
    'common.search':     'Search...',
    'common.filter':     'Filter',
    'common.export':     'Export',
    'common.add':        'Add',
    'common.edit':       'Edit',
    'common.delete':     'Delete',
    'common.view':       'View',
    'common.save':       'Save',
    'common.cancel':     'Cancel',
    'common.confirm':    'Confirm',
    'common.loading':    'Loading...',
    'common.no_data':    'No data found',
    'common.all':        'All',
    'common.active':     'Active',
    'common.inactive':   'Inactive',
    'common.pending':    'Pending',
    'common.approved':   'Approved',
    'common.rejected':   'Rejected',
    'common.critical':   'Critical',
    'common.vendor':     'Vendor',
    'common.user':       'User',
    'common.amount':     'Amount',
    'common.date':       'Date',
    'common.status':     'Status',
    'common.actions':    'Actions',
    'common.name':       'Name',
    'common.email':      'Email',
    'common.phone':      'Phone',
    'common.city':       'City',
    'common.category':   'Category',
    'common.today':      'Today',
    'common.this_week':  'This Week',
    'common.this_month': 'This Month',
    'common.prev':       'Previous',
    'common.next':       'Next',
    'common.of':         'of',
    'common.rows':       'rows',

    // Profile
    'profile.super_admin': 'Super Administrator',
    'profile.admin':       'Administrator',
    'profile.moderator':   'Moderator',
  }
};

// ─── Language Store ────────────────────────────────────────────────────────
function createLangStore() {
  // Initialize from localStorage if available, default to Arabic
  const storedLang = typeof localStorage !== 'undefined'
    ? (localStorage.getItem('zafaf_lang') as Lang | null)
    : null;

  const { subscribe, set, update } = writable<Lang>(storedLang ?? 'ar');

  return {
    subscribe,
    set: (lang: Lang) => {
      // Update DOM direction
      if (typeof document !== 'undefined') {
        document.documentElement.lang = lang;
        document.documentElement.dir  = lang === 'ar' ? 'rtl' : 'ltr';
      }
      // Persist
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('zafaf_lang', lang);
      }
      set(lang);
    },
    toggle: () => {
      update(current => {
        const next: Lang = current === 'ar' ? 'en' : 'ar';
        if (typeof document !== 'undefined') {
          document.documentElement.lang = next;
          document.documentElement.dir  = next === 'ar' ? 'rtl' : 'ltr';
        }
        if (typeof localStorage !== 'undefined') {
          localStorage.setItem('zafaf_lang', next);
        }
        return next;
      });
    }
  };
}

export const lang = createLangStore();

// ─── Translation Function ──────────────────────────────────────────────────
export const t = derived(lang, ($lang) => {
  return (key: string, fallback?: string): string => {
    return translations[$lang]?.[key] ?? fallback ?? key;
  };
});

// ─── Direction Derived Store ───────────────────────────────────────────────
export const dir = derived(lang, ($lang) => $lang === 'ar' ? 'rtl' : 'ltr');
export const isRTL = derived(lang, ($lang) => $lang === 'ar');
