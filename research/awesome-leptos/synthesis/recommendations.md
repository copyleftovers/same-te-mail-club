# Awesome-Leptos Ecosystem: Final Adoption Recommendations

> Research basis: 21 reports, 98 awesome-list entries, 400+ crates.io entries examined, 5 production project inspections, 8 deep technical verifications, 3 project audits.

---

## Executive Summary

Approximately 5% of the Leptos ecosystem is relevant to this project in its current state. The project is well-architected and production-ready; the question is not "what do we need to fix" but "what targeted additions provide net value." The headline answer: adopt exactly one thing now (tracing-subscriber-wasm for dev DX), note that leptos_i18n is already correctly adopted, treat leptosfmt as an optional polish item, and plan three internal refactors that eliminate more pain than any external library. On the component library question — the definitive answer is no for the current scope. The project's hand-rolled Tailwind v4 design system with oklch tokens and data-attribute CSS variants is superior to anything the ecosystem currently offers for this specific stack combination. The only headless library worth a future POC is radix-leptos, but only when new complex interactive components (dialogs, comboboxes, calendars) are added to the product.

---

## Project Context (from audits)

**Stack:** Leptos 0.8.17, Axum 0.8.8, SQLx 0.8.6, Tailwind v4 standalone via cargo-leptos (no Node.js), PostgreSQL.

**Component surface:** 19 Leptos components across 10 files, 10 use ActionForm, 26 server functions, 7 routes, 56 E2E tests (serial, all passing).

**Design system:** Single `style/tailwind.css` with oklch palette, two-tier token system (@theme raw + :root semantic), data-attribute CSS variants, no @apply, no hardcoded colors.

**i18n:** leptos_i18n 0.6.1 already integrated, 134 Ukrainian translation keys, all UI strings externalized, zero inline Cyrillic in production code.

**Pain points ranked by actual impact:**
1. Hydration gate boilerplate: 9 instances x 4 lines = 36 lines of identical signal + Effect code
2. Form field markup repetition: 8 forms x ~12 lines of div/label/input/error-div = ~96 lines
3. Action error display: 3 diverging patterns across pages (per-form signal, global merged, structured response)
4. Context access boilerplate: 19 server functions x 3 lines of pool + parts extraction = 57 lines
5. Admin role check duplication: 13 server functions manually check `user.role != UserRole::Admin`

None of these require external libraries. All are addressable with internal refactors in under a day.

---

## Adoption Tiers

### Tier 1: ADOPT NOW

#### tracing-subscriber-wasm 0.1.0

**What it solves:** The browser WASM bundle currently has zero tracing instrumentation. Server logs events; the client is silent. This closes the gap by routing `tracing` calls to `console.log/warn/error` in the browser DevTools.

**Verified compatibility:** tracing 0.1 and tracing-subscriber 0.3 are exact version matches with what the project already uses. gloo 0.8 (the only new transitive dep) is standard WASM tooling with 2.4M downloads. The crate itself is 296 lines of Rust.

**Integration effort:** 8 lines added to the `hydrate()` function in `src/lib.rs`, two lines to Cargo.toml. Total: ~5 minutes.

**Risks:** The crate has not been updated since January 2023 (single release). This is a feature-completeness signal, not abandonment — the adapter has no moving parts to break. 175k downloads validates production use.

**Integration steps:**
```toml
# Cargo.toml, under [dependencies] (no feature gate needed; crate is WASM-safe and SSR-safe)
tracing-subscriber-wasm = "0.1"
```
```rust
// src/lib.rs, inside the hydrate() function, before leptos::mount::hydrate_body(App)
use tracing_subscriber::fmt;
use tracing_subscriber_wasm::MakeConsoleWriter;
fmt()
    .with_writer(MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG))
    .without_time()  // Required: avoids runtime error in WASM
    .init();
```

The `.without_time()` call is not optional — tracing's time formatting causes a panic in WASM. The `.map_trace_level_to(DEBUG)` suppresses extremely verbose internal Leptos TRACE logs while keeping INFO/WARN/ERROR visible.

