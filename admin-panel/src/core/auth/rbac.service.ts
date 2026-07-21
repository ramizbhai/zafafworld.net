export interface User {
    id: string;
    email: string;
    role: string;
    scopes?: string[];
    first_name: string;
    last_name: string;
}

export class RBACService {
    static isSuperAdmin(user?: User | null): boolean {
        if (!user) return false;
        return user.role === 'Admin' || user.scopes?.includes('super_admin') || false;
    }

    static hasScope(user: User | null | undefined, scope: string): boolean {
        if (!user) return false;
        return user.scopes?.includes(scope) || false;
    }

    // Domain Specific Checks for Vendors
    static canApproveVendor(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'manage_vendors');
    }

    static canSuspendVendor(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'manage_vendors');
    }

    static canOverrideSubscription(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'manage_billing');
    }

    static canPromoteListing(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'manage_marketing');
    }

    static canModerateListings(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'manage_listings');
    }

    static canDeleteThread(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'delete_conversations');
    }

    static canFlagUser(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'flag_users');
    }

    static canBlockMessage(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'block_messages');
    }

    static canManageMarketing(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'manage_marketing');
    }

    static canSendCampaigns(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'send_campaigns');
    }

    static canModeratePromotions(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'moderate_promotions');
    }

    static canViewFinancials(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'view_financials');
    }

    static canViewSystemHealth(user?: User | null): boolean {
        return this.isSuperAdmin(user) || this.hasScope(user, 'view_system_health');
    }
}
