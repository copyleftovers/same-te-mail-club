# biji-ui — Source-Level Verification

Verification date: 2026-03-19. All code quoted is fetched directly from `https://github.com/biji-ui/biji-ui` at the `main` branch, which corresponds to the v0.4.4 publish on crates.io (2026-03-16).

---

## Repo & Crate Metadata

| Field | Value |
|-------|-------|
| Crate | `biji-ui` |
| Latest version | `0.4.4` (published 2026-03-16) |
| Repo | `https://github.com/biji-ui/biji-ui` |
| License | MIT |
| Total versions | 13 (v0.1.0 May 2024 → v0.4.4 Mar 2026) |
| Total downloads | 6,763 |
| Rust toolchain | nightly (pinned in `rust-toolchain.toml`) |
| Edition | 2021 |
| Source lines | ~10,492 across 90 files |

The workspace contains three members: `biji-ui` (the library), `biji-ui-docs` (documentation site), and `jspackages`.

---

## leptos-use Version Conflict

**Exact dependency line from `biji-ui/Cargo.toml`:**

```toml
leptos-use = { version = "0.16" }
```

This is a caret requirement: `^0.16`, meaning `>=0.16.0, <0.17.0`. The target project has `leptos-use = "0.18.3"` as a transitive dependency (via `leptos_i18n`).

**Can they coexist? No.**

Under Cargo's semver rules for `0.x` crates, the leading zero is treated as the major version for compatibility purposes. `^0.16` and `^0.18` are incompatible ranges — Cargo cannot unify them. Cargo resolves a single version per crate in the dependency graph; if two crates require incompatible ranges, the build fails with a version conflict error.

Verified: `leptos-use 0.16.3` and `leptos-use 0.18.3` both depend on `leptos ^0.8`, so the leptos dependency itself is not the blocker — the conflict is within leptos-use itself.

**Workaround options:**
1. Patch `biji-ui` locally (via `[patch.crates-io]`) to point at a fork with `leptos-use = "0.18"`.
2. Wait for biji-ui to publish with updated leptos-use (0.16 was current as of the March 2026 publish — 0.18 was already available at that point, so this may be a deliberate pin or an oversight).
3. Override the transitive dependency from `leptos_i18n` to `leptos-use = "0.16"` instead — but only if `leptos_i18n` has not changed its API surface against 0.17/0.18 features.

---

## Component Source Audit

### Dialog

Source: `biji-ui/src/components/dialog/dialog.rs` and `context.rs`

- **Headless:** Yes. No CSS emitted. The `Content` component accepts `class`, `show_class`, `hide_class` props and passes them verbatim to `CustomAnimatedShow`. Nothing is injected beyond structural HTML and ARIA attributes.

- **Class API:** String props only: `#[prop(into, optional)] class: String`. No CSS Modules, no `@apply`, no stylesheet injection. You write the classes.

- **ARIA (real — quoted from source):**
  ```rust
  view! {
      <CustomAnimatedShow
          // ...
          attr:role="dialog"
          attr:aria-modal="true"
      >
  ```
  The `Content` component correctly sets `role="dialog"` and `aria-modal="true"`. `Close` button and `Overlay` are separate sub-components. The trigger button itself does not set `aria-haspopup` — that is a gap compared to WAI-ARIA recommendations for dialog triggers.

- **Focus management (real, quoted from source):**
  ```rust
  let _focus_eff = RenderEffect::new(move |_| {
      if dialog_ctx.open.get() {
          let _ = leptos::leptos_dom::helpers::set_timeout_with_handle(
              move || {
                  if let Some(el) = content_ref.get() {
                      focus_first_element(&el);
                  }
              },
              std::time::Duration::from_millis(10),
          );
      }
  });

  let _ = use_event_listener(content_ref, leptos::ev::keydown, move |evt| {
      let key = evt.key();
      match key.as_str() {
          "Escape" => { dialog_ctx.close(); /* returns focus to trigger */ }
          "Tab" => { trap_tab_focus(&el, evt.shift_key(), &evt); }
          _ => {}
      }
  });
  ```
  Focus trapping is implemented via `get_focusable_elements` (queries `a[href], button:not([disabled]), textarea, input, select, [tabindex]:not([tabindex="-1"])`) with `Tab`/`Shift+Tab` wrapping. On Escape, focus returns to the trigger. This is genuine, not placeholder.

- **Leptos 0.8 API correctness:** Uses `RwSignal`, `RenderEffect`, `NodeRef<Button>`, `NodeRef<Div>`, `leptos::context::Provider`, `expect_context`, `on_cleanup` — all correct 0.8 patterns. No deprecated `create_signal`, no `cx` argument.

- **`disabled` prop:** Not applicable to Dialog/Overlay. The Close and Trigger sub-components take no `disabled` prop.

---

### Tabs

