#!/bin/bash
echo "Checking tables..."
podman exec -i zafafworld_postgres_1 psql -U zafaf_db_admin -d zafaf_world -t -c "SELECT tablename FROM pg_tables WHERE schemaname = 'public';" | grep -E "(listing_promotions|vendors|vendor_gallery|global_users)"

echo "Checking triggers..."
podman exec -i zafafworld_postgres_1 psql -U zafaf_db_admin -d zafaf_world -t -c "SELECT trigger_name FROM information_schema.triggers WHERE event_object_schema = 'public';" | grep "touch_updated_at" | head -n 5

echo "Checking functions..."
podman exec -i zafafworld_postgres_1 psql -U zafaf_db_admin -d zafaf_world -t -c "SELECT routine_name FROM information_schema.routines WHERE routine_schema = 'public';" | grep "touch_updated_at"
