# Implementation Plan: Launch Blockers U1–U4

HEAD at planning: `1a3c8a5`. Evidence: `readiness/d-ops.md` (F1/F2/F5), `readiness/g-authverify.md` (1b), `readiness/i-swapverify.md`, `readiness/a-scope.md` (3.3 rows). Every claim below re-verified against source at HEAD.

## Preamble

| Unit | Blocker | Verified at |
|---|---|---|
| U1 | Self-registration skips OTP: `register_with_code` / `validate_invite_code` trust a raw, unsigned `pending_phone=<phone>` cookie. curl with any phone + one invite code creates the account. Invite redemption is unthrottled (~39,800-code space). | `src/pages/login.rs:208-219` (cookie = normalized phone), `:320-328`, `:391-399`, `:485-496` |
| U2 | `SAMETE_TEST_MODE=true` = OTP `000000` for every phone incl. admin + OTP rate limit off. No boot guard. Read ad hoc at 5 sites. Undocumented. | `src/auth.rs:77,121`, `src/pages/home.rs:180,398,521`, `src/config.rs` (no check), `README.md:185-192` |
| U3 | No production path to the first admin; invite codes require an admin to exist (circular). | `migrations/20260314000003…:5` (`distributor_id NOT NULL`), `src/admin/invite_codes.rs:90`, only admin insert = `seed/test_admin.sql` |
| U4 | `swap_assignment` exchanges two senders' recipients. Math: in one cycle this always splits it (or self-loops); first UPDATE also violates non-deferrable `UNIQUE(season_id, recipient_id)`; three statements on the pool, no tx, validation after writes; `validate_swap_topology` treats all rows as one cohort; `advance_season` never re-validates; E2E 3.3 asserts only that the viz renders. | `src/admin/assignments.rs:184-226,318-416,435-488`, `migrations/20260314000002…:56-67`, `src/admin/season.rs:144-174`, `end2end/tests/mail_club.spec.ts:649-654`, `fixtures/mail_club_page.ts:483-498` |

Story 3.3 (spec/technical/User Stories.md:228-241): "swap individual sender→recipient pairings while maintaining cycle integrity"; AC "Swaps must preserve the single-loop topology"; "organizer sees the full graph for each cohort". The only swap of two people that preserves every loop is **exchanging their positions in the cycle** (conjugating the sender→recipient map by the transposition (a b)). U4 implements exactly that.

## Design decisions (final — do not revisit)

- **U1:** Server-side `registration_tickets` table is the sole authority. On OTP success for an unknown phone the server inserts `(sha256(token), phone, expires_at = now()+10min, invite_attempts = 0)` and sets cookie `registration_ticket=<random 32-byte base64url token>`. The phone is read ONLY from the ticket row. Registration consumes the row (`DELETE … RETURNING phone`) inside the registration transaction. Throttle falls out of the same row: each invalid/used/revoked code submission increments `invite_attempts`; a ticket with `invite_attempts >= 5` resolves to nothing → user must re-verify via OTP (itself limited to 5/h/phone). No U1b.
- **U2:** `Config.sms: SmsMode { Live{token,sender} | DryRun{test_mode} }` — test mode is unrepresentable with live SMS. Boot refuses `SAMETE_TEST_MODE=true` without `SAMETE_SMS_DRY_RUN=true`, and refuses test mode on a non-loopback bind address (prod container binds `0.0.0.0`; `just e2e`, CI, `isolated-capture.sh`, `just dev` all bind `127.0.0.1`). All 5 env reads replaced by `Config::test_mode()`.
- **U3:** Idempotent startup bootstrap from `SAMETE_ADMIN_PHONE` + `SAMETE_ADMIN_NAME` (both or neither; phone normalized via `phone::normalize`; invalid → boot refuses). Upsert: insert admin, or promote existing user with that phone to admin. Documented in README + `.env.example`.
- **U4:** Position-exchange swap as a pure function in `src/assignment.rs`; cohorts recovered from edges by walking (multi-cohort-correct); swap = one transaction: lock season row, load edges, compute, validate, `DELETE` season rows, bulk `INSERT`, commit. `advance_season` re-validates when leaving Assignment. Preview renders every cohort.

## Write-sets and sequencing

