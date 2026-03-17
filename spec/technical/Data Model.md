# Data Model: The Mail Club

Authoritative schema definition. Supersedes the data model section in Architecture.md and the migration SQL in Implementation Plan.md.

Written 2026-03-14. Produced through first-principles analysis of the spec with explicit stakeholder decisions.

---

## Design Principles Applied

These are the stakeholder's explicit preferences, derived through spec-chef questioning:

1. **Enums over bools when the axis is expandable.** Role and account status are enums because new variants are plausible. Confirmed-ready is a nullable timestamp because it's a one-way latch with exactly two states (happened / hasn't happened) and the "when" matters.

2. **No unnecessary nullability.** Default values preferred. Null is only acceptable when "no value" is semantically distinct from any possible default (e.g., `receipt_note` — absence of a note is not the same as an empty note).

3. **Separate tables for separate concerns.** User identity and delivery logistics are different concerns. Collapsing them into one table complects identity with logistics.

4. **No redundant tables.** If data can be derived from existing tables via query, don't denormalize into a separate table. Only denormalize when query complexity becomes a real operational problem.

5. **Nullable timestamps as one-way latches.** Null = hasn't happened yet. Non-null = happened + when. Replaces boolean flags where the "when" carries information.

6. **Phases must correspond to actual behavioral gates in the app.** If two "phases" produce identical app behavior, they're one phase. Social/process distinctions are not app phases.

---

## Phase Enum

```
Enrollment → Preparation → Assignment → Delivery → Complete
                                                      ↑
                                          Cancelled (from any non-terminal)
```

| Phase | App Behavior | Gated By |
|-------|-------------|----------|
| Enrollment | Participants can enroll. Enroll button visible. | `signup_deadline` timestamp |
| Preparation | Enrollment closed. Participants create mail and confirm ready. Confirm button visible. | `confirm_deadline` timestamp |
| Assignment | Organizer generates, reviews, overrides assignments. Participants see "organizer is preparing." | Manual organizer advance |
| Delivery | Assignments visible to participants. Receipt confirmation available. SMS notifications fire. | Manual organizer advance (release) |
| Complete | Terminal. Nothing mutable. Historical record. | Manual organizer advance |
| Cancelled | Terminal. Reachable from any non-terminal phase. | Manual organizer action |

**Collapsed from original 8 to 6.** Creating/Confirming merged into Preparation (the confirm deadline gates the end, not a phase transition). Sending/Receiving merged into Delivery (receipt timing is driven by `notified_at` timestamp on assignments, not a phase transition).

---

## Tables

### users

Identity and account lifecycle. No logistics data.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK, DEFAULT `gen_random_uuid()` | |
| phone | TEXT | UNIQUE, NOT NULL | E.164: `+380XXXXXXXXX` |
| name | TEXT | NOT NULL | Legal name for Nova Poshta pickup |
| role | user_role enum | NOT NULL, DEFAULT 'participant' | Expandable: Participant, Admin |
| status | user_status enum | NOT NULL, DEFAULT 'active' | Expandable: Active, Deactivated |
| onboarded | BOOLEAN | NOT NULL, DEFAULT false | Explicit lifecycle flag. Survives if onboarding grows new fields |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT `now()` | |

**Why `role` enum over `is_admin` bool:** The role axis is expandable — community managers, trusted hosts are mentioned as future possibilities in the spec. An enum variant addition is a compiler-guided change in Rust.

**Why `status` enum over `is_active` bool:** Same reasoning. Suspended, Pending, Banned are plausible future states. The enum makes each state a named, matchable value.

**Why `onboarded` stays as bool:** It's a one-way latch (false → true, never back). We considered deriving it from `delivery_addresses` row existence, but the stakeholder prefers the explicit flag — it survives if onboarding ever collects more than just an address.

### delivery_addresses

Delivery logistics. Separate concern from user identity. 1:1 with users.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| user_id | UUID | PK, FK → users ON DELETE CASCADE | 1:1 relationship |
| nova_poshta_city | TEXT | NOT NULL | City name |
| nova_poshta_number | INTEGER | NOT NULL | Branch/warehouse number |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT `now()` | |

