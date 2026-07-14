<script lang="ts">
    import { getStatusLabel } from "../../services/inquiries.service";

    let { i18n, inquiriesState } = $props<{ i18n: any, inquiriesState: any }>();
</script>

<div class="filter-bar">
    <div class="search-field">
        <span class="search-icon">🔍</span>
        <input 
            type="text" 
            bind:value={inquiriesState.searchQuery} 
            placeholder={i18n.locale === 'ar' ? 'البحث بالاسم، البريد أو الهاتف...' : 'Search by name, email or phone...'} 
        />
    </div>

    <div class="filter-select-wrap">
        <select bind:value={inquiriesState.statusFilter} class="form-select status-select">
            <option value="all">{i18n.locale === 'ar' ? 'كل الطلبات' : 'All Leads'}</option>
            <option value="pending">{getStatusLabel('pending', i18n)}</option>
            <option value="replied">{getStatusLabel('replied', i18n)}</option>
            <option value="declined">{getStatusLabel('declined', i18n)}</option>
        </select>
    </div>
</div>
