-- ═══════════════════════════════════════════════════════════════════════════════
-- Baseline Migration 06: 0006_business_flow.sql
-- Consolidated ZafafWorld Database Schema Baseline
-- ═══════════════════════════════════════════════════════════════════════════════


--
-- Name: Client_Budgets; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.client_budgets (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    total_budget numeric(12,2) NOT NULL,
    spent_amount numeric(12,2) DEFAULT 0.00 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_positive_budget CHECK ((total_budget >= (0)::numeric)),
    CONSTRAINT chk_positive_spent CHECK ((spent_amount >= (0)::numeric))
);

--
-- Name: Core_Bookings; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.core_bookings (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    booking_number character varying(50) NOT NULL,
    vendor_id uuid NOT NULL,
    client_id uuid,
    product_id uuid NOT NULL,
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    wedding_date date NOT NULL,
    event_type character varying(100) NOT NULL,
    guest_count integer NOT NULL,
    total_price numeric(12,2) NOT NULL,
    deposit_paid numeric(12,2) NOT NULL,
    customer_first_name character varying(100) NOT NULL,
    customer_last_name character varying(100) NOT NULL,
    customer_phone character varying(50) NOT NULL,
    customer_email character varying(255) NOT NULL,
    special_requests text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    city_id uuid,
    CONSTRAINT chk_booking_deposit_paid CHECK ((deposit_paid >= (0)::numeric)),
    CONSTRAINT chk_booking_email CHECK (((customer_email)::text ~~ '%@%'::text)),
    CONSTRAINT chk_booking_guest_count CHECK ((guest_count > 0)),
    CONSTRAINT chk_booking_status CHECK (((status)::text = ANY ((ARRAY['Draft_Inquiry'::character varying, 'Pending_Vendor_Acceptance'::character varying, 'Escrow_Verified'::character varying, 'Booking_Active'::character varying, 'pending'::character varying, 'confirmed'::character varying, 'cancelled'::character varying])::text[]))),
    CONSTRAINT chk_booking_total_price CHECK ((total_price > (0)::numeric))
);

--
-- Name: Lead_Inquiries; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.lead_inquiries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    client_id uuid,
    product_id uuid,
    customer_name character varying(100) NOT NULL,
    phone character varying(50),
    wedding_date date NOT NULL,
    message text NOT NULL,
    resolution_note text,
    status character varying(50) DEFAULT 'new'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    city_id uuid,
    CONSTRAINT chk_lead_inquiry_status CHECK (((status)::text = ANY ((ARRAY['new'::character varying, 'read'::character varying, 'done'::character varying, 'expired'::character varying, 'rejected'::character varying, 'negotiation'::character varying, 'unreachable'::character varying, 'paid'::character varying])::text[])))
);

--

--
-- Name: admin_settings; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.admin_settings (
    key character varying(100) NOT NULL,
    value text NOT NULL,
    updated_by uuid,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: afrah_inquiries; Type: TABLE; Schema: public; Owner: zafaf_db_admin
--

CREATE TABLE public.afrah_inquiries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(255) NOT NULL,
    phone character varying(50) NOT NULL,
    is_whatsapp boolean DEFAULT true NOT NULL,
    event_date date NOT NULL,
    message text,
    email character varying(255),
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    ip_address character varying(50),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_afrah_inquiry_status CHECK (((status)::text = ANY ((ARRAY['pending'::character varying, 'contacted'::character varying, 'resolved'::character varying])::text[])))
);

--
-- Name: assistant_inquiries; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.assistant_inquiries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    message text NOT NULL,
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    city_id uuid,
    CONSTRAINT chk_assistant_inquiry_status CHECK (((status)::text = ANY ((ARRAY['pending'::character varying, 'resolved'::character varying])::text[])))
);

--
-- Name: blog_categories; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.blog_categories (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    slug text NOT NULL
);

--
-- Name: blog_category_map; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.blog_category_map (
    blog_id uuid NOT NULL,
    category_id uuid NOT NULL
);

--
-- Name: blog_comments; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.blog_comments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    blog_id uuid NOT NULL,
    user_id uuid,
    name text NOT NULL,
    email text,
    comment text NOT NULL,
    is_approved boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    parent_id uuid
);

--
-- Name: blog_funnel_events; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.blog_funnel_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    blog_slug character varying(255) NOT NULL,
    event_type character varying(50) NOT NULL,
    session_id character varying(255),
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: blog_tags; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.blog_tags (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    slug text NOT NULL
);

--
-- Name: blog_tags_map; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.blog_tags_map (
    blog_id uuid NOT NULL,
    tag_id uuid NOT NULL
);

--
-- Name: blog_views; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.blog_views (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    blog_id uuid NOT NULL,
    ip_hash text NOT NULL,
    user_id uuid,
    viewed_date date DEFAULT CURRENT_DATE NOT NULL,
    viewed_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: blogs; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.blogs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    slug text NOT NULL,
    title text NOT NULL,
    excerpt text,
    content_html text NOT NULL,
    content_markdown text NOT NULL,
    cover_image_url text,
    author_id uuid NOT NULL,
    meta_title text,
    meta_description text,
    focus_keywords text,
    published_at timestamp with time zone,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    read_time_minutes integer DEFAULT 5 NOT NULL,
    is_published boolean DEFAULT false NOT NULL,
    title_ar text,
    title_en text,
    meta_title_ar text,
    meta_title_en text,
    meta_description_ar text,
    meta_description_en text,
    wp_post_id bigint,
    source character varying(20) DEFAULT 'internal'::character varying NOT NULL,
    lang character varying(5) DEFAULT 'en'::character varying NOT NULL,
    translation_group_id bigint,
    canonical_url text
);

--
-- Name: booking_quotation_revisions; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.booking_quotation_revisions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    quotation_id uuid NOT NULL,
    version integer NOT NULL,
    total_price numeric(12,2) NOT NULL,
    deposit_amount numeric(12,2) NOT NULL,
    sender_role character varying(20) NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now()
);

--
-- Name: booking_quotations; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.booking_quotations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    booking_id uuid NOT NULL,
    client_id uuid NOT NULL,
    vendor_id uuid NOT NULL,
    current_version integer DEFAULT 1 NOT NULL,
    total_price numeric(12,2) NOT NULL,
    deposit_amount numeric(12,2) NOT NULL,
    status character varying(50) DEFAULT 'Pending_Client_Review'::character varying NOT NULL,
    expires_at timestamp with time zone,
    notes text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

--
-- Name: client_budget_items; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.client_budget_items (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    category character varying(100) NOT NULL,
    title character varying(255) NOT NULL,
    planned_amount numeric(12,2) DEFAULT 0.00 NOT NULL,
    actual_amount numeric(12,2) DEFAULT 0.00 NOT NULL,
    status character varying(50) DEFAULT 'Planned'::character varying NOT NULL,
    due_date date,
    notes text,
    booking_id uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

--
-- Name: client_documents; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.client_documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    file_name character varying(255) NOT NULL,
    file_url text NOT NULL,
    category character varying(100) DEFAULT 'Contract'::character varying NOT NULL,
    booking_id uuid,
    created_at timestamp with time zone DEFAULT now(),
    deleted_at timestamp with time zone
);

--
-- Name: client_favorites; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.client_favorites (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    vendor_id uuid NOT NULL,
    shortlist_name character varying(100) DEFAULT 'Default'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);

--
-- Name: client_tasks; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.client_tasks (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    title character varying(255) NOT NULL,
    category character varying(100) DEFAULT 'General'::character varying NOT NULL,
    due_date date,
    priority character varying(20) DEFAULT 'Medium'::character varying NOT NULL,
    is_completed boolean DEFAULT false NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now()
);

--
-- Name: client_timeline_events; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.client_timeline_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    title character varying(255) NOT NULL,
    event_date date NOT NULL,
    event_type character varying(50) DEFAULT 'Custom'::character varying NOT NULL,
    is_completed boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);

--
-- Name: conversation_participants; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.conversation_participants (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    conversation_id uuid NOT NULL,
    user_id uuid NOT NULL,
    joined_at timestamp with time zone DEFAULT now() NOT NULL,
    archived boolean DEFAULT false NOT NULL
);

--
-- Name: conversations; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.conversations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    title character varying(255),
    status character varying(50) DEFAULT 'active'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    product_id uuid,
    city_id uuid,
    CONSTRAINT chk_conversation_status CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'archived'::character varying, 'closed'::character varying])::text[])))
);

--
-- Name: csrf_tokens; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.csrf_tokens (
    token character varying(64) NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone DEFAULT (now() + '01:00:00'::interval) NOT NULL
);

--
-- Name: escrow_accounts; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.escrow_accounts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    booking_id uuid NOT NULL,
    vendor_id uuid NOT NULL,
    client_id uuid NOT NULL,
    amount_held numeric(12,2) NOT NULL,
    status character varying(50) DEFAULT 'Held'::character varying NOT NULL,
    dispute_reason text,
    released_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now()
);

--
-- Name: invoices; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.invoices (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    invoice_number character varying(100) NOT NULL,
    booking_id uuid,
    client_id uuid NOT NULL,
    vendor_id uuid NOT NULL,
    amount numeric(12,2) NOT NULL,
    tax_amount numeric(12,2) DEFAULT 0.00 NOT NULL,
    status character varying(50) DEFAULT 'Issued'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);

--
-- Name: listing_promotions; Type: TABLE; Schema: public; Owner: zafaf_db_admin
--

CREATE TABLE public.listing_promotions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    title_en character varying(255) NOT NULL,
    title_ar character varying(255) NOT NULL,
    description_en text,
    description_ar text,
    discount_percentage integer NOT NULL,
    badge_text_en character varying(50),
    badge_text_ar character varying(50),
    banner_image_url character varying(255),
    start_at timestamp with time zone NOT NULL,
    end_at timestamp with time zone NOT NULL,
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    is_featured boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    rejection_reason text,
    listing_id uuid NOT NULL,
    promo_type character varying(30) DEFAULT 'discount'::character varying NOT NULL,
    discount_type character varying(30),
    discount_fixed_amount numeric(10,2),
    benefit_description_en character varying(255),
    benefit_description_ar character varying(255),
    use_listing_cover_image boolean DEFAULT true NOT NULL,
    custom_banner_image_url character varying(255),
    display_priority integer DEFAULT 0 NOT NULL,
    views_count integer DEFAULT 0 NOT NULL,
    clicks_count integer DEFAULT 0 NOT NULL,
    inquiries_count integer DEFAULT 0 NOT NULL,
    CONSTRAINT chk_discount_type CHECK (((discount_type)::text = ANY ((ARRAY['percentage'::character varying, 'fixed_amount'::character varying])::text[]))),
    CONSTRAINT chk_fixed_amount_positive CHECK (((((discount_type)::text = 'fixed_amount'::text) AND (discount_fixed_amount > 0.00)) OR (((discount_type)::text <> 'fixed_amount'::text) OR (discount_type IS NULL)))),
    CONSTRAINT chk_promo_dates CHECK ((end_at > start_at)),
    CONSTRAINT chk_promo_discount CHECK (((discount_percentage >= 5) AND (discount_percentage <= 90))),
    CONSTRAINT chk_promo_status CHECK (((status)::text = ANY ((ARRAY['draft'::character varying, 'pending'::character varying, 'approved'::character varying, 'rejected'::character varying, 'paused'::character varying, 'cancelled'::character varying])::text[]))),
    CONSTRAINT chk_promo_type CHECK (((promo_type)::text = ANY ((ARRAY['discount'::character varying, 'benefit'::character varying])::text[])))
);

