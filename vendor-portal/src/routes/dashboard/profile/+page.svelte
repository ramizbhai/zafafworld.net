<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { Settings, User, Mail, Phone, MapPin, Building, Lock, Save, Edit3 } from 'lucide-svelte';
    import { getContext } from 'svelte';
    import { page } from '$app/stores';
    import { getApiUrl } from '$lib/utils/api';
    import { uiStore } from '$lib/stores/ui.svelte';

    let { data } = $props();
    const i18n = getI18n();

    // Vendor data from the global layout stream
    let vendor = $state<any>({});
    
    // Form state
    let successMessage = $state('');
    let errorMessage = $state('');

    // Profile editable state
    let isEditingProfile = $state(false);

    let profileData = $state({
        name_en: '',
        name_ar: '',
        email: '',
        phone: '',
        address_en: '',
        address_ar: '',
        description_en: '',
        description_ar: '',
        cr_number: '',
        coordinator_name_en: '',
        coordinator_name_ar: '',
        coordinator_phone: '',
        version: 0
    });

    let passwordData = $state({
        old_password: '',
        new_password: '',
        confirm_password: ''
    });

    $effect(() => {
        if (data.streamed?.telemetry) {
            data.streamed.telemetry.then((res: any) => {
                if (res?.data?.vendor) {
                    vendor = res.data.vendor;
                    profileData = {
                        name_en: vendor.name_en || '',
                        name_ar: vendor.name_ar || '',
                        email: vendor.email || '',
                        phone: vendor.phone || '',
                        address_en: vendor.address_en || '',
                        address_ar: vendor.address_ar || '',
                        description_en: vendor.description_en || '',
                        description_ar: vendor.description_ar || '',
                        cr_number: vendor.cr_number || '',
                        coordinator_name_en: vendor.coordinator_name_en || '',
                        coordinator_name_ar: vendor.coordinator_name_ar || '',
                        coordinator_phone: vendor.coordinator_phone || '',
                        version: vendor.version || 0
                    };
                }
            });
        }
    });

    async function updateProfile(e: Event) {
        e.preventDefault();
        uiStore.setLoading(true);
        errorMessage = '';
        successMessage = '';

        try {
            const res = await fetch(getApiUrl('/api/v1/vendor/profile'), {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${data.sessionToken}`
                },
                body: JSON.stringify(profileData)
            });

            if (res.ok) {
                successMessage = i18n.locale === 'ar' ? 'تم تحديث الملف الشخصي بنجاح' : 'Profile updated successfully';
                isEditingProfile = false;
                // Increment local version
                profileData.version += 1;
            } else {
                const err = await res.json();
                errorMessage = err.message || (i18n.locale === 'ar' ? 'حدث خطأ أثناء التحديث' : 'Failed to update profile');
            }
        } catch (err) {
            errorMessage = i18n.locale === 'ar' ? 'حدث خطأ في الشبكة' : 'Network error occurred';
        } finally {
            uiStore.setLoading(false);
        }
    }

    async function changePassword(e: Event) {
        e.preventDefault();
        
        if (passwordData.new_password !== passwordData.confirm_password) {
            errorMessage = i18n.locale === 'ar' ? 'كلمات المرور غير متطابقة' : 'Passwords do not match';
            return;
        }

        if (passwordData.new_password.length < 8) {
            errorMessage = i18n.locale === 'ar' ? 'يجب أن تتكون كلمة المرور من 8 أحرف على الأقل' : 'Password must be at least 8 characters long';
            return;
        }

        uiStore.setLoading(true);
        errorMessage = '';
        successMessage = '';

        try {
            const res = await fetch(getApiUrl('/api/v1/auth/change-password'), {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${data.sessionToken}`
                },
                body: JSON.stringify({
                    old_password: passwordData.old_password,
                    new_password: passwordData.new_password
                })
            });

            if (res.ok) {
                successMessage = i18n.locale === 'ar' ? 'تم تغيير كلمة المرور بنجاح' : 'Password changed successfully';
                passwordData = { old_password: '', new_password: '', confirm_password: '' };
            } else {
                const err = await res.json();
                errorMessage = err.message || (i18n.locale === 'ar' ? 'حدث خطأ أثناء تغيير كلمة المرور' : 'Failed to change password');
            }
        } catch (err) {
            errorMessage = i18n.locale === 'ar' ? 'حدث خطأ في الشبكة' : 'Network error occurred';
        } finally {
            uiStore.setLoading(false);
        }
    }

