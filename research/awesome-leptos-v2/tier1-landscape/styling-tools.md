# Styling Tools for Leptos 0.8 + Tailwind CSS v4

**Research date:** 2026-03-19
**Scope:** Class management, CSS-in-Rust, icon libraries, theming
**Project constraint:** No Node.js, no npm; Tailwind v4 standalone binary; Leptos 0.8.x required

---

## Category 1 — Class Management (Tailwind merge/conflict resolution)

### tailwind_fuse

- **URL:** https://github.com/gaucho-labs/tailwind-fuse
- **crates.io:** https://crates.io/crates/tailwind_fuse
- **Leptos dep in Cargo.toml:** Not a dependency. Leptos appears only as a keyword.
- **Category:** Class management
- **What it does:** Two utilities: `tw_join!` (concatenate classes, no conflict resolution) and `tw_merge!` (resolve conflicts, rightmost wins — e.g., `p-4 p-6` → `p-6`). Optional `variant` feature adds `#[derive(TwClass)]` and `#[derive(TwVariant)]` macros for type-safe component variants (CVA pattern).
- **Requires Node.js?** No
- **Tailwind v4 compatible?** Unknown — no explicit v4 documentation. Conflict resolution logic is parser-based (uses `nom`); v4's new utility naming could break some conflict rules. Last update predates v4 GA.
- **Last commit:** June 26, 2024 (v0.3.2, January 2025 crates.io publish)
- **Notes:** Framework-agnostic — works in Leptos, Dioxus, Yew. Solid JS-ecosystem parity. No Tailwind v4 explicit support confirmed. Repository inactive since mid-2024.

---

### tw_merge

- **URL:** https://github.com/rust-ui/tw_merge
- **crates.io:** https://crates.io/crates/tw_merge
- **Leptos dep in Cargo.toml:** Not a dependency. No framework dependency at all.
- **Category:** Class management
- **What it does:** Macro-based Tailwind class merging with explicit Tailwind v4 targeting (`tailwind-v4` keyword). Provides merge + variant composition. Part of the rust-ui organization (same org as `leptos_ui`).
- **Requires Node.js?** No
- **Tailwind v4 compatible?** Yes — explicitly targets Tailwind CSS v4. Description: "Macros for merging Tailwind CSS v4 classes or creating variants."
- **Last commit:** March 7, 2026 (v0.1.20)
- **Notes:** Actively maintained as of research date. v4-first design. No direct Leptos dependency — pure class string manipulation. Companion crate `tw_merge_variants` (v0.1.20, same date). Early in development (17 commits, 2 stars) but aligns exactly with this project's stack.

---

### twust

- **URL:** https://github.com/Oyelowo/twust
- **crates.io:** https://crates.io/crates/twust
- **Leptos dep in Cargo.toml:** Not a dependency.
- **Category:** Class management (compile-time validation, not merging)
- **What it does:** Zero-config compile-time static type-checker for Tailwind CSS class names. Validates class strings at compile time via macros — catches typos before runtime. Self-contained, no external tools.
- **Requires Node.js?** No
- **Tailwind v4 compatible?** Unknown — last updated January 2025 (v1.1.0). No v4-specific mentions found.
- **Last commit:** January 2025
- **Notes:** Different category from tailwind_fuse/tw_merge — this validates, not merges. Useful for catching typos. Supports DaisyUI via optional feature. No v4 confirmation.

---

## Category 2 — CSS-in-Rust (scoped styles, CSS modules)

### Stylance

- **URL:** https://github.com/basro/stylance-rs
- **crates.io:** https://crates.io/crates/stylance
- **Leptos dep in Cargo.toml:** None. Framework-agnostic.
- **Category:** CSS modules
- **What it does:** Import hashed class names from `.module.css` or `.module.scss` files into Rust as string constants at compile time. Bundles all CSS module files into a single output file with transformed (hashed) class names. CLI + library.
- **Requires Node.js?** No. Rust-native. Supports SCSS via integration (requires external Sass binary for `.module.scss` files).
- **Tailwind v4 compatible?** Compatible in principle — generates class strings that can contain any CSS. But designed for custom CSS files, not Tailwind utilities. No conflict resolution.
- **Last commit:** November 24, 2025 (v0.7.4)
- **Notes:** Actively maintained. Framework-agnostic — works with any Rust web framework. If SCSS input is used, needs a Sass compiler (not bundled). For pure CSS modules without SCSS, no external deps. Leptos integration would be manual (import class constants into view! macro).

