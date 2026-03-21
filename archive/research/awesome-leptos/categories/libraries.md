# Libraries — Deep Dive

**Date:** 2026-03-19

## Summary

The "Libraries" category contains 34 utility crates spanning reactive utilities, state management, UI primitives, real-time synchronization, internationalization, content delivery, and specialized components. Most are mature (0.1–0.5 release status) and actively maintained (updates within the past 6 months). The ecosystem divides into three tiers: **core utilities** (leptos-use, leptos-fetch) with 50+ stars and strong 0.8 support; **niche but solid** (internationalization, toasters, animations, PDFs); and **experimental/archived** (papelito, archived leptos_sse). For this project's scope (seasonal mail exchange with auth, SMS, season management), only 3–5 are relevant. Most others solve problems outside the domain (maps, charting, WYSIWYG editing, email obfuscation).

---

## Per-Library Analysis

### leptos-use
- **URL:** https://leptos-use.rs/
- **Stars/Activity:** Unknown exact count; active. 89 functions in collection.
- **Leptos version:** 0.8 supported (exact compatibility not documented in landing page)
- **What it does:** VueUse/React-Use-inspired collection of 89+ reactive utility hooks (sensors, composables, lifecycle helpers). Covers common UI patterns: mouse tracking, window resize, focus management, clipboard, storage, etc.
- **Relevance to this project:** MEDIUM — Useful for general reactive patterns, but the project's scope is narrow (forms, auth, season flow). Adopted utilities would likely be few (useStorage for preferences, useLocalStorage for form drafts).
- **Adoption recommendation:** SKIP (for now) — Only if form draft persistence or gesture detection are needed later. The project is currently feature-complete without them.

### leptos-fetch
- **URL:** https://github.com/zakstucke/leptos-fetch
- **Stars/Activity:** 55 stars. Last update: Feb 2025.
- **Leptos version:** 0.8 supported (leptos-fetch 0.4.x)
- **What it does:** Async state management library for data fetching. Provides caching, request deduplication, invalidation, background refetching, optimistic updates, pagination. Successor to "Leptos Query."
- **Relevance to this project:** LOW — The project uses `Resource` + `ActionForm` for data loading and mutation. No complex query management, caching strategies, or pagination currently exist. The simple Resource model is sufficient.
- **Adoption recommendation:** SKIP — Premature abstraction. Adopt if pagination, complex caching, or concurrent request coordination are added in future phases.

### lepticons
- **URL:** https://lepticons.9bits.cc/
- **Stars/Activity:** Unknown (landing page only, no GitHub repo provided)
- **Leptos version:** Unknown
- **What it does:** Icon library with live demo.
- **Relevance to this project:** LOW — The design system specifies SVG logos only, no generic icons. Navigation and UI indicators use text + brand colors.
- **Adoption recommendation:** SKIP — No icon requirement in the product design.

### leptos-icons
- **URL:** https://github.com/Carlosted/leptos-icons
- **Stars/Activity:** 118 stars. Latest version 0.7.0 supports Leptos 0.8.
- **Leptos version:** 0.8 supported (v0.7.0+)
- **What it does:** Icon library wrapper enabling integration of popular icon sets (Heroicons, Font Awesome, etc.) into Leptos apps.
- **Relevance to this project:** LOW — Same as lepticons; design system has no icon requirements.
- **Adoption recommendation:** SKIP.

### leptos_image
- **URL:** https://github.com/gaucho-labs/leptos-image
- **Stars/Activity:** 63 stars. Last update: Feb 2024.
- **Leptos version:** 0.6.x supported (last tested). **Unclear if 0.8 compatible** — documentation does not confirm.
- **What it does:** Image optimization component. Auto-converts to WebP, generates low-quality placeholders, prioritizes critical images.
- **Relevance to this project:** NONE — The mail club has no image assets (no profile pictures, no mail previews, no galleries).
- **Adoption recommendation:** SKIP.

