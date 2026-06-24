Flushed 2026-06-20. Nothing deferred.

## Added 2026-06-22

### Dev-mode WASM hydration intermittent failure
- **Severity:** Medium (mitigated by release-binary E2E, but dev-mode debugging harder)
- **Detail:** 14MB dev WASM intermittently fails to call `hydrate()`. Buttons stay disabled permanently. Affects ~40% of dev-mode E2E runs. No server errors, no WASM fetch failures — client-side init failure. Investigated: cached-context encoding, POM waits, goHome hydration probes, reload fallbacks. None solved it.
- **Mitigation:** `just e2e` now runs release binary (471KB WASM). `just e2e-dev` preserved for manual debugging.
- **Rationale for deferral:** Release binary is stable (3/3 greens). Root-causing the dev WASM init failure requires Leptos/wasm-bindgen level investigation beyond component work scope.
