<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Search, Filter, UserPlus, MoreHorizontal, Shield, CheckCircle2, XCircle, Clock, Download, AlertCircle } from 'lucide-svelte';
  import { goto } from '$app/navigation';
  import { enhance } from '$app/forms';

  interface Props {
    data: {
      users: any[];
      total: number;
      page: number;
      totalPages: number;
      search: string;
      error?: string;
      stats?: {
        totalUsers: number;
        totalClients: number;
        totalVendors: number;
        totalAdmins: number;
        newUsersThisMonth: number;
      };
    };
  }

  let { data }: Props = $props();

  // Local state synced from page data
  let searchInput = $state('');

  // Sync state if url changes externally
  $effect(() => {
    searchInput = data.search || '';
  });

  function applyFilters(page: number = 1) {
    const params = new URLSearchParams();
    if (searchInput.trim()) {
      params.set('search', searchInput.trim());
    }
    params.set('page', String(page));
    goto(`/dashboard/users?${params.toString()}`, { keepFocus: true, noScroll: true });
  }

  function handleSearchSubmit(e: Event) {
    e.preventDefault();
    applyFilters(1);
  }

  function handlePageChange(newPage: number) {
    applyFilters(newPage);
  }

  function statusBadge(status: string): string {
    if (status === 'active') return 'badge badge-dot badge-success';
    if (status === 'suspended') return 'badge badge-dot badge-danger';
    return 'badge badge-dot badge-warning';
  }

  function statusLabel(status: string): string {
    const map: Record<string, string> = {
      active: $lang === 'ar' ? 'نشط' : 'Active',
      suspended: $lang === 'ar' ? 'موقوف' : 'Suspended',
      pending: $lang === 'ar' ? 'معلق' : 'Pending'
    };
    return map[status] ?? status;
  }

  function formatDate(d: string): string {
    if (!d) return '-';
    return new Date(d).toLocaleDateString($lang === 'ar' ? 'ar-SA' : 'en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }

  function roleBadge(role: string): string {
    if (role === 'Admin') return 'badge badge-danger-pill';
    if (role === 'Vendor') return 'badge badge-warning-pill';
    return 'badge badge-purple-pill';
  }

  function roleLabel(role: string): string {
    const map: Record<string, string> = {
      Admin: $lang === 'ar' ? 'مسؤول' : 'Admin',
      Vendor: $lang === 'ar' ? 'مورد' : 'Vendor',
      Client: $lang === 'ar' ? 'عميل' : 'Client'
    };
    return map[role] ?? role;
  }

  function getInitials(firstName: string, lastName: string, email: string): string {
    if (firstName || lastName) {
      return ((firstName?.[0] || '') + (lastName?.[0] || '')).toUpperCase();
    }
    return email.slice(0, 2).toUpperCase();
  }

  let pages = $derived.by(() => {
    const current = data.page;
    const total = data.totalPages;
    if (total <= 7) {
      return Array.from({ length: total }, (_, i) => i + 1);
    }
    if (current <= 4) {
      return [1, 2, 3, 4, 5, '...', total];
    }
    if (current >= total - 3) {
      return [1, '...', total - 4, total - 3, total - 2, total - 1, total];
    }
    return [1, '...', current - 1, current, current + 1, '...', total];
  });
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.users')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'إدارة جميع مستخدمي المنصة من العملاء' : 'Manage all platform client users'}</p>
    </div>
    <div class="page-header-right">
      <button class="btn btn-outline btn-sm" disabled>
        <Download size={14} aria-hidden="true" />
        {$lang === 'ar' ? 'تصدير' : 'Export'}
      </button>
      <button class="btn btn-gold btn-sm" disabled>
        <UserPlus size={14} aria-hidden="true" />
        {$lang === 'ar' ? 'إضافة مستخدم' : 'Add User'}
      </button>
    </div>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
    </div>
  {/if}

  <!-- Stats row -->
  <div class="user-stats-row">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي مستخدمي المنصة' : 'Total Platform Users'}</span>
      <span class="mini-stat-value" style="color: var(--gold)">{data.stats?.totalUsers ?? data.total}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'العملاء' : 'Clients'}</span>
      <span class="mini-stat-value" style="color: var(--success)">{data.stats?.totalClients ?? data.total}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'الموردون' : 'Vendors'}</span>
      <span class="mini-stat-value" style="color: var(--info)">{data.stats?.totalVendors ?? 0}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'المسؤولون' : 'Admins'}</span>
      <span class="mini-stat-value" style="color: var(--text-main)">{data.stats?.totalAdmins ?? 0}</span>
    </div>
  </div>

  <!-- Table -->
  <div class="table-container">
    <div class="table-head-bar">
      <form class="toolbar" onsubmit={handleSearchSubmit} style="margin: 0; flex: 1;">
        <div class="search-box" style="flex: 1; max-width: 320px;">
          <Search size={15} aria-hidden="true" />
          <input 
            type="search" 
            placeholder={$t('common.search')} 
            bind:value={searchInput} 
            aria-label="Search users" 
          />
        </div>
        <button type="submit" class="btn btn-outline btn-sm">
          {$lang === 'ar' ? 'بحث' : 'Search'}
        </button>
      </form>
      <span class="table-title">
        {data.total} {$lang === 'ar' ? 'مستخدم' : 'Users'}
      </span>
    </div>

    <div class="table-scroll">
      <table aria-label="User directory table">
        <thead>
          <tr>
            <th>{$t('common.name')}</th>
            <th>{$t('common.email')}</th>
            <th>{$lang === 'ar' ? 'الدور' : 'Role'}</th>
            <th>{$lang === 'ar' ? 'رقم الهاتف' : 'Phone'}</th>
            <th>{$lang === 'ar' ? 'الحجوزات' : 'Bookings'}</th>
            <th>{$lang === 'ar' ? 'تاريخ الانضمام' : 'Joined'}</th>
            <th>{$lang === 'ar' ? 'تاريخ الزفاف' : 'Wedding Date'}</th>
            <th>{$t('common.status')}</th>
            <th>{$t('common.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#each data.users as user (user.id)}
            <tr>
              <td>
                <div class="cell-avatar">
                  <div 
                    class="cell-avatar-img cell-avatar-round" 
                    style="background: linear-gradient(135deg, hsl({Math.abs(user.id.charCodeAt(3) * 47) % 360}, 60%, 35%), hsl({(Math.abs(user.id.charCodeAt(3) * 47) + 40) % 360}, 70%, 50%))"
                  >
                    {getInitials(user.first_name, user.last_name, user.email)}
                  </div>
                  <div>
                    <div class="cell-label">
                      {user.first_name || user.last_name 
                        ? `${user.first_name} ${user.last_name}`.trim() 
                        : ($lang === 'ar' ? 'مستخدم غير معروف' : 'Unknown User')}
                    </div>
                    <div class="cell-sub">{user.id}</div>
                  </div>
                </div>
              </td>
              <td class="text-muted mono" style="font-size:12.5px">{user.email}</td>
              <td><span class={roleBadge(user.domain_type)}>{roleLabel(user.domain_type)}</span></td>
              <td class="text-muted">{user.phone || '-'}</td>
              <td style="font-weight:600; text-align:center">{user.bookings_count}</td>
              <td class="text-muted" style="font-size:12px">{formatDate(user.created_at)}</td>
              <td class="text-muted" style="font-size:12px">{user.domain_type === 'Client' ? formatDate(user.wedding_date) : '-'}</td>
              <td><span class={statusBadge(user.status)}>{statusLabel(user.status)}</span></td>
              <td>
                <form method="POST" action="?/updateStatus" use:enhance style="display:flex; gap:6px; align-items:center;">
                  <input type="hidden" name="id" value={user.id} />
                  {#if user.status === 'active'}
                    <input type="hidden" name="status" value="suspended" />
                    <button class="btn-icon" aria-label="Suspend user" title="Suspend" type="submit">
                      <XCircle size={14} aria-hidden="true" style="color:var(--danger)" />
                    </button>
                  {:else}
                    <input type="hidden" name="status" value="active" />
                    <button class="btn-icon" aria-label="Reactivate user" title="Reactivate" type="submit">
                      <CheckCircle2 size={14} aria-hidden="true" style="color:var(--success)" />
                    </button>
                  {/if}
                  <button class="btn-icon" aria-label="More actions" title="More" disabled type="button">
                    <MoreHorizontal size={14} aria-hidden="true" />
                  </button>
                </form>
              </td>
            </tr>
          {/each}
          {#if data.users.length === 0}
            <tr>
              <td colspan="9">
                <div class="empty-state">
                  <div class="empty-icon"><Search size={28} /></div>
                  <h3>{$t('common.no_data')}</h3>
                  <p>{$lang === 'ar' ? 'لا يوجد مستخدمون مطابقون للبحث في قاعدة البيانات' : 'No users match your search in the database'}</p>
                </div>
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>

    {#if data.totalPages > 1}
      <div class="pagination">
        <span class="pagination-info">
          {$lang === 'ar'
            ? `عرض الصفحات من ${data.page} إلى ${data.totalPages} (إجمالي ${data.total} عميل)`
            : `Showing page ${data.page} of ${data.totalPages} (Total ${data.total} clients)`}
        </span>
        <div class="pagination-controls">
          <button 
            class="page-btn" 
            onclick={() => handlePageChange(Math.max(1, data.page - 1))} 
            disabled={data.page === 1} 
            aria-label="Previous page"
          >
            ‹
          </button>
          {#each pages as p}
            {#if p === '...'}
              <span class="page-ellipsis" style="padding: 0 8px; color: var(--text-ghost); display: flex; align-items: center;">...</span>
            {:else}
              <button 
                class="page-btn" 
                class:active={p === data.page} 
                onclick={() => handlePageChange(p as number)} 
                aria-label="Page {p}" 
                aria-current={p === data.page ? 'page' : undefined}
              >
                {p}
              </button>
            {/if}
          {/each}
          <button 
            class="page-btn" 
            onclick={() => handlePageChange(Math.min(data.totalPages, data.page + 1))} 
            disabled={data.page === data.totalPages} 
            aria-label="Next page"
          >
            ›
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .user-stats-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat {
    padding: 16px 18px;
    display: flex; flex-direction: column; gap: 6px;
  }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 20px; font-weight: 800; letter-spacing: -0.5px; }
  @media (max-width: 900px) { .user-stats-row { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 560px) { .user-stats-row { grid-template-columns: 1fr; } }
</style>