Source: `biji-ui/src/components/tabs/root.rs` and `context.rs`

- **Headless:** Yes. `data-state`, `data-orientation`, `data-disabled` attributes emitted for CSS targeting. No injected styles.

- **Class API:** `#[prop(into, optional)] class: String` on every sub-component (`Root`, `List`, `Trigger`, `Content`).

- **ARIA (real — quoted from source):**
  ```rust
  // List:
  <div role="tablist" aria-orientation={...}>

  // Trigger:
  <button
      role="tab"
      id={item.trigger_id}
      aria-selected={move || if is_selected.get() { "true" } else { "false" }}
      aria-controls={item.panel_id}
      aria-disabled={if item_ctx.disabled { Some("true") } else { None }}
      tabindex={move || /* roving tabindex logic */}
  >

  // Content:
  <div
      role="tabpanel"
      id={move || ids.get().as_ref().map(|(_, p)| p.clone())}
      aria-labelledby={move || ids.get().as_ref().map(|(t, _)| t.clone())}
      tabindex="0"
  >
  ```
  Full roving tabindex implementation. `aria-selected`, `aria-controls`, `aria-labelledby` all wired with dynamic IDs generated per root instance (`biji-tab-trigger-{root_id}-{index}`). Correct.

- **Keyboard navigation (real):** Arrow keys (horizontal/vertical), Home, End, Enter, Space all handled. Supports both `Automatic` (focus → activate) and `Manual` (focus then Enter/Space to activate) modes.

- **`disabled` prop type:** `#[prop(default = false)] disabled: bool` — static bool, not `Signal<bool>` or `MaybeSignal<bool>`. The value is baked in at component construction time. Dynamic `disabled=move || !hydrated.get()` cannot be passed directly to the Trigger prop without a wrapper.

- **Leptos 0.8 API correctness:** Uses `StoredValue`, `AtomicUsize` for ID generation, `use_event_listener` from leptos-use, `Memo::new`, `on_cleanup`. All correct.

---

### Select

Source: `biji-ui/src/components/select/root.rs`

- **Headless:** Yes. Positioning is computed via `use_element_bounding` and output as an inline `style` string. No stylesheet injected.

- **Class API:** `#[prop(into, optional)] class: String` on `Root`, `Trigger`, `Content`, `Item`. The `Content` also accepts `show_class` / `hide_class` for CSS animation.

- **ARIA (real — quoted from source):**
  ```rust
  // Trigger:
  <button
      role="combobox"
      aria-expanded={move || if ctx.open.get() { "true" } else { "false" }}
      aria-haspopup="listbox"
      aria-controls={ctx.select_id.get_value()}
  >

  // Content:
  attr:role="listbox"
  attr:tabindex="-1"

  // Item:
  <div
      role="option"
      tabindex="-1"
      aria-selected={move || if is_selected.get() { "true" } else { "false" }}
      aria-disabled={if item_ctx.disabled { Some("true") } else { None }}
      data-state={...}
      data-highlighted={...}
  >
  ```
  `combobox` + `listbox` + `option` pattern is correct per WAI-ARIA Listbox pattern. Dynamic IDs, expand/collapse tracking, selected item tracking.

- **Keyboard navigation:** Arrow keys with first/last, Enter/Space to select, Tab closes without selecting, Escape closes and returns focus to trigger. Mouse hover also sets focus (`mouseover` listener). Real implementation.

- **Collision avoidance:** Uses `use_element_bounding` from leptos-use to read trigger position, then computes fixed positioning with viewport-aware collision avoidance (`AvoidCollisions::Flip`). This is what makes it a dependency on leptos-use — it is not trivially removable.

- **`disabled` prop type on Item:** `#[prop(default = false)] disabled: bool` — static bool, same as Tabs.

- **Leptos 0.8 API correctness:** Uses `Effect::new`, `RenderEffect::new`, `Arc<Mutex<>>` for timer handles, `on_click_outside` from leptos-use. All 0.8 patterns.

---

## SSR + Hydration

**There are no `#[cfg(feature = "ssr")]` guards anywhere in the component source.** The entire library, including `prevent_scroll.rs`, `items.rs`, and all components, uses `web_sys`, `wasm_bindgen`, `document()`, `window()`, and `leptos::leptos_dom::helpers::set_timeout` without any SSR feature gates.

This is the critical finding: **biji-ui is a client-only library.** Code like:

```rust
// prevent_scroll.rs
if let Some(doc) = document().body() {
    let client_width = f64::from(doc.client_width());
    let inner_width = window().inner_width().unwrap().as_f64()...
```

...will panic or fail at compile time under SSR (`--features ssr`) because `document()` and `window()` are WASM-only APIs. `leptos_dom::helpers::set_timeout_with_handle` is also WASM-only.

The `CustomAnimatedShow` component (no SSR cfg guards) and `use_event_listener` usage are client-only patterns throughout.