---

### turf

- **URL:** https://github.com/myFavShrimp/turf
- **crates.io:** https://crates.io/crates/turf
- **Leptos dep in Cargo.toml:** None.
- **Category:** CSS-in-Rust (SCSS → CSS at compile time)
- **What it does:** Macro-based compile-time SCSS transpilation using `grass` (pure Rust SCSS compiler) + CSS minification via `lightningcss`. Generates unique, hashed class names. Embeds final CSS into the binary.
- **Requires Node.js?** No. Pure Rust pipeline: grass (SCSS) + lightningcss (optimization).
- **Tailwind v4 compatible?** N/A — competes with Tailwind rather than composing with it. Compiles SCSS, not Tailwind utilities.
- **Last commit:** May 2, 2025 (v0.10.1)
- **Notes:** No Leptos dependency — framework-agnostic. A leptos-example exists in the repo. Pulls in `grass` (SCSS compiler) and `lightningcss` as heavy proc-macro dependencies — significant compile-time cost. Not relevant for a Tailwind v4 project since you'd be running two CSS pipelines.

---

### Stylers

- **URL:** https://github.com/abishekatp/stylers
- **crates.io:** https://crates.io/crates/stylers
- **Leptos dep in Cargo.toml:** Not a direct dependency (leptos is a keyword only).
- **Category:** CSS-in-Rust (compile-time scoped CSS extraction)
- **What it does:** Proc macro that extracts CSS written inline in Leptos `view!` macros, scopes it with hashed class names, and generates a separate CSS file at compile time.
- **Requires Node.js?** No
- **Tailwind v4 compatible?** N/A — competes with Tailwind. Not designed for Tailwind class composition.
- **Last commit:** September 2023 (latest published version: 1.0.0-alpha, September 26, 2023)
- **Notes:** DEAD. Last release is an alpha from 2023. README claims Leptos 0.4+ compatibility but no 0.8 updates. Do not use.

---

### Styled

- **URL:** https://github.com/eboody/styled
- **crates.io:** https://crates.io/crates/styled
- **Leptos dep in Cargo.toml:** Not a direct dependency. Uses `stylist` 0.13.0 (CSS-in-JS runtime for Rust).
- **Category:** CSS-in-Rust (runtime scoped styles via stylist)
- **What it does:** Runtime CSS scoping for Leptos components using the `stylist` crate. Styles are injected into `<style>` tags at runtime with hashed class names.
- **Requires Node.js?** No
- **Tailwind v4 compatible?** N/A — runtime CSS injection conflicts with Tailwind's compile-time approach.
- **Last commit:** June 25, 2025 (v0.3.2)
- **Notes:** Maintained, but no Leptos version pinned. The `stylist` dependency is a runtime CSS-in-JS approach — incompatible with this project's compile-time Tailwind v4 pipeline. Irrelevant for this stack.

---

## Category 3 — Icon Libraries

### leptos_icons + icondata

- **URL:** https://github.com/Carlosted/leptos-icons
- **crates.io:** https://crates.io/crates/leptos_icons (136,864 downloads)
- **Leptos dep in Cargo.toml:** `leptos = ">=0.8.3"`
- **Category:** Icon library (multi-pack)
- **What it does:** Unified `<Icon>` component for Leptos that renders SVG icons from any of the `icondata`-supported icon sets (Font Awesome, Material, Bootstrap Icons, Lucide, Phosphor, Tabler, Heroicons, Feather, Remix, Simple Icons, and many more). Tree-shaking via feature flags per icon pack.
- **Requires Node.js?** No. Pure Rust + inline SVG.
- **Tailwind v4 compatible?** Yes — outputs SVG elements, class styling applied normally.
- **Last commit:** January 4, 2026 (v0.7.1 per crates.io; `rust-version = "1.88"`)
- **Notes:** Most comprehensive icon solution for Leptos. `icondata` (211k downloads) is the data layer, `leptos_icons` is the component layer. Both targeting Leptos 0.8. Actively maintained. Feature flags control which icon packs are compiled in. This is the canonical choice for multi-pack icon needs.

