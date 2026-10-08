# Spec Review: worktree-agent-ac57401dbc39b3668

Verdict: PASS
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-ac57401dbc39b3668
Branch: worktree-agent-ac57401dbc39b3668
HEAD SHA: 107cc23ea3ca4cd4782e18511e6c4385b3200f59
Diff: ff9e2e4 (merge-base with claude/loving-johnson-7l8hn5)..107cc23 (commits f8e718a, merge 85145c3, fmt 107cc23)
Reviewed at: 2026-10-08T07:52Z
Files reviewed:
- README.md
- src/admin/sms.rs
- src/config.rs
- /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-product.md (P-S section, D5, Base recipes)
- /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/PROBLEMS.md (PRB-101)
- worktree-agent-ad7158d9abfc86219:locales/uk.json (P-COPY sms_* bodies)

## Findings

PASS -- Spec compliant. Verified all 9 requirements in code and by running the gates.

1. S.1 tests: the 5 named tests are present verbatim (fmt-wrapped), with the specified imports widened to keep U3's `AdminBootstrap`/`admin_bootstrap_from_vars`.
2. S.1 impl: `DRY_RUN_SITE_URL`, `site_url_from_var`, `ConfigError::{MissingSiteUrl, InvalidSiteUrl}` and the `Config.site_url` last field match the spec text exactly. `from_env` computes `site_url` from `!sms_dry_run` after `sms_dry_run`, and the doc sentence is added.
3. S.2: `with_site_link` (`#[cfg(any(feature = "ssr", test))]`) sits above `// ── Server functions`, and its test is present.
4. S.2 call sites: `grep -c "let message = with_site_link"` = 4. Exactly 4 `send_sms(` lines (206/301/396/486), each ending `&message)`. The confirm-nudge site uses `&format!("{prefix}{deadline_str}.")` as specified.
5. S.3 README: the `SAMETE_SITE_URL` row is directly after `TURBOSMS_SENDER`, with the text verbatim.
6. `SAMETE_SITE_URL` appears in `src/` only in `src/config.rs`.
7. Boot gate: `SQLX_OFFLINE=true cargo build --features ssr` ok. With dry-run, test mode and site URL unset, `./target/debug/samete` exits 101 immediately, and the output contains `MissingSiteUrl` once.
8. Unit-test deltas: bare 80 (only new test is `admin::sms::tests::site_link_follows_body_after_a_space` = +1). ssr 103 passed / 2 ignored (+1 sms + 5 `config::tests::site_url_*` = +6). CI clippy lanes pass with `-D warnings` (`--features ssr --no-default-features` and wasm32 hydrate). `cargo fmt --check` is clean.
9. E2E ×3 (isolated harness, sibling DBs samete_psrev1..3, `SAMETE_SITE_URL` unset):
   - run 1: 121 passed / 2 skipped / 0 failed.
   - runs 2 and 3: 118 passed / 1 failed / 2 skipped / 2 did not run.
   - Both failures are `visual-audit.spec.ts:798` "capture admin — mixed participant statuses (after deactivate)". It is a strict-mode violation: `getByTestId('inactive-status')` resolved to 2 elements (rows "Аудит Учасник В" and "Деактивований Учасник"), raised at `MailClubPage.deactivateParticipant` (mail_club_page.ts:478, the page-wide unscoped wait).
   - This is exactly PRB-101, the same symptom and mechanism it describes. P-S touches no `end2end/` file and no participant/admin rendering, so it cannot cause this.
   - Delta 0 vs base. The dry-run default boots; run 1 is fully green.
   - Dry-run SMS logs show the link appended, e.g. "…реєструйся. http://127.0.0.1:3000" and "Нагадуємо: підтверди готовність листа до 29 жовтня 2026, 02:53. http://127.0.0.1:3000".

Cross-checks requested by dispatch:
- U3 conflict resolution: `git diff ff9e2e4..107cc23 -- src/config.rs` contains only additions plus one removed line, the `use super::{…}` import, which was widened, not dropped. The full U3 content is retained unchanged: the `admin_bootstrap` field, `AdminBootstrap`, `AdminBootstrapIncomplete`/`InvalidAdminPhone`, the `admin_bootstrap_from_vars` call and fn, the U3 doc lines and the U3 tests. In README, U3's `SAMETE_ADMIN_PHONE`/`SAMETE_ADMIN_NAME` rows are intact and follow the new row.
- P-COPY join: the reviewed P-COPY bodies end "…Реєструйся тут:", "…кому надсилати:" and "…Підтверди тут:". `format!("{body} {site_url}")` yields "тут: https://…", with a single space and no newline, which reads correctly. The confirm-nudge prefix is unchanged by P-COPY and joins as "…<date>. https://…".

Extra: none.
Missing / Partial / Misinterpreted: none.

Observation (not P-S, not a finding): `cargo clippy --all-targets --features ssr` fails on pre-existing code outside the write-set (`src/auth.rs:468` items_after_test_module, `src/admin/page.rs:2415` assertions_on_constants). Neither CI nor `just clippy` uses `--all-targets`.

## Reasoning

I read the P-S section and its D5 rationale in full, then read the merge-base diff, not main..HEAD and not the report. I compared each code block in S.1–S.3 token by token against the plan. The only differences are rustfmt line-wrapping (commit 107cc23) and the test import widened to coexist with U3. No unrequested code, fields or abstractions appear.

The ambiguous part was the conflict merge 85145c3. I diffed config.rs against the merge-base, which already contains U3 (commits 1bc1211/067b9bd). The only `-` line is the import, so nothing of U3 was lost. I also diffed f8e718a against 107cc23 to see what the merge brought in; it was exactly U3's additions alongside P-S's.

I ran every gate myself rather than trusting the report. That covers unit tests in both modes with per-test names to attribute the +1/+6 deltas, both CI clippy lanes, fmt, the debug build, the boot gate, and three full E2E runs on isolated sibling DBs. The implementer's "alternating" E2E pattern reproduced: one green run, then two red runs. I did not accept the PRB-101 label on trust. I compared the actual failure text to the PRB-101 statement: the same test, the same unscoped `inactive-status` wait in `deactivateParticipant`, and 2 matched rows, one of them the participant deactivated earlier by mail_club.spec. They match exactly, and P-S's write-set is disjoint from that code path.

Cleanup: the harness dropped all three sibling DBs (verified none remain). The worktree `target/` predates this review (mtime 2026-10-07 21:22, the implementer's build) and was reused, not created by me, so it was not deleted. No files in the worktree were edited and no commits were made.
