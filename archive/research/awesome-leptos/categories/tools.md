# Tools — Deep Dive

## Summary

The Tools category covers six essential build, formatting, and IDE integrations for Leptos development. The ecosystem is anchored by cargo-leptos (the official build orchestrator), complemented by leptosfmt for view! macro formatting, CI automation, and VSCode extensions. These tools form the developer experience layer and are generally well-maintained, with active upstreams and compatibility with current Leptos versions.

---

## Per-Tool Analysis

### cargo-leptos

- **URL:** https://github.com/leptos-rs/cargo-leptos
- **Stars/Activity:** 510 stars, 173 forks, 36 open issues, active (main branch updated regularly)
- **Leptos version:** 0.8+ (no explicit version constraint in docs, but actively maintained by leptos-rs org)
- **What it does:** Official build orchestrator for Leptos. Coordinates parallel compilation of server (Rust/Axum) and client (WASM) sides, provides hot-reload for CSS, SCSS compilation via dart-sass, WASM optimization via Binaryen, supports both single-package and workspace layouts.
- **Relevance to this project:** **HIGH** — This project is already using cargo-leptos as the primary build tool. It is essential infrastructure.
- **Adoption recommendation:** **ALREADY IN USE** — Foundational tool. No action needed. Well-designed and stable.

---

### leptosfmt

- **URL:** https://github.com/bram209/leptosfmt
- **Stars/Activity:** 348 stars, 36 forks, 20 releases, latest v0.1.33 (Jan 30, 2025)
- **Leptos version:** No explicit version constraint; supports `leptos::view` and `view` macros generically
- **What it does:** Formatter for Leptos `view!` macros. Configurable line width, indentation, and formatting style. Supports Tailwind CSS class formatting. Can pipe output through rustfmt. Integrates with Rust Analyzer.
- **Relevance to this project:** **MEDIUM** — Useful for consistency in `view!` macro formatting. Project currently has no explicit formatter for view macros (relies on rustfmt for Rust code). Adding leptosfmt would improve readability of component templates.
- **Adoption recommendation:** **EVALUATE** — Consider for local development workflow (via VSCode extension below), but not a blocker. The project's view! formatting is already readable. Useful as a linter in CI if you want to enforce consistent view macro style.

**Note:** Known limitation — does not support non-doc comments in code blocks due to parser limitations.

---

### leptos-fmt (VSCode plugin)

- **URL:** https://github.com/codeitlikemiley/leptos-fmt
- **Stars/Activity:** 4 stars, 40 commits, latest v0.1.2 (Dec 13, 2024)
- **Leptos version:** No explicit version constraint
- **What it does:** VSCode extension that integrates leptosfmt into the editor. Provides `Leptos Init` command to configure workspace settings, `Format with Leptosfmt` command for manual formatting, custom keybinding support, configurable binary paths.
- **Relevance to this project:** **MEDIUM** — Depends on adoption of leptosfmt above. Useful for IDE-integrated formatting if leptosfmt is adopted.
- **Adoption recommendation:** **CONDITIONAL** — Only adopt if leptosfmt is adopted. Provides convenient IDE integration. Stars and activity are low, but it's a thin wrapper around leptosfmt so stability is reasonable.

---

### leptosfmt-action

- **URL:** https://github.com/LesnyRumcajs/leptosfmt-action
- **Stars/Activity:** 2 stars, 27 commits, latest v0.1.0 (Jan 26, 2025)
- **Leptos version:** N/A (GitHub Action)
- **What it does:** GitHub Actions CI integration for leptosfmt. Downloads and runs leptosfmt on your repo, with configurable args, version pinning, failure modes, and debug output. Provides exit code for conditional workflow steps.
- **Relevance to this project:** **MEDIUM** — Useful for CI if leptosfmt is adopted. Automates formatting checks on PRs.
- **Adoption recommendation:** **CONDITIONAL** — Only adopt if leptosfmt is adopted and you want to enforce formatting in CI. Currently the project uses `just check` which runs `cargo clippy` and tests. Could add a leptosfmt check step if desired.

---

### cargo-runner (VSCode plugin)

- **URL:** https://github.com/codeitlikemiley/cargo-runner
- **Stars/Activity:** 6 stars, 282 commits, latest v1.5.4 (Nov 24, 2024)
- **Leptos version:** N/A (VSCode extension for general Rust)
- **What it does:** VSCode extension providing CodeLens buttons and keyboard shortcuts to run/debug/test Rust code. Configurable via `Ctrl+Shift+R` for quick overrides. Supports features, toolchain channels, environment variables, and nextest.
- **Relevance to this project:** **LOW** — Useful general development QoL but not Leptos-specific. This project already uses `just` for task running (`just dev`, `just test`, `just e2e`).
- **Adoption recommendation:** **SKIP** — The project's task runner (`justfile`) is explicit and clear. This VSCode extension would duplicate that layer. The `bacon` continuous linter (already in use) provides better real-time feedback than CodeLens buttons.

---

### vscode-leptos-snippets

- **URL:** https://github.com/mondeja/vscode-leptos-snippets
- **Stars/Activity:** 5 stars, 24 commits, latest v0.1.2 (May 30, 2025)
- **Leptos version:** N/A (VSCode snippets)
- **What it does:** VSCode extension providing code snippets for Leptos development. Includes triggers for `#[component]`, `#[server]`, Router, App setup, counter examples, etc.
- **Relevance to this project:** **LOW** — Useful for boilerplate generation but the project has established patterns (Leptos idioms guide, existing components as templates). Snippets are helpful for new developers but not essential.
- **Adoption recommendation:** **SKIP** — Low priority. The project's guidance documents and existing component code serve as better templates than snippets. Can be revisited if onboarding new developers becomes common.

---

## Category Verdict

### Top Picks

1. **cargo-leptos** — Already in use. Essential infrastructure. No action needed.

2. **leptosfmt** — Evaluate for adoption. Would improve consistency in `view!` macro formatting. Relatively lightweight. Consider adding to local development workflow and as a CI check if code style enforcement becomes important.

3. **leptos-fmt + leptosfmt-action** — Conditional on leptosfmt adoption. Provides IDE integration and CI automation, both useful if leptosfmt is adopted.

### Not Recommended

- **cargo-runner** — Redundant with the project's task runner (justfile) and bacon continuous linter.
- **vscode-leptos-snippets** — Nice-to-have but not essential. The project's guidance docs and existing code patterns are sufficient.

### Summary Recommendation

**This project's tooling is well-configured.** cargo-leptos is the right choice for build orchestration. If code style consistency becomes a priority, consider adding leptosfmt + its VSCode extension to the development workflow and CI pipeline. The other tools add marginal value given the project's existing patterns and tooling.
