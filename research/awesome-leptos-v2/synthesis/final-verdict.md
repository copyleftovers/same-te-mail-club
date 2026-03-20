# Awesome-Leptos Ecosystem: Final Verdict

> Research basis: 3-tier methodology. 5 sonnet landscape scans + 3 sonnet source-level verifications + 3 project audits. Claims verified from Cargo.toml and source code, not READMEs.

## Executive Summary

The Leptos 0.8 ecosystem is immature for component libraries. After scanning 17 headless UI libraries, 14 form libraries, 11 data display libraries, and 20+ DX/styling tools, the conclusion is stark: **no headless component library exists that is compatible with this project's stack** (Leptos 0.8 + SSR + hydration gate pattern + Tailwind v4 + Playwright E2E). The single most promising candidate (biji-ui) has three independent blocking issues confirmed by source-level inspection. The project should continue hand-rolling its 19 components and invest in two targeted internal refactors that eliminate more boilerplate than any available library. Two utility crates (wasm-tracing, leptos_icons) are ready for adoption when their trigger conditions are met. leptos-use, already a transitive dependency, should be promoted to a direct dependency to unlock hooks for future features.

## The Component Library Answer

This was the primary research question: "Should I adopt a Tailwind-compatible headless component library?"

**No. Continue hand-rolling components.**

### Every candidate, and why it fails

**biji-ui** (26 components, Leptos 0.8, actively maintained) — The only library targeting Leptos 0.8 with real implementations. Source-level verification confirmed genuine code: focus trapping in Dialog, roving tabindex in Tabs, collision-aware positioning in Select. However, three independent blockers were found:

1. **leptos-use version conflict.** biji-ui pins `leptos-use = "0.16"`. The project has `leptos-use = "0.18.3"` via leptos_i18n. Under Cargo's 0.x semver rules, `^0.16` and `^0.18` are incompatible ranges. The build fails. This requires either a fork or waiting for upstream to update (the pin appears to be an oversight — 0.18 was available when 0.4.4 was published on 2026-03-16).

2. **No SSR guards.** Zero `#[cfg(feature = "ssr")]` guards in the entire codebase. Components call `document().body()`, `window().inner_width()`, and `set_timeout_with_handle` at the top level of component bodies — all of which panic or fail to compile under cargo-leptos's SSR compilation pass. Every biji-ui component would need to be wrapped in `#[cfg(feature = "hydrate")]` guards or the library would need a fork adding SSR safety.

3. **Static `disabled` prop.** All `disabled` props are `bool`, not `Signal<bool>` or `MaybeSignal<bool>`. The project's hydration gate pattern (`disabled=move || !hydrated.get()`) cannot be passed to biji-ui components. This breaks the E2E contract where Playwright waits for `enabled` as the hydration signal.

**RustForWeb/radix** (28 primitive packages, genuine Radix UI port, Leptos 0.8.0) — High-quality code, correct ARIA, proper composition patterns. **Archived on 2026-02-02. Read-only repository.** Never published to crates.io. Consumable only via git dependency with no maintenance guarantees. The 28 packages are low-level primitives (focus-scope, dismissable-layer, presence) — not assembled into high-level components like Dialog or Tabs.