### leptos-declarative
- **URL:** https://github.com/jquesada2016/leptos-declarative
- **Stars/Activity:** 16 stars. Last update: Jan 2023 (3 years old).
- **Leptos version:** Unknown (not documented). **Likely outdated for 0.8**.
- **What it does:** Provides `If` component with Then/ElseIf/Else branches and `Portal` component for rendering to other DOM nodes. Declarative control flow.
- **Relevance to this project:** LOW — Leptos's built-in `<Show>` component and pattern matching cover these needs idiomatically. No value-add over native patterns.
- **Adoption recommendation:** SKIP — Use Leptos's native `<Show>`, `<For>`, and `match` instead.

### leptos-tracked
- **URL:** https://docs.rs/leptos-tracked/latest/leptos_tracked/
- **Stars/Activity:** Unknown. Last documented version 0.1.4.
- **Leptos version:** 0.2.x (very old; predates modern Leptos 0.5+)
- **What it does:** Utility traits for signal mutations without closures. Traits like `AddAssign`, `MulAssign`, `Toggle`, `TrackedVec`.
- **Relevance to this project:** NONE — This is for Leptos 0.2.x, not 0.8. Incompatible.
- **Adoption recommendation:** SKIP — Deprecated/abandoned.

### leptos-signals
- **URL:** https://github.com/akesson/leptos-signals
- **Stars/Activity:** 6 stars. Last update: Jan 2023 (3 years old).
- **Leptos version:** Unknown (not documented). **Likely outdated**.
- **What it does:** Provides `KeyedSignal` primitive for caching values by key.
- **Relevance to this project:** NONE — Niche use case. No caching by key needed in the domain.
- **Adoption recommendation:** SKIP.

### leptos-tea
- **URL:** https://github.com/jquesada2016/leptos-tea
- **Stars/Activity:** 25 stars. Last update: March 2023 (3 years old).
- **Leptos version:** Unknown (nightly only). **Not suitable for stable 0.8**.
- **What it does:** Elm Architecture (TEA) pattern for state management. Generates Model/UpdateModel/ViewModel from a derive macro.
- **Relevance to this project:** LOW — The project's state management is simple: phase transitions, form state, Resource-driven data loading. No need for centralized Elm-style architecture. Adds boilerplate for minimal gain.
- **Adoption recommendation:** SKIP — Over-engineered for the domain. Keep forms and Resource-driven patterns.

### leptos-leaflet
- **URL:** https://github.com/headless-studio/leptos-leaflet
- **Stars/Activity:** 52 stars. Latest version 0.10.x supports Leptos 0.8.x (March 2025 release).
- **Leptos version:** 0.8 supported (v0.10.x)
- **What it does:** Leaflet map components for Leptos. Wraps Leaflet.js with Leptos components: MapContainer, Marker, Polygon, TileLayer, etc.
- **Relevance to this project:** NONE — The mail club is location-agnostic. No map features in the spec.
- **Adoption recommendation:** SKIP.

### leptos_maplibre
- **URL:** https://github.com/triesap/leptos_maplibre
- **Stars/Activity:** 1 star. Status: very early/experimental.
- **Leptos version:** Unknown (not documented).
- **What it does:** MapLibre GL JS bindings for Leptos (WebGL-based maps).
- **Relevance to this project:** NONE — No mapping features needed.
- **Adoption recommendation:** SKIP.

### papelito
- **URL:** https://github.com/msmaiaa/papelito
- **Stars/Activity:** 10 stars. **ARCHIVED October 2023**. Last update: May 2023.
- **Leptos version:** Unknown. Library in early experimental stages.
- **What it does:** WYSIWYG editor component for Leptos.
- **Relevance to this project:** NONE — No rich text editing in the mail club (simple text SMS/messages only).
- **Adoption recommendation:** SKIP — Archived anyway.

