# g-authverify — auth flow adversarial verification (static trace)

Binding: stop-yapping / first-principles / kiss / dry read in full. Commitments: I must cut noise -> fragments only. I must derive from source -> file:line per claim. I must keep simple -> one finding, one home. I must not duplicate -> table holds verdicts, proof section holds sequence.

Empirical (Q5): NOT RUN. `ls target/*/` = target/debug/{build,deps,examples,incremental} + wasm32 dir; `find target -maxdepth 3 -name samete -type f` = empty. `pg_isready` = "/var/run/postgresql:5432 - no response". No binary, no DB -> static trace only. Proof below is UNEXECUTED.

## Verdicts

| Q | Verdict | Severity | Evidence |
|---|---|---|---|
| 1a Login for EXISTING phone without OTP | NO | OK | session minted only after `verify_otp_for_phone` Ok: login.rs:~150-205 (`otp_result.is_err()` -> redirect). Session token = 32 random bytes, stored sha256: auth.rs `create_session`. No client-trusted cookie on this path. |
| 1b Registration for NEW phone without OTP | YES (needs a valid unused invite code) | **BLOCKER** | `pending_phone` cookie = raw normalized phone, unsigned/unencrypted/not bound to server state: set at login.rs:~190 via `set_cookie_header("pending_phone", &normalized, 300)`. `register_with_code` trusts it solely: login.rs:~391 `extract_pending_phone_cookie` -> `phone_mod::normalize` -> INSERT users (phone from cookie) login.rs:~420. No server-side OTP-verified record (no table/row; `otp_codes` row deleted on verify). HttpOnly/Secure/SameSite=Strict (login.rs:481) only restrict browsers; curl sets any Cookie header. `validate_invite_code` gates on cookie presence only (login.rs:~265). |
| 1c Takeover of EXISTING account via forged pending_phone | NO | OK | `users.phone UNIQUE` (migrations/...000002:3) -> INSERT fails -> redirect `/login?pending=1` (login.rs:~420). Code not consumed (tx dropped). |
| 2 Invite codes | entropy LOW, no throttle, no expiry; single-use OK | MAJOR | see below |
| 3 OTP | sound; TEST_MODE hazard | MAJOR (config hazard) | see below |
| 4 First admin | NO production path | MAJOR | see below |
| 5 Empirical | not run | n/a | no binary, no DB |

## 1b request-sequence proof (static; unexecuted)

Prereq: attacker holds one unused invite code C (code format "word-word", see Q2).
```
curl -i -X POST https://HOST/api/register_with_code \
  -H 'Cookie: pending_phone=+380671234567' \
  --data-urlencode 'code=C' --data-urlencode 'name=X'
```
Expected (from code): tx locks invite row, inserts user (+380671234567), marks code used, `Set-Cookie: session=<token>`, 302 -> /onboarding. No OTP requested for that phone; no SMS to victim. Endpoint path/encoding is leptos default `#[server]` (`/api/register_with_code`); not confirmed against a running router (no binary). Impact: register any unclaimed phone number as someone else (squat; later real owner gets AccountExists + can log in via OTP into attacker-chosen name/address). Also: attacker needs no phone ownership, defeating OTP purpose. Mitigant: needs valid invite code (admin-distributed).

## Q2 invite codes
- Entropy: 200 words (invite_codes.rs WORD_LIST, counted 200, 0 dupes) x 199 ordered distinct pair = 39,800 codes (~15.3 bits). No suffix. `pick_two_words` = `choose_multiple(rng,2)`; format `a-b` invite_codes.rs:262.
- Throttle on redemption: NONE. `validate_invite_code` / `register_with_code` have no attempt counter, no IP or phone limit (grep rate/attempt in login.rs hits only request_otp/verify). With 1b forgery, attacker enumerates 39,800 guesses, oracle = distinct errors (invalid/used/revoked messages, login.rs:~280-300). Density: N outstanding unused codes -> expected hit rate N/39,800 per guess. Enumeration also leaks which codes are used/revoked.
- Expiry: NONE. Schema (migrations/...000003) has no expires_at.
- Single-use: OK. `SELECT ... FOR UPDATE` + status check + INSERT user + UPDATE code in one tx (login.rs:~407-445); concurrent redeem blocks then sees `used`. Minor: `UPDATE` and `commit` results discarded (`let _ =`, login.rs:~437-445) — failed commit still proceeds to mint a session for a user_id that may not exist -> session FK error -> redirect /login. Minor.

