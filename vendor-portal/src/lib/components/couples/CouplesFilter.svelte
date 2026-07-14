<script lang="ts">
    import { getStatusLabel } from "../../services/couples.service";

    let { i18n, couplesState } = $props<{ i18n: any, couplesState: any }>();
</script>

<div class="filter-bar">
    <div class="search-field">
        <span class="search-icon">🔍</span>
        <input 
            type="text" 
            bind:value={couplesState.searchQuery} 
            placeholder={i18n.t.common.search} 
        />
    </div>

    <div class="filter-select-wrap">
        <select bind:value={couplesState.statusFilter} class="form-select status-select">
            <option value="all">{i18n.locale === 'ar' ? 'كل الحالات' : 'All Statuses'}</option>
            <option value="new">{getStatusLabel('new', i18n)}</option>
            <option value="read">{getStatusLabel('read', i18n)}</option>
            <option value="done">{getStatusLabel('done', i18n)}</option>
            <option value="rejected">{getStatusLabel('rejected', i18n)}</option>
            <option value="negotiation">{getStatusLabel('negotiation', i18n)}</option>
            <option value="unreachable">{getStatusLabel('unreachable', i18n)}</option>
            <option value="paid">{getStatusLabel('paid', i18n)}</option>
        </select>
    </div>

    <div class="filter-select-wrap">
        <select bind:value={couplesState.sortField} class="form-select status-select">
            <option value="created_at">{i18n.locale === 'ar' ? 'ترتيب: الاستلام' : 'Sort: Received'}</option>
            <option value="customer_name">{i18n.locale === 'ar' ? 'ترتيب: الاسم' : 'Sort: Name'}</option>
            <option value="wedding_date">{i18n.locale === 'ar' ? 'ترتيب: تاريخ الحفل' : 'Sort: Wedding Date'}</option>
            <option value="status">{i18n.locale === 'ar' ? 'ترتيب: الحالة' : 'Sort: Status'}</option>
        </select>
    </div>

    <button 
        type="button" 
        onclick={() => { couplesState.sortAsc = !couplesState.sortAsc; couplesState.currentPage = 1; }} 
        class="btn-sort-dir"
        style="height: 40px; width: 40px; padding: 0; display: flex; align-items: center; justify-content: center; font-size: 14px; border: 1.5px solid var(--border); background: var(--white); border-radius: var(--radius-sm); color: var(--text-sec); cursor: pointer;"
        title={i18n.locale === 'ar' ? 'عكس اتجاه الترتيب' : 'Toggle Sort Direction'}
    >
        {couplesState.sortAsc ? '▲' : '▼'}
    </button>

    <button 
        type="button" 
        onclick={() => couplesState.isAddModalOpen = true} 
        class="btn-add-couple"
        style="height: 40px; display: flex; align-items: center; gap: 8px; font-weight: 700; background: var(--teal); border: none; border-radius: var(--radius-sm); color: #fff; padding: 0 16px; cursor: pointer; transition: all 0.2s;"
    >
        <span style="font-size: 18px; line-height: 1;">+</span>
        {i18n.locale === 'ar' ? 'إضافة عميل' : 'Add Couple'}
    </button>
</div>
