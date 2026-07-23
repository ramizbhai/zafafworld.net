-- migrations/0009_create_support_messages.sql

CREATE TABLE IF NOT EXISTS public.support_messages (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    name character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    phone character varying(50),
    subject character varying(255) NOT NULL,
    message text NOT NULL,
    status character varying(20) DEFAULT 'new' NOT NULL,
    assigned_admin_id uuid REFERENCES public.global_users(id) ON DELETE SET NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_support_message_status CHECK (status IN ('new', 'in_progress', 'resolved', 'closed'))
);