| Unit | Files written |
|---|---|
| U1 | `migrations/20261004000001_registration_tickets.sql` (new), `src/auth.rs`, `src/pages/login.rs`, `.sqlx/*`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/mail_club.spec.ts` |
| U2 | `src/config.rs`, `src/main.rs`, `src/sms.rs`, `src/auth.rs`, `src/pages/login.rs`, `src/pages/home.rs`, `README.md`, `.env.example` |
| U3 | `src/config.rs`, `src/main.rs`, `src/db.rs`, `.sqlx/*`, `README.md`, `.env.example` |
| U4 | `src/assignment.rs`, `src/admin/assignments.rs`, `src/admin/season.rs`, `src/admin/page.rs`, `locales/uk.json`, `.sqlx/*`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/visual-audit.spec.ts` |

Overlaps: U1∩U2 = `auth.rs`, `login.rs`. U2∩U3 = `config.rs`, `main.rs`, `README.md`, `.env.example`. U1∩U4 = POM + `mail_club.spec.ts` (disjoint regions: Epic 1 vs Epic 3 / invite vs assignment POM sections) + `.sqlx/` (distinct query files). U3∩U1/U4 = `.sqlx/` only.

**Order:**
- **Wave A (parallel, three worktrees from main):** U1, U3, U4.
- **Integrate wave A** in order U4 → U3 → U1 (each: rebase onto updated main; textual conflicts in `.sqlx/` or test files → resolve, then re-run that unit's sqlx regeneration + all gates).
- **Wave B:** U2, branched from main AFTER U1 and U3 are integrated (U2 edits the post-U1 `auth.rs`/`login.rs` and post-U3 `config.rs`/`main.rs`/`README.md`/`.env.example`).
- After the last integration, the orchestrator runs the Global Gates on main.

## Shared environment (every unit)

Commit style (CLAUDE.md, binding): one-line conventional commit, no body, no `Co-Authored-By`, no AI attribution.

Postgres (once per container):
```bash
pg_isready -h localhost -p 5432 || docker compose -f /home/user/same-te-mail-club/docker-compose.yml up -d
until pg_isready -h localhost -p 5432; do sleep 1; done
```
**REQUIRED OUTPUT:** `localhost:5432 - accepting connections`.

Per-unit sibling DB (NEVER the shared `samete` DB; NEVER `just db-reset`, `just e2e*`, `_kill-stale`, port 3000 — the isolated harness is the e2e path):
```bash
sqlx --version || cargo binstall --no-confirm sqlx-cli --no-default-features --features postgres,rustls
export DATABASE_URL=postgres://samete:samete@localhost:5432/samete_<unit>   # <unit> = u1|u2|u3|u4
sqlx database drop -y; sqlx database create && sqlx migrate run
```

sqlx cache regeneration (after ANY `query!`/`query_as!`/`query_scalar!` change, including whitespace):
```bash
cargo sqlx prepare --workspace -- --features ssr
git status --short .sqlx | head
```
**REQUIRED:** command exits 0; `.sqlx/` has added/modified files for every new query. `ls .sqlx | wc -l` is NOT 0.

Standard gates (run from the worktree root, in this order, after the unit's last code change):
```bash
cargo fmt --all -- --check
SQLX_OFFLINE=true cargo clippy --features ssr --no-default-features -- -D warnings
SQLX_OFFLINE=true cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings
cargo test 2>&1 | grep -E "^test result"
SQLX_OFFLINE=true cargo test --features ssr 2>&1 | grep -E "^test result"
```
**REQUIRED OUTPUT:** fmt prints nothing, exit 0. Both clippy runs end with `Finished` and contain zero `warning:`/`error:` lines. Every `test result:` line reads `ok.` with `0 failed`.

E2E gate (isolated harness — own port + own DB `samete_e2e_<unit>`; requires release build; run in background, output to file, never pipe through `tail`/`head`):
```bash
export PLAYWRIGHT_BROWSERS_PATH=/opt/pw-browsers
[ -d end2end/node_modules ] || (cd end2end && npm ci)
bash scripts/isolated-capture.sh e2e_<unit> full > /tmp/e2e_<unit>.log 2>&1; echo "exit=$?" >> /tmp/e2e_<unit>.log
grep -E "exit=|[0-9]+ (passed|failed|flaky|skipped)" /tmp/e2e_<unit>.log
```
**REQUIRED OUTPUT:** `exit=0`; a `N passed` line; NO `failed` line; NO `flaky` line; `2 skipped`. Every new/changed test title named in the unit appears with a pass mark in the list output (`grep -F "<title>" /tmp/e2e_<unit>.log`). If `orchestration_log/recon/2026-10-04/readiness/f-e2e.md` exists and records a different working recipe for this container, use that recipe's environment deltas (e.g. browser path) — the pass criteria above stay identical. Stability: run the E2E gate 3 times; all 3 must meet the criteria.

---

# U1 — Server-verified registration tickets + invite throttle

## Why This Matters
Any person with one invite code can register any phone number they do not own (no SMS reaches the victim). OTP exists to prove phone ownership; the current cookie proves nothing. Invite codes can be brute-forced with no limit.

## What You Must Do

### U1.1 Migration (new file, exact name and content)
`migrations/20261004000001_registration_tickets.sql`:
```sql
-- Server-side proof that a phone passed OTP verification and may register.
-- The client holds only a random token; the phone is never read from the client.
CREATE TABLE registration_tickets (
    token_hash TEXT PRIMARY KEY,
    phone TEXT NOT NULL,
    invite_attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);
```
Then: `sqlx migrate run` against `samete_u1`.

### U1.2 `src/auth.rs` — TDD, pure helpers first
Step 1 — add failing tests to the existing `#[cfg(test)] mod tests` (change its `use` to `use super::{constant_time_hash_eq, extract_cookie, generate_token, sha256_hex};`):
```rust
    fn parts_with_cookie(header: &str) -> http::request::Parts {
        http::Request::builder()
            .header(http::header::COOKIE, header)
            .body(())
            .expect("request builds")
            .into_parts()
            .0
    }

    #[test]
    fn generate_token_is_43_url_safe_characters() {
        let token = generate_token();
        assert_eq!(token.len(), 43);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn generate_token_is_distinct_per_call() {
        assert_ne!(generate_token(), generate_token());
    }

    #[test]
    fn extract_cookie_finds_named_cookie_among_several() {
        let parts = parts_with_cookie("session=abc; registration_ticket=xyz");
        assert_eq!(extract_cookie(&parts, "registration_ticket").as_deref(), Some("xyz"));
        assert_eq!(extract_cookie(&parts, "session").as_deref(), Some("abc"));
    }

    #[test]
    fn extract_cookie_does_not_match_a_longer_name() {
        let parts = parts_with_cookie("registration_ticket_old=zzz; xsession=1");
        assert_eq!(extract_cookie(&parts, "registration_ticket"), None);
        assert_eq!(extract_cookie(&parts, "session"), None);
    }

    #[test]
    fn extract_cookie_returns_none_without_cookie_header() {
        let parts = http::Request::builder().body(()).expect("request builds").into_parts().0;
        assert_eq!(extract_cookie(&parts, "session"), None);
    }
```
Step 2 — run `SQLX_OFFLINE=true cargo test --features ssr auth::tests 2>&1 | grep -E "error\[|test result"`. **REQUIRED:** compile error naming `generate_token` / `extract_cookie` (red).

Step 3 — implement (replace `extract_session_cookie` entirely; its one caller `current_user` becomes `extract_cookie(parts, "session")`):
```rust
/// Generate a random 32-byte URL-safe token (43 chars, no padding).
pub(crate) fn generate_token() -> String {
    use rand::RngCore as _;
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Read cookie `name` from the request's `Cookie` header (exact name match).
pub(crate) fn extract_cookie(parts: &http::request::Parts, name: &str) -> Option<String> {
    let cookie_header = parts.headers.get(http::header::COOKIE)?.to_str().ok()?;
    cookie_header.split(';').find_map(|pair| {
        let (key, value) = pair.trim().split_once('=')?;
        (key == name).then(|| value.to_owned())
    })
}
```
`create_session`: replace its first four lines (`use rand::RngCore`, `bytes`, `fill_bytes`, `raw_token = …encode`) with `let raw_token = generate_token();`.

Step 4 — add the ticket API (place after `create_session`):
```rust
/// Maximum invalid invite-code submissions per registration ticket.
/// After this many, the ticket stops resolving and the user must re-verify by OTP.
pub const MAX_INVITE_ATTEMPTS: i32 = 5;

/// Record that `phone` passed OTP verification and may self-register.
///
/// Returns the raw token for the `registration_ticket` cookie; only its
/// SHA-256 is stored. Ticket lives 10 minutes.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn create_registration_ticket(pool: &PgPool, phone: &str) -> Result<String, AppError> {
    let raw_token = generate_token();
    let token_hash = sha256_hex(&raw_token);
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM registration_tickets WHERE expires_at < now()")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        r#"
        INSERT INTO registration_tickets (token_hash, phone, expires_at)
        VALUES ($1, $2, now() + interval '10 minutes')
        "#,
        token_hash,
        phone,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(raw_token)
}

/// Resolve a live ticket to its OTP-verified phone without consuming it.
///
/// # Errors
///
/// Returns `Err(AppError::Unauthorized)` if the ticket is unknown, expired,
/// or exhausted; `Err(AppError::Database(_))` on DB failure.
pub async fn registration_ticket_phone(pool: &PgPool, raw_token: &str) -> Result<String, AppError> {
    let token_hash = sha256_hex(raw_token);
    sqlx::query_scalar!(
        r#"
        SELECT phone FROM registration_tickets
        WHERE token_hash = $1 AND expires_at > now() AND invite_attempts < $2
        "#,
        token_hash,
        MAX_INVITE_ATTEMPTS,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)
}

/// Consume a live ticket inside the caller's transaction, returning its phone.
/// Rolling back the transaction restores the ticket.
///
/// # Errors
///
/// Returns `Err(AppError::Unauthorized)` if the ticket is unknown, expired,
/// or exhausted; `Err(AppError::Database(_))` on DB failure.
pub async fn consume_registration_ticket(
    conn: &mut sqlx::PgConnection,
    raw_token: &str,
) -> Result<String, AppError> {
    let token_hash = sha256_hex(raw_token);
    sqlx::query_scalar!(
        r#"
        DELETE FROM registration_tickets
        WHERE token_hash = $1 AND expires_at > now() AND invite_attempts < $2
        RETURNING phone
        "#,
        token_hash,
        MAX_INVITE_ATTEMPTS,
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(AppError::Unauthorized)
}

/// Count one invalid invite-code submission against the ticket.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn record_failed_invite_attempt(pool: &PgPool, raw_token: &str) -> Result<(), AppError> {
    let token_hash = sha256_hex(raw_token);
    sqlx::query!(
        "UPDATE registration_tickets SET invite_attempts = invite_attempts + 1 WHERE token_hash = $1",
        token_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}
```
Step 5 — `sqlx prepare` (shared env), then rerun step-2 command. **REQUIRED:** `test result: ok.` with the 5 new test names passing (`cargo test --features ssr auth::tests -- --list` lists `generate_token_is_43_url_safe_characters`, `generate_token_is_distinct_per_call`, `extract_cookie_finds_named_cookie_among_several`, `extract_cookie_does_not_match_a_longer_name`, `extract_cookie_returns_none_without_cookie_header`).

### U1.3 `src/pages/login.rs`
Define once near `set_cookie_header`:
```rust
/// Cookie carrying the opaque registration-ticket token (never the phone).
#[cfg(feature = "ssr")]
const REGISTRATION_COOKIE: &str = "registration_ticket";
```
1. `verify_otp_code`, `None =>` branch: replace the body with:
```rust
        None => {
            // Unknown phone passed OTP: record server-side proof and hand the
            // client an opaque token. The phone is never read back from the client.
            let raw_ticket = auth::create_registration_ticket(&pool, &normalized)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            let response_options =
                leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
            let cookie = set_cookie_header(REGISTRATION_COOKIE, &raw_ticket, 600);
            response_options.append_header(
                axum::http::header::SET_COOKIE,
                axum::http::HeaderValue::from_str(&cookie)
                    .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
            );
            leptos_axum::redirect("/login?pending=1");
            Ok(false)
        }
```
Update the function's doc bullet "No account → set a short-lived `pending_phone` cookie…" to: "No account → store a registration ticket server-side, set the opaque `registration_ticket` cookie (HttpOnly, 10 min), redirect to `/login?pending=1`". Update the inline comment block above `match` (case 3) the same way; delete the "Deactivated user … pending_phone" wording → "without issuing a registration ticket".

2. `validate_invite_code`: replace the "Require that the phone was OTP-verified" block with:
```rust
    let parts = leptos::context::use_context::<http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;
    let not_verified = || ServerFnError::new(td_string!(Locale::uk, auth_phone_not_verified));
    let ticket = crate::auth::extract_cookie(&parts, REGISTRATION_COOKIE).ok_or_else(not_verified)?;
    crate::auth::registration_ticket_phone(&pool, &ticket)
        .await
        .map_err(|_| not_verified())?;
```
Keep the empty-code check unchanged (empty code does NOT count as an attempt). Replace the final `match status { … }` with:
```rust
    let rejection = match status {
        Some(InviteCodeStatus::Unused) => return Ok(code),
        None => td_string!(Locale::uk, auth_invite_code_invalid),
        Some(InviteCodeStatus::Used) => td_string!(Locale::uk, auth_invite_code_used),
        Some(InviteCodeStatus::Revoked) => td_string!(Locale::uk, auth_invite_code_revoked),
    };
    crate::auth::record_failed_invite_attempt(&pool, &ticket)
        .await
        .map_err(crate::error::AppError::into_server_fn_error)?;
    Err(ServerFnError::new(rejection))
```
(If `td_string!` returns a type that does not unify across arms, bind each arm with `.to_string()`; the `ServerFnError::new` argument stays the localized message.) Doc comment: replace every `pending_phone` mention with "registration ticket"; add bullet "- the ticket is unknown, expired, or exhausted (5 invalid codes)".

3. `register_with_code`: replace the body from `let parts = …` through `let _ = tx.commit().await;` with:
```rust
    let parts = leptos::context::use_context::<http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let Some(ticket) = auth::extract_cookie(&parts, REGISTRATION_COOKIE) else {
        leptos_axum::redirect("/login");
        return Ok(());
    };

    let name = name.trim().to_owned();
    if name.is_empty() {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    }

    let Ok(mut tx) = pool.begin().await else {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };

    // The phone comes ONLY from the server-side ticket. Consumed in this tx;
    // any early return drops the tx, rolling the ticket back.
    let Ok(phone) = auth::consume_registration_ticket(&mut *tx, &ticket).await else {
        leptos_axum::redirect("/login");
        return Ok(());
    };

    let code_row = sqlx::query!(
        r#"SELECT id, status AS "status: String" FROM invite_codes WHERE code = $1 FOR UPDATE"#,
        code,
    )
    .fetch_optional(&mut *tx)
    .await;
    let Ok(Some(code_row)) = code_row.map(|row| row.filter(|r| r.status == "unused")) else {
        drop(tx);
        if let Err(e) = auth::record_failed_invite_attempt(&pool, &ticket).await {
            tracing::warn!(error = %e, "failed to record invite attempt");
        }
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };

    let Ok(user_id) = sqlx::query_scalar!(
        r#"INSERT INTO users (phone, name) VALUES ($1, $2) RETURNING id"#,
        phone,
        name,
    )
    .fetch_one(&mut *tx)
    .await
    else {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };

    let Ok(_) = sqlx::query!(
        r#"UPDATE invite_codes SET status = 'used', redeemer_id = $1, redeemed_at = now() WHERE id = $2"#,
        user_id,
        code_row.id,
    )
    .execute(&mut *tx)
    .await
    else {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };

    let Ok(()) = tx.commit().await else {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };
```
Note: the `SELECT … invite_codes … FOR UPDATE`, `INSERT INTO users` and `UPDATE invite_codes` SQL strings are byte-identical to HEAD (keep them so — do not re-indent). The DB-error case of the code lookup also counts as an attempt; accept that. Remove `phone as phone_mod` from this fn's `use` (phone already normalized at ticket creation). In the cookie-clearing block replace `set_cookie_header("pending_phone", "", 0)` with `set_cookie_header(REGISTRATION_COOKIE, "", 0)`. Rewrite the doc comment steps 1/4/7 to "registration ticket" wording.

4. Delete `extract_pending_phone_cookie` entirely. Delete `check_pending_registration` entirely (server fn, its doc comment and `#[allow]`) — it has zero callers (`grep -rn check_pending_registration src end2end` = only its definition). In the `LoginPage` doc comment delete the paragraph "Cookie detection for step 3/4 is done via a server Resource (`check_pending_registration`) …" and change "server sets `pending_phone` cookie" → "server issues a registration ticket".

5. `NameCollectionForm`: add `data-testid="register-form"` to the `<form method="post" action=RegisterWithCode::url()>` element. No other view! change.

### U1.4 E2E — POM (`end2end/tests/fixtures/mail_club_page.ts`)
- Comments: replace every `pending_phone` with `registration_ticket` (lines ~297, ~313).
- Add after `selfRegister` (add `APIRequestContext` to the existing `@playwright/test` type import in this file):
```ts
  /**
   * Submit one invite code on the invite-code step (reached via
   * reachInviteCodeStep). Waits for the server response; caller asserts.
   */
  async submitInviteCode(code: string) {
    await this.page.getByTestId("invite-code-input").fill(code);
    await this.clickAndWaitForResponse(
      this.page.getByTestId("submit-invite-code-button"),
      "validate_invite_code",
    );
  }

  /**
   * Attack probe (Story 1.1 security AC): POST register_with_code from a
   * cookieless API context with a forged ticket cookie whose value is a phone.
   * Returns the response's Set-Cookie header ("" if none).
   */
  async forgeRegistration(request: APIRequestContext, phone: string, code: string): Promise<string> {
    await this.page.goto("/login");
    const action = await this.page.getByTestId("register-form").getAttribute("action");
    expect(action).toBeTruthy();
    const response = await request.post(action as string, {
      form: { code, name: "Підробка" },
      headers: { Cookie: `registration_ticket=${phone}; pending_phone=${phone}` },
      maxRedirects: 0,
    });
    return response.headers()["set-cookie"] ?? "";
  }
