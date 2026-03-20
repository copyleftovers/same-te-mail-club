# Project Brief — Саме Те Mail Club

Facts extracted from project audits. No opinions, no recommendations.

## Stack

- **Framework:** Leptos 0.8.17 (SSR + hydration via cargo-leptos)
- **Server:** Axum 0.8.8, Tokio 1.50
- **Database:** PostgreSQL via sqlx 0.8.6 (compile-time query verification)
- **CSS:** Tailwind CSS v4 standalone binary (NO Node.js, no npm, no tailwind.config.js)
- **i18n:** leptos_i18n 0.6.1, single locale (Ukrainian), 134 translation keys
- **Tracing:** tracing 0.1.44 shared; tracing-subscriber 0.3.22 SSR-only; zero client-side logging
- **Linting:** clippy all+pedantic = deny, unsafe_code = forbid
- **Testing:** 56 serial Playwright E2E tests, ~15 unit tests

## Transitive Leptos Dependencies Already Present

- `leptos-use 0.18.3` (pulled by leptos_i18n)

## Design System

- oklch color tokens (brand orange, pink, gray, blue, black, cream)
- Two-tier token system: raw tokens in `@theme` → Tailwind utilities; semantic aliases in `:root` → CSS variables
- Variant pattern: `data-*` attributes + CSS custom property hooks (e.g., `.btn[data-variant="secondary"]`)
- Fonts: CyGrotesk (display), Mont (body) — woff2 self-hosted
- All CSS in single file: `style/tailwind.css` using `@layer base/components/utilities`

## Component Surface

- **19 Leptos components** across 10 .rs files
- **10** use ActionForm pattern (server function forms)
- **9** have hydration gate (`signal(false)` + `Effect::new` → `set(true)`)
- **26 server functions**, all return `Result<T, ServerFnError>`
- **7 routes** (1 public login, 1 onboarding, 1 participant home, 4 admin pages)

## CSS Component Classes (in `@layer components`)

| Class | Lines | What it does |
|-------|-------|-------------|
| `.btn` | 58 | Pill button with `data-variant` (primary/secondary/destructive) and `data-size` (sm/default/lg) |
| `.field` / `.field-label` / `.field-input` / `.field-error` | 52 | Form field structure with focus/error/disabled states |
| `.badge` | 33 | Status pill with `data-status` (active/pending/error/inactive/confirmed) |
| `.data-table` | 22 | Full-width table with header styling |
| `.alert` | 9 | Error/warning message box |
| `.prose-page` | 45 | Content container (max-width 65ch, typography hierarchy) |
| `.admin-nav` | 26 | Admin navigation links |
| `.app-header` | 14 | Top-level header with logo |
| `.sms-trigger` | 18 | SMS batch send button group |

## UI Patterns Used

- **ActionForm** for all mutations (Leptos reads FormData from DOM at submit, no signals needed)
- **Hydration gate** on all ActionForm buttons: `disabled=move || !hydrated.get()`
- **Resource + action.version()** for data refetch after mutations
- **`<For>` loops** for lists (participants, assignment cycles)
- **`match` on enums** for state-based rendering (HomeState with 9 variants)
- **`data-testid`** on all interactive/assertable elements (40+ testids)
- **No modals, no dropdowns, no tabs, no date pickers, no tooltips, no toasts**

## Components NOT Present (confirmed absent)

Modals, dialogs, dropdowns/select menus, tabs, date pickers, tooltips, toasts/notifications, accordions, carousels, command palettes, popovers, autocomplete/combobox, sliders, switches/toggles, progress bars, skeleton loaders.

## Identified Boilerplate (quantified)

| Pattern | Instances | Lines each | Total lines |
|---------|-----------|-----------|-------------|
| Hydration gate (signal + Effect) | 9 | 4 | 36 |
| Form field markup (label + input + error + ARIA) | 8 forms | ~12 | ~96 |
| Action error display | 3 variants | ~8 | ~24 |
| Context extraction (pool + parts) | 19 | 3 | 57 |
| Admin role check | 13 | 3 | 39 |

## Version Constraints for New Dependencies

Any new crate must:
- Target Leptos 0.8.x (not 0.7, not 0.9)
- Work with Tokio 1.x async runtime
- Support WASM target if used client-side (`wasm32-unknown-unknown`)
- Not require Node.js or npm (Tailwind v4 is standalone binary)
- Not conflict with sqlx 0.8, axum 0.8, serde 1.x