### leptos_server_signal
- **URL:** https://github.com/tqwewe/leptos_server_signal
- **Stars/Activity:** 68 stars. Last update: May 2023 (2+ years old).
- **Leptos version:** Unknown (wildcard `*` dependency, unclear 0.8 support).
- **What it does:** Keeps client-side signals in sync with server via WebSocket. Server can write, client reads (read-only on client). JSON patch transport.
- **Relevance to this project:** MEDIUM — Could simplify real-time admin features (live season state, participant status updates). But current scope has no live admin dashboards; SMS delivery is dry-run only. Not a core need yet.
- **Adoption recommendation:** EVALUATE (only if live admin features are added). For now, Resource + simple polling is sufficient.

### leptos_sse
- **URL:** https://github.com/messense/leptos_sse
- **Stars/Activity:** 34 stars. **ARCHIVED July 2025**. Deprecated in favor of `leptos-use::use_event_source`.
- **Leptos version:** Unknown (wildcard dependency).
- **What it does:** Server-Sent-Events version of leptos_server_signal. Real-time signal sync via SSE (lighter than WebSocket).
- **Relevance to this project:** NONE — Archived. Migration path is to leptos-use.
- **Adoption recommendation:** SKIP — Deprecated. Use leptos-use instead if SSE is needed.

### leptos_ws
- **URL:** https://github.com/TimTom2016/leptos_ws
- **Stars/Activity:** 34 stars. Latest update: Dec 2024 (recent).
- **Leptos version:** 0.8 supported (0.8–0.9 of leptos_ws); 0.7 supported (0.7.x).
- **What it does:** WebSocket-based real-time signals. Three modes: read-only server signals, bidirectional signals, channel-based messaging. JSON patch transport.
- **Relevance to this project:** MEDIUM — Same as leptos_server_signal. Could enable live admin views, but not a current requirement. Resource-based patterns are sufficient for now.
- **Adoption recommendation:** EVALUATE (only if live admin features are added). Active maintenance (Dec 2024), solid 0.8 support, more recent than leptos_server_signal.

### leptos_i18n
- **URL:** https://github.com/Baptistemontan/leptos_i18n
- **Stars/Activity:** 149 stars. Latest version: 0.6.1, released March 3, 2026 (very recent).
- **Leptos version:** 0.8 supported (v0.6.x)
- **What it does:** Internationalization library. Compile-time locale loading, type-safe translation keys, runtime locale selection.
- **Relevance to this project:** MEDIUM — The product is bilingual (Ukrainian + English confirmed in landing page, SMS templates). Compile-time i18n would prevent translation key mismatches. But current implementation uses string constants. Refactoring would require touching many files.
- **Adoption recommendation:** EVALUATE — Adopt **only if** the team commits to maintaining translation keys long-term. High-activity project (very recent update), excellent 0.8 support, 149 stars. If added, refactor hardcoded strings incrementally per new feature.