```

### U1.5 E2E — spec (`end2end/tests/mail_club.spec.ts`)
- `EXTRA_PHONES`: add `FORGED_COOKIE_TEST: "+380670000008",` and `THROTTLE_TEST: "+380670000009",`.
- `CODES` type and initializer: add `FORGE: string` / `FORGE: ""`.
- Insert immediately after the test `"1.1 — used invite code is rejected"` (inside the same Epic 1 describe):
```ts
    test("setup — generate invite code for registration-security tests", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      CODES.FORGE = await app.generateInviteCode();
      expect(CODES.FORGE.length).toBeGreaterThan(0);
    });

    // Story 1.1 security AC: registration requires a server-verified OTP for that phone
    test("1.1 — forged registration cookie cannot create an account", async ({ page, request }) => {
      const app = new MailClubPage(page);
      const setCookie = await app.forgeRegistration(request, EXTRA_PHONES.FORGED_COOKIE_TEST, CODES.FORGE);
      expect(setCookie).not.toMatch(/session=[^;]+/);
      // The phone is still unregistered: a real OTP verify routes it to the invite step.
      await app.reachInviteCodeStep(EXTRA_PHONES.FORGED_COOKIE_TEST);
      await expect(page.getByTestId("invite-code-step")).toBeVisible();
      // The code was not consumed.
      await app.login(ADMIN_PHONE);
      await app.expectInviteCodeStatus(CODES.FORGE, "unused");
    });

    // Story 1.1 security AC: invite-code guessing is throttled per OTP verification
    test("1.1 — invite code attempts are throttled per verification", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.reachInviteCodeStep(EXTRA_PHONES.THROTTLE_TEST);
      for (let i = 0; i < 5; i++) {
        await app.submitInviteCode(`wrong-guess-${i}`);
        await expect(page.getByTestId("invite-code-error")).not.toBeEmpty();
      }
      // A valid unused code is now refused: the ticket is exhausted.
      await app.submitInviteCode(CODES.FORGE);
      await expect(page.getByTestId("invite-code-error")).toContainText("Почни спочатку");
      await expect(page.getByTestId("legal-name-input")).not.toBeVisible();
    });

    test("setup — revoke registration-security invite code", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.revokeInviteCode(CODES.FORGE);
      await app.expectInviteCodeStatus(CODES.FORGE, "revoked");
    });