---

#### leptos_i18n 0.6.1 (already adopted — verify quarterly)

**Status:** Production-verified across all 6 implementation phases. Zero issues. No action required. The `#[allow(deprecated)]` on `load_locales!()` is acceptable — migration to build.rs is a 1.5-hour task with no urgency.

**Future note:** If English is added as a second locale, budget 10-22 hours (translation + UI toggle + E2E update). The infrastructure supports it with minimal structural changes.

---

### Tier 2: EVALUATE (needs POC)

#### leptosfmt 0.1.33

**What it might solve:** Consistent formatting of `view!` macro content. Currently rustfmt handles Rust code but the view! contents rely on editor conventions.

**POC must test:** Whether it formats the project's existing components without mangling the hydration gate patterns, inline data-testid attributes, or the `<For>` loop structures. Known limitation: does not support non-doc comments in code blocks.

**Decision criteria:** Run against 3 files (`src/pages/home.rs`, `src/admin/season.rs`, `src/admin/assignments.rs`). If output is cleaner and CI-safe, adopt. If it reformats valid patterns incorrectly, skip.

**Effort:** 30 minutes for POC. If adopted: add `leptosfmt-action` to CI, install VSCode extension conditionally.

---

#### radix-leptos 0.9.0 (FOR FUTURE PHASES ONLY)

**What it might solve:** If the product roadmap adds complex interactive components — Dialog, Combobox, DatePicker, Calendar, MultiSelect — radix-leptos provides these as headless, accessible Leptos 0.8-compatible primitives with WCAG 2.1 AA compliance and 1,792+ tests.

**What the POC must test specifically:**
1. Does a `<Button>` inside an `<ActionForm>` correctly dispatch the server function POST? (The deep-dive found the `disabled` prop is a static bool, not a signal — the hydration gate `disabled=move || !hydrated.get()` will not work as-is.)
2. Does the workaround (`<Show>` conditional rendering) produce acceptable E2E behavior?
3. Do the `data-*` attributes radix-leptos renders conflict with the project's existing CSS selectors?

**Decision criteria:** If the ActionForm + Button POC passes E2E, proceed. If not, defer until radix-leptos exposes `disabled` as a `MaybeSignal<bool>` prop.

**Current blocker:** The `DataTable` component the project would benefit from most is in experimental and disabled due to syntax errors. Hand-rolled `.data-table` must remain regardless.

**Effort estimate:** 4-hour POC; 5-8 hours if proceeding to full Badge + Button adoption.

**Do not adopt now.** The project has no dialogs, popovers, or complex interactive components. The existing 19 hand-rolled components work correctly and are E2E-tested.

---

#### biji-ui 0.4.4 (SELECTIVE future adoption only)

**What it might solve:** Headless accessibility primitives for complex future components — Accordion, Dialog, Combobox, Toast, MultiSelect. biji-ui is the lighter alternative to radix-leptos (25+ components, zero dependencies, pure Rust primitives following HeadlessUI/MeltUI patterns).

**POC must test:** Leptos 0.8.17 compatibility (biji-ui 0.4.x targets 0.8.x — stated by maintainer), CSS framework agnosticism with the project's oklch tokens, and whether keyboard nav patterns conflict with the project's existing Form/ActionForm interaction model.

**Decision criteria:** Use biji-ui selectively for new high-interaction components only. Do not refactor existing working components.

**Note vs. radix-leptos:** biji-ui is lighter (no deps) but has fewer total components and an unstable API (13 releases with breaking changes). radix-leptos has more components and better test coverage. Choose based on which specific components are actually needed at that time.

---

### Tier 3: WATCH (trigger conditions)

**leptos_ws 0.8 / leptos_server_signal** — Watch if the admin dashboard grows to require live updates (real-time enrollment counts, live phase progress). Current Resource + action.version() polling is sufficient. leptos_ws (Dec 2024 update) is the more recently maintained option.

