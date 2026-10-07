# Spec Review: worktree-agent-a9394f7e60a701d45

Verdict: PASS
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a9394f7e60a701d45
Branch: worktree-agent-a9394f7e60a701d45
HEAD SHA: 3adddfc862a75862fce878a05821935a60f48b72
Reviewed at: 2026-10-07T10:37:23Z (re-review; round 1 at 10:33:12Z on c82b4eb)
Unit: ENV (PLAN-blockers.md § ENV, lines 139-379) + "ENV addendum (2026-10-07, PRB-102)" (line 50)
Diff: `git diff ae111c8..3adddfc` (ENV commits 799e247, a5fed31, c82b4eb, 3adddfc; 1ce283e = base merge only)
Files reviewed:
- scripts/bootstrap-toolchain.sh
- .claude/settings.json
- .claude/hooks/session-start.sh
- .gitignore
- justfile
- scripts/isolated-capture.sh
- end2end/package.json
- end2end/package-lock.json
- .github/workflows/ci.yml
- README.md
- rust-toolchain.toml
- src/lib.rs
- src/pages/login.rs

## Findings

## Re-review (round 2, 3adddfc)

PASS -- Spec compliant. Verified all ENV.1-ENV.9 requirements plus the PRB-102 addendum in code and by gates.

What 3adddfc changed, checked with `git diff c82b4eb..3adddfc`. Only scripts/bootstrap-toolchain.sh changed (+17/-4):
- `TAILWIND_SHA256_MACOS_ARM64`/`_X64` were added at :16-17. Both are byte-identical to the hashes I computed independently in round 1.
- The `Darwin-arm64` and `Darwin-x86_64` case arms were restored at :59-60, matching the ENV.1 template.
- A `sha256_matches` helper was added at :22-28. It uses `sha256sum` when present, else `shasum -a 256`, and both the existence check and the post-download verify call it.

Helper evidence (I sourced the function and ran it):
- sha256sum branch: correct hash returns 0, wrong hash returns 1.
- shasum branch (PATH with no `sha256sum`, confirmed `no-sha256sum`): correct hash returns 0, wrong hash returns 1.

Static gate re-run: `no-placeholders`; both package files pin `"1.58.2"`; `tracked-ok` ×2; `still-ignored`; `json-ok`; `yaml-ok`; `syntax-ok`; the script is still mode 755.

Idempotence re-run: run1=0 and run2=0. Run 2 printed `present` for all 7 tools (cargo-leptos 0.3.7, sqlx-cli 0.8.6, just 1.58.0, cargo-audit 0.22.2, wasm-opt 0.116.1, tailwindcss v4.2.1, playwright 1208), with `installing_count=0`. The worktree is clean.

The Linux path did not change in behavior (same asset and sha, verified via the helper). No Rust, CI or other file changed, so the round-1 clippy, test, build and clean-shell results below still hold for 3adddfc. The round-1 Missing finding is RESOLVED. No new Missing, Partial, Extra or Misinterpreted findings.

## Round 1 record (c82b4eb, FAIL, superseded by round 2)

FAIL -- Issues found:

Missing:
- Decisions "Tailwind" ("on macOS the matching `macos-arm64`/`macos-x64` asset with its own pinned sha256"), the ENV.1 template (`TAILWIND_SHA256_MACOS_ARM64`/`_X64` + two `Darwin-*` case arms) and the obligation "Replace each `<…>` placeholder with the real value… A placeholder left in = FAIL". The implementer deleted the placeholders instead of filling them. scripts/bootstrap-toolchain.sh:15 defines only `TAILWIND_SHA256_LINUX_X64`. The case at :45-48 has only `Linux-x86_64`, so `*)` FATALs on every Mac. The plan states "Local macOS developers run it too", and the project owner develops on macOS (conventions: macOS linker, brew/Docker notes). As shipped, `just bootstrap` / the README Setup step aborts on the owner's machine. The pins were obtainable here. I fetched them through the proxy (`curl -fsSL …/v4.2.1/tailwindcss-<asset> | sha256sum`):
  - macos-arm64: `e510af7928750c9ee8d5ff2e5e98088bd5b99a8a8e2c554668621c7e151fa91f`
  - macos-x64:   `019e5cfa441992ede2772c6faaeb8d7fb1726aab50b1138c0aa38e88f4b7bd44`
  Fix: restore the two variables and the `Darwin-arm64` / `Darwin-x86_64` case arms exactly as in the ENV.1 template. Note that macOS has no `sha256sum` by default (it has `shasum -a 256`). The script calls `sha256sum -c` at :49/:54, so on Darwin the existence check silently fails and the post-download verify FATALs. The fix must use a portable checksum: `sha256sum` when present, else `shasum -a 256`. Otherwise the restored arms are dead on arrival. This is a plan-template defect, but the plan requires the macOS path to work.

Partial: none.

Extra: none. These deviations are justified and not scope creep:
- justfile:10-12 puts `bootstrap:` after `default:`. ENV.5's literal placement would make `bootstrap` the first recipe, so bare `just` would run the bootstrap instead of `just --list`. Keeping `default` first preserves existing behavior. `export PATH` is directly after the header (:3-4), as specified.
- bootstrap :58-59 adds an `else log "tailwindcss … present"` line. The idempotence gate needs it ("prints `present` for every pinned tool").
- src/lib.rs:7 includes `unknown_lints`. The pinned 1.97.1 clippy predates `unused_async_trait_impl`, so without it the allow is itself a `-D warnings` error on the pin.

