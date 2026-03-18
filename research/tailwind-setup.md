# Tailwind CSS v4 Setup — Agent Reference

Native-binary, zero-Node integration with cargo-leptos. Covers everything an implementing agent needs to go from nothing to a working Tailwind build.

---

## Table of Contents

1. [The Native Binary](#the-native-binary)
2. [cargo-leptos Integration Model](#cargo-leptos-integration-model)
3. [Tailwind v3 vs v4 — Which to Use](#tailwind-v3-vs-v4--which-to-use)
4. [Minimal Working Setup](#minimal-working-setup)
5. [v4 CSS-First Configuration](#v4-css-first-configuration)
6. [Content Detection and Rust/.rs Files](#content-detection-and-rusts-files)
7. [Dynamic Classes and Safelisting](#dynamic-classes-and-safelisting)
8. [Leptos view! Class Patterns](#leptos-view-class-patterns)
9. [Development Workflow](#development-workflow)
10. [CSS Output Pipeline](#css-output-pipeline)
11. [Banned Patterns](#banned-patterns)
12. [Troubleshooting](#troubleshooting)

---

## The Native Binary

Tailwind ships a **standalone CLI binary** — a self-contained executable with no Node.js or npm runtime dependency. It is functionally identical to the npm-installed CLI.

### What It Is

The standalone binary is a single file downloaded from the Tailwind GitHub releases page. It contains an embedded JS runtime (via Bun) and needs nothing else installed on the system.

### Binary Names by Platform

As of v4.2.1 (the current cargo-leptos default), the release assets are named:

| Platform | Binary Name |
|----------|-------------|
| macOS Apple Silicon | `tailwindcss-macos-arm64` |
| macOS Intel | `tailwindcss-macos-x64` |
| Linux x64 (glibc) | `tailwindcss-linux-x64` |
| Linux x64 (musl) | `tailwindcss-linux-x64-musl` |
| Linux arm64 (glibc) | `tailwindcss-linux-arm64` |
| Linux arm64 (musl) | `tailwindcss-linux-arm64-musl` |
| Windows | `tailwindcss-windows-x64.exe` |

Base download URL: `https://github.com/tailwindlabs/tailwindcss/releases/download/v4.2.1/<binary-name>`

### Manual Install (if needed)

```bash
# macOS Apple Silicon
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/download/v4.2.1/tailwindcss-macos-arm64
chmod +x tailwindcss-macos-arm64
mv tailwindcss-macos-arm64 /usr/local/bin/tailwindcss
```

**You do not need to do this manually.** cargo-leptos handles binary acquisition automatically (see next section).

---

## cargo-leptos Integration Model

### How It Works — No Manual Binary Management Required

cargo-leptos manages the Tailwind binary as a first-class tool dependency, alongside Sass, wasm-opt, and wasm-bindgen. The process is fully automatic:

1. At build time, cargo-leptos checks for `tailwindcss` via `which tailwindcss` on `PATH`.
2. If not found globally, it downloads the correct platform binary from GitHub releases and caches it.
3. It invokes the binary with `--input <tailwind-input-file> --output <tmp-file>`, optionally `--minify` in release mode.
4. The output CSS is merged with `style-file` output (if any) and processed by Lightning CSS.

### Activation

Tailwind is **only activated when `tailwind-input-file` is set** in `[package.metadata.leptos]`. If it is absent, no Tailwind build runs.

### Hardcoded Default Version

cargo-leptos hardcodes `v4.2.1` as the default Tailwind version. Override with the env var:

```
LEPTOS_TAILWIND_VERSION=v4.2.1  # or any other semver tag
```

The version controls which release binary is downloaded. Set this in `.env` or your CI environment if you need a specific version.

### Binary Name Expected

cargo-leptos invokes the binary as `tailwindcss` (via `which`) or downloads it using the platform-specific name from the releases URL. The binary you install globally must be named `tailwindcss`.

### v4 Config File Behavior

With v4, cargo-leptos **does not require and does not generate** `tailwind.config.js`. If `tailwind-config-file` is absent and `tailwind.config.js` does not exist, cargo-leptos skips the `--config` argument entirely — which is correct v4 behavior. A warning is logged if either the config field or the default JS file is detected, informing you that JS config is no longer required in v4.

### CSS Output Location

The compiled Tailwind CSS is written to `<site-root>/<site-pkg-dir>/<output-name>.css`. For this project: `target/site/pkg/samete.css`. This file is served as a static asset alongside the WASM bundle.

---

## Tailwind v3 vs v4 — Which to Use

**Use v4.** cargo-leptos defaults to v4.2.1 (as of 2026-03). Starting a new project on v3 requires explicit `LEPTOS_TAILWIND_VERSION` override and a JS config file — extra work with no benefit.

Key v4 advantages relevant to Leptos projects:

| Feature | v3 | v4 |
|---------|----|----|
| Config file | Required `tailwind.config.js` | No JS config needed |
| CSS import | Three `@tailwind` directives | Single `@import "tailwindcss"` |
| Content detection | Must configure `content` array | Auto-detects from project root |
| Safelisting | `safelist` in JS config | `@source inline(...)` in CSS |
| Design tokens | JS theme object | `@theme { --token: value }` in CSS |
| Build speed | Baseline | 3-8x faster |

---

## Minimal Working Setup

### 1. Cargo.toml additions

In `[package.metadata.leptos]`:

```toml
[package.metadata.leptos]
# ... existing fields ...
tailwind-input-file = "style/tailwind.css"
# tailwind-config-file is NOT needed for v4
```

The `style-file` field (`style/main.css`) can coexist. Both are compiled and concatenated: `style-file` output first, then Tailwind output.

### 2. CSS Entry File

Create `style/tailwind.css`:

```css
@import "tailwindcss";
```

That is the complete minimal input file. No `@tailwind base`, `@tailwind components`, `@tailwind utilities` — those are v3 directives. v4 uses a single import.

### 3. No Other Files Needed

- No `tailwind.config.js`
- No `postcss.config.js`
- No `package.json`
- No `.npmrc`

The binary is downloaded automatically on first `cargo leptos build` or `cargo leptos watch`.

---

## v4 CSS-First Configuration

### Design Tokens via @theme

All design customization lives in the CSS file. The `@theme` block defines CSS custom properties that become utility classes:

```css
@import "tailwindcss";

@theme {
  /* Colors — creates text-brand, bg-brand, border-brand, etc. */
  --color-brand: oklch(0.60 0.20 250);
  --color-brand-light: oklch(0.80 0.12 250);

  /* Typography */
  --font-sans: "Inter", ui-sans-serif, system-ui, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;

  /* Custom breakpoint */
  --breakpoint-xs: 30rem;

  /* Spacing override */
  --spacing-18: 4.5rem;
}
```

These become CSS custom properties on `:root` and generate utility classes:
- `--color-brand` → `text-brand`, `bg-brand`, `border-brand`, `ring-brand`
- `--breakpoint-xs` → `xs:flex`, `xs:hidden`, etc.

### Custom Utilities via @utility

```css
@utility scrollbar-hidden {
  scrollbar-width: none;
  &::-webkit-scrollbar { display: none; }
}
```

This utility works with all variants: `hover:scrollbar-hidden`, `lg:scrollbar-hidden`.

### Custom Variants via @custom-variant

```css
/* Target elements when a [data-theme="dark"] ancestor is present */
@custom-variant dark (&:where([data-theme="dark"] *));
```

Enables: `dark:bg-gray-900`, `dark:text-white`.

---

## Content Detection and Rust/.rs Files

### What v4 Auto-Detects

v4's auto-detection scans all files in the project **except**:
- Files listed in `.gitignore`
- `node_modules/`
- Binary files (images, videos, archives)
- CSS files themselves
- Package manager lock files

Rust `.rs` files are **not explicitly excluded** and would be scanned if they were in a location v4 considers "the project". However, in a cargo-leptos setup, the CSS file lives in `style/` and the scanner starts from the CSS file's directory or the configured source root.

**Do not rely on auto-detection for Rust sources.** Be explicit.

### Explicit Source Configuration — Required for Leptos

Add `@source` directives pointing at your Rust sources:

```css
@import "tailwindcss";

/* Scan all Rust source files */
@source "../src";

/* Also scan HTML templates if any exist */
@source "../index.html";
```

The path is relative to the CSS input file (`style/tailwind.css`), so `../src` resolves to the project root's `src/` directory.

### Verifying Detection

Run the binary directly to test what classes are found:

```bash
tailwindcss --input style/tailwind.css --output /tmp/tw-test.css
```

Inspect the output to confirm utility classes from your Rust files appear.

### The Leptos class: Transform — v3 Only

In v3, the community used a `transform` function in `tailwind.config.js` to strip `class:` prefixes from Leptos's conditional class directive syntax:

```js
// v3 only — NOT needed in v4
transform: {
  rs: (content) => content.replace(/(?:^|\s)class:/g, ' '),
}
```

In v4 there is no JS config and therefore no transform hook. The scanner reads source files as plain text. This works correctly because:

- `class="flex items-center"` — scanned as-is, classes detected
- `class:hidden=move || !visible.get()` — Tailwind sees the literal string `hidden`, detects the class
- `class=("text-red-500", condition)` — Tailwind sees `text-red-500`, detects the class

The scanner is intentionally simple: it finds any token that matches a known utility pattern regardless of surrounding syntax. The `class:` prefix in Leptos is handled naturally because `hidden` (the class name) appears as a discrete token in the file.

---

## Dynamic Classes and Safelisting

### The Fundamental Constraint

Tailwind's scanner reads source files as plain text. It cannot evaluate Rust code at scan time. **Any class name that is constructed at runtime rather than written as a literal string will not be detected.**

```rust
// WRONG — "red" and "green" appear in source but "text-red-500" does not
let color = if error { "red" } else { "green" };
view! { <div class=format!("text-{}-500", color)></div> }

// CORRECT — both full class names appear as literals in the source
let class = if error { "text-red-500" } else { "text-green-500" };
view! { <div class=class></div> }
```

### Safelisting via @source inline()

When you cannot write class names as literals (e.g., they come from a database, config, or match arms returning strings computed elsewhere), use `@source inline()` to force generation:

```css
@import "tailwindcss";
@source "../src";

/* Force-generate specific classes regardless of scan results */
@source inline("underline line-through");

/* With variants — generates hover:text-red-500 and focus:text-red-500 */
@source inline("{hover:,focus:,}text-red-500");

/* Brace expansion for color scales */
@source inline("bg-red-{50,100,200,300,400,500,600,700,800,900,950}");
@source inline("bg-blue-{50,100,200,300,400,500,600,700,800,900,950}");

/* Responsive variants of a status set */
@source inline("{,sm:,lg:}{text-green-600,text-yellow-600,text-red-600}");
```

### When to Safelist vs When to Refactor

Prefer writing full literal class names in Rust source — it is the correct pattern and requires no safelist maintenance. Use `@source inline()` only when:
1. Class names come from external data (database-driven theming, user preferences)
2. A library generates class names you cannot control
3. A match arm returns a pre-computed string that contains full class names — in this case the class names do appear as literals in the match arm bodies, so no safelist is needed

---

## Leptos view! Class Patterns

### Static Classes — Always Detected

```rust
view! {
    <div class="flex items-center gap-4 rounded-lg bg-white p-6 shadow-md">
        <p class="text-sm text-gray-600">Content</p>
    </div>
}
```

The scanner reads the string literal. All classes are detected.

### Conditional Classes — Detected if Full Names Are Literals

```rust
// CORRECT — both class names are literal strings in source
let status_class = if active { "bg-green-100 text-green-800" } else { "bg-gray-100 text-gray-600" };
view! { <span class=status_class>Status</span> }

// CORRECT — class: directive with a literal class name
view! { <div class:hidden=move || !visible.get()>Content</div> }

// CORRECT — class tuple with literal name
view! { <div class=("ring-2 ring-offset-2 ring-brand", move || focused.get())>Field</div> }
```

### Multiple Class: Directives

```rust
view! {
    <button
        class="px-4 py-2 rounded font-medium transition-colors"
        class:bg-brand=move || !disabled.get()
        class:bg-gray-300=move || disabled.get()
        class:text-white=move || !disabled.get()
        class:text-gray-500=move || disabled.get()
        class:cursor-not-allowed=move || disabled.get()
    >
        "Submit"
    </button>
}
```

All six class names appear as literals — all are detected.

### Dynamically Computed Classes — NOT Detected

```rust
// WRONG — "sm", "lg", "xl" appear in source but "text-sm", "text-lg", "text-xl" do not
let size_class = format!("text-{}", size_variant);

// FIX — use a lookup table
let size_class = match size_variant.as_str() {
    "sm" => "text-sm",
    "lg" => "text-lg",
    "xl" => "text-xl",
    _ => "text-base",
};
```

### The [&_*] Arbitrary Variant

For styling Leptos-generated markup (e.g., server-rendered HTML injected via `inner_html`):

```rust
view! {
    <div class="prose [&_h2]:text-brand [&_a]:underline [&_a:hover]:no-underline">
        <div inner_html=html_content />
    </div>
}
```

The arbitrary variant `[&_*]` applies styles to descendant elements. Tailwind's scanner detects the full string including the brackets.

---

## Development Workflow

### cargo-leptos Handles Everything

`cargo leptos watch` (or `just dev`) runs the full pipeline:
1. Downloads Tailwind binary if not cached or on `PATH`
2. Invokes `tailwindcss --input style/tailwind.css --output <tmp>` on file changes to `*.rs` or the CSS input file
3. Concatenates with `style-file` output
4. Runs Lightning CSS for autoprefixing
5. Hot-reloads via the reload port

**No separate watcher process is needed.** Do not run `tailwindcss --watch` alongside `cargo leptos watch` — that would create a race condition on the output file.

### Triggering a CSS Rebuild

Changes that trigger a Tailwind rebuild:
- Any `*.rs` file change (cargo-leptos watches source dirs)
- Changes to `style/tailwind.css` (the input file)
- Changes to `style/main.css` (the `style-file`)

### Release Build

`cargo leptos build --release` passes `--minify` to the Tailwind binary, then runs Lightning CSS minification on the combined output.

### Justfile Integration

Add to `justfile` only if you need to run Tailwind outside of cargo-leptos (e.g., for standalone CSS inspection):

```just
# Standalone Tailwind run (debugging only — not needed in normal dev)
tw-check:
    tailwindcss --input style/tailwind.css --output /tmp/tw-check.css
    wc -c /tmp/tw-check.css
```

For normal development, `just dev` (`cargo leptos watch`) is all you need.

---

## CSS Output Pipeline

The full pipeline when both `style-file` and `tailwind-input-file` are configured:

```
style/main.css          ──┐
                           ├─► concat (style-file + tailwind output)
style/tailwind.css      ──┤                │
  │ (tailwindcss binary)  │       Lightning CSS (autoprefixer, minify in release)
  └─► tmp tailwind output ┘                │
                                           ▼
                              target/site/pkg/samete.css
```

Key implementation details (from `src/compile/style.rs`):
- Both builds run concurrently in separate tokio tasks
- The `style-file` CSS comes first in the concatenated output, then Tailwind
- If either build fails, the overall build fails (no partial output)
- Lightning CSS parser errors fall back gracefully — the raw CSS is used without optimization

### Relationship Between style-file and tailwind-input-file

They are independent inputs that get concatenated. Use this to your advantage:

- `style/main.css` — hand-written CSS, base resets, component overrides using `@apply`, `@layer`
- `style/tailwind.css` — the `@import "tailwindcss"` entry point plus `@theme`, `@source`, `@source inline()` directives

You can also put everything in one file. If you set `tailwind-input-file` but not `style-file`, only the Tailwind output is served. If you set only `style-file` to a `.css` file, it is served as-is without Tailwind processing.

---

## Banned Patterns

### Do Not Use tailwind.config.js for v4

The JS config file is a v3 concept. In v4 it is a compatibility shim with known limitations (`corePlugins`, `safelist`, `separator` are not supported). cargo-leptos will log a warning if it finds one. Use CSS-first config exclusively.

### Do Not Run tailwindcss --watch Separately

Running the Tailwind watcher in parallel with `cargo leptos watch` creates a write race on the output file. cargo-leptos owns the Tailwind build lifecycle.

### Do Not Construct Class Names via format!() or String Concatenation

```rust
// BANNED — classes not detectable by scanner
let color_class = format!("bg-{}-500", color_name);
```

Use a static lookup (match or array indexing) that produces full literal class names visible in source.

### Do Not Use Node-Dependent Install Methods

```bash
# BANNED — introduces Node/npm dependency
npm install tailwindcss
npx tailwindcss --input ...
```

Use the standalone binary or let cargo-leptos download it.

### Do Not Set tailwind-config-file Without tailwind-input-file

cargo-leptos will error: `tailwind-input-file is required when using tailwind-config-file`. There is no config-only mode.

---

## Troubleshooting

### Classes Not Appearing in Output

1. Check that `@source "../src"` is in your CSS input file — v4 auto-detection may not reach `src/` from `style/tailwind.css`.
2. Verify the class name appears as a complete literal in a `.rs` file (not constructed by format!).
3. Run `tailwindcss --input style/tailwind.css --output /tmp/check.css` and inspect the output.
4. If the binary is cached from a previous version: delete the cargo-leptos tool cache (`~/.cache/cargo-leptos` or equivalent) and let it re-download.

### Tailwind Binary Not Downloaded

If the build fails with "tailwindcss is required but was not found":
- If running in a network-restricted environment, install the binary manually (see [Manual Install](#the-native-binary)) and ensure `tailwindcss` is on `PATH`.
- cargo-leptos has a `no_downloads` feature flag. Check if your cargo-leptos install was compiled with it.

### LEPTOS_TAILWIND_VERSION Mismatch

If you have a globally installed `tailwindcss` binary at a different version than the cargo-leptos default:
- cargo-leptos uses the global binary if found on `PATH`, regardless of version
- Set `LEPTOS_TAILWIND_VERSION` to match your global install, or rename the global binary to avoid the `which` match

### CSS Not Updating in Dev

If file changes aren't triggering CSS rebuilds:
- Check that `cargo leptos watch` is running (not `cargo run`)
- Verify the input file path in `Cargo.toml` matches the actual file location (path is relative to the directory containing `Cargo.toml`)
- Check for errors in the terminal — a Tailwind build failure silently produces no output update

### v4 Content Detection Misses Library Classes

If you use a Rust component library that ships pre-computed class strings:

```css
@source "../node_modules/@some/ui-lib";
/* or for a local crate workspace member: */
@source "../../ui-components/src";
```

---

## Sources Consulted

- cargo-leptos source: `src/config/tailwind.rs`, `src/config/version.rs`, `src/compile/tailwind.rs`, `src/compile/style.rs`
- Tailwind v4.2.1 GitHub release assets (binary naming confirmed via GitHub API)
- Tailwind v4 official docs: installation, directives, content detection, v3 upgrade guide
- cargo-leptos example project: `examples/project/`