**leptoaster / leptos_toaster** — Watch if the UX spec adds toast notifications for success feedback (e.g., "Enrollment confirmed!"). Currently all feedback is inline. Prefer leptoaster (May 2025 update) over leptos_toaster (May 2024).

**leptos-chartistry** — Watch if the admin dashboard adds enrollment trend charts or gift distribution analytics. 136 stars, actively developed, extensible charting.

**leptos-struct-table 0.18** — Watch if the admin section grows to 3+ tables requiring sorting or pagination. Currently one table exists (participant list), it is simple, and the `TailwindClassesPreset` generates hardcoded v3-style class strings that do not integrate with the project's custom oklch tokens without a custom `TableClassesProvider` implementation. Revisit when Tailwind v4 integration is explicitly documented by the maintainer, or when a second complex table is needed.

**leptos-use (direct)** — Already pulled in transitively by leptos_i18n (0.18.3). If ever needed directly (e.g., useStorage for draft persistence, useEventSource for SSE), it is already in the dependency graph at the correct version. 4 of 5 surveyed production Leptos projects use it.

---

### Tier 4: SKIP (with reasons)

#### Dead projects (archived Feb 2, 2026 — do not depend on read-only code)
- **Rust Radix (RustForWeb/radix)** — archived, read-only, unmaintained
- **Rust shadcn/ui (RustForWeb/shadcn-ui)** — archived, read-only, unmaintained

#### Incompatible Leptos version
- **leptos_query 0.5.3** — targets Leptos 0.6; Leptos 0.8's implicit context API is a breaking change. 47.5k downloads but no 0.8 port exists. The project's 7 Resources are simple non-overlapping queries; TanStack Query-style caching adds complexity with zero benefit at this scale.
- **leptos_drag_reorder** — explicitly documents Leptos 0.7 only, no 0.8 support.
- **leptos-tracked, leptos-signals** — Leptos 0.2.x era, abandoned.
- **leptos-declarative, leptos-tea** — outdated for 0.8 or nightly-only.

#### Architectural mismatch with this project's design system
- **Thaw** — Fluent Design System aesthetics hardcoded; no path to matching the project's oklch orange/cream brand without a complete CSS override.
- **leptos-material** — Material Design contradicts the brand; low repository activity is a secondary concern.
- **Leptodon** — Flowbite-inspired aesthetics that conflict with the custom brand; retheme cost exceeds the cost of keeping hand-rolled components.
- **leptix** — Not headless despite the category label. Injects opinionated CSS into `<head>` at mount time via `inject_style_once`. CSS variables for theming are fragile. 2 weeks old at verification time (40 total downloads), self-described "no guarantees." The Input component uses two-way signal binding which conflicts directly with ActionForm's DOM FormData read pattern.
- **leptos_ui (macro utility)** — Not a component library. Provides `clx!` class merging. The project uses semantic CSS variables for styling, which sidesteps the class merging problem entirely. No GitHub repository makes maintenance risk unacceptable.

#### CSS styling approach mismatch
- **Stylers, Styled, Stylance, Turf** — CSS-in-Rust and CSS modules approaches. The project intentionally keeps all CSS in one `tailwind.css` file. Adding a second CSS generation system violates the single-file architecture principle.

#### Out of scope for this domain
- **leptos-leaflet, leptos_maplibre** — no location features in the product
- **leptos_image** — no image assets; mail club is text and SMS
- **papelito** — archived; no WYSIWYG editing needed
- **leptos-captcha** — SMS OTP is sufficient bot prevention
- **leptos-obfuscate** — no email addresses displayed
- **leptos_oidc** — auth is SMS OTP, not OAuth/OIDC
- **leptos_meilisearch** — no search features in the product spec
- **leptos_darkmode** — design system already handles dark mode via prefers-color-scheme; no manual toggle
- **leptos-fluent** — leptos_i18n is already integrated and serves the same need with a simpler API, explicit 0.8 docs, and more recent update cadence

#### Premature for current scale
- **leptos-fetch** — the project's 7 simple Resources do not need caching, deduplication, or SWR. Adopting this before it's needed adds hidden complexity without measurable benefit.

