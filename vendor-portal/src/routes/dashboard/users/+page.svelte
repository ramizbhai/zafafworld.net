<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { enhance } from '$app/forms';
    import { invalidateAll } from '$app/navigation';
    import type { PageData } from './$types';

    const i18n = getI18n();

    // Svelte 5 runes: receive page data
    let { data }: { data: PageData } = $props();

    // Client-side search query
    let searchQuery = $state('');

    // Modal states
    let isAddOpen = $state(false);
    let isEditOpen = $state(false);
    let selectedUser = $state<any | null>(null);

    // Form fields mapped to modal inputs
    let id = $state('');
    let name = $state('');
    let email = $state('');
    let role = $state<'admin' | 'editor' | 'viewer'>('editor');
    let status = $state<'active' | 'inactive'>('active');

    // Computed filtered roster
    let filteredUsers = $derived.by(() => {
        const staffList = data.staff || [];
        if (!searchQuery.trim()) return staffList;
        const q = searchQuery.toLowerCase().trim();
        return staffList.filter((u: any) =>
            (u.name || '').toLowerCase().includes(q) ||
            (u.email || '').toLowerCase().includes(q)
        );
    });

    // Modal control actions
    function openAddModal() {
        name = '';
        email = '';
        role = 'editor';
        status = 'active';
        isAddOpen = true;
    }

    function openEditModal(u: any) {
        selectedUser = u;
        id = u.id;
        name = u.name;
        email = u.email;
        role = u.role;
        status = u.status;
        isEditOpen = true;
    }

    // Role helper
    function getRoleLabel(r: string) {
        if (r === 'admin') return i18n.t.users?.roleAdmin || 'Administrator';
        if (r === 'editor') return i18n.t.users?.roleEditor || 'Editor';
        return i18n.t.users?.roleViewer || 'Viewer';
    }

    // Status helper
    function getStatusLabel(s: string) {
        return s === 'active' 
            ? (i18n.t.users?.statusActive || 'Active') 
            : (i18n.t.users?.statusInactive || 'Inactive');
    }
</script>

<svelte:head>
    <title>{i18n.t.nav.users} | {i18n.t.common.appName}</title>
</svelte:head>