--
-- Name: message_attachments; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.message_attachments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    message_id uuid NOT NULL,
    file_name character varying(255) NOT NULL,
    file_url text NOT NULL,
    file_type character varying(100) NOT NULL,
    file_size integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: message_read_receipts; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.message_read_receipts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    message_id uuid NOT NULL,
    user_id uuid NOT NULL,
    read_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: messages; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.messages (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    conversation_id uuid NOT NULL,
    sender_id uuid NOT NULL,
    body text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: notification_outbox; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.notification_outbox (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    event_type character varying(100) NOT NULL,
    aggregate_type character varying(100) NOT NULL,
    aggregate_id uuid NOT NULL,
    payload jsonb NOT NULL,
    status character varying(50) DEFAULT 'PENDING'::character varying NOT NULL,
    attempt_count integer DEFAULT 0 NOT NULL,
    last_attempt_at timestamp with time zone,
    next_retry_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    delivered_at timestamp with time zone,
    error_message text,
    channel_delivery jsonb DEFAULT '{}'::jsonb NOT NULL,
    CONSTRAINT chk_notification_outbox_attempts CHECK ((attempt_count >= 0)),
    CONSTRAINT chk_notification_outbox_status CHECK (((status)::text = ANY ((ARRAY['PENDING'::character varying, 'PROCESSING'::character varying, 'DELIVERED'::character varying, 'FAILED'::character varying, 'RETRYING'::character varying])::text[])))
);

--
-- Name: password_reset_tokens; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.password_reset_tokens (
    token character varying(64) NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL
);

--
-- Name: payment_intents; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.payment_intents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    booking_id uuid,
    client_id uuid NOT NULL,
    vendor_id uuid NOT NULL,
    amount numeric(12,2) NOT NULL,
    currency character varying(10) DEFAULT 'SAR'::character varying NOT NULL,
    payment_method character varying(50) NOT NULL,
    provider character varying(50) DEFAULT 'Tap'::character varying NOT NULL,
    status character varying(50) DEFAULT 'Pending'::character varying NOT NULL,
    transaction_reference character varying(255),
    idempotency_key character varying(255),
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

--
-- Name: payout_requests; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.payout_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    amount numeric(12,2) NOT NULL,
    status character varying(50) DEFAULT 'Pending'::character varying NOT NULL,
    bank_name character varying(255),
    iban character varying(255),
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

--
-- Name: promotion_audit_logs; Type: TABLE; Schema: public; Owner: zafaf_db_admin
--

CREATE TABLE public.promotion_audit_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    promotion_id uuid NOT NULL,
    actor_user_id uuid NOT NULL,
    action character varying(50) NOT NULL,
    previous_status character varying(50),
    new_status character varying(50),
    payload jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_audit_action CHECK (((action)::text = ANY ((ARRAY['create'::character varying, 'update'::character varying, 'approve'::character varying, 'reject'::character varying, 'pause'::character varying, 'resume'::character varying, 'archive'::character varying, 'cancel'::character varying, 'delete'::character varying])::text[]))),
    CONSTRAINT chk_audit_new_status CHECK (((new_status)::text = ANY ((ARRAY['draft'::character varying, 'pending'::character varying, 'approved'::character varying, 'rejected'::character varying, 'paused'::character varying, 'cancelled'::character varying])::text[]))),
    CONSTRAINT chk_audit_prev_status CHECK (((previous_status)::text = ANY ((ARRAY['draft'::character varying, 'pending'::character varying, 'approved'::character varying, 'rejected'::character varying, 'paused'::character varying, 'cancelled'::character varying])::text[])))
);

--
-- Name: seo_articles; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.seo_articles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    slug character varying(255) NOT NULL,
    category character varying(100) NOT NULL,
    title_ar character varying(255) NOT NULL,
    title_en character varying(255) NOT NULL,
    summary_ar text,
    summary_en text,
    body_ar text,
    body_en text,
    published boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: system_events; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.system_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    target_vendor_id uuid,
    event_type character varying(50) NOT NULL,
    message_ar text NOT NULL,
    message_en text NOT NULL,
    is_read boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: vendor_inquiries; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_inquiries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid,
    vendor_id uuid NOT NULL,
    event_date date NOT NULL,
    guest_count integer NOT NULL,
    message text NOT NULL,
    status character varying(50) DEFAULT 'unread'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    product_id uuid,
    name character varying(100),
    phone character varying(50),
    email character varying(255),
    conversation_id uuid,
    city_id uuid,
    CONSTRAINT chk_vendor_inquiry_status CHECK (((status)::text = ANY ((ARRAY['unread'::character varying, 'viewed'::character varying, 'pending'::character varying, 'replied'::character varying, 'closed'::character varying, 'declined'::character varying])::text[]))),
    CONSTRAINT chk_vi_guest_count CHECK ((guest_count > 0))
);

--
-- Name: vendor_inquiry_admin_notes; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_inquiry_admin_notes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    inquiry_id uuid NOT NULL,
    admin_id uuid NOT NULL,
    note text NOT NULL,
    note_type character varying(50) DEFAULT 'internal'::character varying NOT NULL,
    is_internal boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_vian_note_type CHECK (((note_type)::text = ANY ((ARRAY['internal'::character varying, 'customer_followup'::character varying, 'vendor_followup'::character varying, 'escalation'::character varying, 'resolution'::character varying])::text[])))
);

--
-- Name: vendor_inquiry_management; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_inquiry_management (
    inquiry_id uuid NOT NULL,
    assigned_admin_id uuid,
    escalation_status character varying(50) DEFAULT 'none'::character varying NOT NULL,
    resolution_status character varying(50) DEFAULT 'unresolved'::character varying NOT NULL,
    priority character varying(50) DEFAULT 'medium'::character varying NOT NULL,
    assigned_at timestamp with time zone,
    escalated_at timestamp with time zone,
    resolved_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_vim_escalation_status CHECK (((escalation_status)::text = ANY ((ARRAY['none'::character varying, 'pending'::character varying, 'escalated'::character varying, 'resolved'::character varying])::text[]))),
    CONSTRAINT chk_vim_priority CHECK (((priority)::text = ANY ((ARRAY['low'::character varying, 'medium'::character varying, 'high'::character varying, 'critical'::character varying])::text[]))),
    CONSTRAINT chk_vim_resolution_status CHECK (((resolution_status)::text = ANY ((ARRAY['unresolved'::character varying, 'in_progress'::character varying, 'resolved'::character varying, 'closed_no_action'::character varying])::text[])))
);

--
-- Name: vendor_staff; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_staff (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    role character varying(50) NOT NULL,
    status character varying(50) DEFAULT 'active'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT vendor_staff_role_check CHECK (((role)::text = ANY ((ARRAY['admin'::character varying, 'editor'::character varying, 'viewer'::character varying])::text[]))),
    CONSTRAINT vendor_staff_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying])::text[])))
);