Misinterpreted: none.

## Gate evidence (all run by me, not taken from the report)

Static (ENV gate): `no-placeholders`; package.json:11 and package-lock.json:12 both `"1.58.2"`, and the lock diff touches only the root `packages[""]` line; `tracked-ok` ×2; `still-ignored`; `json-ok`; `yaml-ok`; `syntax-ok`. Both scripts are mode 755. `docker compose up -d db` matches docker-compose.yml service `db`. The wasm-opt probe is the dedicated `version 116` line (:36-41), as obliged. cargo-audit pin 0.22.2 = `cargo binstall --dry-run cargo-audit` resolution today, and it is the same pin in CI (ci.yml:47).

Idempotence: two runs, run1=0 and run2=0. Run 2 printed `present` for cargo-leptos 0.3.7, sqlx-cli 0.8.6, just 1.58.0, cargo-audit 0.22.2, wasm-opt 0.116.1, tailwindcss v4.2.1 and playwright chromium-headless-shell 1208, with `grep -c installing` = 0.

Addendum:
- rust-toolchain.toml = 1.97.1 + clippy, rustfmt + wasm32. That is the exact version CI last resolved: readiness/l-toolchain.md:29 records the CI log header `rustc 1.97.1 (8bab26f4f 2026-07-14)`. Both CI jobs use `rustup toolchain install` (ci.yml:20-21, 81-82), and dtolnay is gone. Locally, `rustup toolchain install` (rustup 1.29.1) with no args installed the toml toolchain with its components and target. It has not been exercised in CI.
- login.rs:615 and :634 `.ok().is_some_and` → `.is_ok_and`. Grep shows no remaining sites.
- `cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings`: pin 1.97.1 exit 0; stable 1.99.0 exit 0.
- Span proof (1.99.0 with the allow temporarily removed, reverted after): exit 101, 30× `error: unused async for async trait impl function with no .await statements`. All 30 spans point at a `#[server]` attribute line (`26 | #[server]` / `^^^^^^^^^`), and all 30 carry "this error originates in the attribute macro `server`". The crate has 0 hand-written async trait impls (`grep impl…for` with async = 0), so the crate-level allow masks only generated code today. The condition for a crate-level `#![allow]` is met, and lib.rs:4-7 has the WHY comment.

Standard gates (pin unless noted): `cargo fmt --check` 0. `cargo clippy --no-default-features --features ssr -- -D warnings` 0 on 1.97.1 and 0 on 1.99.0. `cargo test` 0 (79 passed). `cargo test --features ssr` 0 (92 passed, 2 ignored). SSR + wasm release build succeeded inside the clean-shell gate below. That needed the plan-sanctioned RUSTFLAGS recursion_limit shim, because no T0 branch exists (`git branch -a --list '*t0*'` is empty).

Clean-shell acceptance gate, run with `env -i` plus a login shell and the hook (`CLAUDE_CODE_REMOTE=true … session-start.sh`, then `source` of the env file). I adapted it in two ways:
- (a) The dispatch forbids `samete`, so instead of `just e2e-release` I ran `bash scripts/isolated-capture.sh envspec full`: sibling DB, free port, same Playwright suite.
- (b) A concurrent lane (U1) was mid-capture, so instead of moving the shared `/root/.cache/cargo-leptos` I pointed `XDG_CACHE_HOME` at an empty scratch dir. That is a stronger proof that no cached tailwind is used, and it does not disturb other lanes.

Result: exit=0; `command -v tailwindcss` = `<worktree>/.tools/bin/tailwindcss`; rustc 1.97.1 from the toml; `playwright ran: expected=116 unexpected=0 flaky=0 skipped=2`; no FATAL; no `tailwindcss-linux-x64-musl` line. The fresh cargo-leptos cache afterwards held only `wasm-bindgen-0.2.114` and no tailwind, which proves cargo-leptos used the PATH binary. The sibling DB was dropped by the trap, and the worktree is clean at c82b4eb.

## Reasoning

I read the ENV section and the addendum line by line and compared each ENV.1-ENV.9 block against the diff. All but one match the plan text, either verbatim or with a justified deviation (listed under Extra). I re-ran every gate myself instead of trusting the report. That includes the SSR build and `cargo test`, which the implementer had not run. I also ran a stricter clean-shell gate than the implementer did, because they had not isolated the cargo-leptos cache.

The addendum is fully met. The pin version traces to primary CI evidence. Both wasm clippy targets are green. The crate-level allow is backed by span output: every hit originates in `#[server]` and points at the attribute. No narrower fix inside the crate's own code exists.

The single failure is explicit plan text: macOS tailwind pins, with "a placeholder left in = FAIL" plus a stated macOS-developer use case. Deleting the placeholders defeats the requirement rather than satisfying it, and it makes bootstrap FATAL on the platform the owner uses. The values are trivially obtainable (supplied above), so this was not a blocked requirement. The fix is small: two variables, two case arms, and a portable `sha256sum`/`shasum -a 256` check. After that, only the static and idempotence gates need re-running; Linux behavior is unchanged.

Re-rooted: 0 paths (verdict path exempt per _spec-reviewer.md).
