-- Cohort seed for the large-cohort cycle-viz capture pass (Unit 7).
--
-- Creates 12 participants with long double-barrel Ukrainian names (stress-tests
-- label wrapping at the ring node), one active season in the assignment phase,
-- 12 enrollments, and 12 assignment rows forming a single ring.
--
-- The admin user (id 00000000-0000-0000-0000-000000000001, phone +380670000001)
-- is inserted by the harness via seed/test_admin.sql before this file runs.
--
-- All IDs are deterministic fixed UUIDs so the ring ordering is stable.
--
-- Idempotent (mirrors seed/test_admin.sql): every INSERT carries
-- ON CONFLICT DO NOTHING, so a re-run against the same sibling DB is a no-op.
-- The bare form (no conflict target) suppresses ANY unique violation — needed
-- because these tables have multiple unique constraints per row (users: PK id
-- + phone; assignments: generated PK + both UNIQUE season pairs).

-- 12 cohort participants
INSERT INTO users (id, phone, name, role, status, onboarded, created_at) VALUES
  ('c0000000-0000-0000-0000-000000000001', '+380990000101', 'Олександра-Вікторія Кравченко-Мельниченко', 'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000002', '+380990000102', 'Максимілян-Богдан Шевченко-Бондаренко',     'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000003', '+380990000103', 'Анастасія-Марія Коваленко-Тимошенко',       'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000004', '+380990000104', 'Христофор-Данило Петренко-Іваненко',        'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000005', '+380990000105', 'Валентина-Ірина Сидоренко-Гриценко',        'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000006', '+380990000106', 'Євгенія-Тетяна Лисенко-Савченко',           'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000007', '+380990000107', 'Микола-Сергій Ткаченко-Мороз',              'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000008', '+380990000108', 'Ярослава-Оксана Василенко-Хоменко',         'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000009', '+380990000109', 'Зоряна-Надія Романенко-Поліщук',            'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000010', '+380990000110', 'Артем-Олег Захаренко-Гаврилюк',             'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000011', '+380990000111', 'Катерина-Людмила Дяченко-Руденко',          'participant', 'active', true, NOW()),
  ('c0000000-0000-0000-0000-000000000012', '+380990000112', 'Владислав-Ростислав Козаченко-Бабенко',     'participant', 'active', true, NOW())
ON CONFLICT DO NOTHING;

-- Delivery addresses (one per participant; nova_poshta_number is INTEGER)
INSERT INTO delivery_addresses (user_id, nova_poshta_city, nova_poshta_number, updated_at) VALUES
  ('c0000000-0000-0000-0000-000000000001', 'Кривий Ріг', 128, NOW()),
  ('c0000000-0000-0000-0000-000000000002', 'Київ',        1,   NOW()),
  ('c0000000-0000-0000-0000-000000000003', 'Львів',       5,   NOW()),
  ('c0000000-0000-0000-0000-000000000004', 'Харків',      12,  NOW()),
  ('c0000000-0000-0000-0000-000000000005', 'Одеса',       7,   NOW()),
  ('c0000000-0000-0000-0000-000000000006', 'Дніпро',      3,   NOW()),
  ('c0000000-0000-0000-0000-000000000007', 'Запоріжжя',   9,   NOW()),
  ('c0000000-0000-0000-0000-000000000008', 'Вінниця',     2,   NOW()),
  ('c0000000-0000-0000-0000-000000000009', 'Полтава',     4,   NOW()),
  ('c0000000-0000-0000-0000-000000000010', 'Чернівці',    6,   NOW()),
  ('c0000000-0000-0000-0000-000000000011', 'Житомир',     8,   NOW()),
  ('c0000000-0000-0000-0000-000000000012', 'Суми',        11,  NOW())
ON CONFLICT DO NOTHING;

-- One season in the assignment phase.
-- one_active_season partial index (phase NOT IN ('complete','cancelled')) allows
-- exactly one row here because this is a fresh sibling DB with no prior season.
INSERT INTO seasons (id, phase, signup_deadline, confirm_deadline, theme, launched_at, created_at)
VALUES (
  'a0000000-0000-0000-0000-000000000001',
  'assignment',
  NOW() + INTERVAL '30 days',
  NOW() + INTERVAL '60 days',
  'Великий сезон: Книги та листи від друзів з усієї України',
  NOW() - INTERVAL '1 hour',
  NOW() - INTERVAL '2 hours'
)
ON CONFLICT DO NOTHING;

-- 12 enrollment rows (all confirmed ready)
INSERT INTO enrollments (user_id, season_id, confirmed_ready_at, created_at)
SELECT
  id,
  'a0000000-0000-0000-0000-000000000001',
  NOW() - INTERVAL '30 minutes',
  NOW() - INTERVAL '1 hour'
FROM users
WHERE id LIKE 'c0000000%'
ON CONFLICT DO NOTHING;

-- 12-node ring: participant[i] sends to participant[i+1 mod 12]
INSERT INTO assignments (season_id, sender_id, recipient_id, created_at) VALUES
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000002', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000002', 'c0000000-0000-0000-0000-000000000003', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000003', 'c0000000-0000-0000-0000-000000000004', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000004', 'c0000000-0000-0000-0000-000000000005', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000005', 'c0000000-0000-0000-0000-000000000006', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000006', 'c0000000-0000-0000-0000-000000000007', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000007', 'c0000000-0000-0000-0000-000000000008', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000008', 'c0000000-0000-0000-0000-000000000009', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000009', 'c0000000-0000-0000-0000-000000000010', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000010', 'c0000000-0000-0000-0000-000000000011', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000011', 'c0000000-0000-0000-0000-000000000012', NOW()),
  ('a0000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000012', 'c0000000-0000-0000-0000-000000000001', NOW())
ON CONFLICT DO NOTHING;
