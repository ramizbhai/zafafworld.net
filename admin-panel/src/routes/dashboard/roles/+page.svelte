<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Shield, Lock, Plus, Check, X } from 'lucide-svelte';

  const roles = [
    { id: 'SuperAdmin', name_ar: 'مدير النظام الرئيسي', name_en: 'Super Admin', users: '∞', color: 'var(--gold)' },
    { id: 'SupportAdmin', name_ar: 'مدير الدعم', name_en: 'Support Admin', users: '∞', color: 'var(--info)' },
    { id: 'Owner', name_ar: 'مالك المنشأة', name_en: 'Vendor Owner', users: '∞', color: 'var(--success)' },
    { id: 'Staff', name_ar: 'موظف المنشأة', name_en: 'Vendor Staff', users: '∞', color: 'var(--purple)' },
  ];

  const permissions = [
    { key: 'platform.manage', label_ar: 'إدارة المنصة الشاملة', label_en: 'Manage Platform' },
    { key: 'vendors.approve', label_ar: 'موافقة الموردين', label_en: 'Approve Vendors' },
    { key: 'support.manage', label_ar: 'إدارة الدعم الفني', label_en: 'Manage Support' },
    { key: 'vendor.manage', label_ar: 'إدارة بيانات المنشأة', label_en: 'Manage Own Vendor' },
  ];

  // Verified Backend Scope Matrix
  const matrix: Record<string, Record<string, boolean>> = {
    SuperAdmin: { 'platform.manage': true, 'vendors.approve': true, 'support.manage': true, 'vendor.manage': true },
    SupportAdmin: { 'platform.manage': false, 'vendors.approve': true, 'support.manage': true, 'vendor.manage': false },
    Owner: { 'platform.manage': false, 'vendors.approve': false, 'support.manage': false, 'vendor.manage': true },
    Staff: { 'platform.manage': false, 'vendors.approve': false, 'support.manage': false, 'vendor.manage': false },
  };
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.roles')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'إدارة الأدوار والصلاحيات للفريق الإداري' : 'Manage roles and permissions for admin staff'}</p>
    </div>
    <button class="btn btn-gold btn-sm"><Plus size={14} /> {$lang === 'ar' ? 'دور جديد' : 'New Role'}</button>
  </div>

  <!-- Role cards -->
  <div class="roles-grid">
    {#each roles as role}
      <div class="role-card card">
        <div class="role-icon" style="background: {role.color}20; border: 1px solid {role.color}33; color: {role.color}">
          <Shield size={18} />
        </div>
        <div class="role-info">
          <span class="role-name">{$lang === 'ar' ? role.name_ar : role.name_en}</span>
          <span class="role-users">{role.users} {$lang === 'ar' ? 'مستخدم' : 'user(s)'}</span>
        </div>
      </div>
    {/each}
  </div>

  <!-- Permission Matrix -->
  <div class="table-container">
    <div class="table-head-bar">
      <span class="table-title">{$lang === 'ar' ? 'مصفوفة الصلاحيات' : 'Permission Matrix'}</span>
    </div>
    <div class="table-scroll">
      <table class="perm-table">
        <thead>
          <tr>
            <th>{$lang === 'ar' ? 'الصلاحية' : 'Permission'}</th>
            {#each roles as role}
              <th style="text-align:center; min-width:110px">
                <span style="color: {role.color}">{$lang === 'ar' ? role.name_ar : role.name_en}</span>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each permissions as perm}
            <tr>
              <td style="font-weight:600; font-size:13px">{$lang === 'ar' ? perm.label_ar : perm.label_en}</td>
              {#each roles as role}
                <td style="text-align:center;">
                  {#if matrix[role.id]?.[perm.key]}
                    <span class="perm-check" style="color:var(--success)" aria-label="Granted"><Check size={16} /></span>
                  {:else}
                    <span class="perm-check" style="color:var(--text-ghost)" aria-label="Not granted"><X size={16} /></span>
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .roles-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: 12px; margin-bottom: 20px; }
  .role-card { padding: 16px 18px; display: flex; align-items: center; gap: 12px; }
  .role-icon { width: 38px; height: 38px; border-radius: 10px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
  .role-info { display: flex; flex-direction: column; gap: 3px; }
  .role-name { font-size: 13px; font-weight: 700; color: var(--text-primary); }
  .role-users { font-size: 11px; color: var(--text-ghost); }
  .perm-check { display: inline-flex; }
  .perm-table td, .perm-table th { padding: 11px 14px; }
  @media (max-width: 1200px) { .roles-grid { grid-template-columns: repeat(3, 1fr); } }
  @media (max-width: 768px) { .roles-grid { grid-template-columns: repeat(2, 1fr); } }
</style>