#### Tools with no gaps to fill
- **cargo-runner** — redundant with justfile + bacon
- **vscode-leptos-snippets** — the project's guidance docs and existing component code serve this purpose better
- **wasm-bindgen-struct** — Leptos abstracts WASM bindings; no direct wasm-bindgen use in application code

#### Alternate macros
- **leptos-mview** — zero functional gain from migrating existing working code. Ecosystem tooling (formatting, IDE support) less mature than standard `view!`. Not worth introducing.

---

## The Component Library Question

**Answer: No component library. Not now. Possibly radix-leptos primitives in a future phase, selectively.**

### What the project actually has

19 hand-rolled components built on 6 CSS component classes (`.btn`, `.field`, `.badge`, `.data-table`, `.alert`, `.prose-page`) plus 3 layout classes. All forms use ActionForm with named attributes reading DOM FormData. WCAG 2.1 AA compliance achieved. 56 E2E tests passing. The design system is a precise brand spec (oklch palette, CyGrotesk/Mont fonts, specific spacing density, pill-shaped buttons at exact pixel values). This is not "we haven't gotten around to a library" — this is a deliberate, correct choice.

### Why each candidate fails for the current scope

**radix-leptos 0.9:** Headless and Leptos 0.8.8 compatible — these are real strengths. But the critical gap is the `disabled` prop: it is a static `bool`, not a `MaybeSignal<bool>`. The project's hydration gate pattern `disabled=move || !hydrated.get()` is used on every ActionForm submit button (9 components). Without this gate, E2E tests break. The workaround requires wrapping every button in `<Show>`, which trades 4 lines of hydration boilerplate for 6 lines of Show boilerplate — net negative. The DataTable component the project would most benefit from is disabled due to compile errors in the experimental module. Additionally: 4,666 total downloads at time of research suggests limited production battle-hardening despite impressive test count.

**biji-ui 0.4:** Genuinely headless (zero CSS), actively maintained through March 2026, 25+ components, Leptos 0.8 compatible. The project's current components are already accessible — biji-ui only provides value for new complex interactive components (Dialog, Combobox, Calendar) that do not yet exist in the product spec. Unstable API is acceptable for selective future adoption but not worth disrupting currently-working code.

**Thaw / Leptodon / leptix:** All impose design opinions that conflict with the project's oklch brand system.

**leptos-shadcn-ui (cloud-shuttle):** Production-grade, Tailwind v4 compatible, 38+ components with 500+ tests. But it brings shadcn/ui aesthetics (neutral grays, specific shadows, Inter font). Adapting these to match the project's orange/cream brand with CyGrotesk display font would require overriding virtually every component's default CSS — at which point you are building a custom design system on top of a custom design system.

### What production Leptos projects actually do

Of 5 production Leptos projects inspected: zero use a component library for their core UI. One (ccf-deadlines, a CSR conference tracker) uses Thaw 0.5-beta. The pattern across the others is handwritten semantic HTML + Tailwind utilities + minimal external dependencies. This project is already at the production-grade end of that spectrum.

### The correct threshold for adopting radix-leptos

Adopt radix-leptos (or biji-ui) when the product spec adds any of: Dialog with focus trap, Combobox with keyboard navigation, DatePicker, Calendar, MultiSelect, or Toast notifications. These components require 100-300 lines of correct accessibility implementation each. A headless library pays for itself at that point. For the current scope (text inputs, submit buttons, status badges, one data table), it does not.

---

## Integration Notes

### tracing-subscriber-wasm — exact steps

Files to change:
- `/Users/ryzhakar/pp/same-te-mail-club/Cargo.toml`: add `tracing-subscriber-wasm = "0.1"` to `[dependencies]`
- `/Users/ryzhakar/pp/same-te-mail-club/src/lib.rs`: add 8 lines to the `hydrate()` function as shown in Tier 1

No other files change. No E2E test changes required — this is dev DX only, invisible to tests.

### Internal refactors that eliminate more pain than any library

