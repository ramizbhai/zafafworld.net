export function formatDate(iso: string | null | undefined): string {
    if (!iso) return "—";
    return new Date(iso).toLocaleDateString("en-SA", {
        year: "numeric",
        month: "short",
        day: "numeric",
    });
}

const statusColors: Record<string, string> = {
    approved: "#10b981",
    active: "#10b981",
    pending: "#f59e0b",
    pending_approval: "#f59e0b",
    suspended: "#ef4444",
    rejected: "#ef4444",
    draft: "#64748b",
    archived: "#374151",
};

export function getStatusColor(s: string): string {
    return statusColors[s] ?? "#64748b";
}

export function genderLabel(g: string | null | undefined): string {
    if (!g) return "—";
    const map: Record<string, string> = {
        women_only: "♀ Women Only",
        men_only: "♂ Men Only",
        dual_parallel: "⚡ Dual Parallel",
        mixed: "👥 Mixed",
        family: "🏠 Family",
    };
    return map[g] ?? g;
}

export function capacity(p: any): string {
    const attr = p.attributes || {};
    if (!attr.gender_section) return "—";
    if (attr.gender_section === "dual_parallel") {
        return `${attr.men_capacity ?? 0}M / ${attr.women_capacity ?? 0}W`;
    } else if (attr.gender_section === "men_only") {
        return `${attr.men_capacity ?? 0} Men`;
    }
    return `${attr.women_capacity ?? 0} Women`;
}

export function categoryLabel(slug: string): string {
    return slug?.replace(/-/g, " ").replace(/_/g, " ") ?? "—";
}

const VENUE_CATEGORIES = new Set([
    "wedding_hall",
    "reception_hall",
    "private_villa",
    "bridal_suite",
    "boardroom",
    "outdoor_garden",
    "rooftop",
    "private_beach",
    "hotel-venue",
    "wedding-palace",
    "villa-resort",
    "restaurant-event",
]);

export function isVenueListing(p: any): boolean {
    return VENUE_CATEGORIES.has(p.product_category ?? "");
}