---

### phosphor-leptos

- **URL:** https://github.com/SorenHolstHansen/phosphor-leptos
- **crates.io:** https://crates.io/crates/phosphor-leptos (33,359 downloads)
- **Leptos dep in Cargo.toml:** `leptos = "0.8"`
- **Category:** Icon library (Phosphor Icons only)
- **What it does:** Leptos wrapper for the Phosphor icon family (6 styles: regular, bold, duotone, fill, light, thin). 19 feature-flagged categories. Each icon is a Leptos component.
- **Requires Node.js?** No. Inline SVG.
- **Tailwind v4 compatible?** Yes.
- **Last commit:** May 9, 2025 (v0.8.0)
- **Notes:** Leptos 0.8 explicit. Single icon family — elegant if Phosphor is the chosen set. Less flexible than leptos_icons for mixed icon needs.

---

### lucide-leptos (Rust Lucide)

- **URL:** https://github.com/RustForWeb/lucide
  Package path: `packages/leptos/`
- **crates.io:** https://crates.io/crates/lucide-leptos (25,871 downloads)
- **Leptos dep in Cargo.toml:** `leptos = "0.8.0"` (workspace root)
- **Category:** Icon library (Lucide only)
- **What it does:** Leptos port of Lucide icons (beautiful, consistent, MIT-licensed). Feature flags per icon category (`accessibility`, `animals`, `arrows`, `brands`, etc.) with `all-icons` meta-feature. Part of the RustForWeb org.
- **Requires Node.js?** No. Inline SVG.
- **Tailwind v4 compatible?** Yes.
- **Last commit:** March 4, 2026 (v2.577.0 — version tracks Lucide upstream)
- **Notes:** Leptos 0.8.0 pinned. Actively maintained. Version number tracks Lucide release (2.577 icons). Clean RustForWeb organization with consistent quality across ports. Also available via `leptos_icons` + `icondata`.

---

### leptos-remix-icon

- **URL:** https://github.com/opeolluwa/leptos-remix-icon
- **crates.io:** https://crates.io/crates/leptos-remix-icon (5,054 downloads)
- **Leptos dep in Cargo.toml:** `leptos = "^0.6.11"` — Leptos 0.6 only
- **Category:** Icon library (Remix Icons only)
- **What it does:** Loads Remix Icons via CDN `<link>` in `index.html`, then provides a `<Icon icon="name">` component. Requires Remix CDN — not self-contained SVGs.
- **Requires Node.js?** No, but requires CDN connectivity (external network dependency).
- **Tailwind v4 compatible?** N/A — font-icon approach via CDN.
- **Last commit:** May 2024 (14 commits total)
- **Notes:** INCOMPATIBLE — targets Leptos 0.6.x. CDN dependency contradicts self-hosted asset requirements. Dead project relative to this stack. Also listed in awesome-leptos but should be skipped.

---

## Category 4 — Theming (dark mode, design tokens)

### leptos_darkmode

- **URL:** https://gitlab.com/kerkmann/leptos_darkmode
- **crates.io:** https://crates.io/crates/leptos_darkmode (12,100 downloads)
- **Leptos dep in Cargo.toml:** `leptos = "0.8"` (default-features = false)
- **Category:** Theming (dark mode state management)
- **What it does:** Helper for managing Tailwind CSS dark mode in Leptos. Handles system preference detection (`prefers-color-scheme`) and manual toggle persistence via `localStorage`. Uses `web-sys` for MediaQueryList + Storage APIs.
- **Requires Node.js?** No
- **Tailwind v4 compatible?** Designed for Tailwind's `dark:` variant class system. Tailwind v4 dark mode works via CSS `prefers-color-scheme` or `[data-theme]` attribute — this crate manages the attribute/class toggling. Likely compatible but untested with v4 specifically.
- **Last commit:** June 23, 2025 (v0.4.0)
- **Notes:** Leptos 0.8 explicit. Actively maintained. Uses `web-sys` (WASM-safe). Only needed if adding a dark mode toggle UI — this project uses system preference only (`prefers-color-scheme: dark`) via pure CSS, no JS needed, so this crate has no value here.

---

### leptos_theme (archived candidate)

