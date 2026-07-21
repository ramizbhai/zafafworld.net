<script lang="ts">
    import { Search, RefreshCw } from 'lucide-svelte';
    import { invalidateAll } from '$app/navigation';

    let { vendorsState, activeCount, stoppedCount, totalCount } = $props<{ vendorsState: any, activeCount: number, stoppedCount: number, totalCount: number }>();
</script>

<div class="controls-bar" style="display: flex; flex-direction: column; gap: 20px; margin-bottom: 24px; padding: 20px; background: var(--bg-elevated); border: 1px solid var(--glass-border); border-radius: 12px; box-shadow: var(--shadow-sm);">
    
    <!-- Top Row: Navigation Tabs & Refresh -->
    <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 16px;">
        <div class="tabs-container" style="display: flex; gap: 6px; background: var(--bg-surface); padding: 4px; border-radius: 8px; border: 1px solid var(--glass-border);">
            <button class="tab-btn" class:active={vendorsState.activeTab === 'active'} onclick={() => { vendorsState.activeTab = 'active'; }} style="padding: 6px 14px; font-size: 13px; font-weight: 600; border-radius: 6px; border: none; cursor: pointer; transition: all 0.2s;">
                Active / نشط <span style="margin-left: 6px; font-size: 11px; padding: 2px 6px; background: var(--success-dim); color: var(--success); border-radius: 4px; font-weight: 700;">{activeCount}</span>
            </button>
            <button class="tab-btn" class:active={vendorsState.activeTab === 'stopped'} onclick={() => { vendorsState.activeTab = 'stopped'; }} style="padding: 6px 14px; font-size: 13px; font-weight: 600; border-radius: 6px; border: none; cursor: pointer; transition: all 0.2s;">
                Stopped / موقوف <span style="margin-left: 6px; font-size: 11px; padding: 2px 6px; background: var(--danger-dim); color: var(--danger); border-radius: 4px; font-weight: 700;">{stoppedCount}</span>
            </button>
            <button class="tab-btn" class:active={vendorsState.activeTab === 'all'} onclick={() => { vendorsState.activeTab = 'all'; }} style="padding: 6px 14px; font-size: 13px; font-weight: 600; border-radius: 6px; border: none; cursor: pointer; transition: all 0.2s;">
                All / الكل <span style="margin-left: 6px; font-size: 11px; padding: 2px 6px; background: var(--glass-border); color: var(--text-secondary); border-radius: 4px; font-weight: 700;">{totalCount}</span>
            </button>
        </div>

        <button class="btn btn-outline" onclick={() => invalidateAll()} style="display: inline-flex; align-items: center; gap: 8px; padding: 8px 16px; font-size: 13px; font-weight: 700; border-radius: 8px; cursor: pointer;">
            <RefreshCw size={14} /> Refresh List
        </button>
    </div>

    <!-- Bottom Row: Filter Inputs in Responsive Flex Row Layout -->
    <div class="filters-row-layout" style="display: flex; flex-wrap: wrap; gap: 16px; align-items: flex-end; width: 100%;">
        
        <!-- Keyword Search Input (Fills extra space) -->
        <div class="filter-item" style="flex: 1.5; min-width: 280px; display: flex; flex-direction: column; gap: 6px;">
            <label style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase; letter-spacing: 0.5px;">Search Keywords</label>
            <div style="position: relative; display: flex; align-items: center;">
                <Search size={14} style="position: absolute; left: 12px; color: var(--text-ghost);" />
                <input 
                    type="search" 
                    placeholder="Search name, email, phone, category..." 
                    bind:value={vendorsState.search} 
                    style="width: 100%; height: 38px; padding: 8px 12px 8px 36px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none; transition: border-color 0.2s;"
                />
            </div>
        </div>

        <!-- Account Status Filter -->
        <div class="filter-item" style="flex: 1; min-width: 180px; display: flex; flex-direction: column; gap: 6px;">
            <label for="acct_status_sel" style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase; letter-spacing: 0.5px;">Account Status</label>
            <select id="acct_status_sel" bind:value={vendorsState.accountStatusFilter} style="width: 100%; height: 38px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none; cursor: pointer;">
                <option value="all">All Statuses</option>
                <option value="active">Active</option>
                <option value="pending">Pending</option>
                <option value="suspended">Suspended</option>
                <option value="rejected">Rejected</option>
                <option value="banned">Banned</option>
            </select>
        </div>

        <!-- Subscription Tier Filter -->
        <div class="filter-item" style="flex: 1; min-width: 180px; display: flex; flex-direction: column; gap: 6px;">
            <label for="sub_plan_sel" style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase; letter-spacing: 0.5px;">Subscription Plan</label>
            <select id="sub_plan_sel" bind:value={vendorsState.subTierFilter} style="width: 100%; height: 38px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none; cursor: pointer;">
                <option value="all">All Plans</option>
                <option value="Free">Free Plan</option>
                <option value="Gold">Golden Plan</option>
                <option value="VIP">VIP Plan</option>
                <option value="Diamond">Diamond Plan</option>
            </select>
        </div>

        <!-- Subscription Status Filter -->
        <div class="filter-item" style="flex: 1; min-width: 180px; display: flex; flex-direction: column; gap: 6px;">
            <label for="billing_state_sel" style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase; letter-spacing: 0.5px;">Billing State</label>
            <select id="billing_state_sel" bind:value={vendorsState.subStatusFilter} style="width: 100%; height: 38px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none; cursor: pointer;">
                <option value="all">All Billing</option>
                <option value="active">Active Sub</option>
                <option value="trial">Trial Sub</option>
                <option value="stopped">Stopped</option>
            </select>
        </div>
    </div>
</div>

<style>
    .tab-btn { background: transparent; color: var(--text-secondary); }
    .tab-btn:hover { color: var(--text-primary); }
    .tab-btn.active { background: var(--bg-elevated); color: var(--gold); border: 1px solid var(--glass-border); box-shadow: var(--shadow-sm); }
</style>