**cloud-shuttle/radix-leptos** (claims 57+ components, Leptos 0.8.8) — **Fraudulent.** Source inspection reveals placeholder code: `aria-selected="false"` hardcoded and never updated, `handle_keydown` stubs that `prevent_default` but perform no keyboard navigation, Dialog always renders children regardless of `open` state with no focus trapping. The "1,792+ passing tests" claim is unverifiable. Open SSR feature flag bug (issue #2, unfixed since 2025-11-15) that breaks Leptos split compilation. Last commit was a mass "remediation" of compilation errors.

**leptix_primitives** (16 real components with correct ARIA) — Genuine quality. Targets **Leptos 0.6**, not 0.8. No upgrade path announced. Inactive since November 2024.

**All opensass crates** (accordion-rs, scroll-rs, input-rs) — Target Leptos 0.7. Incompatible.

**leptail, leptos_aria, ankarhem/leptos-components, thaw_snowpack, guillotine-ui** — Archived, abandoned, or non-existent. None have functional code for Leptos 0.8.

### The definitive answer

The project should **continue hand-rolling components** because:

1. The project uses zero complex interactive patterns (no modals, no dropdowns, no tabs, no date pickers, no tooltips, no toasts — confirmed absent in the component audit).
2. The existing 19 components are simple: ActionForm wrappers, status displays, admin tables. The CSS component layer (`.btn`, `.badge`, `.field`, `.data-table`) totals ~277 lines.
3. The only library with real implementations for Leptos 0.8 (biji-ui) has three independent blockers that each require forking to resolve.
4. The component surface is small enough (19 components, 10 files) that hand-rolling remains cheaper than maintaining a fork.

If the project later adds modals, dropdowns, or tooltips, re-evaluate biji-ui at that point — the leptos-use version pin and SSR guards may be fixed upstream given the library's active development pace (5 releases in March 2026 alone).

## Adoption Tiers

### Tier 1: ADOPT NOW

**leptos-use (promote to direct dependency)**

- **What it solves:** SSR-safe browser API access (`use_window`, `use_document`), media queries (`use_media_query`, `use_preferred_dark`), event utilities (`use_event_listener`, `on_click_outside`), timing (`use_debounce_fn`, `use_throttle_fn`). Already a transitive dependency via leptos_i18n — zero additional compilation cost.
- **Integration steps:**
  1. Add `leptos-use = { version = "0.18", features = ["axum"] }` to `[dependencies]` in Cargo.toml.
  2. Add `"leptos-use/ssr"` to the `ssr` feature list.
  3. Use hooks as needed — no other changes required.
- **Version to pin:** `0.18` (caret range, compatible with current 0.18.3 transitive dep).
- **Risks:** None. Already compiled. Promoting to direct dep only makes feature flags explicit.

### Tier 2: EVALUATE (conditional adoption)

**wasm-tracing 2.1.0** — Client-side tracing to browser console + DevTools performance timeline.

- **Condition:** When debugging hydration issues, client-side signal behavior, or server function errors from the browser.
- **What to test:** Add to `hydrate()` entry point with `wasm_tracing::set_as_global_default()`. Verify that `tracing::info!()` calls in WASM-compiled code appear in browser console. Verify no double-init panic if the server subscriber is already set.
- **Decision criteria:** If it produces useful output for one debugging session, keep it. If the project never needs client-side tracing, remove.
- **Dependency compatibility:** Verified — `tracing ^0.1` and `tracing-subscriber ^0.3` match the project's exact versions. `wasm-bindgen ^0.2` matches Leptos's transitive dep.
- **Integration:**
  ```toml
  [target.'cfg(target_arch = "wasm32")'.dependencies]
  wasm-tracing = "2.1"
  ```

**leptos_icons + icondata** — SVG icon components from any major icon set.

- **Condition:** When the project adds any icon to the UI.
- **What to test:** Add one icon from any set (e.g., Lucide). Verify SVG renders, Tailwind classes apply to the `<Icon>` component, SSR produces the SVG in HTML.
- **Decision criteria:** If icons are needed, this is the canonical choice. No alternative is close.
- **Version to pin:** `leptos_icons >= 0.7.1`, `icondata` latest. Feature-flag only the icon packs you use.

**leptos-struct-table** — Derive-based data tables with sorting, pagination, virtualization.

- **Condition:** When admin pages need sortable/paginated tables beyond the current static `.data-table`.
- **What to test:** Implement a custom `TableClassesProvider` using the project's existing Tailwind classes (bypass `TailwindClassesPreset` which emits v3 classes invisible to the v4 scanner). Verify SSR works with `leptos-use/ssr` feature.
- **Decision criteria:** If the custom class provider works and SSR is clean, adopt for admin data tables. If the class provider is more work than hand-rolling the table, skip.
- **Risks:** `TailwindClassesPreset` emits hardcoded Tailwind v3 class strings inside compiled crate code. The Tailwind v4 standalone scanner cannot see them. You MUST implement a custom `TableClassesProvider` or use `@source inline()` to declare the classes manually.

### Tier 3: WATCH

**biji-ui** — Re-evaluate when: (a) leptos-use dependency is updated to 0.18+, (b) SSR guards are added, (c) `disabled` prop accepts `MaybeSignal<bool>`. Track the GitHub repo — development pace is high (10 commits in 3 days as of research date). If all three blockers are resolved upstream, this becomes Tier 1 for any future complex interactive components (modals, selects, comboboxes).

**tw_merge** — Tailwind v4-aware class merging. Watch for when the project needs dynamic class composition (e.g., a shared component accepting a `class` prop for override utilities). Currently the project uses static class literals and `data-*` variant attributes — no merging needed. tw_merge is a fork of tailwind_fuse with explicit v4 support (oklch validation, v4 utility patterns in collision detection).

**leptos_form_tool** — ActionForm-compatible form builder. The `get_action_form()` path genuinely works with ActionForm (verified: inputs have `name` attributes, on-success does not prevent default, FormData flows normally). Watch for when form count or complexity grows enough that implementing a custom `FormStyle` trait (to emit the project's `.field`/`.field-label`/`.field-input`/`.field-error` classes) becomes cheaper than hand-writing field markup. At 8 forms, hand-writing is still cheaper.

**vld / vld-leptos** — Shared server/client validation. Interesting architecture (validate once, run on both targets). Watch for adoption signals — 15 total downloads as of research date. The `validate_args!` macro works inside `#[server]` functions without touching form rendering, so it is orthogonal to the ActionForm pattern.

**leptosfmt** — view! macro formatter. No Leptos runtime dependency — parses macro AST. Works with any Leptos version. Watch for when view! macro formatting becomes a team-consistency concern.

### Tier 4: SKIP

**Incompatible Leptos version (targeting 0.6 or 0.7):**
- leptix_primitives (0.6) — genuine quality but frozen at 0.6
- leptos_form (tlowerison, 0.6.5) — abandoned RC since Feb 2024
- leptos_query (nicoburniske, 0.6) — superseded by leptos-fetch for 0.8
- leptos-hotkeys (0.6) — stale since Jul 2024
- leptos_toaster (0.7) — needs version update
- accordion-rs, scroll-rs, input-rs (all 0.7.7) — opensass ecosystem, all frozen
- leptab (0.6) — table with sorting, flatlined activity
- leptos_datatable (0.6.9) — abandoned alpha
- urlap (0.7) — no documentation, 6 commits
- table-rs (0.7.7) — Leptos integration marked "TODO" in README
- leptos-infinity (0.4.8) — version 0.0.0, abandoned
- leptail (0.7-alpha) — archived with "requires complete rewrite" note
- leptos_aria (pre-0.1 git pin) — dormant since 2023
- ankarhem/leptos-components (0.4) — abandoned 2023
- leptos-tracked, leptos-signals (0.1-era) — pre-stable, abandoned
- leptos-remix-icon (0.6) — CDN-dependent, dead
- leptos_theme (0.6.5) — dead

**Archived/unmaintained:**
- RustForWeb/radix — archived 2026-02-02, never published to crates.io. Genuine quality code but no maintenance path.
- tracing-wasm (storyai) — abandoned 2021, forked as wasm-tracing
- tracing-subscriber-wasm — single release (2023), depends on gloo 0.8 which may conflict

**Fraudulent or misleading:**
- cloud-shuttle/radix-leptos — placeholder code masquerading as 57+ components. Hardcoded ARIA states, stub event handlers, unfixed SSR bug. Source code inspection directly contradicts README claims of quality and test coverage.

**Wrong architecture for this project:**
- leptos-forms-rs — signal-driven (`use_form` hook), incompatible with ActionForm mandate and Playwright E2E
- borang — signal-driven (`RwSignal<String>` fields), same incompatibility
- formidable — signal-driven (`RwSignal<T>` props), same incompatibility
- input-rs — signal-driven (`handle` signal tuple), same incompatibility
- autoform — references internal `crate::registry`, not a standalone library
- leptos-shadcn-form — likely signal-driven (shadcn/ui Form port wraps react-hook-form equivalent)

**Solves no current problem:**
- leptos-store — enterprise state management; project uses simple per-component signals + Resources
- leptos_async_signal — SSR-resolved async signals; no current use case
- leptos-animate — FLIP animations; design system specifies `120ms ease` transitions only
- leptoaster — toast notifications; project has no toasts
- leptos-darkmode — manual dark mode toggle; project uses system preference only via CSS
- leptos-routes, leptos-routable — type-safe routing; project has 7 routes with no evidence of string-path bugs
- leptos-mview — alternate view! macro; introduces non-standard syntax for the team
- leptos-unique-ids — unique ARIA IDs; project forms are not multiply-instantiated (each route renders one form)
- Stylance, turf, Stylers, Styled — CSS-in-Rust approaches that compete with Tailwind v4, not compose with it
- tailwind-rs-leptos — runtime Tailwind class generation, incompatible with standalone binary pipeline

**Not a component library (utility only):**
- floating-ui-leptos — positioning primitive (`use_floating` hook), actively maintained, useful as a building block if the project later needs tooltips/popovers but not a standalone adoption target
- leptos-floating (aonyx) — simpler alternative to floating-ui-leptos, 5 commits, 0 stars
- leptos_context_menu — CSR-only, no SSR, single purpose
- leptos-modal — no repository URL, cannot evaluate
- ankurah-virtual-scroll — framework-agnostic state machine, requires building rendering layer
- leptos_virtual_scroller — CSR-only by design

## Internal Refactors More Valuable Than Any Library

### 1. Hydration gate helper function (~30 minutes)

**Problem:** 9 instances of the same 4-line pattern:
```rust
let (hydrated, set_hydrated) = signal(false);
Effect::new(move |_| set_hydrated.set(true));
```

**No library provides this.** Confirmed: leptos-use 0.18.3 with 89 hooks has no `use_hydrated` or `use_mounted`. This is an ecosystem gap.

**Refactor:** Extract a `use_hydrated() -> ReadSignal<bool>` function in a shared module. One function, used in 9 components, eliminates 36 lines of boilerplate. Implementation is trivial:
```rust
pub fn use_hydrated() -> ReadSignal<bool> {
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| set_hydrated.set(true));
    hydrated
}
```

**Effort:** 30 minutes including updating all 9 call sites and running `just check`.

### 2. Server function context macro (~1 hour)

**Problem:** 19 server functions extract pool + request parts with 3 lines each (57 lines total). 13 also check admin role (39 additional lines).

**Refactor:** A small helper macro or function:
```rust
fn server_ctx() -> Result<(PgPool, Parts), ServerFnError> {
    let pool = expect_context::<PgPool>();
    let parts = expect_context::<Parts>();
    Ok((pool, parts))
}
```

And an admin variant that also verifies role. Eliminates ~96 lines across 19 functions.

**Effort:** 1 hour including updating all call sites.

### 3. Form field component (~2 hours, optional)

**Problem:** 8 forms with ~12 lines each of label + input + error + ARIA markup (~96 lines).

**Refactor:** A `<FormField>` Leptos component that accepts `name`, `label`, `field_type`, `error_signal`, and `testid` props, emitting the standard `.field` / `.field-label` / `.field-input` / `.field-error` markup with correct ARIA wiring. This is what `leptos_form_tool` provides via `FormStyle` trait, but implementing it internally means zero dependency, zero custom trait, and perfect alignment with the existing CSS design system.

**Effort:** 2 hours. More speculative than the first two — the 8 forms have enough variation (phone input, OTP input, textarea, select) that a generic component may not cover all cases cleanly. Evaluate after the first two refactors.

## Ecosystem Gaps

These are needs that **nothing** in the Leptos ecosystem addresses:

1. **Hydration lifecycle hook.** No `use_hydrated()` or `use_mounted()` exists in any library. The 4-line `signal(false) + Effect::new` pattern is the only approach. This is the single most common piece of boilerplate in Leptos SSR applications and no one has extracted it.

2. **Structured field-level error transport from ServerFnError.** Every form library either handles errors internally (signal-driven) or leaves error display to the developer. No library maps `ServerFnError` back to specific form fields in a type-safe way. The project's current approach (display the error string from `action.value()`) is the ecosystem standard.

3. **ActionForm-native form generation.** Of 14 form libraries surveyed, exactly one (`leptos_form_tool`) supports ActionForm's FormData submission path. The rest all use signal-driven state, which is fundamentally incompatible with the Leptos SSR idiom that Playwright can reliably test. The ecosystem overwhelmingly follows the React/hook pattern rather than Leptos's progressive-enhancement-first ActionForm pattern.

4. **SSR-safe headless components.** biji-ui is the closest and it has no SSR guards. RustForWeb/radix is archived. Every other option is either dead, on the wrong Leptos version, or fraudulent. Building production-quality headless components (dialog, select, combobox) with correct ARIA, focus management, and SSR safety remains a hand-roll or fork task.

5. **Tailwind v4 class scanning for crate-embedded classes.** Any library that emits Tailwind utility strings from inside its compiled code (leptos-struct-table's `TailwindClassesPreset`, Leptodon's preset) is invisible to the v4 standalone scanner's `@source` directive. The workaround (`@source inline()` or custom class provider) is manual. No tooling bridges this gap.

## Research Corrections

Cases where source-level verification (Tier 2) overrode surface-level claims (Tier 1 or earlier haiku-level research):

1. **cloud-shuttle/radix-leptos is not a Radix UI port.** Earlier research treated the README's claims of "57+ components" and "1,792+ tests" at face value. Source inspection of `tabs.rs` revealed hardcoded `aria-selected="false"` that is never updated, `handle_keydown` stubs that prevent default but navigate nowhere. The Dialog always renders children regardless of `open` state. This is scaffold code, not a component library.

2. **biji-ui is not SSR-safe.** Tier 1 noted "needs verification for SSR usage." Tier 2 confirmed: zero `#[cfg(feature = "ssr")]` guards in the entire codebase. Direct `document()`, `window()`, and `set_timeout_with_handle` calls at component body top level will fail the SSR compilation pass. This was a critical upgrade from "needs verification" to "confirmed broken."

3. **biji-ui's `disabled` prop is static, not reactive.** Tier 1 did not flag this. Tier 2 source inspection confirmed all `disabled` props are `bool`, not `Signal<bool>`. This makes the project's hydration gate pattern (`disabled=move || !hydrated.get()`) incompatible without forking.

4. **wasm-tracing is NOT a fork of tracing-subscriber-wasm.** Earlier research conflated the two. wasm-tracing forks `tracing-wasm` (abandoned 2021). `tracing-subscriber-wasm` is a separate, independent crate using a different integration approach (`MakeWriter` vs `Layer`). They are architecturally different and not substitutable.

5. **leptos_form_tool has two submission paths with different safety profiles.** Tier 1 characterized it as "ActionForm compatible." Tier 2 source inspection revealed: `get_action_form()` genuinely uses ActionForm's FormData path (safe), but `get_form()` uses signal-driven `action.dispatch()` (unsafe for Playwright). The distinction is critical — only one of the two paths is compatible.

6. **formidable is signal-driven despite i18n integration claims.** Tier 1 highlighted formidable's leptos_i18n integration as a potential match. Tier 2 confirmed `RwSignal<T>` props and `Callback<Result<T, FormError>>` submission — fundamentally incompatible with ActionForm regardless of i18n features.

## Confidence Assessment

### Tier 1: ADOPT NOW

**leptos-use (direct dependency):** Confidence: **very high**. Already compiled as a transitive dependency. Zero risk. The only question is which feature flags to enable — start with `features = ["axum"]` and add individual hook features as needed.

### Tier 2: EVALUATE

**wasm-tracing:** Confidence: **high** that it will work as documented. Dependency versions verified compatible. Risk: the 3.0.0-alpha on main suggests a breaking change may land — pin to `2.1` explicitly. What could change: if Leptos 0.9 changes the hydrate entry point, the initialization code would need updating.

**leptos_icons:** Confidence: **high**. 136k downloads, targets `>=0.8.3`, inline SVG approach has no SSR complications. What could change: nothing likely — icon rendering is stable territory.

**leptos-struct-table:** Confidence: **medium**. The library works, but the Tailwind v4 scanner issue is real and the custom `TableClassesProvider` implementation is untested for this project. The POC must verify: (a) custom class provider emits classes the v4 scanner finds, (b) SSR renders correctly with `leptos-use/ssr`, (c) sorting/pagination work with the project's server functions.

### Tier 3: WATCH

**biji-ui:** Confidence that blockers will be resolved: **medium**. The development pace is high (sole maintainer @jonsaw shipping daily), but the leptos-use pin and missing SSR guards suggest the library is primarily used in CSR-only contexts. The `disabled: bool` prop type is a design decision that may require persuasion to change upstream. Monitor monthly.

**leptos_form_tool:** Confidence that `get_action_form()` path works: **high** (source-verified). Confidence that implementing a custom `FormStyle` is worth the effort at 8 forms: **low**. The crossover point is likely 15+ forms or when forms share enough structure to amortize the trait implementation.

**tw_merge:** Confidence in v4 support: **high** (source-verified oklch validation, v4 utility patterns). Confidence the project needs it: **low**. All current class strings are static. Watch for when a shared component with a `class` override prop is introduced.