- **URL:** https://github.com/friendlymatthew/leptos-theme
- **crates.io:** https://crates.io/crates/leptos_theme (5,769 downloads)
- **Leptos dep in Cargo.toml:** `leptos = "0.6.5"` — Leptos 0.6 only
- **Category:** Theming
- **What it does:** Theme abstraction for Leptos with system/manual dark mode toggle.
- **Requires Node.js?** No
- **Tailwind v4 compatible?** N/A — Leptos 0.6 only
- **Last commit:** February 2024
- **Notes:** INCOMPATIBLE — Leptos 0.6. Dead relative to this stack.

---

## Additional Candidates Evaluated (Rejected)

| Crate | Reason rejected |
|-------|----------------|
| `tailwind-rs-leptos` (cloud-shuttle) | Builder-pattern Tailwind utilities generating class strings at runtime; framework-invasive approach; incompatible with `cargo-leptos` + standalone Tailwind v4 binary pipeline |
| `leptab` | Leptos table component with Tailwind styling; functional but targets older Leptos (last update May 2024, no 0.8 confirmation) |
| `leptos_twelements` | Tailwind Elements wrapper; last updated October 2023, Leptos 0.4 era. Dead. |
| `Stylers` | Compile-time scoped CSS; dead since September 2023 alpha |
| `hobo_css` | Separate Rust frontend framework, not Leptos |
| `leptodon` | Flowbite component library; requires checking Leptos version but out of scope for styling primitives |
| `sabry` | SCSS-in-Rust, framework-agnostic, interesting but competes with Tailwind v4 pipeline |

---

## Landscape Summary

### Class Management

Two viable options exist:

**`tailwind_fuse`** (47k downloads, Jan 2025) is the established choice with a JavaScript-ecosystem parity API (`tw_join!`/`tw_merge!` + CVA variants). No Leptos dependency. Framework-agnostic. However, **last commit was June 2024** and **no Tailwind v4 explicit support**.

**`tw_merge`** (13k downloads, March 2026) is newer, actively maintained, and explicitly targets **Tailwind CSS v4**. No Leptos dependency — pure class string manipulation. Part of the rust-ui organization which also maintains `leptos_ui`. Aligns precisely with this project's stack.

Neither is necessary for this project's current component surface (CSS custom property variant pattern via `data-*` attributes satisfies all current use cases without class merging). If dynamic class composition is ever needed, prefer `tw_merge` for v4 alignment.

### CSS-in-Rust

**None are applicable** for this project:
- `Stylance` (CSS modules) and `turf` (SCSS→CSS compile-time) introduce parallel CSS pipelines alongside Tailwind v4 — redundant complexity.
- `Stylers` is dead (2023 alpha).
- `Styled` uses runtime CSS injection (stylist) — incompatible with Tailwind's compile-time approach.

The project's existing `@layer components` approach in `style/tailwind.css` is superior to any of these for the current scale.

### Icon Libraries

**`leptos_icons` + `icondata`** is the clear winner for multi-pack needs: Leptos `>=0.8.3`, 136k downloads, actively maintained, covers virtually all major icon sets via feature flags.

**`phosphor-leptos`** is the alternative if committing to a single Phosphor-only icon family: Leptos 0.8 explicit, clean API, 33k downloads.

**`lucide-leptos`** is the RustForWeb-quality choice for Lucide specifically: Leptos 0.8.0, tracks upstream Lucide releases, March 2026 update.

**`leptos-remix-icon`** is dead and incompatible (Leptos 0.6, CDN-dependent).

This project currently has no icon components, so none are immediately needed.

### Theming

**`leptos_darkmode`** is the only viable option (Leptos 0.8, actively maintained). However, this project implements dark mode via pure CSS `prefers-color-scheme: dark` in `style/tailwind.css` — no JavaScript state management needed, no toggle UI. This crate provides no value for the current architecture.

`leptos_theme` is dead (Leptos 0.6).

### Net verdict for this project

| Category | Recommendation |
|----------|---------------|
| Class merging | `tw_merge` if dynamic class composition is needed (currently: not needed) |
| CSS modules | None — existing `@layer components` is sufficient |
| Icons | `leptos_icons` + `icondata` when/if icons are needed |
| Dark mode | None — pure CSS solution already in place |