**Implication for cargo-leptos dual compilation:** cargo-leptos compiles the codebase twice — once with `--features ssr` (server binary) and once with `--features hydrate` (WASM bundle). biji-ui will fail the SSR compilation pass unless the integration wraps every biji-ui usage in `#[cfg(feature = "hydrate")]` or the components are only rendered client-side (not in SSR path).

In Leptos 0.8, the idiomatic fix is to guard client-only code inside `Effect::new` or `create_effect` (which only run on the client), or to use `#[cfg(not(feature = "ssr"))]`. biji-ui does not do this internally — the `RenderEffect` and `use_event_listener` calls are at the top level of component bodies, which run during SSR.

Note: leptos-use's `use_event_listener` and `on_click_outside` have their own SSR guards internally, so those calls may be safe. But the direct `document().body()`, `window().inner_width()`, and `set_timeout_with_handle` calls in `prevent_scroll.rs` and Select's focus logic will not compile under SSR.

**Verdict on SSR:** Not SSR-safe as shipped. Would require either a fork with `#[cfg]` guards added, or all biji-ui components wrapped to only render client-side.

---

## Reactive Props (disabled, open, etc.)

All prop types verified from source:

| Prop | Component | Type | Notes |
|------|-----------|------|-------|
| `disabled` | Tabs::Trigger | `bool` | Static. Baked in at component construction. |
| `disabled` | Select::Item | `bool` | Static. |
| `open` | Dialog | `RwSignal<bool>` in context | Not a prop on Root — state is internal. |
| `class` | All components | `String` (via `#[prop(into, optional)]`) | Accepts `&str`, `String`, `Cow<str>`, anything `Into<String>`. |
| `value` | Tabs::Root | `Option<String>` | Initial value only; not a reactive signal. |
| `value` | Select::Root | `Option<String>` | Initial value only; not a reactive signal. |
| `on_value_change` | Tabs::Root, Select::Root | `Option<Callback<String>>` | Callback fired on selection, but the component owns state internally. |
| `prevent_scroll` | Dialog::Root | `bool` | Static. |

**The `disabled=move || !hydrated.get()` pattern used in this project is incompatible with biji-ui's prop signature.** biji-ui's Trigger and Item components take `bool`, not `Signal<bool>` or `MaybeSignal<bool>`. To pass a reactive disabled value, you would need to either:
1. Fork biji-ui to change prop types to `MaybeSignal<bool>`.
2. Wrap biji-ui components in a reactive outer component that re-mounts them when `hydrated` changes (expensive and awkward).
3. Accept that biji-ui's internal buttons are not hydration-gated — which violates the project's E2E contract (Playwright waits for `enabled` as the hydration signal).

---

## Activity

- **Commits:** 10 commits in the last 3 days (Mar 17-19, 2026). Actively maintained.
- **Contributors:** 1 — Jon Saw (@jonsaw), sole contributor across all 13 versions.
- **Open issues:** 0 as of 2026-03-19.
- **PRs merged recently:** #22 (toast component), #24 (progress fix), and several animation/refactoring PRs in March 2026.
- **Version cadence:** 5 releases in March 2026 alone (0.4.0 through 0.4.4). Active iteration.

There are no open issues about Leptos version compatibility. The library was last published with `leptos-use = "0.16"` on 2026-03-16 despite `leptos-use 0.18` being available since January 2026 — suggesting the leptos-use pin is not being actively tracked.

---

## Verdict (factual, not strategic)

**Is it real?** Yes. 26 components with genuine implementations. Keyboard navigation (roving tabindex, Arrow/Home/End/Escape), focus trapping (Tab cycling in Dialog), focus management (auto-focus on open/close), and collision-aware positioning (Select). Not placeholder code.

**Does it work with Leptos 0.8?** The Leptos API usage is correct 0.8 (`RwSignal`, `RenderEffect`, `StoredValue`, `NodeRef`, `Callback`, `expect_context`). It uses leptos 0.8 correctly in the WASM/client target.

**Is the version conflict blocking?**
- `leptos-use = "0.16"` vs the project's `leptos-use = "0.18.3"` is a hard incompatibility. Cargo cannot resolve both in one build. This is a blocking dependency conflict.
- No `#[cfg(feature = "ssr")]` guards anywhere — the library will fail to compile under cargo-leptos's SSR pass due to direct use of `document()`, `window()`, and `set_timeout_with_handle`. This is an independent blocking issue.
- `disabled` prop is static `bool`, incompatible with the project's `disabled=move || !hydrated.get()` hydration gate pattern. This would require forking or wrapping.

**Summary:** Three independent blockers: (1) leptos-use version conflict, (2) no SSR guards, (3) static `disabled` prop incompatible with the hydration gate pattern. Using biji-ui as published requires resolving all three, likely via a maintained fork or upstream patches.