</script>

<svelte:head>
    <title>{i18n.locale === 'ar' ? 'الملف الشخصي | زفاف وورلد' : 'Profile | ZafafWorld'}</title>
</svelte:head>

<div class="profile-container animate-fade-in" dir={i18n.isRtl ? 'rtl' : 'ltr'}>
    <header class="page-header">
        <div class="header-content">
            <h1 class="page-title">
                <User size={28} class="text-primary" />
                {i18n.locale === 'ar' ? 'إدارة الملف الشخصي' : 'Profile Management'}
            </h1>
            <p class="page-subtitle">
                {i18n.locale === 'ar' ? 'قم بتحديث بيانات شركتك وإعدادات الأمان الخاصة بك' : 'Update your company details and security settings'}
            </p>
        </div>
    </header>

    {#if errorMessage}
        <div class="alert alert-error">
            {errorMessage}
        </div>
    {/if}

    {#if successMessage}
        <div class="alert alert-success">
            {successMessage}
        </div>
    {/if}

    <div class="grid-layout">
        <!-- Main Profile Details -->
        <div class="card profile-card">
            <div class="card-header">
                <h2>
                    <Building size={20} />
                    {i18n.locale === 'ar' ? 'بيانات الشركة' : 'Company Details'}
                </h2>
                <button class="btn btn-outline" onclick={() => isEditingProfile = !isEditingProfile}>
                    <Edit3 size={16} />
                    {i18n.locale === 'ar' ? (isEditingProfile ? 'إلغاء' : 'تعديل') : (isEditingProfile ? 'Cancel' : 'Edit')}
                </button>
            </div>
            <div class="card-body">
                <form onsubmit={updateProfile}>
                    <div class="form-grid">
                        <div class="form-group">
                            <label for="name_ar">{i18n.locale === 'ar' ? 'الاسم (عربي)' : 'Name (Arabic)'}</label>
                            <input id="name_ar" type="text" bind:value={profileData.name_ar} class="input-field" disabled={!isEditingProfile} required />
                        </div>
                        <div class="form-group">
                            <label for="name_en">{i18n.locale === 'ar' ? 'الاسم (إنجليزي)' : 'Name (English)'}</label>
                            <input id="name_en" type="text" bind:value={profileData.name_en} class="input-field" disabled={!isEditingProfile} required />
                        </div>
                        <div class="form-group">
                            <label><Mail size={16} /> {i18n.locale === 'ar' ? 'البريد الإلكتروني' : 'Email'}</label>
                            <input type="email" bind:value={profileData.email} class="input-field" disabled={!isEditingProfile} />
                        </div>
                        <div class="form-group">
                            <label><Phone size={16} /> {i18n.locale === 'ar' ? 'رقم الهاتف' : 'Phone Number'}</label>
                            <input type="tel" bind:value={profileData.phone} class="input-field" disabled={!isEditingProfile} />
                        </div>
                        <div class="form-group full-width">
                            <label><MapPin size={16} /> {i18n.locale === 'ar' ? 'العنوان' : 'Address'}</label>
                            <div class="grid-2">
                                <input type="text" bind:value={profileData.address_ar} placeholder={i18n.locale === 'ar' ? 'العنوان (عربي)' : 'Address (Arabic)'} class="input-field" disabled={!isEditingProfile} />
                                <input type="text" bind:value={profileData.address_en} placeholder={i18n.locale === 'ar' ? 'العنوان (إنجليزي)' : 'Address (English)'} class="input-field" disabled={!isEditingProfile} />
                            </div>
                        </div>
                        <div class="form-group full-width">
                            <label for="description_ar">{i18n.locale === 'ar' ? 'الوصف' : 'Description'}</label>
                            <div class="grid-2">
                                <textarea id="description_ar" bind:value={profileData.description_ar} placeholder={i18n.locale === 'ar' ? 'الوصف (عربي)' : 'Description (Arabic)'} class="input-field textarea" disabled={!isEditingProfile} rows="3"></textarea>
                                <textarea bind:value={profileData.description_en} placeholder={i18n.locale === 'ar' ? 'الوصف (إنجليزي)' : 'Description (English)'} class="input-field textarea" disabled={!isEditingProfile} rows="3" aria-label={i18n.locale === 'ar' ? 'الوصف (إنجليزي)' : 'Description (English)'}></textarea>
                            </div>
                        </div>
                    </div>

                    <div class="divider"></div>
                    <h3 class="section-title">{i18n.locale === 'ar' ? 'معلومات إضافية' : 'Additional Information'}</h3>
                    
                    <div class="form-grid">
                        <div class="form-group">
                            <label for="cr_number">{i18n.locale === 'ar' ? 'السجل التجاري' : 'CR Number'}</label>
                            <input id="cr_number" type="text" bind:value={profileData.cr_number} class="input-field" disabled={!isEditingProfile} />
                        </div>
                        <div class="form-group">
                            <label for="coordinator_phone">{i18n.locale === 'ar' ? 'هاتف المنسق' : 'Coordinator Phone'}</label>
                            <input id="coordinator_phone" type="tel" bind:value={profileData.coordinator_phone} class="input-field" disabled={!isEditingProfile} />
                        </div>
                    </div>

                    {#if isEditingProfile}
                        <div class="form-actions">
                            <button type="submit" class="btn btn-primary">
                                <Save size={18} />
                                {i18n.locale === 'ar' ? 'حفظ التغييرات' : 'Save Changes'}
                            </button>
                        </div>
                    {/if}
                </form>
            </div>
        </div>

        <!-- Security / Password Card -->
        <div class="card security-card">
            <div class="card-header">
                <h2>
                    <Lock size={20} />
                    {i18n.locale === 'ar' ? 'تغيير كلمة المرور' : 'Change Password'}
                </h2>
            </div>
            <div class="card-body">
                <form onsubmit={changePassword}>
                    <div class="form-group">
                        <label for="old_password">{i18n.locale === 'ar' ? 'كلمة المرور الحالية' : 'Current Password'}</label>
                        <input id="old_password" type="password" bind:value={passwordData.old_password} class="input-field" required />
                    </div>
                    <div class="form-group">
                        <label for="new_password">{i18n.locale === 'ar' ? 'كلمة المرور الجديدة' : 'New Password'}</label>
                        <input id="new_password" type="password" bind:value={passwordData.new_password} class="input-field" required minlength="8" />
                    </div>
                    <div class="form-group">
                        <label for="confirm_password">{i18n.locale === 'ar' ? 'تأكيد كلمة المرور' : 'Confirm Password'}</label>
                        <input id="confirm_password" type="password" bind:value={passwordData.confirm_password} class="input-field" required minlength="8" />
                    </div>

                    <div class="form-actions" style="margin-top: 1.5rem;">
                        <button type="submit" class="btn btn-secondary" disabled={!passwordData.old_password || !passwordData.new_password}>
                            <Lock size={18} />
                            {i18n.locale === 'ar' ? 'تحديث كلمة المرور' : 'Update Password'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</div>

<style>
    .profile-container {
        padding: 1rem 0;
        max-width: 1200px;
        margin: 0 auto;
    }

    .page-header {
        margin-bottom: 2rem;
    }

    .header-content {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .page-title {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 1.75rem;
        font-weight: 700;
        color: var(--text);
        margin: 0;
    }

    .page-subtitle {
        color: var(--text-sec);
        margin: 0;
        font-size: 0.95rem;
    }

    .grid-layout {
        display: grid;
        grid-template-columns: 2fr 1fr;
        gap: 2rem;
        align-items: flex-start;
    }

    @media (max-width: 1024px) {
        .grid-layout {
            grid-template-columns: 1fr;
        }
    }

    .card {
        background: var(--white);
        border: 1px solid var(--border-light);
        border-radius: var(--radius-xl);
        box-shadow: var(--shadow-sm);
        overflow: hidden;
        transition: box-shadow var(--duration-base) ease;
    }

    .card:hover {
        box-shadow: var(--shadow-md);
    }

    .card-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1.25rem 1.5rem;
        border-bottom: 1px solid var(--border-light);
        background: rgba(250, 250, 250, 0.5);
    }

    .card-header h2 {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 1.1rem;
        font-weight: 600;
        margin: 0;
        color: var(--text);
    }

    .card-body {
        padding: 1.5rem;
    }

    .form-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.25rem;
    }

    @media (max-width: 768px) {
        .form-grid {
            grid-template-columns: 1fr;
        }
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .full-width {
        grid-column: 1 / -1;
    }

    .grid-2 {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    @media (max-width: 640px) {
        .grid-2 {
            grid-template-columns: 1fr;
        }
    }

    label {
        font-size: 0.85rem;
        font-weight: 500;
        color: var(--text-sec);
        display: flex;
        align-items: center;
        gap: 0.35rem;
    }

    .input-field {
        padding: 0.65rem 1rem;
        border: 1px solid var(--border);
        border-radius: var(--radius-md);
        font-family: inherit;
        font-size: 0.95rem;
        color: var(--text);
        background: var(--white);
        transition: all var(--duration-fast);
        width: 100%;
    }

    .input-field:focus {
        outline: none;
        border-color: var(--teal);
        box-shadow: 0 0 0 3px var(--teal-light);
    }

    .input-field:disabled {
        background: var(--bg);
        color: var(--text-sec);
        cursor: not-allowed;
        opacity: 0.8;
    }

    .textarea {
        resize: vertical;
        min-height: 80px;
    }

    .divider {
        height: 1px;
        background: var(--border-light);
        margin: 2rem 0;
    }

    .section-title {
        font-size: 1rem;
        font-weight: 600;
        margin: 0 0 1rem 0;
        color: var(--text-light);
    }

    .form-actions {
        display: flex;
        justify-content: flex-end;
        margin-top: 2rem;
    }

    .btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.65rem 1.25rem;
        border-radius: var(--radius-md);
        font-weight: 600;
        font-size: 0.9rem;
        cursor: pointer;
        transition: all var(--duration-fast);
        border: none;
    }

    .btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .btn-primary {
        background: var(--teal);
        color: white;
    }

    .btn-primary:hover:not(:disabled) {
        background: var(--teal-dark);
        transform: translateY(-1px);
        box-shadow: var(--shadow-sm);
    }

    .btn-secondary {
        background: var(--text);
        color: white;
    }

    .btn-secondary:hover:not(:disabled) {
        background: #000;
        transform: translateY(-1px);
        box-shadow: var(--shadow-sm);
    }

    .btn-outline {
        background: transparent;
        border: 1px solid var(--border);
        color: var(--text);
        padding: 0.4rem 0.85rem;
        font-size: 0.85rem;
    }

    .btn-outline:hover {
        background: var(--bg);
        border-color: var(--text-sec);
    }

    .alert {
        padding: 1rem;
        border-radius: var(--radius-md);
        margin-bottom: 1.5rem;
        font-weight: 500;
        font-size: 0.95rem;
        animation: fade-in 0.3s ease;
    }

    .alert-success {
        background: rgba(16, 185, 129, 0.1);
        border: 1px solid rgba(16, 185, 129, 0.2);
        color: #047857;
    }

    .alert-error {
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.2);
        color: #b91c1c;
    }


    .animate-fade-in {
        animation: fade-in 0.4s ease-out forwards;
    }

    @keyframes fade-in {
        from { opacity: 0; transform: translateY(10px); }
        to { opacity: 1; transform: translateY(0); }
    }

    /* Additional color utilities assuming they exist in global CSS or can be inherited */
    :global(.text-primary) {
        color: var(--teal);
    }
</style>