These are not library adoptions but address the top gaps identified in the architecture audit. Each is faster to implement than evaluating a library.

**use_hydrated() hook** (30 minutes, saves 36 lines across 9 components):
```rust
// src/hooks.rs or similar
pub fn use_hydrated() -> ReadSignal<bool> {
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| set_hydrated.set(true));
    hydrated
}
```

**extract_context! macro** (30 minutes, saves ~57 lines across 19 server functions):
```rust
macro_rules! extract_context {
    ($($ty:ty),+) => {
        ( $( leptos::context::use_context::<$ty>()
                 .ok_or_else(|| ServerFnError::new(
                     concat!("missing context: ", stringify!($ty))
                 ))? ),+ )
    }
}
```

**require_admin helper** (30 minutes, reduces 13 server function copy-paste checks):
```rust
fn require_admin(user: &CurrentUser) -> Result<(), ServerFnError> {
    if user.role != UserRole::Admin {
        Err(ServerFnError::new("forbidden: admin only"))
    } else {
        Ok(())
    }
}
```

These three refactors address the DX gaps flagged in the architecture audit and are collectively a better investment than any library evaluation.

---

## Coverage Gaps

The following project needs have no ecosystem solution and require internal implementation:

**Structured server function errors (HIGH impact):** All server function errors are converted to string immediately via `ServerFnError::new(to_string())`. The client receives opaque strings and cannot distinguish auth errors (should redirect to login) from user input errors (should show inline) from server errors (should show "try again"). No ecosystem library addresses this. The solution is an `AppErrorResponse` enum that serializes/deserializes across the server function boundary with a discriminant that clients can match on.

**Server function instrumentation (MEDIUM impact):** 19 server functions have no `#[tracing::instrument]` spans. Failures are invisible beyond the HTTP 500. Adding `tracing-subscriber-wasm` closes the client-side gap; adding `#[tracing::instrument]` to server functions closes the server-side gap. No library needed.

**Transaction boundaries (LOW impact):** Multi-step server functions (generate_assignments, SMS broadcasts) run without database transactions. A partial failure leaves the DB inconsistent. `pool.begin().await?` and `.commit()` is the solution — no library needed.

**Icon system (DEFERRED):** The design system explicitly defers icons. When icons become necessary, `leptos-icons 0.7.0` (Leptos 0.8 compatible, 118 stars) with `icondata 0.6` is the battle-tested choice — seen in 3 of 5 surveyed production projects.

---

## Research Confidence

**Tier 1 — HIGH confidence.** tracing-subscriber-wasm: exact version compatibility verified from Cargo.toml, integration code copy-paste ready, 175k download track record. leptos_i18n: already in production for 6 complete E2E phases.

**Tier 2 — MEDIUM confidence.** leptosfmt: tool works but project-specific view! formatting behavior requires a POC run. radix-leptos: Leptos 0.8.8 compatibility verified from source, ActionForm + hydration gate incompatibility documented from Button component source code inspection — but upstream could fix `disabled` prop at any time. biji-ui: 0.8 compatibility stated but not independently verified against this project's feature flag configuration.

**Tier 3 (WATCH) — LOW confidence.** Libraries assessed from README, download counts, and category surveys. None were deeply verified. Compatibility claims are from maintainer documentation.

**Tier 4 (SKIP) — HIGH confidence for rejections.** Dead project status (Radix, shadcn/ui RustForWeb) verified via GitHub API. Version incompatibility (leptos_query) verified against crate Cargo.toml source. Design system conflicts (Thaw, Leptodon, leptix) verified via CSS and source inspection.

**What could change these recommendations:**
- radix-leptos adds `MaybeSignal<bool>` support for the `disabled` prop → ActionForm compatibility becomes unblocked → POC becomes a high-priority action
- leptos-struct-table explicitly documents Tailwind v4 custom token integration → table adoption becomes viable for a second admin table
- leptos_query receives an official Leptos 0.8 port → re-evaluate if the app grows to 30+ Resources
- Any WATCH library releases 1.0 → stability threshold changes for that specific library
