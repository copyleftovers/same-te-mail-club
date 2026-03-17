# i18n Migration — Externalize User-Facing Strings

## Problem

~103 Ukrainian string literals are hardcoded directly in Leptos `view!` macros and server function bodies across 8+ source files. Changing any copy requires grepping the codebase and hoping nothing is missed. There is no single source of truth for UI text.

## Decision

Adopt **`leptos_i18n`** (v0.6.x, Leptos 0.8 native).

- Compile-time safety: invalid string keys are compiler errors
- JSON locale files — one file per language, clear inventory of all strings
- `t!(i18n, "key")` macro drops in wherever a string literal was
- Strings baked into the binary — no runtime loading, no SSR complexity
- E2E tests are already decoupled from string content (all selectors use `data-testid`)

## Scope

### 1. Dependency & config

- Add `leptos_i18n` to `Cargo.toml` (SSR + hydrate features)
- Create `i18n.json`: `{ "default": "uk", "locales": ["uk"] }`
- Create `locales/uk.json` with all extracted strings (see inventory below)

### 2. String categories to extract

| Category | Location | Count (approx) |
|----------|----------|----------------|
| UI labels, buttons, headings | `view!` macros across all pages/admin | ~70 |
| SMS message bodies | `src/admin/sms.rs` server functions | ~5 |
| Error/status messages | `AppError`, server fn returns | ~10 |
| Placeholder text | `<input placeholder=...>` | ~5 |
| Loading/fallback text | `<Suspense fallback=...>` | ~5 |

### 3. Component changes

- Wrap root component in `<I18nContextProvider>`
- Replace every inline string literal in `view!` with `t!(i18n, "key")`
- SMS bodies in server functions: use `i18n` context via `expect_context` or a standalone translation fn (server-side `use_i18n` equivalent)

### 4. Key naming convention

```
{component}_{element}_{variant?}
```

Examples:
- `login_phone_label`
- `login_send_button`
- `home_enroll_button`
- `admin_season_launch_button`
- `sms_season_open_body`
- `common_loading`

## What this is NOT

- No multi-language support yet. Single locale (`uk`) to start.
- No user-facing language switcher.
- No changes to routing, data model, or business logic.

Adding a second language later = add `locales/en.json` + update `i18n.json`. No code changes.

## Verification gates

1. `cargo clippy --features ssr -- -D warnings` — clean
2. `cargo test` — passing
3. `just e2e` — 56/56 passing (E2E is already string-decoupled, so this should be mechanical)
4. No raw Ukrainian string literals remain in `src/` (verify with `grep -r '"[А-Яа-яІіЇїЄєҐґ]' src/`)