**Why structured fields over free text:** Nova Poshta branches have standardized numbers (~10,000 branches). Structured city + number catches a class of errors that free text misses. Not API-validated in season 1, but the shape provides basic consistency.

**Why separate table:** User identity (phone, name, role, status) is one concern. Delivery logistics (which post office to use) is another. Collapsing them complects identity with logistics. The address is read live at assignment-view time — no snapshot.

### sessions

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| token_hash | TEXT | PK | SHA-256 of raw token |
| user_id | UUID | NOT NULL, FK → users ON DELETE CASCADE | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT `now()` | |
| expires_at | TIMESTAMPTZ | NOT NULL | 90 days from creation |

### otp_codes

Retained (not upserted) to support rate limit queries from existing rows.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK, DEFAULT `gen_random_uuid()` | |
| phone | TEXT | NOT NULL | |
| code_hash | TEXT | NOT NULL | SHA-256 |
| attempts | INTEGER | NOT NULL, DEFAULT 0 | Per-code attempt counter |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT `now()` | |
| expires_at | TIMESTAMPTZ | NOT NULL | 10 minutes from creation |

**Why retained instead of upsert:** Rate limiting requires counting OTP requests per phone per hour. Upsert destroys old rows, making `COUNT` impossible. Retaining old rows means rate limit queries are simple:
- 1 per 60s: `WHERE phone = $1 AND created_at > now() - interval '60 seconds'`
- 5 per hour: `COUNT(*) WHERE phone = $1 AND created_at > now() - interval '1 hour'`

Only the most recent non-expired code for a given phone is valid for verification. Expired rows can be cleaned up lazily or via periodic admin action.

**Why `id` PK instead of `phone` PK:** `phone` is no longer unique per row since old codes are retained. UUID PK is the natural choice.

### seasons

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK, DEFAULT `gen_random_uuid()` | |
| phase | season_phase enum | NOT NULL, DEFAULT 'enrollment' | See Phase Enum section |
| signup_deadline | TIMESTAMPTZ | NOT NULL | End of enrollment window |
| confirm_deadline | TIMESTAMPTZ | NOT NULL | End of confirmation window |
| theme | TEXT | NULLABLE | Optional season theme |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT `now()` | |

**Constraint:** Partial unique index ensures at most one active season:
```sql
CREATE UNIQUE INDEX one_active_season ON seasons ((true))
    WHERE phase NOT IN ('complete', 'cancelled');
```

### enrollments

Junction between users and seasons. No logistics data — the delivery address is read live from `delivery_addresses`.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| user_id | UUID | FK → users ON DELETE CASCADE | |
| season_id | UUID | FK → seasons ON DELETE CASCADE | |
| confirmed_ready_at | TIMESTAMPTZ | NULLABLE | Null = not confirmed. Non-null = confirmed + when. One-way latch. |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT `now()` | |
| | | PRIMARY KEY (user_id, season_id) | |

**Why `confirmed_ready_at` instead of `confirmed_ready` bool:** The timestamp IS the confirmation. Null = hasn't confirmed. Non-null = confirmed at this time. The bool is derivable (`confirmed_ready_at IS NOT NULL`). One representation, carrying more information.

**Why no `nova_poshta_branch`:** Enrollment means "I'm in this season." The delivery address is a separate concern, always read live from `delivery_addresses`. If a participant updates their address after enrollment, the sender sees the current address — which is the correct behavior (sender needs the address that works *now*, not the one from signup day).

### assignments