<div class="workforce-page" dir={i18n.isRtl ? 'rtl' : 'ltr'}>
    <!-- ─── HEADER / ACTION BAR ────────────────────────────────────────────── -->
    <div class="page-header">
        <div class="title-section">
            <span class="category-tag">{i18n.locale === 'ar' ? 'إدارة الطاقم' : 'Staff Operations'}</span>
            <h1>{i18n.t.nav.users}</h1>
            <p class="subtitle">{i18n.locale === 'ar' ? 'إضافة موظفين جدد وتعديل صلاحيات الوصول وتنشيط الحسابات للمشاركة في التنظيم.' : 'Manage collaborator authorization levels, register staff credentials, and update system accesses.'}</p>
        </div>
        <button onclick={openAddModal} class="btn btn-primary">
            <span>➕</span> {i18n.t.users?.addBtn || 'Add Member'}
        </button>
    </div>

    <!-- ─── SEARCH / FILTER BAR ────────────────────────────────────────────── -->
    <div class="filter-bar" style="margin-bottom: 24px;">
        <div class="search-field">
            <span class="search-icon">🔍</span>
            <input 
                type="text" 
                bind:value={searchQuery} 
                placeholder={i18n.t.common.search} 
            />
        </div>
    </div>

    <!-- ─── COLLABORATORS TABLE ────────────────────────────────────────────── -->
    {#if filteredUsers.length === 0}
        <div class="empty-state">
            <div class="empty-illustration">👥</div>
            <h3>{i18n.t.nav.users}</h3>
            <p>{i18n.t.users?.empty || 'No workforce staff members found.'}</p>
            <button onclick={openAddModal} class="btn btn-primary">{i18n.t.users?.addBtn || 'Add Member'}</button>
        </div>
    {:else}
        <div class="table-card">
            <div class="table-wrap">
                <table>
                    <thead>
                        <tr>
                            <th>{i18n.t.users?.name || 'Name'}</th>
                            <th>{i18n.t.users?.email || 'Email'}</th>
                            <th>{i18n.t.users?.role || 'System Role'}</th>
                            <th>{i18n.t.users?.status || 'Account Access'}</th>
                            <th>{i18n.t.common.actions}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each filteredUsers as item (item.id)}
                            <tr>
                                <td>
                                    <div class="user-meta-cell">
                                        <div class="user-avatar">{(item.name || '').slice(0, 2).toUpperCase()}</div>
                                        <span class="user-name-txt">{item.name}</span>
                                    </div>
                                </td>
                                <td>{item.email}</td>
                                <td>
                                    <span class="role-indicator role-{item.role}">
                                        {getRoleLabel(item.role)}
                                    </span>
                                </td>
                                <td>
                                    <form method="POST" action="?/toggleStatus" use:enhance={() => {
                                        return async ({ result, update }) => {
                                            if (result.type === 'success') await invalidateAll();
                                            await update();
                                        };
                                    }} style="display: inline-block;">
                                        <input type="hidden" name="id" value={item.id} />
                                        <input type="hidden" name="status" value={item.status === 'active' ? 'inactive' : 'active'} />
                                        <button
                                            type="submit"
                                            class="status-toggle-btn badge"
                                            class:badge-done={item.status === 'active'}
                                            class:badge-expired={item.status === 'inactive'}
                                            title={i18n.locale === 'ar' ? 'تعديل حالة الدخول' : 'Click to change status'}
                                        >
                                            {getStatusLabel(item.status)}
                                        </button>
                                    </form>
                                </td>
                                <td>
                                    <div class="action-wrap">
                                        <button class="action-btn" onclick={() => openEditModal(item)} title={i18n.t.common.edit}>
                                            ✏️
                                        </button>
                                        <form method="POST" action="?/delete" use:enhance={() => {
                                            return async ({ result, update }) => {
                                                if (result.type === 'success') await invalidateAll();
                                                await update();
                                            };
                                        }} style="display: inline-block;">
                                            <input type="hidden" name="id" value={item.id} />
                                            <button
                                                type="submit"
                                                class="action-btn delete-btn"
                                                onclick={(e) => {
                                                    if (!confirm(i18n.locale === 'ar' ? 'هل أنت متأكد من حذف هذا العضو؟' : 'Are you sure you want to delete this member?')) {
                                                        e.preventDefault();
                                                    }
                                                }}
                                                title={i18n.t.common.delete}
                                            >
                                                🗑️
                                            </button>
                                        </form>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    {/if}

    <!-- ─── CREATE / EDIT MODALS ───────────────────────────────────────────── -->
    {#if isAddOpen}
        <div class="modal-backdrop" onclick={() => { isAddOpen = false; }} role="presentation">
            <div
                class="modal"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
                role="dialog"
                aria-modal="true"
                tabindex="-1"
            >
                <form method="POST" action="?/add" use:enhance={() => {
                    return async ({ result, update }) => {
                        if (result.type === 'success') {
                            isAddOpen = false;
                            await invalidateAll();
                        }
                        await update();
                    };
                }}>
                    <div class="modal-header">
                        <h2 class="modal-title">
                            {i18n.t.users?.addBtn || 'Add Member'}
                        </h2>
                        <button type="button" onclick={() => { isAddOpen = false; }} class="modal-close" aria-label={i18n.t.common.close}>✕</button>
                    </div>

                    <div class="modal-body">
                        <div class="form-group">
                            <label class="form-label" for="member-name">{i18n.t.users?.name || 'Name'}</label>
                            <input
                                id="member-name"
                                name="name"
                                type="text"
                                class="form-input"
                                bind:value={name}
                                placeholder={i18n.t.users?.namePl || 'Enter full name'}
                                required
                            />
                        </div>

                        <div class="form-group">
                            <label class="form-label" for="member-email">{i18n.t.users?.email || 'Email'}</label>
                            <input
                                id="member-email"
                                name="email"
                                type="email"
                                class="form-input"
                                bind:value={email}
                                placeholder={i18n.t.users?.emailPl || 'colleague@domain.com'}
                                required
                            />
                        </div>

                        <div class="form-group">
                            <label class="form-label" for="member-role">{i18n.t.users?.role || 'System Role'}</label>
                            <div style="position: relative;">
                                <select id="member-role" name="role" class="form-select" bind:value={role}>
                                    <option value="admin">{i18n.t.users?.roleAdmin || 'Administrator'}</option>
                                    <option value="editor">{i18n.t.users?.roleEditor || 'Editor'}</option>
                                    <option value="viewer">{i18n.t.users?.roleViewer || 'Viewer'}</option>
                                </select>
                            </div>
                        </div>

                        <div class="form-group">
                            <label class="form-label" for="member-status">{i18n.t.users?.status || 'Access Status'}</label>
                            <div style="position: relative;">
                                <select id="member-status" name="status" class="form-select" bind:value={status}>
                                    <option value="active">{getStatusLabel('active')}</option>
                                    <option value="inactive">{getStatusLabel('inactive')}</option>
                                </select>
                            </div>
                        </div>
                    </div>

                    <div class="modal-footer">
                        <button type="button" class="btn btn-ghost" onclick={() => { isAddOpen = false; }}>{i18n.t.common.cancel}</button>
                        <button type="submit" class="btn btn-primary">
                            {i18n.t.common.save}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    {/if}

    {#if isEditOpen && selectedUser}
        <div class="modal-backdrop" onclick={() => { isEditOpen = false; selectedUser = null; }} role="presentation">
            <div
                class="modal"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
                role="dialog"
                aria-modal="true"
                tabindex="-1"
            >
                <form method="POST" action="?/update" use:enhance={() => {
                    return async ({ result, update }) => {
                        if (result.type === 'success') {
                            isEditOpen = false;
                            selectedUser = null;
                            await invalidateAll();
                        }
                        await update();
                    };
                }}>
                    <input type="hidden" name="id" value={id} />
                    <div class="modal-header">
                        <h2 class="modal-title">
                            {i18n.locale === 'ar' ? 'تعديل بيانات العضو' : 'Edit Collaborator'}
                        </h2>
                        <button type="button" onclick={() => { isEditOpen = false; selectedUser = null; }} class="modal-close" aria-label={i18n.t.common.close}>✕</button>
                    </div>

                    <div class="modal-body">
                        <div class="form-group">
                            <label class="form-label" for="edit-member-name">{i18n.t.users?.name || 'Name'}</label>
                            <input
                                id="edit-member-name"
                                name="name"
                                type="text"
                                class="form-input"
                                bind:value={name}
                                placeholder={i18n.t.users?.namePl || 'Enter full name'}
                                required
                            />
                        </div>

                        <div class="form-group">
                            <label class="form-label" for="edit-member-email">{i18n.t.users?.email || 'Email'}</label>
                            <input
                                id="edit-member-email"
                                name="email"
                                type="email"
                                class="form-input"
                                bind:value={email}
                                placeholder={i18n.t.users?.emailPl || 'colleague@domain.com'}
                                required
                            />
                        </div>

                        <div class="form-group">
                            <label class="form-label" for="edit-member-role">{i18n.t.users?.role || 'System Role'}</label>
                            <div style="position: relative;">
                                <select id="edit-member-role" name="role" class="form-select" bind:value={role}>
                                    <option value="admin">{i18n.t.users?.roleAdmin || 'Administrator'}</option>
                                    <option value="editor">{i18n.t.users?.roleEditor || 'Editor'}</option>
                                    <option value="viewer">{i18n.t.users?.roleViewer || 'Viewer'}</option>
                                </select>
                            </div>
                        </div>

                        <div class="form-group">
                            <label class="form-label" for="edit-member-status">{i18n.t.users?.status || 'Access Status'}</label>
                            <div style="position: relative;">
                                <select id="edit-member-status" name="status" class="form-select" bind:value={status}>
                                    <option value="active">{getStatusLabel('active')}</option>
                                    <option value="inactive">{getStatusLabel('inactive')}</option>
                                </select>
                            </div>
                        </div>
                    </div>

                    <div class="modal-footer">
                        <button type="button" class="btn btn-ghost" onclick={() => { isEditOpen = false; selectedUser = null; }}>{i18n.t.common.cancel}</button>
                        <button type="submit" class="btn btn-primary">
                            {i18n.t.common.save}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    {/if}
</div>

<style>
    .workforce-page {
        display: flex;
        flex-direction: column;
        gap: 24px;
        max-width: 1200px;
        margin: 0 auto;
        animation: fade-in 0.3s ease-out;
    }

    @keyframes fade-in {
        from { opacity: 0; transform: translateY(8px); }
        to { opacity: 1; transform: translateY(0); }
    }

    .page-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 16px;
        border-bottom: 1px solid var(--border);
        padding-bottom: 16px;
    }

    .title-section {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .category-tag {
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 1px;
        font-weight: 700;
        color: var(--teal);
    }

    .page-header h1 {
        margin: 0;
        font-size: 24px;
        font-weight: 800;
        color: var(--text);
    }

    .subtitle {
        margin: 0;
        font-size: 13px;
        color: var(--text-sec);
    }

    .filter-bar {
        display: flex;
        gap: 12px;
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        padding: 12px;
        align-items: center;
    }

    .search-field {
        flex: 1;
        position: relative;
        display: flex;
        align-items: center;
    }

    .search-icon {
        position: absolute;
        right: 12px;
        font-size: 14px;
        color: var(--text-light);
    }

    :global(html[dir="rtl"]) .search-icon {
        right: auto;
        left: 12px;
    }

    .search-field input {
        width: 100%;
        border: 1.5px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 10px 36px 10px 16px;
        font-family: var(--font);
        font-size: 13px;
        color: var(--text);
        outline: none;
    }

    :global(html[dir="rtl"]) .search-field input {
        padding: 10px 16px 10px 36px;
    }

    .search-field input:focus {
        border-color: var(--teal);
    }

    .user-meta-cell {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .user-avatar {
        width: 36px;
        height: 36px;
        background: var(--teal-light);
        color: var(--teal);
        font-weight: 700;
        font-size: 13px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        text-transform: uppercase;
    }

    .user-name-txt {
        font-weight: 600;
        color: var(--text);
    }

    .role-indicator {
        display: inline-block;
        font-size: 12px;
        font-weight: 600;
        padding: 2px 8px;
        border-radius: 6px;
    }

    .role-admin {
        background: rgba(29, 78, 216, 0.08);
        color: #1d4ed8;
    }

    .role-editor {
        background: rgba(190, 24, 93, 0.08);
        color: #be185d;
    }

    .role-viewer {
        background: rgba(71, 85, 105, 0.08);
        color: #475569;
    }

    .status-toggle-btn {
        border: none;
        cursor: pointer;
        transition: transform 0.15s;
        font-family: var(--font);
    }

    .status-toggle-btn:hover {
        transform: scale(1.05);
    }

    .action-wrap {
        display: flex;
        gap: 8px;
    }
</style>
