Flushed 2026-06-20. Nothing deferred.

## Added 2026-06-22

### Dev-mode WASM hydration intermittent failure
- **Severity:** Medium (mitigated by release-binary E2E, but dev-mode debugging harder)
- **Detail:** 14MB dev WASM intermittently fails to call `hydrate()`. Buttons stay disabled permanently. Affects ~40% of dev-mode E2E runs. No server errors, no WASM fetch failures — client-side init failure. Investigated: cached-context encoding, POM waits, goHome hydration probes, reload fallbacks. None solved it.
- **Mitigation:** `just e2e` now runs release binary (471KB WASM). `just e2e-dev` preserved for manual debugging.
- **Rationale for deferral:** Release binary is stable (3/3 greens). Root-causing the dev WASM init failure requires Leptos/wasm-bindgen level investigation beyond component work scope.

## Added 2026-06-24

### Unreviewed commits on main
- **Severity:** High (process violation)
- **Detail:** 5 commits integrated without spec/quality review chain: b2a88cd, 8bdd7a1, ddac0bd, 543bbbc, 72d029a. Covers TurboSMS field fix, error prefix strip, onboarding redirect (2 commits), enrollment address form + POM.
- **Action:** Run post-hoc review chain on diff range b2a88cd~1..72d029a before next push.

### CSS layout appears broken despite classes being present
- **Severity:** High (user-facing)
- **Detail:** User reports layouts broken everywhere. CSS investigation confirmed all 33 component classes compile and serve at correct file. Deep investigation (checking whether classes are applied to HTML elements) was in progress when session ended.
- **Action:** Complete the deep investigation. Check SSR HTML output for class application. Check if Rust components actually use the component classes vs raw utilities.

### E2E suite not re-run after enrollment fix
- **Severity:** Medium
- **Detail:** Commits 543bbbc (address form) and 72d029a (POM update) not verified with E2E. 3 consecutive greens needed.
- **Action:** Run `just e2e` 3x before pushing.

### IP-based rate limiting absent
- **Severity:** Low (per-phone limits exist)
- **Detail:** No IP-based rate limiting on OTP requests. Per-phone limits (1/60s, 5/hr) mitigate but don't prevent distributed attacks across phones.
- **Rationale for deferral:** Architectural — needs middleware or external store. Different category from the quick fixes addressed this session.