--
-- Name: vendor_tasks; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_tasks (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    title_ar character varying(255) NOT NULL,
    title_en character varying(255) NOT NULL,
    is_completed boolean DEFAULT false NOT NULL,
    due_date timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: vendor_whatsapp_templates; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_whatsapp_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    template_name character varying(255) NOT NULL,
    body_text_ar text,
    body_text_en text,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: fn_check_promotion_ownership_and_overlap(); Type: FUNCTION; Schema: public; Owner: zafaf_db_admin
--

CREATE FUNCTION public.fn_check_promotion_ownership_and_overlap() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    product_vendor_id UUID;
BEGIN
    -- 1. Verify Listing Ownership
    SELECT vendor_id INTO product_vendor_id FROM vendor_products WHERE id = NEW.listing_id;
    IF product_vendor_id IS NULL OR product_vendor_id != NEW.vendor_id THEN
        RAISE EXCEPTION 'Ownership violation: Listing % does not belong to vendor %.', 
            NEW.listing_id, NEW.vendor_id;
    END IF;

    -- 2. Verify Date Overlap (only for active/pending/paused states)
    IF NEW.status IN ('approved', 'pending', 'paused') THEN
        -- Row lock to prevent race conditions during parallel transactions
        PERFORM 1 FROM vendor_products WHERE id = NEW.listing_id FOR UPDATE;

        IF EXISTS (
            SELECT 1 FROM listing_promotions
            WHERE listing_id = NEW.listing_id
              AND status IN ('approved', 'pending', 'paused')
              AND id != NEW.id
              AND tstzrange(start_at, end_at, '[]') && tstzrange(NEW.start_at, NEW.end_at, '[]')
        ) THEN
            RAISE EXCEPTION 'Overlap conflict: Listing % already has an active, pending, or paused promotion during this range (% to %).',
                NEW.listing_id, NEW.start_at, NEW.end_at;
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

--
-- Name: fn_promo_is_active(character varying, timestamp with time zone, timestamp with time zone); Type: FUNCTION; Schema: public; Owner: zafaf_db_admin
--

CREATE FUNCTION public.fn_promo_is_active(status character varying, start_at timestamp with time zone, end_at timestamp with time zone) RETURNS boolean
    LANGUAGE sql STABLE
    AS $$
    SELECT status = 'approved' AND NOW() >= start_at AND NOW() <= end_at;
$$;

--
-- Name: is_conversation_participant(uuid, uuid); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.is_conversation_participant(conv_id uuid, u_id uuid) RETURNS boolean
    LANGUAGE sql SECURITY DEFINER
    SET search_path = public, pg_temp
    AS $$
    SELECT EXISTS (
        SELECT 1 FROM conversation_participants
        WHERE conversation_id = conv_id AND user_id = u_id
    );
$$;

--
-- Name: is_conversation_participant_or_empty(uuid, uuid); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.is_conversation_participant_or_empty(conv_id uuid, u_id uuid) RETURNS boolean
    LANGUAGE sql SECURITY DEFINER
    SET search_path = public, pg_temp
    AS $$
    SELECT NOT EXISTS (
        SELECT 1 FROM conversation_participants WHERE conversation_id = conv_id
    ) OR EXISTS (
        SELECT 1 FROM conversation_participants
        WHERE conversation_id = conv_id AND user_id = u_id
    );
$$;

--
-- Name: idx_assistant_inquiries_city; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_assistant_inquiries_city ON public.assistant_inquiries USING btree (city_id);

--
-- Name: idx_assistant_inquiries_client; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_assistant_inquiries_client ON public.assistant_inquiries USING btree (client_id);

--
-- Name: idx_assistant_inquiries_created_at; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_assistant_inquiries_created_at ON public.assistant_inquiries USING btree (created_at);

--
-- Name: idx_assistant_inquiries_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_assistant_inquiries_status ON public.assistant_inquiries USING btree (status);

--
-- Name: idx_bc_approved_created; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_bc_approved_created ON public.blog_comments USING btree (blog_id, created_at) WHERE (is_approved = true);

--
-- Name: idx_blog_funnel_event; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_blog_funnel_event ON public.blog_funnel_events USING btree (event_type);

--
-- Name: idx_blog_funnel_slug; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_blog_funnel_slug ON public.blog_funnel_events USING btree (blog_slug);

--
-- Name: idx_blog_views_unique_daily; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE UNIQUE INDEX idx_blog_views_unique_daily ON public.blog_views USING btree (blog_id, ip_hash, viewed_date);

--
-- Name: idx_blogs_lang; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_blogs_lang ON public.blogs USING btree (lang);

--
-- Name: idx_blogs_pub_date; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_blogs_pub_date ON public.blogs USING btree (published_at DESC) WHERE (is_published = true);

--
-- Name: idx_blogs_source; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_blogs_source ON public.blogs USING btree (source);

--
-- Name: idx_blogs_translation_group_id; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_blogs_translation_group_id ON public.blogs USING btree (translation_group_id);

--
-- Name: idx_blogs_wp_post_id; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_blogs_wp_post_id ON public.blogs USING btree (wp_post_id) WHERE (wp_post_id IS NOT NULL);

--
-- Name: idx_cb_vendor_created; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_cb_vendor_created ON public.core_bookings USING btree (vendor_id, created_at DESC);

--
-- Name: idx_conversation_participants_user; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_conversation_participants_user ON public.conversation_participants USING btree (user_id);

--
-- Name: idx_conversations_city; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_conversations_city ON public.conversations USING btree (city_id);

--
-- Name: idx_conversations_product; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_conversations_product ON public.conversations USING btree (product_id) WHERE (product_id IS NOT NULL);

--
-- Name: idx_core_bookings_city; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_core_bookings_city ON public.core_bookings USING btree (city_id);

--
-- Name: idx_core_bookings_client; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_core_bookings_client ON public.core_bookings USING btree (client_id);

--
-- Name: idx_core_bookings_product; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_core_bookings_product ON public.core_bookings USING btree (product_id) WHERE (product_id IS NOT NULL);

--
-- Name: idx_core_bookings_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_core_bookings_status ON public.core_bookings USING btree (status);

--
-- Name: idx_core_bookings_created_at; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_core_bookings_created_at ON public.core_bookings USING btree (created_at DESC);

--
-- Name: idx_core_bookings_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_core_bookings_vendor ON public.core_bookings USING btree (vendor_id);

--
-- Name: idx_core_bookings_wedding_date; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_core_bookings_wedding_date ON public.core_bookings USING btree (wedding_date);

--
-- Name: idx_csrf_tokens_expires; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_csrf_tokens_expires ON public.csrf_tokens USING btree (expires_at);

--
-- Name: idx_csrf_tokens_user; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_csrf_tokens_user ON public.csrf_tokens USING btree (user_id);

--
-- Name: idx_lead_inq_product; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_lead_inq_product ON public.lead_inquiries USING btree (product_id) WHERE (product_id IS NOT NULL);

--
-- Name: idx_lead_inquiries_city; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_lead_inquiries_city ON public.lead_inquiries USING btree (city_id);

--
-- Name: idx_lead_inquiries_client; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_lead_inquiries_client ON public.lead_inquiries USING btree (client_id);

--
-- Name: idx_lead_inquiries_product; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_lead_inquiries_product ON public.lead_inquiries USING btree (product_id) WHERE (product_id IS NOT NULL);

--
-- Name: idx_lead_inquiries_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_lead_inquiries_status ON public.lead_inquiries USING btree (status);

--
-- Name: idx_lead_inquiries_created_at; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_lead_inquiries_created_at ON public.lead_inquiries USING btree (created_at DESC);

--
-- Name: idx_lead_inquiries_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_lead_inquiries_vendor ON public.lead_inquiries USING btree (vendor_id);

--
-- Name: idx_listing_promotions_approved_active; Type: INDEX; Schema: public; Owner: zafaf_db_admin
--

CREATE INDEX idx_listing_promotions_approved_active ON public.listing_promotions USING btree (start_at, end_at) WHERE ((status)::text = 'approved'::text);

--
-- Name: idx_listing_promotions_dates; Type: INDEX; Schema: public; Owner: zafaf_db_admin
--

CREATE INDEX idx_listing_promotions_dates ON public.listing_promotions USING btree (start_at, end_at);

--
-- Name: idx_listing_promotions_listing; Type: INDEX; Schema: public; Owner: zafaf_db_admin
--

CREATE INDEX idx_listing_promotions_listing ON public.listing_promotions USING btree (listing_id);

--
-- Name: idx_listing_promotions_priority; Type: INDEX; Schema: public; Owner: zafaf_db_admin
--

CREATE INDEX idx_listing_promotions_priority ON public.listing_promotions USING btree (display_priority DESC);

--
-- Name: idx_listing_promotions_status; Type: INDEX; Schema: public; Owner: zafaf_db_admin
--

CREATE INDEX idx_listing_promotions_status ON public.listing_promotions USING btree (status);

--
-- Name: idx_listing_promotions_type; Type: INDEX; Schema: public; Owner: zafaf_db_admin
--

CREATE INDEX idx_listing_promotions_type ON public.listing_promotions USING btree (promo_type);

--
-- Name: idx_listing_promotions_vendor; Type: INDEX; Schema: public; Owner: zafaf_db_admin
--

CREATE INDEX idx_listing_promotions_vendor ON public.listing_promotions USING btree (vendor_id);

--
-- Name: idx_message_attachments_message; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_message_attachments_message ON public.message_attachments USING btree (message_id);

--
-- Name: idx_message_read_receipts_message; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_message_read_receipts_message ON public.message_read_receipts USING btree (message_id);

--
-- Name: idx_messages_conversation_created; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_messages_conversation_created ON public.messages USING btree (conversation_id, created_at DESC);

--
-- Name: idx_notification_outbox_poll; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_notification_outbox_poll ON public.notification_outbox USING btree (status, next_retry_at) WHERE ((status)::text = ANY ((ARRAY['PENDING'::character varying, 'RETRYING'::character varying])::text[]));

--
-- Name: idx_password_reset_tokens_expires; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_password_reset_tokens_expires ON public.password_reset_tokens USING btree (expires_at);

--
-- Name: idx_promotion_audit_logs_promo; Type: INDEX; Schema: public; Owner: zafaf_db_admin
--

CREATE INDEX idx_promotion_audit_logs_promo ON public.promotion_audit_logs USING btree (promotion_id);

--
-- Name: idx_seo_articles_category; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_seo_articles_category ON public.seo_articles USING btree (category);

--
-- Name: idx_seo_articles_published; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_seo_articles_published ON public.seo_articles USING btree (published);

--
-- Name: idx_system_events_type; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_system_events_type ON public.system_events USING btree (event_type);

--
-- Name: idx_system_events_user; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_system_events_user ON public.system_events USING btree (user_id);

--
-- Name: idx_system_events_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_system_events_vendor ON public.system_events USING btree (target_vendor_id);

--
-- Name: idx_vendor_inquiries_city; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_inquiries_city ON public.vendor_inquiries USING btree (city_id);

--
-- Name: idx_vendor_inquiries_client; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_inquiries_client ON public.vendor_inquiries USING btree (client_id);

--
-- Name: idx_vendor_inquiries_client_general_duplicate; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE UNIQUE INDEX idx_vendor_inquiries_client_general_duplicate ON public.vendor_inquiries USING btree (client_id, vendor_id, event_date) WHERE ((client_id IS NOT NULL) AND (product_id IS NULL));

--
-- Name: idx_vendor_inquiries_client_product_duplicate; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE UNIQUE INDEX idx_vendor_inquiries_client_product_duplicate ON public.vendor_inquiries USING btree (client_id, vendor_id, product_id, event_date) WHERE ((client_id IS NOT NULL) AND (product_id IS NOT NULL));

--
-- Name: idx_vendor_inquiries_conversation; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_inquiries_conversation ON public.vendor_inquiries USING btree (conversation_id) WHERE (conversation_id IS NOT NULL);

--
-- Name: idx_vendor_inquiries_created_at; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_inquiries_created_at ON public.vendor_inquiries USING btree (created_at DESC);

--
-- Name: idx_vendor_inquiries_guest_general_duplicate; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE UNIQUE INDEX idx_vendor_inquiries_guest_general_duplicate ON public.vendor_inquiries USING btree (phone, vendor_id, event_date) WHERE ((phone IS NOT NULL) AND (client_id IS NULL) AND (product_id IS NULL));

--
-- Name: idx_vendor_inquiries_guest_product_duplicate; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE UNIQUE INDEX idx_vendor_inquiries_guest_product_duplicate ON public.vendor_inquiries USING btree (phone, vendor_id, product_id, event_date) WHERE ((phone IS NOT NULL) AND (client_id IS NULL) AND (product_id IS NOT NULL));

--
-- Name: idx_vendor_inquiries_product; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_inquiries_product ON public.vendor_inquiries USING btree (product_id) WHERE (product_id IS NOT NULL);

--
-- Name: idx_vendor_inquiries_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_inquiries_status ON public.vendor_inquiries USING btree (status);

--
-- Name: idx_vendor_inquiries_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_inquiries_vendor ON public.vendor_inquiries USING btree (vendor_id);

--
-- Name: idx_vendor_staff_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_staff_vendor ON public.vendor_staff USING btree (vendor_id);

--
-- Name: idx_vendor_tasks_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_tasks_vendor ON public.vendor_tasks USING btree (vendor_id);

--
-- Name: idx_vendor_whatsapp_templates_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_whatsapp_templates_vendor ON public.vendor_whatsapp_templates USING btree (vendor_id);

--
-- Name: idx_vi_vendor_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vi_vendor_status ON public.vendor_inquiries USING btree (vendor_id, status);

--
-- Name: idx_vian_inquiry_id; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vian_inquiry_id ON public.vendor_inquiry_admin_notes USING btree (inquiry_id);

--
-- Name: idx_vim_assigned_admin; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vim_assigned_admin ON public.vendor_inquiry_management USING btree (assigned_admin_id);

--
-- Name: idx_vim_escalation_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vim_escalation_status ON public.vendor_inquiry_management USING btree (escalation_status);

--
-- Name: idx_vim_priority; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vim_priority ON public.vendor_inquiry_management USING btree (priority);

--
-- Name: idx_vim_resolution_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vim_resolution_status ON public.vendor_inquiry_management USING btree (resolution_status);

--
-- Name: blog_views Admins can view all views; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Admins can view all views" ON public.blog_views FOR SELECT TO app_admin_role USING (true);

--
-- Name: blog_funnel_events Admins can view funnel events; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Admins can view funnel events" ON public.blog_funnel_events FOR SELECT TO app_admin_role USING (true);

--
-- Name: blogs Admins have full access to blogs; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Admins have full access to blogs" ON public.blogs TO app_admin_role USING (true);

--
-- Name: blog_categories Admins have full access to categories; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Admins have full access to categories" ON public.blog_categories TO app_admin_role USING (true);

--
-- Name: blog_category_map Admins have full access to category maps; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Admins have full access to category maps" ON public.blog_category_map TO app_admin_role USING (true);

--
-- Name: blog_comments Admins have full access to comments; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Admins have full access to comments" ON public.blog_comments TO app_admin_role USING (true);

--
-- Name: blog_tags_map Admins have full access to tag maps; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Admins have full access to tag maps" ON public.blog_tags_map TO app_admin_role USING (true);

--
-- Name: blog_tags Admins have full access to tags; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Admins have full access to tags" ON public.blog_tags TO app_admin_role USING (true);

--
-- Name: blog_funnel_events Anyone can insert funnel events via backend; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Anyone can insert funnel events via backend" ON public.blog_funnel_events FOR INSERT WITH CHECK (true);

--
-- Name: blog_views Anyone can insert views via backend; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Anyone can insert views via backend" ON public.blog_views FOR INSERT WITH CHECK (true);

-- Authenticated comments insertion check is combined in the unified comments_insert policy below.

--
-- Name: blog_comments Public can view approved comments; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Public can view approved comments" ON public.blog_comments FOR SELECT USING ((is_approved = true));

--
-- Name: blog_categories Public can view categories; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Public can view categories" ON public.blog_categories FOR SELECT USING (true);

--
-- Name: blog_category_map Public can view category maps; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Public can view category maps" ON public.blog_category_map FOR SELECT USING (true);

--
-- Name: blogs Public can view published blogs; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Public can view published blogs" ON public.blogs FOR SELECT USING ((is_published = true));

--
-- Name: blog_tags_map Public can view tag maps; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Public can view tag maps" ON public.blog_tags_map FOR SELECT USING (true);

--
-- Name: blog_tags Public can view tags; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Public can view tags" ON public.blog_tags FOR SELECT USING (true);

--
-- Name: features Public read access for features; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Public read access for features" ON public.features FOR SELECT TO app_client_role, app_vendor_role, app_admin_role USING (true);

--
-- Name: subscription_tiers Public read access for subscription tiers; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY "Public read access for subscription tiers" ON public.subscription_tiers FOR SELECT TO app_client_role, app_vendor_role, app_admin_role USING (true);

--
-- Name: message_attachments admin_attachments_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY admin_attachments_policy ON public.message_attachments TO app_admin_role USING (true);

--
-- Name: conversations admin_conversations_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY admin_conversations_policy ON public.conversations TO app_admin_role USING (true);

--
-- Name: messages admin_messages_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY admin_messages_policy ON public.messages TO app_admin_role USING (true);

--
-- Name: conversation_participants admin_participants_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY admin_participants_policy ON public.conversation_participants TO app_admin_role USING (true);

--
-- Name: message_read_receipts admin_receipts_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY admin_receipts_policy ON public.message_read_receipts TO app_admin_role USING (true);

--
-- Name: admin_settings admin_settings_modify; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY admin_settings_modify ON public.admin_settings USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: admin_settings admin_settings_select; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY admin_settings_select ON public.admin_settings FOR SELECT USING (true);

--
-- Name: system_events admin_system_events_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY admin_system_events_policy ON public.system_events TO app_admin_role USING (true);

--
-- Name: afrah_inquiries afrah_inquiries_admin_policy; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY afrah_inquiries_admin_policy ON public.afrah_inquiries USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text)) WITH CHECK ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: afrah_inquiries afrah_inquiries_public_insert_policy; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY afrah_inquiries_public_insert_policy ON public.afrah_inquiries FOR INSERT WITH CHECK (true);

--
-- Name: assistant_inquiries assistant_inquiries_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY assistant_inquiries_isolation ON public.assistant_inquiries USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: message_attachments attachments_insert_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY attachments_insert_policy ON public.message_attachments FOR INSERT TO app_client_role, app_vendor_role WITH CHECK ((EXISTS ( SELECT 1
   FROM public.messages m
  WHERE ((m.id = message_attachments.message_id) AND (m.sender_id = (current_setting('app.current_user_id'::text, true))::uuid)))));