## Q3 OTP
- Entropy: 6 digits, uniform `random_range(0..1_000_000)` (auth.rs create_otp). Hash: unsalted sha256 of 6 digits (auth.rs) — DB read leak = instant recovery (10^6 brute); low value given 10-min TTL.
- Expiry 10 min (auth.rs create_otp). Attempts: 3 per row, then row deleted (auth.rs verify / login.rs `verify_otp_for_phone`). Attempt increment is read-then-write non-atomic (SELECT then UPDATE): parallel requests can exceed 3 on one row; bounded by request fan-out per row. Issue rate: 1/60s, 5/h per phone (auth.rs check_otp_rate_limit) -> ~15 guesses/h/phone sequential; MINOR.
- Rate-limit failure is silent (returns AccountExists) — fine.
- TEST_MODE: read as `std::env::var("SAMETE_TEST_MODE").as_deref()==Ok("true")` per call (auth.rs create_otp, check_otp_rate_limit; home.rs:180,398,521). Only exact string "true" enables. Default off (unset). When on: OTP fixed "000000", rate limit skipped -> anyone logs in as any existing phone incl. admin. `.env.example:2` exports it =true; Dockerfile sets no env for it (Dockerfile:21-22 only LEPTOS_*). No startup guard refusing TEST_MODE with non-dry-run SMS (config.rs has no check). Misconfig in Coolify env = full auth bypass. Also `SAMETE_SMS_DRY_RUN=true` in prod silently stops SMS (config.rs).

## Q4 first admin
- Only `INSERT INTO users` anywhere outside register_with_code: seed/test_admin.sql (fixed id ...0001, phone +380670000001, role admin), run by `just db-seed` (justfile:68-71; README.md:58 "used by E2E pipeline"). Migrations: no admin insert (grep `INSERT INTO users` in migrations = none). Self-registration requires an invite code whose `distributor_id NOT NULL REFERENCES users` (migrations/...000003) and codes are generated only via `require_admin` (admin/invite_codes.rs:90) -> chicken-and-egg: fresh prod DB cannot get an admin via the app. README Deployment section (README.md:194-203) and spec: no bootstrap doc (`grep -i "first admin|bootstrap"` = none). Prod path = manual psql INSERT, or running test seed (puts a publicly known phone +380670000001 as admin — takeover-able if that number's OTP is obtainable; combined with TEST_MODE hazard = trivial). MAJOR: undocumented, undelivered.

## Other trust edges
- `session` cookie: HttpOnly; Secure; SameSite=Strict; 90d (login.rs:481, 7_776_000); server-side sha256 lookup (auth.rs validate_session) — OK. Cookie parser takes first `session=` match; fine.
- No CSRF token (config.rs comment: SameSite=Strict is the mitigation). OK for same-site.
- `Secure` flag: plain-http dev (non-localhost) drops cookies; prod behind Traefik TLS — OK.
- Phone-existence enumeration: `request_otp` returns distinct enum (AccountExists/NewAccount) to anyone, no auth, only OTP rate limit; docstring claims "only visible to phone holder" — false, the response goes to the requester. MINOR/MAJOR (privacy: reveals membership of any phone). SMS also sent to arbitrary victim phones (SMS-bomb limited 5/h/phone, no IP limit — known deferred item).

## Fix direction (not applied)
Bind verified phone server-side (signed cookie via HMAC with config secret, or DB row `verified_phones(token_hash, phone, expires_at)`; cookie carries only random token). Add attempt limit on invite redemption; add suffix entropy to codes; startup refuse TEST_MODE unless dry-run SMS; document/automate first-admin bootstrap (env-driven one-shot).