### leptos-fluent
- **URL:** https://github.com/mondeja/leptos-fluent
- **Stars/Activity:** 88 stars. Latest version: 0.3.1, released Dec 29, 2025 (very recent).
- **Leptos version:** Unknown (not explicitly documented, but crate structure suggests 0.8).
- **What it does:** i18n using fluent-templates (Mozilla's Fluent format). YAML/JSON/JSON5 translation files, server + client rendering, browser auto-detection.
- **Relevance to this project:** MEDIUM — Alternative to leptos_i18n. Fluent format is more powerful for pluralization, gender, etc., but overkill if translations are simple.
- **Adoption recommendation:** SKIP (over leptos_i18n) — leptos_i18n has clearer 0.8 docs, simpler setup, more recent activity specifically dated. Use leptos_i18n if adopting i18n libraries.

### leptos_darkmode
- **URL:** https://gitlab.com/kerkmann/leptos_darkmode
- **Stars/Activity:** Unknown. Created Oct 2023. 13 commits.
- **Leptos version:** Unknown.
- **What it does:** Dark mode helper with Tailwind support.
- **Relevance to this project:** LOW — Design system uses `prefers-color-scheme: dark` only; no manual dark mode toggle. Library would be unnecessary.
- **Adoption recommendation:** SKIP.

### leptos_oidc
- **URL:** https://gitlab.com/kerkmann/leptos_oidc
- **Stars/Activity:** Unknown. Created Nov 2023. 74 commits.
- **Leptos version:** Unknown.
- **What it does:** OpenID Connect (OAuth) integration for Leptos.
- **Relevance to this project:** NONE — Auth is phone-based (SMS OTP), not OAuth/OIDC.
- **Adoption recommendation:** SKIP.

### leptos_meilisearch
- **URL:** https://gitlab.com/kerkmann/leptos_meilisearch
- **Stars/Activity:** Unknown. Created Nov 2023.
- **Leptos version:** Unknown.
- **What it does:** Meilisearch search engine integration for Leptos.
- **Relevance to this project:** NONE — No search features in the mail club (seasonal snapshots, not searchable archives).
- **Adoption recommendation:** SKIP.

### leptos-captcha
- **URL:** https://github.com/sebadob/leptos-captcha
- **Stars/Activity:** 29 stars. Latest version: 0.4.0, released May 6, 2025 (recent).
- **Leptos version:** 0.8 supported (0.4.x)
- **What it does:** Self-hosted Captcha/PoW component. No third-party service. Uses Proof of Work instead of puzzles.
- **Relevance to this project:** LOW — SMS OTP is already sufficient for bot prevention. No additional CAPTCHA needed unless spam becomes a problem.
- **Adoption recommendation:** SKIP (for now) — Only add if registration abuse increases.

### leptos-obfuscate
- **URL:** https://github.com/sebadob/leptos-obfuscate
- **Stars/Activity:** 11 stars. Latest version: 0.5.0, released May 6, 2025 (recent).
- **Leptos version:** 0.8 supported (0.5.x)
- **What it does:** Email obfuscation component to prevent bot harvesting.
- **Relevance to this project:** NONE — The mail club handles no email addresses as displays (contact info is via SMS/user names, not email). No need to obfuscate.
- **Adoption recommendation:** SKIP.

### cinnog
- **URL:** https://github.com/NiklasEi/cinnog
- **Stars/Activity:** 91 stars. Last update: Dec 2023 (2+ years old).
- **Leptos version:** Unknown (not documented). Experimental.
- **What it does:** Static site generator using Leptos + Bevy ECS. Island mode: static HTML + interactive WASM islands.
- **Relevance to this project:** NONE — This is a server application, not a static site.
- **Adoption recommendation:** SKIP.

### leptoaster
- **URL:** https://github.com/KiaShakiba/leptoaster
- **Stars/Activity:** 14 stars. Latest version: 0.2.3, released May 23, 2025 (very recent).
- **Leptos version:** Unknown (not explicitly documented).
- **What it does:** Minimal toast/notification library. Supports info, success, warn, error levels.
- **Relevance to this project:** MEDIUM — Toast notifications are useful for form success/error feedback. But the design system makes no mention of toasts; currently using inline error containers in forms.
- **Adoption recommendation:** EVALUATE — If the product adds brief success messages (e.g., "Enrollment confirmed!"), adopt a toast library. This one is lightweight and recent. Decision: delay until UX adds toast requirements.

### leptos_toaster
- **URL:** https://github.com/SorenHolstHansen/leptos_toaster
- **Stars/Activity:** 35 stars. Latest version: 0.1.7, released May 20, 2024 (12 months old).
- **Leptos version:** Unknown (not explicitly documented).
- **What it does:** Toast component inspired by sonner (popular React toast library). Notifications with positioning, expiry, styling.
- **Relevance to this project:** MEDIUM — Same as leptoaster. Slightly more active (35 vs 14 stars), but older (May 2024 vs May 2025).
- **Adoption recommendation:** EVALUATE (same as leptoaster). If adopting, prefer leptoaster (more recent). But defer pending UX requirements.

### leptos-hotkeys
- **URL:** https://github.com/gaucho-labs/leptos-hotkeys
- **Stars/Activity:** 63 stars. Latest version: 0.2.1, released May 27, 2024.
- **Leptos version:** Not explicitly documented, but maintained actively. Likely 0.8 compatible.
- **What it does:** Declarative keyboard shortcuts. Global, scoped, or focus-trapped hotkeys. Macro-based.
- **Relevance to this project:** LOW — No keyboard shortcuts in the current product. SMS + form-based navigation is primary UX.
- **Adoption recommendation:** SKIP (for now) — Add if power-user admin shortcuts are needed (e.g., Cmd+E to export, Cmd+L to launch season).

### leptos-chartistry
- **URL:** https://github.com/feral-dot-io/leptos-chartistry
- **Stars/Activity:** 136 stars. Active development.
- **Leptos version:** Unknown (not explicitly documented in provided content).
- **What it does:** Extensible charting library. Provides `<Chart>` component for rendering data visualizations.
- **Relevance to this project:** MEDIUM-LOW — Admin dashboard might benefit from charts (e.g., enrollment trends, gift distribution heatmaps). But not a core feature in Phase 6. Defer.
- **Adoption recommendation:** EVALUATE — Add only if admin analytics are prioritized. Good community interest (136 stars) suggests maturity.

### leptos_drag_reorder
- **URL:** https://github.com/tqwewe/leptos_drag_reorder
- **Stars/Activity:** 25 stars. Last update: Nov 2024.
- **Leptos version:** 0.7 **only** (explicitly documented). **Does not support 0.8.**
- **What it does:** Draggable panel reordering.
- **Relevance to this project:** NONE — Not 0.8 compatible, and no drag-and-drop requirements in the spec.
- **Adoption recommendation:** SKIP — Incompatible with Leptos 0.8.

### Rust Floating UI
- **URL:** https://floating-ui.rustforweb.org/
- **Stars/Activity:** Unknown. Community port of Floating UI (JavaScript library).
- **Leptos version:** Unknown (not documented).
- **What it does:** Positioning primitives for floating elements (tooltips, popovers, dropdowns). Collision detection, viewport management.
- **Relevance to this project:** MEDIUM-LOW — Useful if the UI adds dropdowns or complex popovers. Not required for current form-based design. Design system mentions no popovers or complex overlays.
- **Adoption recommendation:** SKIP (for now) — Use native `<select>` and CSS positioning for now. Add if advanced dropdown UX is needed.

### Rust Lucide
- **URL:** https://lucide.rustforweb.org/
- **Stars/Activity:** Unknown (community port).
- **Leptos version:** Unknown (not documented).
- **What it does:** Icon toolkit (Lucide icons ported to Rust for use in Leptos/Dioxus/Yew).
- **Relevance to this project:** LOW — Same as other icon libraries; no icon requirements in the design system.
- **Adoption recommendation:** SKIP.

### leptos_async_signal
- **URL:** https://github.com/demiurg-dev/leptos_async_signal
- **Stars/Activity:** 3 stars. Last update unknown (very early stage).
- **Leptos version:** 0.8 supported (0.6.0)
- **What it does:** Async signals for SSR. Combines RwSignal + Resource to handle async value generation before rendering. Use case: generating breadcrumbs.
- **Relevance to this project:** LOW — The project uses `Resource` directly for async data. No need for wrapper pattern unless breadcrumbs or similar dynamic metadata are added.
- **Adoption recommendation:** SKIP — Use Resource directly for clarity.

### leptos_animate
- **URL:** https://github.com/brofrain/leptos-animate
- **Stars/Activity:** 14 stars. Latest update recent (commit hash visible, exact date unknown).
- **Leptos version:** 0.8 supported (0.1.x)
- **What it does:** Animation utilities. FLIP for reordering, CSS class enter/exit animations, custom animation builders.
- **Relevance to this project:** MEDIUM-LOW — Could enhance UX (e.g., fade-in form fields, smooth season transitions). But design system specifies minimal motion (`prefers-reduced-motion: reduce`). Deferred feature.
- **Adoption recommendation:** SKIP (for now) — Design system prioritizes accessibility. Motion is secondary. Evaluate if explicit animations are added to UX spec.

### leptos-unique-ids
- **URL:** https://github.com/mondeja/leptos-unique-ids
- **Stars/Activity:** 8 stars. Latest version: 0.1.1, released June 16, 2025 (recent).
- **Leptos version:** Wildcard (`*`) dependency, supports all versions.
- **What it does:** Ensures globally unique DOM IDs. Macro for ID generation, Dylint lints to prevent conflicts.
- **Relevance to this project:** LOW — The project is small enough that ID conflicts are unlikely. HTML forms and sections can safely use manual IDs or no IDs. Overkill for current scope.
- **Adoption recommendation:** SKIP — Manual ID management is sufficient.

### leptos-pdf
- **URL:** https://github.com/dmanuel64/leptos-pdf
- **Stars/Activity:** 2 stars. Latest version: 0.8.1, released Dec 27, 2025 (recent).
- **Leptos version:** 0.8 supported (0.8.x)
- **What it does:** PDF rendering and viewing component. Uses PDFium backend, canvas-based.
- **Relevance to this project:** NONE — No PDF generation or viewing in the mail club. SMS + web forms only.
- **Adoption recommendation:** SKIP.

---

## Category Verdict

### Top Picks for This Project

**ADOPT NOW:**
1. **leptos_i18n** (if translation maintenance is committed) — 149 stars, 0.6.1 March 2026, excellent 0.8 support. Refactor hardcoded strings incrementally as new features are added. Risk: requires discipline to maintain translation keys.

**EVALUATE FOR FUTURE PHASES:**
1. **leptos_ws** or **leptos_server_signal** (if live admin features are added) — Real-time signal sync via WebSocket. leptos_ws is more recent (Dec 2024). Decide when admin analytics/live dashboards are spec'd.
2. **leptoaster** or **leptos_toaster** (if toast notifications are added to UX spec) — Lightweight, recent (May 2025), good community interest. Prefer leptoaster (more recent). Defer pending UX requirements.
3. **leptos-chartistry** (if admin dashboards require analytics) — 136 stars, extensible. Evaluate when analytics features are prioritized.

**SKIP:**
- All icon libraries (lepticons, leptos-icons, Rust Lucide) — no icon requirements
- All mapping libraries (leptos-leaflet, leptos_maplibre) — location-agnostic product
- leptos-image, papelito, cinnog — no images, WYSIWYG, or static site generation needed
- Email obfuscation, CAPTCHA, OIDC, Meilisearch — out of scope for the domain
- leptos_drag_reorder — Leptos 0.7 only, incompatible with 0.8
- Archived libraries (leptos_sse, papelito) — use active alternatives
- Experimental/unmaintained (leptos-declarative, leptos-tea, leptos-tracked, leptos-signals, cinnog) — outdated for 0.8 or add unnecessary boilerplate

### Summary by Use Case

| Category | Library | Status |
|----------|---------|--------|
| **i18n** | leptos_i18n | ADOPT (pending translation commitment) |
| **Real-time sync** | leptos_ws | EVALUATE (phase 2+) |
| **Notifications** | leptoaster | EVALUATE (UX-pending) |
| **Analytics** | leptos-chartistry | EVALUATE (phase 2+) |
| **Everything else** | (see SKIP list) | SKIP |

**Most valuable decision:** Adopt **leptos_i18n** now if the team is ready to maintain bilingual translations. This prevents silent key mismatches and ensures UI consistency across Ukrainian and English. Cost: one refactor pass to extract hardcoded strings into i18n keys. Benefit: eliminates a class of bugs and scales well as features are added.

All other libraries are either premature (not needed yet), out-of-scope (wrong domain), or superseded (archived). Avoid adoption until UX or product requirements explicitly call for those features.