```
(The code is revoked so no extra active participant or unused code leaks into later count assertions such as 5.3's "3 active participants".)

## Verification Gates (U1)
```bash
grep -rn "pending_phone" src/
```
**REQUIRED:** zero matches.
```bash
grep -rn "extract_pending_phone_cookie\|check_pending_registration\|CheckPendingRegistration" src/ end2end/tests/
```
**REQUIRED:** zero matches.
```bash
grep -n "fn extract_session_cookie" src/auth.rs
```
**REQUIRED:** zero matches.
```bash
awk '/pub async fn register_with_code/,/^}/' src/pages/login.rs | grep -c "normalize"
```
**REQUIRED:** `0` (the registered phone comes only from the ticket).
```bash
awk '/pub async fn register_with_code/,/^}/' src/pages/login.rs | grep -c "let _ ="
```
**REQUIRED:** `0`.
```bash
ls migrations/ | tail -1
```
**REQUIRED:** `20261004000001_registration_tickets.sql`.
Standard gates, sqlx regeneration, E2E gate (titles: `1.1 — forged registration cookie cannot create an account`, `1.1 — invite code attempts are throttled per verification`, plus all pre-existing `1.1`/`1.6` titles passing). Expected passed count = baseline + 4.

Manual adversarial gate (debug binary on its own port + `samete_u1`; `samete_u1` must be migrated):
```bash
SQLX_OFFLINE=true cargo build --features ssr
DATABASE_URL=postgres://samete:samete@localhost:5432/samete_u1 SAMETE_SMS_DRY_RUN=true LEPTOS_SITE_ADDR=127.0.0.1:3911 ./target/debug/samete > /tmp/u1-server.log 2>&1 &
SRV=$!; until curl -sf http://127.0.0.1:3911/login >/dev/null; do sleep 1; done
ACTION=$(curl -s http://127.0.0.1:3911/login | grep -o 'action="[^"]*register_with_code[^"]*"' | head -1 | cut -d'"' -f2)
echo "ACTION=$ACTION"   # REQUIRED: non-empty path
curl -s -o /dev/null -D - -X POST "http://127.0.0.1:3911$ACTION" -H 'Cookie: registration_ticket=+380671234567; pending_phone=+380671234567' --data-urlencode 'code=x-y' --data-urlencode 'name=X' | grep -i "set-cookie: session=[^;]"
psql postgres://samete:samete@localhost:5432/samete_u1 -tAc "SELECT count(*) FROM users WHERE phone='+380671234567'"
kill $SRV
```
**REQUIRED:** the `grep -i set-cookie: session=` prints nothing; psql prints `0`.

---

# U2 — Test mode structurally confined to dry-run + loopback

(Branch from main AFTER U1 and U3 are integrated.)

## Why This Matters
One stray env var in Coolify turns OTP into `000000` for every account including admin. The flag must be impossible to combine with real SMS and impossible on a network-exposed bind.

## What You Must Do

### U2.1 `src/config.rs` — TDD
Step 1 — append a test module (failing: types do not exist):
```rust
#[cfg(test)]
mod tests {
    use super::{Config, ConfigError, SmsMode, sms_mode_from_vars};

    fn config_with(sms: SmsMode) -> Config {
        Config { database_url: String::new(), sms, admin_bootstrap: None }
    }

    #[test]
    fn dry_run_without_test_mode() {
        assert_eq!(sms_mode_from_vars(Some("true"), None, None, None).ok(), Some(SmsMode::DryRun { test_mode: false }));
    }

    #[test]
    fn dry_run_with_test_mode() {
        assert_eq!(sms_mode_from_vars(Some("true"), Some("true"), None, None).ok(), Some(SmsMode::DryRun { test_mode: true }));
    }

    #[test]
    fn test_mode_without_dry_run_is_refused_even_with_credentials() {
        let result = sms_mode_from_vars(None, Some("true"), Some("tok".into()), Some("snd".into()));
        assert!(matches!(result, Err(ConfigError::TestModeRequiresDryRun)));
    }

    #[test]
    fn live_requires_token() {
        assert!(matches!(sms_mode_from_vars(None, None, None, Some("snd".into())), Err(ConfigError::MissingTurbosmsToken)));
    }

    #[test]
    fn live_rejects_empty_sender() {
        assert!(matches!(sms_mode_from_vars(None, None, Some("tok".into()), Some(String::new())), Err(ConfigError::EmptyTurbosmsSender)));
    }

    #[test]
    fn live_with_credentials() {
        assert_eq!(
            sms_mode_from_vars(None, None, Some("tok".into()), Some("snd".into())).ok(),
            Some(SmsMode::Live { token: "tok".into(), sender: "snd".into() })
        );
    }

    #[test]
    fn only_literal_true_enables_flags() {
        assert!(matches!(sms_mode_from_vars(Some("1"), Some("TRUE"), None, None), Err(ConfigError::MissingTurbosmsToken)));
    }

    #[test]
    fn test_mode_refuses_non_loopback_bind() {
        let config = config_with(SmsMode::DryRun { test_mode: true });
        assert!(matches!(config.check_bind_addr("0.0.0.0:3000".parse().expect("addr")), Err(ConfigError::TestModeRequiresLoopback(_))));
        assert!(config.check_bind_addr("127.0.0.1:3000".parse().expect("addr")).is_ok());
    }

    #[test]
    fn non_test_mode_allows_any_bind() {
        let config = config_with(SmsMode::DryRun { test_mode: false });
        assert!(config.check_bind_addr("0.0.0.0:3000".parse().expect("addr")).is_ok());
    }
}
```
(`admin_bootstrap: None` exists because U3 is integrated first. If U3's field name differs, use U3's actual name.)
Step 2 — `SQLX_OFFLINE=true cargo test --features ssr config::tests` → **REQUIRED:** compile errors (red).
Step 3 — implement. Add `#[derive(PartialEq, Eq)]`-capable types:
```rust
/// How SMS is delivered. Test mode exists only inside `DryRun`, so a server
/// that sends real SMS can never run with the fixed test OTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmsMode {
    /// Real `TurboSMS` delivery.
    Live { token: String, sender: String },
    /// SMS is logged, never sent. `test_mode` = fixed OTP `000000`, no OTP
    /// rate limits, no deadline gates (local dev / E2E only).
    DryRun { test_mode: bool },
}
```
`Config`: remove fields `turbosms_token`, `turbosms_sender`, `sms_dry_run` and the stray CSRF comment; add `pub sms: SmsMode`. `ConfigError`: add
```rust
    #[error("SAMETE_TEST_MODE=true requires SAMETE_SMS_DRY_RUN=true")]
    TestModeRequiresDryRun,
    #[error("SAMETE_TEST_MODE=true requires a loopback bind address, got {0}")]
    TestModeRequiresLoopback(std::net::SocketAddr),
```
`from_env` body after `database_url`:
```rust
        let sms = sms_mode_from_vars(
            std::env::var("SAMETE_SMS_DRY_RUN").ok().as_deref(),
            std::env::var("SAMETE_TEST_MODE").ok().as_deref(),
            std::env::var("TURBOSMS_TOKEN").ok(),
            std::env::var("TURBOSMS_SENDER").ok(),
        )?;
```
and construct `Self { database_url, sms, <U3 field> }`. Rewrite the `from_env` doc to describe both flags and both refusals (keep `# Errors`). Add:
```rust
impl Config {
    /// True only in dry-run SMS mode with `SAMETE_TEST_MODE=true`.
    #[must_use]
    pub fn test_mode(&self) -> bool {
        matches!(self.sms, SmsMode::DryRun { test_mode: true })
    }

    /// Refuse test mode on any non-loopback bind address.
    ///
    /// # Errors
    ///
    /// Returns `Err(ConfigError::TestModeRequiresLoopback)` when test mode is
    /// on and `addr` is not loopback.
    pub fn check_bind_addr(&self, addr: std::net::SocketAddr) -> Result<(), ConfigError> {
        if self.test_mode() && !addr.ip().is_loopback() {
            return Err(ConfigError::TestModeRequiresLoopback(addr));
        }
        Ok(())
    }
}

/// Parse SMS delivery mode from raw env values. Only the literal `"true"` enables a flag.
fn sms_mode_from_vars(
    dry_run: Option<&str>,
    test_mode: Option<&str>,
    token: Option<String>,
    sender: Option<String>,
) -> Result<SmsMode, ConfigError> {
    let test_mode = test_mode == Some("true");
    if dry_run == Some("true") {
        return Ok(SmsMode::DryRun { test_mode });
    }
    if test_mode {
        return Err(ConfigError::TestModeRequiresDryRun);
    }
    let token = token.ok_or(ConfigError::MissingTurbosmsToken)?;
    if token.is_empty() {
        return Err(ConfigError::EmptyTurbosmsToken);
    }
    let sender = sender.ok_or(ConfigError::MissingTurbosmsSender)?;
    if sender.is_empty() {
        return Err(ConfigError::EmptyTurbosmsSender);
    }
    Ok(SmsMode::Live { token, sender })
}
```
(Merge `check_bind_addr`/`test_mode` into the existing `impl Config` block — one impl block.)
Step 4 — rerun step-2 command: **REQUIRED:** `test result: ok.` with all 9 tests.

### U2.2 `src/sms.rs`
`send_sms`: replace `if config.sms_dry_run { … return Ok(()); }` with
```rust
    let (token, sender) = match &config.sms {
        SmsMode::DryRun { .. } => {
            tracing::info!(phone = phone, message = message, "[DRY RUN] SMS would be sent");
            return Ok(());
        }
        SmsMode::Live { token, sender } => (token.as_str(), sender.as_str()),
    };
```
JSON body `"sender": config.turbosms_sender` → `"sender": sender`. `post_to_turbosms(client, config, &body)` (both call sites) → `post_to_turbosms(client, token, &body)`; its signature `config: &Config` → `token: &str`; `.bearer_auth(&config.turbosms_token)` → `.bearer_auth(token)`. Import `use crate::config::{Config, SmsMode};`. Doc line "When `config.sms_dry_run` is true" → "In `SmsMode::DryRun`".

### U2.3 `src/auth.rs`
- `create_otp(pool, phone)` → `create_otp(pool: &PgPool, phone: &str, test_mode: bool)`; first statement `let code = if test_mode { "000000".to_owned() } else { … };`. Doc: "When `test_mode` (see `Config::test_mode`) is true, always returns `"000000"`."
- `check_otp_rate_limit(pool, phone)` → `check_otp_rate_limit(pool: &PgPool, phone: &str, test_mode: bool)`; first statement `if test_mode { return Ok(()); }`. Doc: add "Skipped entirely when `test_mode` is true."
- SQL strings unchanged.

### U2.4 `src/pages/login.rs` (`request_otp` only)
`auth::check_otp_rate_limit(&pool, &normalized)` → `auth::check_otp_rate_limit(&pool, &normalized, config.test_mode())`; `auth::create_otp(&pool, &normalized)` → `auth::create_otp(&pool, &normalized, config.test_mode())`. Comment "(test mode returns "000000")" stays.

### U2.5 `src/pages/home.rs`
Add (next to the other `#[cfg(feature = "ssr")]` helpers, before `resolve_preparation_state`):
```rust
/// Test mode (deadline gates bypassed) from the server `Config` context.
#[cfg(feature = "ssr")]
fn test_mode() -> Result<bool, ServerFnError> {
    leptos::context::use_context::<crate::config::Config>()
        .map(|config| config.test_mode())
        .ok_or_else(|| ServerFnError::new("no config in context"))
}
```
Replace each of the three lines `let test_mode = std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true");` (in `resolve_preparation_state`, `enroll_in_season`, `confirm_ready`) with `let test_mode = test_mode()?;`. `is_past_deadline` and its tests unchanged.

### U2.6 `src/main.rs`
Move these lines from step 4 to directly after step 2 (before the database step):
```rust
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    config.check_bind_addr(addr).expect("configuration error");
    if config.test_mode() {
        tracing::warn!(
            "SAMETE_TEST_MODE=true: fixed OTP 000000, OTP rate limits and deadline gates disabled (local/E2E only)"
        );
    }
```
Step 4 keeps `let leptos_options = conf.leptos_options;` and `let routes = generate_route_list(App);`.

### U2.7 Docs
`README.md` Environment Variables table — final rows (replace the whole table body; keep U3's two rows, which already exist):
```
| `DATABASE_URL` | Yes | Postgres connection string |
| `TURBOSMS_TOKEN` | Yes, unless `SAMETE_SMS_DRY_RUN=true` | TurboSMS API bearer token |
| `TURBOSMS_SENDER` | Yes, unless `SAMETE_SMS_DRY_RUN=true` | Registered alpha-name |
| `SAMETE_SMS_DRY_RUN` | No (dev/E2E only) | `true` = SMS are logged (including OTP codes), never sent. Never set in production |
| `SAMETE_TEST_MODE` | No (dev/E2E only) | `true` = fixed OTP `000000`, no OTP rate limits, no deadline gates. Boot is refused unless `SAMETE_SMS_DRY_RUN=true` AND the server binds a loopback address. Never set in production |
| `SAMETE_ADMIN_PHONE` / `SAMETE_ADMIN_NAME` | (U3 rows, unchanged) | |
| `RUST_LOG` | No | Log filter; default `samete=info,tower_http=info` |
```
Delete the `CSRF_SECRET` row (the variable does not exist; `grep -rn CSRF_SECRET src` = 0).
`.env.example` — final content (keep U3's admin lines below):
```sh
export DATABASE_URL="postgres://samete:samete@localhost/samete"
# Local dev / E2E only. SAMETE_TEST_MODE=true (fixed OTP 000000, no rate limits,
# no deadline gates) is refused at boot unless SAMETE_SMS_DRY_RUN=true and the
# server binds a loopback address. Production: set TURBOSMS_TOKEN and
# TURBOSMS_SENDER and leave both SAMETE_* flags unset.
export SAMETE_TEST_MODE=true
export SAMETE_SMS_DRY_RUN=true
```

## Verification Gates (U2)
```bash
grep -rn "SAMETE_TEST_MODE\|SAMETE_SMS_DRY_RUN" src/
```
**REQUIRED:** matches only in `src/config.rs` and `src/main.rs`.
```bash
grep -rn "std::env::var" src/ | grep -v "src/config.rs" | grep -v "SAMETE_LOG_POOL"
```
**REQUIRED:** zero matches.
```bash
grep -rn "sms_dry_run\|turbosms_token\|turbosms_sender\|CSRF_SECRET" src/ README.md
```
**REQUIRED:** zero matches.
Boot-refusal gates (no DB needed — refusal precedes DB connect):
```bash
SQLX_OFFLINE=true cargo build --features ssr
env -u TURBOSMS_TOKEN -u SAMETE_SMS_DRY_RUN DATABASE_URL=postgres://x SAMETE_TEST_MODE=true LEPTOS_SITE_ADDR=127.0.0.1:3921 ./target/debug/samete 2>&1 | grep -c "TestModeRequiresDryRun"
DATABASE_URL=postgres://x SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true LEPTOS_SITE_ADDR=0.0.0.0:3922 ./target/debug/samete 2>&1 | grep -c "TestModeRequiresLoopback"
```
**REQUIRED:** each prints `1`; both processes exit non-zero immediately.
Standard gates. E2E gate (passed count unchanged vs the post-U1/U3 baseline; proves `isolated-capture.sh` still boots test mode on 127.0.0.1).

---

# U3 — First-admin bootstrap from env

## Why This Matters
A fresh production DB has no admin; the only ways in are hand-written SQL or running the test seed (which installs a publicly known phone as admin).

## What You Must Do

### U3.1 `src/config.rs` — TDD
Step 1 — append test module (if U2 already created one — it has not; U3 precedes U2 — create it):
```rust
#[cfg(test)]
mod tests {
    use super::{AdminBootstrap, ConfigError, admin_bootstrap_from_vars};

    #[test]
    fn neither_var_means_no_bootstrap() {
        assert_eq!(admin_bootstrap_from_vars(None, None).ok(), Some(None));
    }

    #[test]
    fn blank_vars_count_as_unset() {
        assert_eq!(admin_bootstrap_from_vars(Some("  ".into()), Some(String::new())).ok(), Some(None));
    }

    #[test]
    fn both_vars_normalize_phone_and_trim_name() {
        assert_eq!(
            admin_bootstrap_from_vars(Some("067 123 45 67".into()), Some("  Організатор ".into())).ok(),
            Some(Some(AdminBootstrap { phone: "+380671234567".into(), name: "Організатор".into() }))
        );
    }

    #[test]
    fn phone_without_name_is_refused() {
        assert!(matches!(admin_bootstrap_from_vars(Some("+380671234567".into()), None), Err(ConfigError::AdminBootstrapIncomplete)));
    }

    #[test]
    fn name_without_phone_is_refused() {
        assert!(matches!(admin_bootstrap_from_vars(None, Some("Організатор".into())), Err(ConfigError::AdminBootstrapIncomplete)));
    }

    #[test]
    fn invalid_phone_is_refused() {
        assert!(matches!(admin_bootstrap_from_vars(Some("12345".into()), Some("A".into())), Err(ConfigError::InvalidAdminPhone(_))));
    }
}
```
Step 2 — `SQLX_OFFLINE=true cargo test --features ssr config::tests` → **REQUIRED:** compile errors (red).
Step 3 — implement:
```rust
/// First admin ensured at every boot (idempotent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminBootstrap {
    /// E.164-normalized phone.
    pub phone: String,
    pub name: String,
}
```
`Config`: add `pub admin_bootstrap: Option<AdminBootstrap>,`. `ConfigError`: add
```rust
    #[error("SAMETE_ADMIN_PHONE and SAMETE_ADMIN_NAME must be set together")]
    AdminBootstrapIncomplete,
    #[error("SAMETE_ADMIN_PHONE is not a valid Ukrainian phone: {0}")]
    InvalidAdminPhone(String),
```
In `from_env`, before `Ok(Self { … })`:
```rust
        let admin_bootstrap = admin_bootstrap_from_vars(
            std::env::var("SAMETE_ADMIN_PHONE").ok(),
            std::env::var("SAMETE_ADMIN_NAME").ok(),
        )?;
```
and include it in `Self { … }`. Add to `from_env` doc `# Errors`: "or if exactly one of `SAMETE_ADMIN_PHONE`/`SAMETE_ADMIN_NAME` is set, or the phone is invalid."
```rust
/// Parse the optional first-admin bootstrap. Blank values count as unset.
fn admin_bootstrap_from_vars(
    phone: Option<String>,
    name: Option<String>,
) -> Result<Option<AdminBootstrap>, ConfigError> {
    let present = |v: Option<String>| v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    match (present(phone), present(name)) {
        (None, None) => Ok(None),
        (Some(phone), Some(name)) => {
            let phone = crate::phone::normalize(&phone)
                .map_err(|_| ConfigError::InvalidAdminPhone(phone.clone()))?;
            Ok(Some(AdminBootstrap { phone, name }))
        }
        _ => Err(ConfigError::AdminBootstrapIncomplete),
    }
}
```
Step 4 — rerun: **REQUIRED:** `test result: ok.` with the 6 tests.

### U3.2 `src/db.rs`
```rust
/// Ensure the configured first admin exists: insert it, or promote the
/// existing user with that phone to admin. Idempotent; runs at every boot.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn ensure_admin(
    pool: &PgPool,
    admin: &crate::config::AdminBootstrap,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (phone, name, role, onboarded)
        VALUES ($1, $2, 'admin', true)
        ON CONFLICT (phone) DO UPDATE SET role = 'admin'
        "#,
        admin.phone,
        admin.name,
    )
    .execute(pool)
    .await?;
    Ok(())
}
```
(`onboarded = true` mirrors `seed/test_admin.sql`; existing users keep their name/onboarded/status.)

### U3.3 `src/main.rs`
Directly after the `run_migrations(...).expect("migrations failed");` statement:
```rust
    if let Some(admin) = &config.admin_bootstrap {
        samete::db::ensure_admin(&pool, admin)
            .await
            .expect("admin bootstrap failed");
        tracing::info!(phone = %admin.phone, "admin bootstrap ensured");
    }
```

### U3.4 Docs
`README.md` Environment Variables table: add two rows after `TURBOSMS_SENDER`:
```
| `SAMETE_ADMIN_PHONE` | No (set both or neither) | Phone of the first organizer. At every boot the app ensures this user exists with role `admin` (creates it, or promotes an existing account). Any Ukrainian format; stored as E.164 |
| `SAMETE_ADMIN_NAME` | No (set both or neither) | Display name used only when the admin user is created |
```
Add subsection at the end of `## Deployment`:
```markdown
### First admin

A fresh database has no organizer, and invite codes can only be created by one. Set `SAMETE_ADMIN_PHONE` and `SAMETE_ADMIN_NAME` in the Coolify environment and deploy: on boot the app creates that admin (or promotes the existing account with that phone). The operation is idempotent — leaving the variables set is safe. Then sign in at `/login` with that phone (real SMS OTP) and generate invite codes from `/admin`. Never run `seed/test_admin.sql` against production: it installs a fixed, publicly known test phone as admin.
```
`.env.example`: append
```sh
# First organizer (optional; both or neither). Ensured at every boot.
# export SAMETE_ADMIN_PHONE="+380XXXXXXXXX"
# export SAMETE_ADMIN_NAME="Організатор"
```

## Verification Gates (U3)
sqlx regeneration (new query in `db.rs`), standard gates.
Behavioral gate:
```bash
export DATABASE_URL=postgres://samete:samete@localhost:5432/samete_u3
sqlx database drop -y; sqlx database create
SQLX_OFFLINE=true cargo build --features ssr
for run in 1 2; do
  SAMETE_SMS_DRY_RUN=true SAMETE_ADMIN_PHONE="067 123 45 67" SAMETE_ADMIN_NAME="Організатор" LEPTOS_SITE_ADDR=127.0.0.1:3931 ./target/debug/samete > /tmp/u3-boot-$run.log 2>&1 &
  SRV=$!; until grep -q "listening on" /tmp/u3-boot-$run.log || ! kill -0 $SRV 2>/dev/null; do sleep 1; done
  grep -c "admin bootstrap ensured" /tmp/u3-boot-$run.log; kill $SRV; wait $SRV 2>/dev/null
done
psql "$DATABASE_URL" -tAc "SELECT phone, role, count(*) OVER () FROM users"
SAMETE_SMS_DRY_RUN=true SAMETE_ADMIN_PHONE="+380671234567" LEPTOS_SITE_ADDR=127.0.0.1:3932 ./target/debug/samete 2>&1 | grep -c AdminBootstrapIncomplete
```
**REQUIRED OUTPUT:** `1`, `1` (one per boot); psql prints exactly `+380671234567|admin|1`; last command prints `1`.
E2E gate: passed count equal to baseline (bootstrap vars unset in harness → no behavior change).

---

# U4 — Swap as position exchange, transactional, re-validated at release

## Why This Matters
Story 3.3 swap never succeeds (UNIQUE violation on the first UPDATE), and if it ever wrote, it would persist a split graph with no rollback that `advance_season` would release to participants. The E2E test is vacuous.

## What You Must Do

### U4.1 `src/assignment.rs` — TDD (pure, ungated, runs under bare `cargo test`)
Step 1 — add to `mod tests` (failing):
```rust
    // ── swap_positions / cycles_from_edges ───────────────────────────────────

    fn ring_edges(ids: &[Uuid]) -> Vec<(Uuid, Uuid)> {
        (0..ids.len()).map(|i| (ids[i], ids[(i + 1) % ids.len()])).collect()
    }

    #[test]
    fn swap_positions_preserves_single_loop_for_every_pair() {
        for n in 3..=11 {
            let ids = make_uuids(n);
            for i in 0..n {
                for j in (i + 1)..n {
                    let swapped = swap_positions(&ring_edges(&ids), ids[i], ids[j]).expect("valid swap");
                    let result = validated_cycles_from_edges(&swapped).expect("single loop");
                    assert_eq!(result.cohorts.len(), 1, "n={n} i={i} j={j}");
                    assert_eq!(result.cohorts[0].participants.len(), n);
                }
            }
        }
    }

    #[test]
    fn swap_positions_exchanges_the_two_positions() {
        let ids = make_uuids(4); // a→b→c→d→a
        let (a, b, c, d) = (ids[0], ids[1], ids[2], ids[3]);
        let mut swapped = swap_positions(&ring_edges(&ids), a, c).expect("valid swap");
        swapped.sort();
        let mut expected = vec![(c, b), (b, a), (a, d), (d, c)]; // c→b→a→d→c
        expected.sort();
        assert_eq!(swapped, expected);
    }

    #[test]
    fn swap_positions_adjacent_pair_never_self_assigns() {
        let ids = make_uuids(3);
        let swapped = swap_positions(&ring_edges(&ids), ids[0], ids[1]).expect("valid swap");
        assert!(swapped.iter().all(|(s, r)| s != r));
        assert!(validated_cycles_from_edges(&swapped).is_ok());
    }

    #[test]
    fn swap_positions_across_cohorts_keeps_both_loops() {
        let left = make_uuids(3);
        let right = make_uuids(4);
        let mut edges = ring_edges(&left);
        edges.extend(ring_edges(&right));
        let swapped = swap_positions(&edges, left[0], right[2]).expect("valid swap");
        let result = validated_cycles_from_edges(&swapped).expect("two loops");
        let mut sizes: Vec<usize> = result.cohorts.iter().map(|c| c.participants.len()).collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![3, 4]);
    }

    #[test]
    fn swap_positions_rejects_same_participant() {
        let ids = make_uuids(3);
        assert_eq!(swap_positions(&ring_edges(&ids), ids[0], ids[0]), Err(SwapError::SameParticipant));
    }

    #[test]
    fn swap_positions_rejects_unassigned_participant() {
        let ids = make_uuids(3);
        let stranger = Uuid::new_v4();
        assert_eq!(swap_positions(&ring_edges(&ids), ids[0], stranger), Err(SwapError::NotAssigned(stranger)));
    }

    #[test]
    fn cycles_from_edges_recovers_each_cohort() {
        let mut edges = ring_edges(&make_uuids(3));
        edges.extend(ring_edges(&make_uuids(5)));
        let result = cycles_from_edges(&edges).expect("decomposes");
        assert_eq!(result.cohorts.len(), 2);
    }

    #[test]
    fn cycles_from_edges_rejects_dangling_recipient() {
        let ids = make_uuids(4);
        let edges = vec![(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[3])];
        assert!(cycles_from_edges(&edges).is_err());
    }

    #[test]
    fn cycles_from_edges_rejects_duplicate_sender() {
        let ids = make_uuids(3);
        let mut edges = ring_edges(&ids);
        edges.push((ids[0], ids[2]));
        assert!(cycles_from_edges(&edges).is_err());
    }

    #[test]
    fn recipient_exchange_splits_the_loop() {
        // Regression: the old swap semantics (exchange two senders' recipients).
        let ids = make_uuids(5);
        let mut edges = ring_edges(&ids);
        edges[0].1 = ids[3];
        edges[2].1 = ids[1];
        assert!(validated_cycles_from_edges(&edges).is_err());
    }

    #[test]
    fn validated_cycles_from_edges_rejects_empty() {
        assert!(validated_cycles_from_edges(&[]).is_err());
    }
```
Step 2 — `cargo test assignment::tests 2>&1 | grep -E "error\[|test result"` → **REQUIRED:** compile errors (red).
Step 3 — implement (after `validate_cycles`, before `mod tests`; no `cfg` gate):
```rust
/// Why a position swap was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum SwapError {
    /// Both slots name the same participant.
    SameParticipant,
    /// The participant has no assignment in this set.
    NotAssigned(Uuid),
}

/// Exchange the positions of `a` and `b` in the assignment graph.
///
/// Every edge `s → r` becomes `σ(s) → σ(r)` where σ swaps `a` and `b`.
/// Relabelling preserves the cycle structure exactly: every loop stays one
/// loop of the same length, within or across cohorts.
///
/// # Errors
///
/// `SwapError::SameParticipant` if `a == b`; `SwapError::NotAssigned` if
/// either is not a sender in `edges`.
pub fn swap_positions(
    edges: &[(Uuid, Uuid)],
    a: Uuid,
    b: Uuid,
) -> Result<Vec<(Uuid, Uuid)>, SwapError> {
    if a == b {
        return Err(SwapError::SameParticipant);
    }
    for participant in [a, b] {
        if !edges.iter().any(|&(sender, _)| sender == participant) {
            return Err(SwapError::NotAssigned(participant));
        }
    }
    let sigma = |u: Uuid| {
        if u == a {
            b
        } else if u == b {
            a
        } else {
            u
        }
    };
    Ok(edges.iter().map(|&(s, r)| (sigma(s), sigma(r))).collect())
}

/// Recover the ordered cycles from sender → recipient edges.
///
/// Does not check cohort size or receiver uniqueness — see `validated_cycles_from_edges`.
///
/// # Errors
///
/// Returns `Err` if a sender appears twice, a recipient has no outgoing
/// edge, or a walk re-enters a participant before closing its loop.
pub fn cycles_from_edges(edges: &[(Uuid, Uuid)]) -> Result<AssignmentResult, String> {
    let mut next: HashMap<Uuid, Uuid> = HashMap::with_capacity(edges.len());
    for &(sender, recipient) in edges {
        if next.insert(sender, recipient).is_some() {
            return Err(format!("participant {sender} has more than one assignment"));
        }
    }
    let mut visited = std::collections::HashSet::with_capacity(edges.len());
    let mut cohorts = Vec::new();
    for &(start, _) in edges {
        if visited.contains(&start) {
            continue;
        }
        let mut participants = Vec::new();
        let mut current = start;
        loop {
            if !visited.insert(current) {
                return Err(format!("participant {current} is reached twice"));
            }
            participants.push(current);
            let Some(&recipient) = next.get(&current) else {
                return Err(format!("participant {current} has no assignment"));
            };
            if recipient == start {
                break;
            }
            current = recipient;
        }
        cohorts.push(Cycle { participants, score: 0 });
    }
    Ok(AssignmentResult { cohorts })
}

/// Recover cycles from edges and validate them as a releasable assignment set.
///
/// # Errors
///
/// Returns `Err` if `edges` is empty, does not decompose into loops, or any
/// loop fails `validate_cycles`.
pub fn validated_cycles_from_edges(edges: &[(Uuid, Uuid)]) -> Result<AssignmentResult, String> {
    if edges.is_empty() {
        return Err("no assignments".to_owned());
    }
    let result = cycles_from_edges(edges)?;
    validate_cycles(&result)?;
    Ok(result)
}
```
Step 4 — rerun step-2: **REQUIRED:** `test result: ok.` including the 11 new names.

### U4.2 `src/admin/assignments.rs`
1. `AssignmentLink`: add field `pub recipient_id: String,` (between `sender_name` and `recipient_name`). In `store_and_build_preview` add `recipient_id: recipient_id.to_string(),` to the `AssignmentLink { … }` literal.
2. Delete `validate_swap_topology` entirely.
3. Replace `swap_assignment` body (signature and `#[server(SwapAssignment)]` unchanged) with:
```rust
    use crate::{
        assignment::{self, SwapError},
        auth,
        i18n::i18n::{Locale, td_string},
        types::Phase,
    };

    let (pool, _user) = auth::require_admin().await?;

    let sid: uuid::Uuid = season_id
        .parse()
        .map_err(|_| ServerFnError::new("invalid season_id"))?;
    let sa: uuid::Uuid = sender_a
        .parse()
        .map_err(|_| ServerFnError::new("invalid sender_a"))?;
    let sb: uuid::Uuid = sender_b
        .parse()
        .map_err(|_| ServerFnError::new("invalid sender_b"))?;

    let mut tx = pool.begin().await.map_err(db_err)?;

    // Lock the season row: serializes concurrent swaps and advance_season.
    let phase = sqlx::query_scalar!(
        r#"
        SELECT phase AS "phase: Phase"
        FROM seasons
        WHERE id = $1 AND launched_at IS NOT NULL
        FOR UPDATE
        "#,
        sid,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_err)?;
    if phase != Some(Phase::Assignment) {
        return Err(ServerFnError::new(td_string!(Locale::uk, assignments_error_wrong_phase)));
    }

    let edges: Vec<(uuid::Uuid, uuid::Uuid)> = sqlx::query!(
        r#"SELECT sender_id, recipient_id FROM assignments WHERE season_id = $1"#,
        sid,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(db_err)?
    .into_iter()
    .map(|row| (row.sender_id, row.recipient_id))
    .collect();

    let swapped = assignment::swap_positions(&edges, sa, sb).map_err(|e| {
        ServerFnError::new(match e {
            SwapError::SameParticipant => td_string!(Locale::uk, assignments_error_swap_same_participant),
            SwapError::NotAssigned(id) if id == sa => td_string!(Locale::uk, assignments_error_sender_a_not_found),
            SwapError::NotAssigned(_) => td_string!(Locale::uk, assignments_error_sender_b_not_found),
        })
    })?;

    // Validate BEFORE any write. Holds by construction; checked anyway.
    assignment::validated_cycles_from_edges(&swapped).map_err(|_| {
        ServerFnError::new(td_string!(Locale::uk, assignments_error_swap_breaks_cycle))
    })?;

    // Rewrite the season's rows in one statement pair: UNIQUE(season_id,
    // sender_id) and UNIQUE(season_id, recipient_id) are non-deferrable, so
    // per-row UPDATEs would collide mid-swap. Safe to recreate rows: in the
    // Assignment phase no SMS/receipt state exists yet (notified_at,
    // receipt_* stay at defaults).
    sqlx::query!("DELETE FROM assignments WHERE season_id = $1", sid)
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
    let (senders, recipients): (Vec<uuid::Uuid>, Vec<uuid::Uuid>) = swapped.into_iter().unzip();
    sqlx::query!(
        r#"
        INSERT INTO assignments (season_id, sender_id, recipient_id)
        SELECT $1, s, r FROM UNNEST($2::uuid[], $3::uuid[]) AS t(s, r)
        "#,
        sid,
        &senders[..],
        &recipients[..],
    )
    .execute(&mut *tx)
    .await
    .map_err(db_err)?;

    tx.commit().await.map_err(db_err)?;
    Ok(())
```
(If `td_string!` arms do not unify in the `match`, append `.to_string()` to each arm.) Doc comment of `swap_assignment` becomes:
```
/// Exchange two participants' positions in the assignment cycle (admin only).
///
/// Every loop keeps its members and length (see `assignment::swap_positions`).
/// Runs in one transaction: lock season, read, compute, validate, rewrite, commit.
/// Any error rolls back; nothing partial is ever persisted.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, the season is not launched and in the
/// Assignment phase, both senders are the same, either has no assignment, or
/// the result fails validation.
```
4. `get_assignment_preview`: keep the query (append `, a.sender_id` to its `ORDER BY a.created_at` → `ORDER BY a.created_at, a.sender_id`); replace everything from `// Build chain from next_map` to the final `Ok(Some(AssignmentPreview { … }))` with:
```rust
    let by_sender: std::collections::HashMap<uuid::Uuid, &AssignmentRow> =
        assignments.iter().map(|a| (a.sender_id, a)).collect();
    let edges: Vec<(uuid::Uuid, uuid::Uuid)> = assignments
        .iter()
        .map(|a| (a.sender_id, a.recipient_id))
        .collect();
    let result = crate::assignment::cycles_from_edges(&edges).map_err(|_| {
        ServerFnError::new(td_string!(Locale::uk, season_error_assignments_invalid))
    })?;

    let cohorts = result
        .cohorts
        .iter()
        .map(|cycle| CohortPreview {
            score: cycle.score,
            chain: cycle
                .participants
                .iter()
                .filter_map(|id| by_sender.get(id))
                .map(|a| AssignmentLink {
                    sender_id: a.sender_id.to_string(),
                    sender_name: a.sender_name.clone(),
                    recipient_id: a.recipient_id.to_string(),
                    recipient_name: a.recipient_name.clone(),
                })
                .collect(),
        })
        .collect();

    Ok(Some(AssignmentPreview {
        season_id: season.id.to_string(),
        cohorts,
        phase: season.phase,
    }))
```
and change the fn's `use crate::auth;` to `use crate::{auth, i18n::i18n::{Locale, td_string}};`.

### U4.3 `src/admin/season.rs` — release guard in `advance_season`
Insert after `let next_phase = …?;` and before the `UPDATE seasons`:
```rust
    // Release guard: leaving Assignment publishes the graph to participants.
    if season.phase == Phase::Assignment {
        let edges: Vec<(uuid::Uuid, uuid::Uuid)> = sqlx::query!(
            r#"SELECT sender_id, recipient_id FROM assignments WHERE season_id = $1"#,
            season.id,
        )
        .fetch_all(&pool)
        .await
        .map_err(db_err)?
        .into_iter()
        .map(|row| (row.sender_id, row.recipient_id))
        .collect();
        crate::assignment::validated_cycles_from_edges(&edges).map_err(|_| {
            ServerFnError::new(td_string!(Locale::uk, season_error_assignments_invalid))
        })?;
    }
```
(Same SQL string as in `swap_assignment` → one `.sqlx` entry.) Add to the doc `# Errors`: "or, when leaving the Assignment phase, the stored assignments are missing or not valid loops."

### U4.4 `src/admin/page.rs` — accessible link list carrying the graph (testid contract)
In `render_cycle_ring`, inside `<figure class="cycle-viz-container" data-testid="cycle-visualization">`, directly after `</svg>` and before `</figure>`, insert:
```rust
            <ol class="sr-only" data-testid="cycle-link-list">
                {chain
                    .iter()
                    .map(|link| view! {
                        <li
                            data-testid="cycle-link"
                            data-sender-id=link.sender_id.clone()
                            data-sender-name=link.sender_name.clone()
                            data-recipient-id=link.recipient_id.clone()
                        >
                            {link.sender_name.clone()} " → " {link.recipient_name.clone()}
                        </li>
                    })
                    .collect_view()}
            </ol>
```
No `#[cfg]` tokens and no bare `>` comparisons in view! attributes. `chain` is the existing `&[AssignmentLink]` parameter. No other page.rs change (swap form keeps `season_id` hidden input and both selects).

### U4.5 `locales/uk.json`
- Add `"assignments_error_swap_same_participant": "Обери двох різних учасників.",` after `assignments_error_swap_breaks_cycle`.
- Add `"season_error_assignments_invalid": "Призначення некоректні — згенеруй їх знову.",` after `season_error_no_active_season`.
- Change value of `assignments_swap_description` to `"Обери двох учасників, щоб поміняти їх місцями в циклі."`.
- Delete key `assignments_error_broken_cycle` (only user was `validate_swap_topology`; verify `grep -rn assignments_error_broken_cycle src` = 0 before deleting).

### U4.6 E2E
POM (`end2end/tests/fixtures/mail_club_page.ts`), assignments section:
- `swapAssignment`: replace the final `await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();` and its preceding comment with:
```ts
    // The caller asserts the graph change via readCycleEdges() + expect.poll.
    await expect(this.page.getByTestId("action-error")).toBeEmpty();
```
- Add:
```ts
  /**
   * Read the assignment graph from the admin cycle visualization's link list.
   * Returns "senderId>recipientId" strings, sorted (order-independent).
   */
  async readCycleEdges(): Promise<string[]> {
    const links = this.page.getByTestId("cycle-link");
    await expect(links.first()).toBeAttached();
    const count = await links.count();
    const edges: string[] = [];
    for (let i = 0; i < count; i++) {
      const link = links.nth(i);
      edges.push(`${await link.getAttribute("data-sender-id")}>${await link.getAttribute("data-recipient-id")}`);
    }
    return edges.sort();
  }

  /** Sender UUID for a participant name, from the cycle link list. */
  async cycleSenderId(name: string): Promise<string> {
    const link = this.page.getByTestId("cycle-link").and(this.page.locator(`[data-sender-name="${name}"]`));
    await expect(link).toBeAttached();
    return (await link.getAttribute("data-sender-id")) as string;
  }
```
Spec (`end2end/tests/mail_club.spec.ts`), replace the body of `"3.3 — admin swaps two assignments"` with:
```ts
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      const before = await app.readCycleEdges();
      expect(before).toHaveLength(3);
      const idA = await app.cycleSenderId(NAMES.A);
      const idB = await app.cycleSenderId(NAMES.B);
      const sigma = (u: string) => (u === idA ? idB : u === idB ? idA : u);
      const expected = before
        .map((edge) => edge.split(">"))
        .map(([s, r]) => `${sigma(s)}>${sigma(r)}`)
        .sort();
      expect(expected).not.toEqual(before);

      await app.swapAssignment(NAMES.A, NAMES.B);

      await expect.poll(() => app.readCycleEdges()).toEqual(expected);
      // Still one loop through all three: no self-assignment.
      for (const edge of expected) {
        const [s, r] = edge.split(">");
        expect(s).not.toEqual(r);
      }
```
(For 3 participants position-exchange of A and B is the reverse orientation — `expected !== before` always holds.)
`end2end/tests/visual-audit.spec.ts`: replace the `// ── Note: admin swap form error (A40) is unreachable via UI ──` comment block (6 lines) with:
```ts
  // ── Note: admin swap form error (A40) not captured ──────────────────────────
  // Choosing the same participant in both slots is now rejected server-side
  // (assignments_error_swap_same_participant → action-error). The state is
  // reachable but not yet part of the capture set.
```

## Verification Gates (U4)
```bash
grep -rn "validate_swap_topology\|assignments_error_broken_cycle" src/ locales/
```
**REQUIRED:** zero matches.
```bash
awk '/pub async fn swap_assignment/,/^}/' src/admin/assignments.rs | grep -c "&pool)"
```
**REQUIRED:** `0` (every statement in swap runs on `&mut *tx`).
```bash
awk '/pub async fn swap_assignment/,/^}/' src/admin/assignments.rs | grep -n "validated_cycles_from_edges\|DELETE FROM assignments"
```
**REQUIRED:** the `validated_cycles_from_edges` line number is lower than the `DELETE FROM assignments` line number.
```bash
grep -n "validated_cycles_from_edges" src/admin/season.rs
```
**REQUIRED:** exactly one match, inside `advance_season`.
```bash
cargo test assignment::tests 2>&1 | grep -E "^test result"
```
**REQUIRED:** `test result: ok.` and `0 failed`; `cargo test assignment::tests -- --list` lists all 11 names from U4.1.
sqlx regeneration, standard gates, E2E gate (titles `3.3 — admin swaps two assignments`, `phase — advance assignment → delivery`, `2.3 — participant sees recipient details` pass; passed count equals baseline).

---

## Testing Decisions
- Behavior tested through public pure interfaces; DB-bound server fns are covered by E2E (project rule: DB = E2E).
- `auth`: `generate_token`, `extract_cookie` (unit). Ticket lifecycle (create/resolve/consume/attempts) — E2E (forged cookie, throttle, existing registration tests).
- `config`: `sms_mode_from_vars`, `Config::check_bind_addr`, `admin_bootstrap_from_vars` (unit, `--features ssr`; never mutate process env in tests — `unsafe_code = "forbid"` and edition 2024 make `set_var` unsafe).
- `assignment`: `swap_positions`, `cycles_from_edges`, `validated_cycles_from_edges` (unit, bare `cargo test`), exhaustive over n=3..=11 and all pairs.
- Boot behavior (U2 refusals, U3 bootstrap) — executable binary gates above.
- Runner: `cargo test` / `cargo test --features ssr`; names describe behavior in snake_case (prior art: `src/auth.rs` tests, `src/assignment.rs` tests). E2E titles trace to story numbers.

## Forbidden Patterns

### BANNED: reading identity from the client
```rust
// BANNED — any phone, user id, or role taken from a cookie/form/query value
let phone = extract_cookie(&parts, "registration_ticket"); // then used as a phone
```
The cookie value is only ever hashed and looked up.

### BANNED: signed-cookie / HMAC / secret-key schemes for U1
No new secret, no `CSRF_SECRET`, no `blake2`/`hmac` usage. Server state is the authority.

### BANNED: `let _ =` on writes in auth/registration/swap paths
```rust
let _ = tx.commit().await;          // BANNED
let _ = sqlx::query!(...).execute(&mut *tx).await; // BANNED
```

### BANNED: per-row UPDATE of `recipient_id` for swaps, or writes before validation
Also BANNED: making `UNIQUE(season_id, recipient_id)` DEFERRABLE or dropping it.

### BANNED: reading `SAMETE_TEST_MODE` / `SAMETE_SMS_DRY_RUN` outside `src/config.rs`
### BANNED: weakening the guards
No "allow test mode in prod" override var, no warning-only fallback instead of refusing boot, no `0.0.0.0` exception.

### BANNED: E2E shortcuts
`waitForTimeout`, `networkidle`, `waitForLoadState`, `force: true`, `page.evaluate`, `getByText`, `getByRole` with name, CSS-class selectors, non-retrying assertions on DOM state where a web-first/`expect.poll` form exists. Import `test`/`expect` only from `./fixtures/cached-context`.

### BANNED: environment-touching commands
`just e2e`, `just e2e-release`, `just db-reset`, `just _kill-stale`, anything on port 3000 or DB `samete`, bare `cargo sqlx prepare` without `-- --features ssr`, `git push`, `git merge` into main, commits outside your worktree.

### BANNED: lint escapes
No new `#[allow(clippy::…)]` without a one-line WHY comment. No `#[cfg]` tokens inside `view!`. No `attr:`-prefixed attributes on native elements. Every new `pub fn` returning `Result` has a `# Errors` doc section (clippy pedantic `missing_errors_doc`).

### BANNED: scope creep
No changes to: `request_otp` outcome enum, IP rate limiting, OTP hashing, Dockerfile, CI workflow, assignment generation, swap form markup, visual-audit captures (only the A40 comment text changes). Out-of-scope findings → report in DONE_WITH_CONCERNS, do not fix.

## Definition of Done (per unit; binary)
1. Every "What You Must Do" step applied; no step skipped or substituted.
2. Every unit-specific gate shows its REQUIRED OUTPUT (paste the command + output in the report).
3. Standard gates: fmt clean; both clippy runs zero warnings; both `cargo test` runs `ok`, `0 failed`.
4. `.sqlx/` regenerated with `-- --features ssr` (U1, U3, U4) and committed; `SQLX_OFFLINE=true` clippy passes.
5. E2E gate green 3 consecutive times via `scripts/isolated-capture.sh e2e_<unit> full`; new/changed titles listed as passed.
6. `git status --short` clean after commit; exactly the unit's write-set files changed (`git diff --stat main...HEAD`).
7. One-line conventional commit(s) in the worktree; report the SHA(s). Suggested messages: `fix(auth): require server-side registration ticket for self-registration` (U1), `fix(config): confine test mode to dry-run SMS and loopback bind` (U2), `feat(admin): bootstrap first admin from env at startup` (U3), `fix(assignments): make swap a transactional position exchange and guard release` (U4).
8. Report status: DONE / DONE_WITH_CONCERNS / BLOCKED with gate evidence. A gate failing twice → STOP and report BLOCKED (task, failing output, attempts).

## Global Gates (orchestrator, on main after U2 integration)
```bash
SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr -- -D warnings
cargo test && SQLX_OFFLINE=true cargo test --features ssr
bash scripts/isolated-capture.sh e2e_final full > /tmp/e2e_final.log 2>&1; echo "exit=$?"
grep -rn "pending_phone\|SAMETE_TEST_MODE" src/ | grep -v "src/config.rs\|src/main.rs"
```
**REQUIRED:** clippy clean; tests ok; `exit=0` with 0 failed (baseline + 4 passed); last grep zero matches.