--
-- Name: message_attachments attachments_select_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY attachments_select_policy ON public.message_attachments FOR SELECT TO app_client_role, app_vendor_role USING ((EXISTS ( SELECT 1
   FROM public.messages m
  WHERE ((m.id = message_attachments.message_id) AND public.is_conversation_participant(m.conversation_id, (current_setting('app.current_user_id'::text, true))::uuid)))));

--
-- Name: Core_Bookings bookings_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY bookings_isolation ON public.core_bookings USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: client_budget_items budget_items_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY budget_items_isolation ON public.client_budget_items USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: Client_Budgets budgets_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY budgets_isolation ON public.client_budgets USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: blogs catalog_modify_blogs; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_modify_blogs ON public.blogs USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: blog_categories catalog_modify_categories; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_modify_categories ON public.blog_categories USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: blog_category_map catalog_modify_category_map; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_modify_category_map ON public.blog_category_map USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: blog_tags catalog_modify_tags; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_modify_tags ON public.blog_tags USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: blog_tags_map catalog_modify_tags_map; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_modify_tags_map ON public.blog_tags_map USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: blogs catalog_select_blogs; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_select_blogs ON public.blogs FOR SELECT USING (((is_published = true) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: blog_categories catalog_select_categories; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_select_categories ON public.blog_categories FOR SELECT USING (true);

--
-- Name: blog_category_map catalog_select_category_map; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_select_category_map ON public.blog_category_map FOR SELECT USING (true);

--
-- Name: blog_tags catalog_select_tags; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_select_tags ON public.blog_tags FOR SELECT USING (true);

--
-- Name: blog_tags_map catalog_select_tags_map; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_select_tags_map ON public.blog_tags_map FOR SELECT USING (true);

--
-- Name: blog_comments comments_insert; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY comments_insert ON public.blog_comments FOR INSERT WITH CHECK (((user_id IS NULL) OR (user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)));

--
-- Name: blog_comments comments_modify; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY comments_modify ON public.blog_comments USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: blog_comments comments_select; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY comments_select ON public.blog_comments FOR SELECT USING (((is_approved = true) OR (user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: conversations conversations_delete_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY conversations_delete_policy ON public.conversations FOR DELETE TO app_client_role, app_vendor_role USING (public.is_conversation_participant(id, (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: conversations conversations_insert_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY conversations_insert_policy ON public.conversations FOR INSERT TO app_client_role, app_vendor_role WITH CHECK (true);

--
-- Name: conversations conversations_select_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY conversations_select_policy ON public.conversations FOR SELECT TO app_client_role, app_vendor_role USING (public.is_conversation_participant_or_empty(id, (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: conversations conversations_update_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY conversations_update_policy ON public.conversations FOR UPDATE TO app_client_role, app_vendor_role USING (public.is_conversation_participant(id, (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: csrf_tokens csrf_tokens_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY csrf_tokens_isolation ON public.csrf_tokens USING (((user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: listing_promotions delete_vendor_promotions; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY delete_vendor_promotions ON public.listing_promotions FOR DELETE USING (((current_setting('app.current_user_role'::text, true) = 'admin'::text) OR (vendor_id = ( SELECT vendors.id
   FROM public.vendors
  WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)))));

--
-- Name: client_documents documents_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY documents_isolation ON public.client_documents USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: escrow_accounts escrow_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY escrow_isolation ON public.escrow_accounts USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: client_favorites favorites_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY favorites_isolation ON public.client_favorites USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: blog_funnel_events funnel_events_all; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY funnel_events_all ON public.blog_funnel_events USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: blog_funnel_events funnel_events_insert; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY funnel_events_insert ON public.blog_funnel_events FOR INSERT WITH CHECK (true);

--
-- Name: promotion_audit_logs insert_audit_logs; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY insert_audit_logs ON public.promotion_audit_logs FOR INSERT WITH CHECK (((current_setting('app.current_user_role'::text, true) = 'admin'::text) OR (actor_user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)));

--
-- Name: listing_promotions insert_vendor_promotions; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY insert_vendor_promotions ON public.listing_promotions FOR INSERT WITH CHECK (((current_setting('app.current_user_role'::text, true) = 'admin'::text) OR (vendor_id = ( SELECT vendors.id
   FROM public.vendors
  WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)))));

--
-- Name: invoices invoices_isolation_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY invoices_isolation_policy ON public.invoices USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: Lead_Inquiries leads_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY leads_isolation ON public.lead_inquiries USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: messages messages_delete_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY messages_delete_policy ON public.messages FOR DELETE TO app_client_role, app_vendor_role USING ((sender_id = (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: messages messages_insert_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY messages_insert_policy ON public.messages FOR INSERT TO app_client_role, app_vendor_role WITH CHECK (((sender_id = (current_setting('app.current_user_id'::text, true))::uuid) AND public.is_conversation_participant(conversation_id, (current_setting('app.current_user_id'::text, true))::uuid)));

--
-- Name: messages messages_select_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY messages_select_policy ON public.messages FOR SELECT TO app_client_role, app_vendor_role USING (public.is_conversation_participant(conversation_id, (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: messages messages_update_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY messages_update_policy ON public.messages FOR UPDATE TO app_client_role, app_vendor_role USING ((sender_id = (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: notification_outbox outbox_insert_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY outbox_insert_policy ON public.notification_outbox FOR INSERT WITH CHECK (true);

--
-- Name: notification_outbox outbox_worker_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY outbox_worker_policy ON public.notification_outbox USING (((NULLIF(current_setting('app.current_user_role'::text, true), ''::text) IS NULL) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: conversation_participants participants_delete_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY participants_delete_policy ON public.conversation_participants FOR DELETE TO app_client_role, app_vendor_role USING ((user_id = (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: conversation_participants participants_insert_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY participants_insert_policy ON public.conversation_participants FOR INSERT TO app_client_role, app_vendor_role WITH CHECK (true);

--
-- Name: conversation_participants participants_select_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY participants_select_policy ON public.conversation_participants FOR SELECT TO app_client_role, app_vendor_role USING (public.is_conversation_participant(conversation_id, (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: conversation_participants participants_update_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY participants_update_policy ON public.conversation_participants FOR UPDATE TO app_client_role, app_vendor_role USING ((user_id = (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: payment_intents payment_intents_isolation_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY payment_intents_isolation_policy ON public.payment_intents USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: payout_requests payout_requests_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY payout_requests_isolation ON public.payout_requests USING (((vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: booking_quotation_revisions quotation_revisions_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY quotation_revisions_isolation ON public.booking_quotation_revisions USING ((EXISTS ( SELECT 1
   FROM public.booking_quotations q
  WHERE (q.id = booking_quotation_revisions.quotation_id))));

--
-- Name: booking_quotations quotations_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY quotations_isolation ON public.booking_quotations USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: message_read_receipts receipts_insert_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY receipts_insert_policy ON public.message_read_receipts FOR INSERT TO app_client_role, app_vendor_role WITH CHECK ((user_id = (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: message_read_receipts receipts_select_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY receipts_select_policy ON public.message_read_receipts FOR SELECT TO app_client_role, app_vendor_role USING ((EXISTS ( SELECT 1
   FROM public.messages m
  WHERE ((m.id = message_read_receipts.message_id) AND public.is_conversation_participant(m.conversation_id, (current_setting('app.current_user_id'::text, true))::uuid)))));

--
-- Name: promotion_audit_logs select_audit_logs; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY select_audit_logs ON public.promotion_audit_logs FOR SELECT USING (((current_setting('app.current_user_role'::text, true) = 'admin'::text) OR (promotion_id IN ( SELECT listing_promotions.id
   FROM public.listing_promotions
  WHERE (listing_promotions.vendor_id = ( SELECT vendors.id
           FROM public.vendors
          WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)))))));

--
-- Name: listing_promotions select_public_promotions; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY select_public_promotions ON public.listing_promotions FOR SELECT USING (public.fn_promo_is_active(status, start_at, end_at));

--
-- Name: listing_promotions select_vendor_promotions; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY select_vendor_promotions ON public.listing_promotions FOR SELECT USING (((current_setting('app.current_user_role'::text, true) = 'admin'::text) OR (vendor_id = ( SELECT vendors.id
   FROM public.vendors
  WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)))));

--
-- Name: system_events system_events_delete_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY system_events_delete_policy ON public.system_events FOR DELETE TO app_client_role, app_vendor_role USING ((user_id = (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: system_events system_events_insert_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY system_events_insert_policy ON public.system_events FOR INSERT TO app_client_role, app_vendor_role WITH CHECK (true);

--
-- Name: system_events system_events_select_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY system_events_select_policy ON public.system_events FOR SELECT TO app_client_role, app_vendor_role USING ((user_id = (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: system_events system_events_update_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY system_events_update_policy ON public.system_events FOR UPDATE TO app_client_role, app_vendor_role USING ((user_id = (current_setting('app.current_user_id'::text, true))::uuid)) WITH CHECK ((user_id = (current_setting('app.current_user_id'::text, true))::uuid));

--
-- Name: client_tasks tasks_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY tasks_isolation ON public.client_tasks USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: client_timeline_events timeline_events_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY timeline_events_isolation ON public.client_timeline_events USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: listing_promotions update_vendor_promotions; Type: POLICY; Schema: public; Owner: zafaf_db_admin
--

CREATE POLICY update_vendor_promotions ON public.listing_promotions FOR UPDATE USING (((current_setting('app.current_user_role'::text, true) = 'admin'::text) OR (vendor_id = ( SELECT vendors.id
   FROM public.vendors
  WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid))))) WITH CHECK (((current_setting('app.current_user_role'::text, true) = 'admin'::text) OR (vendor_id = ( SELECT vendors.id
   FROM public.vendors
  WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)))));

--
-- Name: vendor_inquiries vendor_inquiries_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY vendor_inquiries_isolation ON public.vendor_inquiries USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: blog_views views_all; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY views_all ON public.blog_views USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: blog_views views_insert; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY views_insert ON public.blog_views FOR INSERT WITH CHECK (true);

--
-- Name: listing_promotions trg_check_promotion_ownership_and_overlap; Type: TRIGGER; Schema: public; Owner: zafaf_db_admin
--

CREATE TRIGGER trg_check_promotion_ownership_and_overlap BEFORE INSERT OR UPDATE ON public.listing_promotions FOR EACH ROW EXECUTE FUNCTION public.fn_check_promotion_ownership_and_overlap();

--
-- Name: listing_promotions trg_listing_promotions_updated_at; Type: TRIGGER; Schema: public; Owner: zafaf_db_admin
--

CREATE TRIGGER trg_listing_promotions_updated_at BEFORE UPDATE ON public.listing_promotions FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

ALTER TABLE ONLY public.client_budgets FORCE ROW LEVEL SECURITY;

ALTER TABLE public.client_budgets OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.core_bookings FORCE ROW LEVEL SECURITY;

ALTER TABLE public.core_bookings OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.lead_inquiries FORCE ROW LEVEL SECURITY;

ALTER TABLE public.lead_inquiries OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.admin_settings FORCE ROW LEVEL SECURITY;

ALTER TABLE public.admin_settings OWNER TO zafaf_schema_owner;

ALTER TABLE public.afrah_inquiries OWNER TO zafaf_db_admin;

ALTER TABLE ONLY public.assistant_inquiries FORCE ROW LEVEL SECURITY;

ALTER TABLE public.assistant_inquiries OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.blog_categories FORCE ROW LEVEL SECURITY;

ALTER TABLE public.blog_categories OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.blog_category_map FORCE ROW LEVEL SECURITY;

ALTER TABLE public.blog_category_map OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.blog_comments FORCE ROW LEVEL SECURITY;

ALTER TABLE public.blog_comments OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.blog_funnel_events FORCE ROW LEVEL SECURITY;

ALTER TABLE public.blog_funnel_events OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.blog_tags FORCE ROW LEVEL SECURITY;

ALTER TABLE public.blog_tags OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.blog_tags_map FORCE ROW LEVEL SECURITY;

ALTER TABLE public.blog_tags_map OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.blog_views FORCE ROW LEVEL SECURITY;

ALTER TABLE public.blog_views OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.blogs FORCE ROW LEVEL SECURITY;

ALTER TABLE public.blogs OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.booking_quotation_revisions FORCE ROW LEVEL SECURITY;

ALTER TABLE public.booking_quotation_revisions OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.booking_quotations FORCE ROW LEVEL SECURITY;

ALTER TABLE public.booking_quotations OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.client_budget_items FORCE ROW LEVEL SECURITY;

ALTER TABLE public.client_budget_items OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.client_documents FORCE ROW LEVEL SECURITY;

ALTER TABLE public.client_documents OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.client_favorites FORCE ROW LEVEL SECURITY;

ALTER TABLE public.client_favorites OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.client_tasks FORCE ROW LEVEL SECURITY;

ALTER TABLE public.client_tasks OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.client_timeline_events FORCE ROW LEVEL SECURITY;

ALTER TABLE public.client_timeline_events OWNER TO zafaf_schema_owner;

ALTER TABLE public.conversation_participants OWNER TO zafaf_schema_owner;

ALTER TABLE public.conversations OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.csrf_tokens FORCE ROW LEVEL SECURITY;

ALTER TABLE public.csrf_tokens OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.escrow_accounts FORCE ROW LEVEL SECURITY;

ALTER TABLE public.escrow_accounts OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.invoices FORCE ROW LEVEL SECURITY;

ALTER TABLE public.invoices OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.listing_promotions FORCE ROW LEVEL SECURITY;

ALTER TABLE public.listing_promotions OWNER TO zafaf_db_admin;

ALTER TABLE public.message_attachments OWNER TO zafaf_schema_owner;

ALTER TABLE public.message_read_receipts OWNER TO zafaf_schema_owner;

ALTER TABLE public.messages OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.notification_outbox FORCE ROW LEVEL SECURITY;

ALTER TABLE public.notification_outbox OWNER TO zafaf_schema_owner;

ALTER TABLE public.password_reset_tokens OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.payment_intents FORCE ROW LEVEL SECURITY;

ALTER TABLE public.payment_intents OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.payout_requests FORCE ROW LEVEL SECURITY;

ALTER TABLE public.payout_requests OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.promotion_audit_logs FORCE ROW LEVEL SECURITY;

ALTER TABLE public.promotion_audit_logs OWNER TO zafaf_db_admin;

ALTER TABLE public.seo_articles OWNER TO zafaf_schema_owner;

ALTER TABLE public.system_events OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.vendor_inquiries FORCE ROW LEVEL SECURITY;

ALTER TABLE public.vendor_inquiries OWNER TO zafaf_schema_owner;

ALTER TABLE public.vendor_inquiry_admin_notes OWNER TO zafaf_schema_owner;

ALTER TABLE public.vendor_inquiry_management OWNER TO zafaf_schema_owner;

ALTER TABLE public.vendor_staff OWNER TO zafaf_schema_owner;

ALTER TABLE public.vendor_tasks OWNER TO zafaf_schema_owner;

ALTER TABLE public.vendor_whatsapp_templates OWNER TO zafaf_schema_owner;

--
-- Name: client_budgets client_budgets_client_id_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_budgets
    ADD CONSTRAINT client_budgets_client_id_key UNIQUE (client_id);

--
-- Name: client_budgets client_budgets_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_budgets
    ADD CONSTRAINT client_budgets_pkey PRIMARY KEY (id);

--
-- Name: core_bookings core_bookings_booking_number_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.core_bookings
    ADD CONSTRAINT core_bookings_booking_number_key UNIQUE (booking_number);

--
-- Name: core_bookings core_bookings_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.core_bookings
    ADD CONSTRAINT core_bookings_pkey PRIMARY KEY (id);

--
-- Name: lead_inquiries lead_inquiries_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.lead_inquiries
    ADD CONSTRAINT lead_inquiries_pkey PRIMARY KEY (id);

--

--
-- Name: admin_settings admin_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.admin_settings
    ADD CONSTRAINT admin_settings_pkey PRIMARY KEY (key);

--
-- Name: afrah_inquiries afrah_inquiries_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE ONLY public.afrah_inquiries
    ADD CONSTRAINT afrah_inquiries_pkey PRIMARY KEY (id);

--
-- Name: assistant_inquiries assistant_inquiries_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.assistant_inquiries
    ADD CONSTRAINT assistant_inquiries_pkey PRIMARY KEY (id);

--
-- Name: blog_categories blog_categories_name_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_categories
    ADD CONSTRAINT blog_categories_name_key UNIQUE (name);

--
-- Name: blog_categories blog_categories_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_categories
    ADD CONSTRAINT blog_categories_pkey PRIMARY KEY (id);

--
-- Name: blog_categories blog_categories_slug_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_categories
    ADD CONSTRAINT blog_categories_slug_key UNIQUE (slug);

--
-- Name: blog_category_map blog_category_map_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_category_map
    ADD CONSTRAINT blog_category_map_pkey PRIMARY KEY (blog_id, category_id);

--
-- Name: blog_comments blog_comments_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_comments
    ADD CONSTRAINT blog_comments_pkey PRIMARY KEY (id);

--
-- Name: blog_funnel_events blog_funnel_events_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_funnel_events
    ADD CONSTRAINT blog_funnel_events_pkey PRIMARY KEY (id);

--
-- Name: blog_tags_map blog_tags_map_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_tags_map
    ADD CONSTRAINT blog_tags_map_pkey PRIMARY KEY (blog_id, tag_id);

--
-- Name: blog_tags blog_tags_name_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_tags
    ADD CONSTRAINT blog_tags_name_key UNIQUE (name);

--
-- Name: blog_tags blog_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_tags
    ADD CONSTRAINT blog_tags_pkey PRIMARY KEY (id);

--
-- Name: blog_tags blog_tags_slug_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_tags
    ADD CONSTRAINT blog_tags_slug_key UNIQUE (slug);

--
-- Name: blog_views blog_views_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_views
    ADD CONSTRAINT blog_views_pkey PRIMARY KEY (id);

--
-- Name: blogs blogs_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blogs
    ADD CONSTRAINT blogs_pkey PRIMARY KEY (id);

--
-- Name: blogs blogs_slug_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blogs
    ADD CONSTRAINT blogs_slug_key UNIQUE (slug);

--
-- Name: blogs blogs_wp_post_id_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blogs
    ADD CONSTRAINT blogs_wp_post_id_key UNIQUE (wp_post_id);

--
-- Name: booking_quotation_revisions booking_quotation_revisions_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.booking_quotation_revisions
    ADD CONSTRAINT booking_quotation_revisions_pkey PRIMARY KEY (id);

--
-- Name: booking_quotations booking_quotations_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.booking_quotations
    ADD CONSTRAINT booking_quotations_pkey PRIMARY KEY (id);

--
-- Name: client_budget_items client_budget_items_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_budget_items
    ADD CONSTRAINT client_budget_items_pkey PRIMARY KEY (id);

--
-- Name: client_documents client_documents_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_documents
    ADD CONSTRAINT client_documents_pkey PRIMARY KEY (id);

--
-- Name: client_favorites client_favorites_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_favorites
    ADD CONSTRAINT client_favorites_pkey PRIMARY KEY (id);

--
-- Name: client_tasks client_tasks_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_tasks
    ADD CONSTRAINT client_tasks_pkey PRIMARY KEY (id);

--
-- Name: client_timeline_events client_timeline_events_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_timeline_events
    ADD CONSTRAINT client_timeline_events_pkey PRIMARY KEY (id);

--
-- Name: conversation_participants conversation_participants_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.conversation_participants
    ADD CONSTRAINT conversation_participants_pkey PRIMARY KEY (id);

--
-- Name: conversations conversations_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.conversations
    ADD CONSTRAINT conversations_pkey PRIMARY KEY (id);

--
-- Name: csrf_tokens csrf_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.csrf_tokens
    ADD CONSTRAINT csrf_tokens_pkey PRIMARY KEY (token);

--
-- Name: escrow_accounts escrow_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.escrow_accounts
    ADD CONSTRAINT escrow_accounts_pkey PRIMARY KEY (id);

--
-- Name: invoices invoices_invoice_number_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.invoices
    ADD CONSTRAINT invoices_invoice_number_key UNIQUE (invoice_number);

--
-- Name: invoices invoices_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.invoices
    ADD CONSTRAINT invoices_pkey PRIMARY KEY (id);

--
-- Name: listing_promotions listing_promotions_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE ONLY public.listing_promotions
    ADD CONSTRAINT listing_promotions_pkey PRIMARY KEY (id);

--
-- Name: message_attachments message_attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.message_attachments
    ADD CONSTRAINT message_attachments_pkey PRIMARY KEY (id);

--
-- Name: message_read_receipts message_read_receipts_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.message_read_receipts
    ADD CONSTRAINT message_read_receipts_pkey PRIMARY KEY (id);

--
-- Name: messages messages_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_pkey PRIMARY KEY (id);

--
-- Name: notification_outbox notification_outbox_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.notification_outbox
    ADD CONSTRAINT notification_outbox_pkey PRIMARY KEY (id);

--
-- Name: password_reset_tokens password_reset_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_pkey PRIMARY KEY (token);

--
-- Name: payment_intents payment_intents_idempotency_key_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.payment_intents
    ADD CONSTRAINT payment_intents_idempotency_key_key UNIQUE (idempotency_key);

--
-- Name: payment_intents payment_intents_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.payment_intents
    ADD CONSTRAINT payment_intents_pkey PRIMARY KEY (id);

--
-- Name: payout_requests payout_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.payout_requests
    ADD CONSTRAINT payout_requests_pkey PRIMARY KEY (id);

--
-- Name: promotion_audit_logs promotion_audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE ONLY public.promotion_audit_logs
    ADD CONSTRAINT promotion_audit_logs_pkey PRIMARY KEY (id);

--
-- Name: seo_articles seo_articles_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.seo_articles
    ADD CONSTRAINT seo_articles_pkey PRIMARY KEY (id);

--
-- Name: seo_articles seo_articles_slug_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.seo_articles
    ADD CONSTRAINT seo_articles_slug_key UNIQUE (slug);

--
-- Name: system_events system_events_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.system_events
    ADD CONSTRAINT system_events_pkey PRIMARY KEY (id);

--
-- Name: client_favorites unique_client_vendor_shortlist; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_favorites
    ADD CONSTRAINT unique_client_vendor_shortlist UNIQUE (client_id, vendor_id, shortlist_name);

--
-- Name: conversation_participants uq_conversation_participant; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.conversation_participants
    ADD CONSTRAINT uq_conversation_participant UNIQUE (conversation_id, user_id);

--
-- Name: message_read_receipts uq_message_user_read; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.message_read_receipts
    ADD CONSTRAINT uq_message_user_read UNIQUE (message_id, user_id);

--
-- Name: vendor_inquiries vendor_inquiries_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiries
    ADD CONSTRAINT vendor_inquiries_pkey PRIMARY KEY (id);

--
-- Name: vendor_inquiry_admin_notes vendor_inquiry_admin_notes_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiry_admin_notes
    ADD CONSTRAINT vendor_inquiry_admin_notes_pkey PRIMARY KEY (id);

--
-- Name: vendor_inquiry_management vendor_inquiry_management_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiry_management
    ADD CONSTRAINT vendor_inquiry_management_pkey PRIMARY KEY (inquiry_id);

--
-- Name: vendor_staff vendor_staff_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_staff
    ADD CONSTRAINT vendor_staff_pkey PRIMARY KEY (id);

--
-- Name: vendor_tasks vendor_tasks_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_tasks
    ADD CONSTRAINT vendor_tasks_pkey PRIMARY KEY (id);

--
-- Name: vendor_whatsapp_templates vendor_whatsapp_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_whatsapp_templates
    ADD CONSTRAINT vendor_whatsapp_templates_pkey PRIMARY KEY (id);

--
-- Name: Client_Budgets; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.client_budgets ENABLE ROW LEVEL SECURITY;

--
-- Name: Core_Bookings; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.core_bookings ENABLE ROW LEVEL SECURITY;

--
-- Name: Lead_Inquiries; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.lead_inquiries ENABLE ROW LEVEL SECURITY;

--
-- Name: admin_settings; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.admin_settings ENABLE ROW LEVEL SECURITY;

--
-- Name: afrah_inquiries; Type: ROW SECURITY; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE public.afrah_inquiries ENABLE ROW LEVEL SECURITY;

--
-- Name: assistant_inquiries; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.assistant_inquiries ENABLE ROW LEVEL SECURITY;

--
-- Name: blog_categories; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.blog_categories ENABLE ROW LEVEL SECURITY;

--
-- Name: blog_category_map; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.blog_category_map ENABLE ROW LEVEL SECURITY;

--
-- Name: blog_comments; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.blog_comments ENABLE ROW LEVEL SECURITY;

--
-- Name: blog_funnel_events; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.blog_funnel_events ENABLE ROW LEVEL SECURITY;

--
-- Name: blog_tags; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.blog_tags ENABLE ROW LEVEL SECURITY;

--
-- Name: blog_tags_map; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.blog_tags_map ENABLE ROW LEVEL SECURITY;

--
-- Name: blog_views; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.blog_views ENABLE ROW LEVEL SECURITY;

--
-- Name: blogs; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.blogs ENABLE ROW LEVEL SECURITY;

--
-- Name: booking_quotation_revisions; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.booking_quotation_revisions ENABLE ROW LEVEL SECURITY;

--
-- Name: booking_quotations; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.booking_quotations ENABLE ROW LEVEL SECURITY;

--
-- Name: client_budget_items; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.client_budget_items ENABLE ROW LEVEL SECURITY;

--
-- Name: client_documents; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.client_documents ENABLE ROW LEVEL SECURITY;

--
-- Name: client_favorites; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.client_favorites ENABLE ROW LEVEL SECURITY;

--
-- Name: client_tasks; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.client_tasks ENABLE ROW LEVEL SECURITY;

--
-- Name: client_timeline_events; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.client_timeline_events ENABLE ROW LEVEL SECURITY;

--
-- Name: conversation_participants; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.conversation_participants ENABLE ROW LEVEL SECURITY;

--
-- Name: conversations; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.conversations ENABLE ROW LEVEL SECURITY;

--
-- Name: csrf_tokens; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.csrf_tokens ENABLE ROW LEVEL SECURITY;

--
-- Name: escrow_accounts; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.escrow_accounts ENABLE ROW LEVEL SECURITY;

--
-- Name: invoices; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.invoices ENABLE ROW LEVEL SECURITY;

--
-- Name: listing_promotions; Type: ROW SECURITY; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE public.listing_promotions ENABLE ROW LEVEL SECURITY;

--
-- Name: message_attachments; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.message_attachments ENABLE ROW LEVEL SECURITY;

--
-- Name: message_read_receipts; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.message_read_receipts ENABLE ROW LEVEL SECURITY;

--
-- Name: messages; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.messages ENABLE ROW LEVEL SECURITY;

--
-- Name: notification_outbox; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.notification_outbox ENABLE ROW LEVEL SECURITY;

--
-- Name: payment_intents; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.payment_intents ENABLE ROW LEVEL SECURITY;

--
-- Name: payout_requests; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.payout_requests ENABLE ROW LEVEL SECURITY;

--
-- Name: promotion_audit_logs; Type: ROW SECURITY; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE public.promotion_audit_logs ENABLE ROW LEVEL SECURITY;

--
-- Name: system_events; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.system_events ENABLE ROW LEVEL SECURITY;

--
-- Name: vendor_inquiries; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.vendor_inquiries ENABLE ROW LEVEL SECURITY;

--
-- Name: client_budgets client_budgets_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_budgets
    ADD CONSTRAINT client_budgets_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: core_bookings core_bookings_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.core_bookings
    ADD CONSTRAINT core_bookings_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE SET NULL;

--
-- Name: core_bookings core_bookings_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.core_bookings
    ADD CONSTRAINT core_bookings_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: core_bookings core_bookings_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.core_bookings
    ADD CONSTRAINT core_bookings_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.vendor_products(id) ON DELETE RESTRICT;

--
-- Name: core_bookings core_bookings_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.core_bookings
    ADD CONSTRAINT core_bookings_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: lead_inquiries lead_inquiries_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.lead_inquiries
    ADD CONSTRAINT lead_inquiries_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE SET NULL;

--
-- Name: lead_inquiries lead_inquiries_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.lead_inquiries
    ADD CONSTRAINT lead_inquiries_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: lead_inquiries lead_inquiries_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.lead_inquiries
    ADD CONSTRAINT lead_inquiries_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.vendor_products(id) ON DELETE SET NULL;

--
-- Name: lead_inquiries lead_inquiries_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.lead_inquiries
    ADD CONSTRAINT lead_inquiries_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: admin_settings admin_settings_updated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.admin_settings
    ADD CONSTRAINT admin_settings_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public.global_users(id) ON DELETE SET NULL;

--
-- Name: assistant_inquiries assistant_inquiries_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.assistant_inquiries
    ADD CONSTRAINT assistant_inquiries_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE SET NULL;

--
-- Name: assistant_inquiries assistant_inquiries_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.assistant_inquiries
    ADD CONSTRAINT assistant_inquiries_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: blog_category_map blog_category_map_blog_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_category_map
    ADD CONSTRAINT blog_category_map_blog_id_fkey FOREIGN KEY (blog_id) REFERENCES public.blogs(id) ON DELETE CASCADE;

--
-- Name: blog_category_map blog_category_map_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_category_map
    ADD CONSTRAINT blog_category_map_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.blog_categories(id) ON DELETE CASCADE;

--
-- Name: blog_comments blog_comments_blog_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_comments
    ADD CONSTRAINT blog_comments_blog_id_fkey FOREIGN KEY (blog_id) REFERENCES public.blogs(id) ON DELETE CASCADE;

--
-- Name: blog_comments blog_comments_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_comments
    ADD CONSTRAINT blog_comments_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.blog_comments(id) ON DELETE CASCADE;

--
-- Name: blog_comments blog_comments_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_comments
    ADD CONSTRAINT blog_comments_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE SET NULL;

--
-- Name: blog_tags_map blog_tags_map_blog_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_tags_map
    ADD CONSTRAINT blog_tags_map_blog_id_fkey FOREIGN KEY (blog_id) REFERENCES public.blogs(id) ON DELETE CASCADE;

--
-- Name: blog_tags_map blog_tags_map_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_tags_map
    ADD CONSTRAINT blog_tags_map_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.blog_tags(id) ON DELETE CASCADE;

--
-- Name: blog_views blog_views_blog_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_views
    ADD CONSTRAINT blog_views_blog_id_fkey FOREIGN KEY (blog_id) REFERENCES public.blogs(id) ON DELETE CASCADE;

--
-- Name: blog_views blog_views_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_views
    ADD CONSTRAINT blog_views_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE SET NULL;

--
-- Name: blogs blogs_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blogs
    ADD CONSTRAINT blogs_author_id_fkey FOREIGN KEY (author_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: booking_quotation_revisions booking_quotation_revisions_quotation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.booking_quotation_revisions
    ADD CONSTRAINT booking_quotation_revisions_quotation_id_fkey FOREIGN KEY (quotation_id) REFERENCES public.booking_quotations(id) ON DELETE CASCADE;

--
-- Name: booking_quotations booking_quotations_booking_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.booking_quotations
    ADD CONSTRAINT booking_quotations_booking_id_fkey FOREIGN KEY (booking_id) REFERENCES public.core_bookings(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_booking_quotations_vendor FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_booking_quotations_client FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: client_budget_items client_budget_items_booking_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_budget_items
    ADD CONSTRAINT client_budget_items_booking_id_fkey FOREIGN KEY (booking_id) REFERENCES public.core_bookings(id) ON DELETE SET NULL;

--
-- Name: client_documents client_documents_booking_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_documents
    ADD CONSTRAINT client_documents_booking_id_fkey FOREIGN KEY (booking_id) REFERENCES public.core_bookings(id) ON DELETE SET NULL;

--
-- Name: client_favorites client_favorites_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_favorites
    ADD CONSTRAINT client_favorites_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: conversation_participants conversation_participants_conversation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.conversation_participants
    ADD CONSTRAINT conversation_participants_conversation_id_fkey FOREIGN KEY (conversation_id) REFERENCES public.conversations(id) ON DELETE CASCADE;

--
-- Name: conversation_participants conversation_participants_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.conversation_participants
    ADD CONSTRAINT conversation_participants_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: conversations conversations_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.conversations
    ADD CONSTRAINT conversations_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE SET NULL;

--
-- Name: conversations conversations_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.conversations
    ADD CONSTRAINT conversations_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.vendor_products(id) ON DELETE SET NULL;

--
-- Name: csrf_tokens csrf_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.csrf_tokens
    ADD CONSTRAINT csrf_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: escrow_accounts escrow_accounts_booking_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.escrow_accounts
    ADD CONSTRAINT escrow_accounts_booking_id_fkey FOREIGN KEY (booking_id) REFERENCES public.core_bookings(id) ON DELETE CASCADE;

--
-- Name: blog_funnel_events fk_blog_slug; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.blog_funnel_events
    ADD CONSTRAINT fk_blog_slug FOREIGN KEY (blog_slug) REFERENCES public.blogs(slug) ON DELETE CASCADE;

--
-- Name: invoices invoices_booking_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.invoices
    ADD CONSTRAINT invoices_booking_id_fkey FOREIGN KEY (booking_id) REFERENCES public.core_bookings(id) ON DELETE SET NULL;

--
-- Name: listing_promotions listing_promotions_listing_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE ONLY public.listing_promotions
    ADD CONSTRAINT listing_promotions_listing_id_fkey FOREIGN KEY (listing_id) REFERENCES public.vendor_products(id) ON DELETE CASCADE;

--
-- Name: listing_promotions listing_promotions_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE ONLY public.listing_promotions
    ADD CONSTRAINT listing_promotions_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: message_attachments message_attachments_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.message_attachments
    ADD CONSTRAINT message_attachments_message_id_fkey FOREIGN KEY (message_id) REFERENCES public.messages(id) ON DELETE CASCADE;

--
-- Name: message_read_receipts message_read_receipts_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.message_read_receipts
    ADD CONSTRAINT message_read_receipts_message_id_fkey FOREIGN KEY (message_id) REFERENCES public.messages(id) ON DELETE CASCADE;

--
-- Name: message_read_receipts message_read_receipts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.message_read_receipts
    ADD CONSTRAINT message_read_receipts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: messages messages_conversation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_conversation_id_fkey FOREIGN KEY (conversation_id) REFERENCES public.conversations(id) ON DELETE CASCADE;

--
-- Name: messages messages_sender_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_sender_id_fkey FOREIGN KEY (sender_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: password_reset_tokens password_reset_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: payment_intents payment_intents_booking_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.payment_intents
    ADD CONSTRAINT payment_intents_booking_id_fkey FOREIGN KEY (booking_id) REFERENCES public.core_bookings(id) ON DELETE CASCADE;

--
-- Name: promotion_audit_logs promotion_audit_logs_promotion_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_db_admin
--

ALTER TABLE ONLY public.promotion_audit_logs
    ADD CONSTRAINT promotion_audit_logs_promotion_id_fkey FOREIGN KEY (promotion_id) REFERENCES public.listing_promotions(id) ON DELETE CASCADE;

--
-- Name: system_events system_events_target_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.system_events
    ADD CONSTRAINT system_events_target_vendor_id_fkey FOREIGN KEY (target_vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: system_events system_events_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.system_events
    ADD CONSTRAINT system_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: vendor_inquiries vendor_inquiries_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiries
    ADD CONSTRAINT vendor_inquiries_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE SET NULL;

--
-- Name: vendor_inquiries vendor_inquiries_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiries
    ADD CONSTRAINT vendor_inquiries_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: vendor_inquiries vendor_inquiries_conversation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiries
    ADD CONSTRAINT vendor_inquiries_conversation_id_fkey FOREIGN KEY (conversation_id) REFERENCES public.conversations(id) ON DELETE SET NULL;

--
-- Name: vendor_inquiries vendor_inquiries_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiries
    ADD CONSTRAINT vendor_inquiries_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.vendor_products(id) ON DELETE SET NULL;

--
-- Name: vendor_inquiries vendor_inquiries_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiries
    ADD CONSTRAINT vendor_inquiries_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: vendor_inquiry_admin_notes vendor_inquiry_admin_notes_admin_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiry_admin_notes
    ADD CONSTRAINT vendor_inquiry_admin_notes_admin_id_fkey FOREIGN KEY (admin_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: vendor_inquiry_admin_notes vendor_inquiry_admin_notes_inquiry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiry_admin_notes
    ADD CONSTRAINT vendor_inquiry_admin_notes_inquiry_id_fkey FOREIGN KEY (inquiry_id) REFERENCES public.vendor_inquiries(id) ON DELETE CASCADE;

--
-- Name: vendor_inquiry_management vendor_inquiry_management_assigned_admin_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiry_management
    ADD CONSTRAINT vendor_inquiry_management_assigned_admin_id_fkey FOREIGN KEY (assigned_admin_id) REFERENCES public.global_users(id) ON DELETE SET NULL;

--
-- Name: vendor_inquiry_management vendor_inquiry_management_inquiry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_inquiry_management
    ADD CONSTRAINT vendor_inquiry_management_inquiry_id_fkey FOREIGN KEY (inquiry_id) REFERENCES public.vendor_inquiries(id) ON DELETE CASCADE;

--
-- Name: vendor_staff vendor_staff_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_staff
    ADD CONSTRAINT vendor_staff_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: vendor_tasks vendor_tasks_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_tasks
    ADD CONSTRAINT vendor_tasks_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: vendor_whatsapp_templates vendor_whatsapp_templates_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_whatsapp_templates
    ADD CONSTRAINT vendor_whatsapp_templates_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: EXTENSION pg_trgm; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';

--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';

--
-- Name: FUNCTION compute_search_rank(p_quality_score integer, p_verification_level public.vendor_verification_level, p_avg_rating numeric, p_updated_at timestamp with time zone, p_is_featured boolean); Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON FUNCTION public.compute_search_rank(p_quality_score integer, p_verification_level public.vendor_verification_level, p_avg_rating numeric, p_updated_at timestamp with time zone, p_is_featured boolean) IS 'Computes normalized search ranking score (0.0–1.5+) for a listing. Used in ORDER BY for search results.';

--
-- Name: FUNCTION sync_product_search_fields(); Type: COMMENT; Schema: public; Owner: zafaf_db_admin
--

COMMENT ON FUNCTION public.sync_product_search_fields() IS 'Trigger function: recomputes total_capacity (using category capacity_mode) and searchable_amenities (from both attributes and cultural_attributes) on every insert/update of vendor_products.';

--
-- Name: COLUMN blogs.wp_post_id; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.blogs.wp_post_id IS 'WordPress post ID — null for internal posts, set for wordpress-sourced posts.';

--
-- Name: COLUMN blogs.source; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.blogs.source IS '''internal'' = legacy admin-panel content | ''wordpress'' = WordPress CMS shadow row';

--
-- Name: COLUMN blogs.lang; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.blogs.lang IS 'Language code (e.g. en or ar) representing the post locale.';

--
-- Name: COLUMN blogs.translation_group_id; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.blogs.translation_group_id IS 'Polylang post ID of the primary (English) post to group translations together.';

--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: pg_database_owner
--

GRANT USAGE ON SCHEMA public TO app_client_role;

GRANT USAGE ON SCHEMA public TO app_vendor_role;

GRANT USAGE ON SCHEMA public TO app_admin_role;

GRANT USAGE ON SCHEMA public TO zafaf_app_user;

GRANT ALL ON SCHEMA public TO zafaf_schema_owner;

--
-- Name: FUNCTION is_conversation_participant(conv_id uuid, u_id uuid); Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT ALL ON FUNCTION public.is_conversation_participant(conv_id uuid, u_id uuid) TO app_client_role;

GRANT ALL ON FUNCTION public.is_conversation_participant(conv_id uuid, u_id uuid) TO app_vendor_role;

GRANT ALL ON FUNCTION public.is_conversation_participant(conv_id uuid, u_id uuid) TO app_admin_role;

--
-- Name: FUNCTION is_conversation_participant_or_empty(conv_id uuid, u_id uuid); Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT ALL ON FUNCTION public.is_conversation_participant_or_empty(conv_id uuid, u_id uuid) TO app_client_role;

GRANT ALL ON FUNCTION public.is_conversation_participant_or_empty(conv_id uuid, u_id uuid) TO app_vendor_role;

GRANT ALL ON FUNCTION public.is_conversation_participant_or_empty(conv_id uuid, u_id uuid) TO app_admin_role;

--
-- Name: TABLE client_budgets; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_budgets TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_budgets TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_budgets TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_budgets TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_budgets TO zafaf_db_admin;

--
-- Name: TABLE core_bookings; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.core_bookings TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.core_bookings TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.core_bookings TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.core_bookings TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.core_bookings TO zafaf_db_admin;

--
-- Name: TABLE lead_inquiries; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.lead_inquiries TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.lead_inquiries TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.lead_inquiries TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.lead_inquiries TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.lead_inquiries TO zafaf_db_admin;

--

--
-- Name: TABLE admin_settings; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.admin_settings TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.admin_settings TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.admin_settings TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.admin_settings TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.admin_settings TO zafaf_db_admin;

--
-- Name: TABLE afrah_inquiries; Type: ACL; Schema: public; Owner: zafaf_db_admin
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.afrah_inquiries TO zafaf_schema_owner;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.afrah_inquiries TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.afrah_inquiries TO PUBLIC;

--
-- Name: TABLE assistant_inquiries; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.assistant_inquiries TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.assistant_inquiries TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.assistant_inquiries TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.assistant_inquiries TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.assistant_inquiries TO zafaf_db_admin;

--
-- Name: TABLE blog_categories; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_categories TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_categories TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_categories TO app_admin_role;

GRANT SELECT ON TABLE public.blog_categories TO app_vendor_role;

GRANT SELECT ON TABLE public.blog_categories TO app_client_role;

--
-- Name: TABLE blog_category_map; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_category_map TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_category_map TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_category_map TO app_admin_role;

GRANT SELECT ON TABLE public.blog_category_map TO app_vendor_role;

GRANT SELECT ON TABLE public.blog_category_map TO app_client_role;

--
-- Name: TABLE blog_comments; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_comments TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_comments TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_comments TO app_admin_role;

GRANT SELECT ON TABLE public.blog_comments TO app_vendor_role;

GRANT SELECT ON TABLE public.blog_comments TO app_client_role;

--
-- Name: TABLE blog_funnel_events; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_funnel_events TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_funnel_events TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_funnel_events TO app_admin_role;

GRANT SELECT ON TABLE public.blog_funnel_events TO app_vendor_role;

GRANT SELECT ON TABLE public.blog_funnel_events TO app_client_role;

--
-- Name: TABLE blog_tags; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_tags TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_tags TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_tags TO app_admin_role;

GRANT SELECT ON TABLE public.blog_tags TO app_vendor_role;

GRANT SELECT ON TABLE public.blog_tags TO app_client_role;

--
-- Name: TABLE blog_tags_map; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_tags_map TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_tags_map TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_tags_map TO app_admin_role;

GRANT SELECT ON TABLE public.blog_tags_map TO app_vendor_role;

GRANT SELECT ON TABLE public.blog_tags_map TO app_client_role;

--
-- Name: TABLE blog_views; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_views TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_views TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blog_views TO app_admin_role;

GRANT SELECT ON TABLE public.blog_views TO app_vendor_role;

GRANT SELECT ON TABLE public.blog_views TO app_client_role;

--
-- Name: TABLE blogs; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blogs TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blogs TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.blogs TO app_admin_role;

GRANT SELECT ON TABLE public.blogs TO app_vendor_role;

GRANT SELECT ON TABLE public.blogs TO app_client_role;

--
-- Name: TABLE booking_quotation_revisions; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.booking_quotation_revisions TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.booking_quotation_revisions TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.booking_quotation_revisions TO app_admin_role;

GRANT SELECT ON TABLE public.booking_quotation_revisions TO app_vendor_role;

GRANT SELECT ON TABLE public.booking_quotation_revisions TO app_client_role;

--
-- Name: TABLE booking_quotations; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.booking_quotations TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.booking_quotations TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.booking_quotations TO app_admin_role;

GRANT SELECT ON TABLE public.booking_quotations TO app_vendor_role;

GRANT SELECT ON TABLE public.booking_quotations TO app_client_role;

--
-- Name: TABLE client_budget_items; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_budget_items TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_budget_items TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_budget_items TO app_admin_role;

GRANT SELECT ON TABLE public.client_budget_items TO app_vendor_role;

GRANT SELECT ON TABLE public.client_budget_items TO app_client_role;

--
-- Name: TABLE client_documents; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_documents TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_documents TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_documents TO app_admin_role;

GRANT SELECT ON TABLE public.client_documents TO app_vendor_role;

GRANT SELECT ON TABLE public.client_documents TO app_client_role;

--
-- Name: TABLE client_favorites; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_favorites TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_favorites TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_favorites TO app_admin_role;

GRANT SELECT ON TABLE public.client_favorites TO app_vendor_role;

GRANT SELECT ON TABLE public.client_favorites TO app_client_role;

--
-- Name: TABLE client_tasks; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_tasks TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_tasks TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_tasks TO app_admin_role;

GRANT SELECT ON TABLE public.client_tasks TO app_vendor_role;

GRANT SELECT ON TABLE public.client_tasks TO app_client_role;

--
-- Name: TABLE client_timeline_events; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_timeline_events TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_timeline_events TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_timeline_events TO app_admin_role;

GRANT SELECT ON TABLE public.client_timeline_events TO app_vendor_role;

GRANT SELECT ON TABLE public.client_timeline_events TO app_client_role;

--
-- Name: TABLE conversation_participants; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversation_participants TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversation_participants TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversation_participants TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversation_participants TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversation_participants TO zafaf_db_admin;

--
-- Name: TABLE conversations; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversations TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversations TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversations TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversations TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.conversations TO zafaf_db_admin;

--
-- Name: TABLE csrf_tokens; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE ON TABLE public.csrf_tokens TO app_client_role;

GRANT SELECT,INSERT,DELETE ON TABLE public.csrf_tokens TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.csrf_tokens TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.csrf_tokens TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.csrf_tokens TO zafaf_db_admin;

--
-- Name: TABLE escrow_accounts; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.escrow_accounts TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.escrow_accounts TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.escrow_accounts TO app_admin_role;

GRANT SELECT ON TABLE public.escrow_accounts TO app_vendor_role;

GRANT SELECT ON TABLE public.escrow_accounts TO app_client_role;

--
-- Name: COLUMN global_users.failed_login_attempts; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT UPDATE(failed_login_attempts) ON TABLE public.global_users TO app_client_role;

GRANT UPDATE(failed_login_attempts) ON TABLE public.global_users TO app_vendor_role;

GRANT UPDATE(failed_login_attempts) ON TABLE public.global_users TO app_admin_role;

--
-- Name: COLUMN global_users.locked_until; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT UPDATE(locked_until) ON TABLE public.global_users TO app_client_role;

GRANT UPDATE(locked_until) ON TABLE public.global_users TO app_vendor_role;

GRANT UPDATE(locked_until) ON TABLE public.global_users TO app_admin_role;

--
-- Name: TABLE invoices; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.invoices TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.invoices TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.invoices TO app_admin_role;

GRANT SELECT ON TABLE public.invoices TO app_vendor_role;

GRANT SELECT ON TABLE public.invoices TO app_client_role;

--
-- Name: TABLE listing_promotions; Type: ACL; Schema: public; Owner: zafaf_db_admin
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.listing_promotions TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.listing_promotions TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.listing_promotions TO app_vendor_role;

GRANT SELECT ON TABLE public.listing_promotions TO app_client_role;

--
-- Name: TABLE message_attachments; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_attachments TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_attachments TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_attachments TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_attachments TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_attachments TO zafaf_db_admin;

--
-- Name: TABLE message_read_receipts; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_read_receipts TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_read_receipts TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_read_receipts TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_read_receipts TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.message_read_receipts TO zafaf_db_admin;

--
-- Name: TABLE messages; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.messages TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.messages TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.messages TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.messages TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.messages TO zafaf_db_admin;

--
-- Name: TABLE notification_outbox; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.notification_outbox TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.notification_outbox TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.notification_outbox TO app_admin_role;

GRANT SELECT ON TABLE public.notification_outbox TO app_vendor_role;

GRANT SELECT ON TABLE public.notification_outbox TO app_client_role;

--
-- Name: TABLE password_reset_tokens; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE ON TABLE public.password_reset_tokens TO app_client_role;

GRANT SELECT,INSERT,DELETE ON TABLE public.password_reset_tokens TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.password_reset_tokens TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.password_reset_tokens TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.password_reset_tokens TO zafaf_db_admin;

--
-- Name: TABLE payment_intents; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.payment_intents TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.payment_intents TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.payment_intents TO app_admin_role;

GRANT SELECT ON TABLE public.payment_intents TO app_vendor_role;

GRANT SELECT ON TABLE public.payment_intents TO app_client_role;

--
-- Name: TABLE payout_requests; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.payout_requests TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.payout_requests TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.payout_requests TO app_admin_role;

GRANT SELECT ON TABLE public.payout_requests TO app_vendor_role;

GRANT SELECT ON TABLE public.payout_requests TO app_client_role;

--
-- Name: TABLE promotion_audit_logs; Type: ACL; Schema: public; Owner: zafaf_db_admin
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.promotion_audit_logs TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.promotion_audit_logs TO app_admin_role;

GRANT SELECT,INSERT ON TABLE public.promotion_audit_logs TO app_vendor_role;

GRANT SELECT ON TABLE public.promotion_audit_logs TO app_client_role;

--
-- Name: TABLE seo_articles; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.seo_articles TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.seo_articles TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.seo_articles TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.seo_articles TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.seo_articles TO zafaf_db_admin;

--
-- Name: TABLE system_events; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.system_events TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.system_events TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.system_events TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.system_events TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.system_events TO zafaf_db_admin;

--
-- Name: TABLE vendor_inquiries; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiries TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiries TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiries TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiries TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiries TO zafaf_db_admin;

--
-- Name: TABLE vendor_inquiry_admin_notes; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiry_admin_notes TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiry_admin_notes TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiry_admin_notes TO app_admin_role;

GRANT SELECT ON TABLE public.vendor_inquiry_admin_notes TO app_vendor_role;

GRANT SELECT ON TABLE public.vendor_inquiry_admin_notes TO app_client_role;

--
-- Name: TABLE vendor_inquiry_management; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiry_management TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiry_management TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_inquiry_management TO app_admin_role;

GRANT SELECT ON TABLE public.vendor_inquiry_management TO app_vendor_role;

GRANT SELECT ON TABLE public.vendor_inquiry_management TO app_client_role;

--
-- Name: TABLE vendor_staff; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_staff TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_staff TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_staff TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_staff TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_staff TO zafaf_db_admin;

--
-- Name: TABLE vendor_tasks; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_tasks TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_tasks TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_tasks TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_tasks TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_tasks TO zafaf_db_admin;

--
-- Name: TABLE vendor_whatsapp_templates; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_whatsapp_templates TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_whatsapp_templates TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_whatsapp_templates TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_whatsapp_templates TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_whatsapp_templates TO zafaf_db_admin;

--
-- PostgreSQL database dump
--

-- Dumped from database version 16.14
-- Dumped by pg_dump version 16.14

--

ALTER FUNCTION public.fn_check_promotion_ownership_and_overlap() OWNER TO zafaf_db_admin;

ALTER FUNCTION public.fn_promo_is_active(status character varying, start_at timestamp with time zone, end_at timestamp with time zone) OWNER TO zafaf_db_admin;

ALTER FUNCTION public.is_conversation_participant(conv_id uuid, u_id uuid) OWNER TO zafaf_schema_owner;

ALTER FUNCTION public.is_conversation_participant_or_empty(conv_id uuid, u_id uuid) OWNER TO zafaf_schema_owner;

--
-- PostgreSQL database dump complete
--


-- ─── DATABASE INTEGRITY OVERHAUL (Phase 3 Re-architecture) ───────────────────

-- Add missing foreign keys to tables created in create_missing_tables
ALTER TABLE public.client_documents 
    ADD CONSTRAINT fk_client_documents_client FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS file_id UUID REFERENCES public.uploaded_files(id) ON DELETE SET NULL;

ALTER TABLE public.message_attachments
    ADD COLUMN IF NOT EXISTS file_id UUID REFERENCES public.uploaded_files(id) ON DELETE SET NULL;

ALTER TABLE public.vendor_review_attachments
    ADD COLUMN IF NOT EXISTS file_id UUID REFERENCES public.uploaded_files(id) ON DELETE SET NULL;

ALTER TABLE public.invoices
    ADD CONSTRAINT fk_invoices_client FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_invoices_vendor FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD CONSTRAINT chk_invoice_amount CHECK (amount >= 0.00 AND tax_amount >= 0.00);

ALTER TABLE public.payment_intents
    ADD CONSTRAINT fk_payment_intents_client FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_payment_intents_vendor FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD CONSTRAINT chk_payment_amount CHECK (amount > 0.00);

ALTER TABLE public.payout_requests
    ADD CONSTRAINT fk_payout_requests_vendor FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD CONSTRAINT chk_payout_amount CHECK (amount > 0.00);

ALTER TABLE public.escrow_accounts
    ADD CONSTRAINT fk_escrow_accounts_vendor FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_escrow_accounts_client FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE,
    ADD CONSTRAINT chk_escrow_amount CHECK (amount_held >= 0.00);

ALTER TABLE public.client_budget_items
    ADD CONSTRAINT fk_client_budget_items_client FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

ALTER TABLE public.client_favorites
    ADD CONSTRAINT fk_client_favorites_client FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

ALTER TABLE public.user_notification_preferences
    ADD CONSTRAINT fk_user_notification_preferences_user FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

ALTER TABLE public.vendor_wallets
    ADD CONSTRAINT fk_vendor_wallets_vendor FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

-- Create optimization indexes for joins and RLS filter lookups
CREATE INDEX IF NOT EXISTS idx_client_documents_client ON public.client_documents(client_id);
CREATE INDEX IF NOT EXISTS idx_client_documents_file ON public.client_documents(file_id);
CREATE INDEX IF NOT EXISTS idx_client_documents_deleted_at ON public.client_documents(deleted_at) WHERE (deleted_at IS NULL);
CREATE INDEX IF NOT EXISTS idx_message_attachments_file ON public.message_attachments(file_id);
CREATE INDEX IF NOT EXISTS idx_vendor_review_attachments_file ON public.vendor_review_attachments(file_id);
CREATE INDEX IF NOT EXISTS idx_invoices_client ON public.invoices(client_id);
CREATE INDEX IF NOT EXISTS idx_invoices_vendor ON public.invoices(vendor_id);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON public.invoices(status);
CREATE INDEX IF NOT EXISTS idx_invoices_created_at ON public.invoices(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_payment_intents_client ON public.payment_intents(client_id);
CREATE INDEX IF NOT EXISTS idx_payment_intents_vendor ON public.payment_intents(vendor_id);
CREATE INDEX IF NOT EXISTS idx_payout_requests_vendor ON public.payout_requests(vendor_id);
CREATE INDEX IF NOT EXISTS idx_escrow_accounts_vendor ON public.escrow_accounts(vendor_id);
CREATE INDEX IF NOT EXISTS idx_escrow_accounts_client ON public.escrow_accounts(client_id);
CREATE INDEX IF NOT EXISTS idx_client_budget_items_client ON public.client_budget_items(client_id);
CREATE INDEX IF NOT EXISTS idx_client_favorites_client ON public.client_favorites(client_id);
CREATE INDEX IF NOT EXISTS idx_user_notification_preferences_user ON public.user_notification_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_vendor_wallets_vendor ON public.vendor_wallets(vendor_id);

--
-- Name: admin_audit_logs; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.admin_audit_logs (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    entity_type character varying(50) NOT NULL,
    entity_id uuid NOT NULL,
    actor_id uuid REFERENCES public.global_users(id) ON DELETE SET NULL,
    action character varying(50) NOT NULL,
    before_state jsonb,
    after_state jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);
CREATE INDEX idx_admin_audit_logs_entity ON public.admin_audit_logs(entity_type, entity_id);
CREATE INDEX idx_admin_audit_logs_actor ON public.admin_audit_logs(actor_id, created_at DESC);

--
-- Name: status_history; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.status_history (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    entity_type character varying(50) NOT NULL,
    entity_id uuid NOT NULL,
    old_status character varying(50),
    new_status character varying(50) NOT NULL,
    changed_by uuid REFERENCES public.global_users(id) ON DELETE SET NULL,
    changed_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text,
    vendor_id uuid REFERENCES public.vendors(id) ON DELETE SET NULL,
    client_id uuid REFERENCES public.global_users(id) ON DELETE CASCADE
);
CREATE INDEX idx_status_history_entity ON public.status_history(entity_type, entity_id);
CREATE INDEX idx_status_history_vendor ON public.status_history(vendor_id) WHERE (vendor_id IS NOT NULL);
CREATE INDEX idx_status_history_client ON public.status_history(client_id) WHERE (client_id IS NOT NULL);


-- Automated Metadata Sync for message_attachments
CREATE OR REPLACE FUNCTION public.sync_message_attachment_metadata()
RETURNS trigger AS $$
DECLARE
    v_file RECORD;
BEGIN
    IF NEW.file_id IS NOT NULL THEN
        SELECT file_path, file_url, media_type, file_size
        INTO v_file
        FROM public.uploaded_files
        WHERE id = NEW.file_id;

        IF FOUND THEN
            NEW.file_name := COALESCE(NEW.file_name, substring(v_file.file_path from '[^/]+$'));
            NEW.file_url := COALESCE(NEW.file_url, v_file.file_url);
            NEW.file_type := COALESCE(NEW.file_type, v_file.media_type);
            NEW.file_size := COALESCE(NEW.file_size, v_file.file_size::integer);
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

ALTER FUNCTION public.sync_message_attachment_metadata() OWNER TO zafaf_schema_owner;

CREATE TRIGGER trg_sync_message_attachment_metadata
BEFORE INSERT OR UPDATE OF file_id ON public.message_attachments
FOR EACH ROW
EXECUTE FUNCTION public.sync_message_attachment_metadata();


-- Automated Metadata Sync for client_documents
CREATE OR REPLACE FUNCTION public.sync_client_document_metadata()
RETURNS trigger AS $$
DECLARE
    v_file RECORD;
BEGIN
    IF NEW.file_id IS NOT NULL THEN
        SELECT file_path, file_url
        INTO v_file
        FROM public.uploaded_files
        WHERE id = NEW.file_id;

        IF FOUND THEN
            NEW.file_name := COALESCE(NEW.file_name, substring(v_file.file_path from '[^/]+$'));
            NEW.file_url := COALESCE(NEW.file_url, v_file.file_url);
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

ALTER FUNCTION public.sync_client_document_metadata() OWNER TO zafaf_schema_owner;

CREATE TRIGGER trg_sync_client_document_metadata
BEFORE INSERT OR UPDATE OF file_id ON public.client_documents
FOR EACH ROW
EXECUTE FUNCTION public.sync_client_document_metadata();


-- Missing touch_updated_at trigger bindings for core tables
CREATE TRIGGER trg_client_budgets_updated_at BEFORE UPDATE ON public.client_budgets FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_core_bookings_updated_at BEFORE UPDATE ON public.core_bookings FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_afrah_inquiries_updated_at BEFORE UPDATE ON public.afrah_inquiries FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_assistant_inquiries_updated_at BEFORE UPDATE ON public.assistant_inquiries FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_blogs_updated_at BEFORE UPDATE ON public.blogs FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_booking_quotations_updated_at BEFORE UPDATE ON public.booking_quotations FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_client_budget_items_updated_at BEFORE UPDATE ON public.client_budget_items FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_conversations_updated_at BEFORE UPDATE ON public.conversations FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_notification_outbox_updated_at BEFORE UPDATE ON public.notification_outbox FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_payment_intents_updated_at BEFORE UPDATE ON public.payment_intents FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_payout_requests_updated_at BEFORE UPDATE ON public.payout_requests FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_seo_articles_updated_at BEFORE UPDATE ON public.seo_articles FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_vendor_inquiries_updated_at BEFORE UPDATE ON public.vendor_inquiries FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_vendor_inquiry_admin_notes_updated_at BEFORE UPDATE ON public.vendor_inquiry_admin_notes FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_vendor_inquiry_management_updated_at BEFORE UPDATE ON public.vendor_inquiry_management FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_vendor_whatsapp_templates_updated_at BEFORE UPDATE ON public.vendor_whatsapp_templates FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

