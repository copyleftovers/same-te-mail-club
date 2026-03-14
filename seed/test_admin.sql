-- Test admin user for E2E tests.
-- Requires migrations to have run (users table must exist).
-- Seeded by `just db-seed`, called from `just e2e`.
INSERT INTO users (id, phone, name, nova_poshta_branch, is_admin, is_active, onboarded, created_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    '+380670000001',
    'Організатор',
    'Відділення №10, Київ',
    true,
    true,
    true,
    NOW()
) ON CONFLICT (phone) DO NOTHING;