Includes receipt data (merged from previously separate `receipts` table).

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK, DEFAULT `gen_random_uuid()` | |
| season_id | UUID | NOT NULL, FK → seasons ON DELETE CASCADE | |
| sender_id | UUID | NOT NULL, FK → users | |
| recipient_id | UUID | NOT NULL, FK → users | |
| notified_at | TIMESTAMPTZ | NULLABLE | Null = SMS not delivered yet. Non-null = when SMS succeeded |
| receipt_status | receipt_status enum | NOT NULL, DEFAULT 'no_response' | NoResponse / Received / NotReceived |
| receipt_note | TEXT | NULLABLE | Organizer-facing. "Anything the organizer should know?" |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT `now()` | |
| | | UNIQUE (season_id, sender_id) | One send per participant per season |
| | | UNIQUE (season_id, recipient_id) | One receive per participant per season |

**Why `notified_at` instead of `released` bool:** Release authorization is a phase concern (phase = Delivery means released). `notified_at` tracks a different fact: whether the SMS notification was successfully delivered to this specific participant. Null = SMS hasn't reached them. Organizer uses this to see who hasn't been reached.

**Why `receipt_status` enum instead of bool or nullable bool:** Three genuinely distinct states: (1) no response yet, (2) actively confirmed received, (3) actively reported not received. A bool collapses states 1 and 3. A nullable bool uses null-as-signal. The enum names all three states explicitly.

**Why no separate `receipts` table:** The separation was incidental, not deliberate. Receipt data is always accessed in the context of an assignment. No join needed. Row existence as a signal for "has responded" is replaced by the enum default (`NoResponse`).

### known_groups

Social graph. Organizer-managed directly in DB for season 1.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| id | UUID | PK, DEFAULT `gen_random_uuid()` | |
| name | TEXT | NOT NULL | |
| weight | INTEGER | NOT NULL, DEFAULT 1 | Higher = stronger connection |

### known_group_members

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| group_id | UUID | FK → known_groups ON DELETE CASCADE | |
| user_id | UUID | FK → users ON DELETE CASCADE | |
| | | PRIMARY KEY (group_id, user_id) | |

---

## Removed from Original Spec

| Original | Decision | Reasoning |
|----------|----------|-----------|
| `past_pairings` table | Removed. Query `assignments` joined with completed seasons instead. | No redundancy. The assignment table IS the pairing history. |
| `receipts` table | Merged into `assignments` as `receipt_status` enum + `receipt_note`. | Separation was incidental. Receipt is always accessed with its assignment. |
| `assignments.released` bool | Replaced by phase semantics + `notified_at` timestamptz. | Release = phase concern. Notification delivery = per-assignment tracking. |
| `users.is_admin` bool | Replaced by `role` enum. | Expandable axis. |
| `users.is_active` bool | Replaced by `status` enum. | Expandable axis. |
| `users.nova_poshta_branch` text | Moved to `delivery_addresses` table, structured as city + number. | Separate concerns. Structured over free text. |
| `enrollments.nova_poshta_branch` | Removed. Address read live from `delivery_addresses`. | Enrollment doesn't need logistics data. |
| `enrollments.confirmed_ready` bool | Replaced by `confirmed_ready_at` nullable timestamptz. | Timestamp IS the latch + carries "when." |
| `otp_codes` upsert (phone as PK) | Retained rows, UUID PK. | Rate limit queries need historical row counts. |
| 8 season phases | Collapsed to 6. | Creating/Confirming and Sending/Receiving were social timelines, not app behavioral gates. |

---

## Custom Postgres Types

```sql
CREATE TYPE season_phase AS ENUM (
    'enrollment', 'preparation', 'assignment', 'delivery', 'complete', 'cancelled'
);

CREATE TYPE user_role AS ENUM (
    'participant', 'admin'
);

CREATE TYPE user_status AS ENUM (
    'active', 'deactivated'
);

CREATE TYPE receipt_status AS ENUM (
    'no_response', 'received', 'not_received'
);
```

---

## Social Weight Computation (No Table)

Past pairings are computed at assignment-generation time by querying:

```sql
SELECT sender_id, recipient_id
FROM assignments a
JOIN seasons s ON s.id = a.season_id
WHERE s.phase IN ('complete', 'cancelled')
```

Combined with known_group_members data to produce a weight matrix. No materialized table needed at N≤50.
