-- Test admin user for E2E tests.
-- Requires migrations to have run (users table must exist).
-- Seeded by `just db-seed`, called from `just e2e`.
INSERT INTO users (id, phone, name, role, status, onboarded, created_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    '+380670000001',
    'Організатор',
    'admin',
    'active',
    true,
    NOW()
) ON CONFLICT (phone) DO NOTHING;
