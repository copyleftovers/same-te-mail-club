# Form & Styling — Source-Level Verification

## leptos_form_tool

### Crate Metadata

- **Version:** 0.4.1
- **Total downloads:** 7,818
- **Repository:** https://github.com/MitchellMarinoDev/leptos_form_tool
- **Last published:** August 13, 2025
- **Stars:** 7

### Leptos Version

From `Cargo.toml`:
```toml
leptos = "0.8"
leptos_router = "0.8"
```

Compatible with the project's Leptos 0.8.17.

### ActionForm Integration (VERIFIED from source)

The library exposes **two distinct submission paths**. They have different behavior and the distinction is critical for this project.

**Path 1: `get_action_form()` → `build_action_form()`**

The `on:submit:capture` handler in `build_action_form()`:
1. Checks `if ev.default_prevented()` — if already prevented, skip
2. Runs client-side validation callbacks
3. On failure: calls `ev.prevent_default()` and returns
4. On success: calls the user `on_submit` callback, then **does NOT call `ev.prevent_default()` and does NOT call `action.dispatch()`**

This means on success, the `ActionForm` native behavior runs: the browser serializes form fields as `FormData` and submits to the server function. The name attribute on each input IS the key used by `FormData`.

From `src/controls/text_input.rs`:
```rust
pub struct TextInputData {
    pub name: String,
    // ...
}
```

From `src/styles/grid_form.rs` (the default `FormStyle` impl):
```rust
name=control.data.name.clone()
```

The `name` attribute is set on the HTML `<input>` element, matching the server function parameter name.

**Path 2: `get_form()` → `build_form()`**

The `on:submit:capture` handler:
1. Always calls `ev.prevent_default()`
2. Runs validation
3. On success: **calls `action.dispatch(ServFn::from(fd.get_untracked()))`**

This path bypasses `ActionForm`'s `FormData` reading entirely. It reads the signal state (`fd.get_untracked()`), converts it to the server function type via a `From` impl, and dispatches directly. This is the signal-driven path.

### on:input vs FormData

From `src/controls/mod.rs`:
```rust
#[derive(Default)]
pub enum UpdateEvent {
    OnFocusout,
    OnInput,
    #[default]
    OnChange,  // default
}
```

From `src/styles/grid_form.rs`, all three events attach handlers:
```rust
on:input:target=move |ev| {
    if update_event == UpdateEvent::OnInput {
        value_setter.set(ev.target().value())
    }
}
on:focusout:target=move |ev| {
    if update_event == UpdateEvent::OnFocusout {
        value_setter.set(ev.target().value())
    }
}
on:change:target=move |ev| {
    if update_event == UpdateEvent::OnChange {
        value_setter.set(ev.target().value())
    }
}
```

These event handlers update an internal `RwSignal` for client-side validation display. They do NOT control form submission data.

**For the `get_action_form()` path:** Form submission reads `FormData` from the DOM (via `ActionForm`'s native behavior), not from signals. If Playwright's `.fill()` doesn't fire `on:input` or `on:change`, the signal state will be stale or empty — but the `FormData` will still contain the correct value from the DOM. **Form submission is correct regardless of whether Playwright fires the event handlers.**

The only consequence of Playwright not firing event handlers: client-side validation feedback will not display during test runs. The form will still submit the correct data to the server.

**For the `get_form()` path (the signal path):** Playwright breaking `on:input`/`on:change` WOULD break form submission, since dispatch uses signal state. This path is unsafe to use in this project.

### Boilerplate Reduction

The project has 8 forms. From `spec/product/project-brief.md`: "Form field markup (label + input + error + ARIA)" is ~12 lines per field.

`leptos_form_tool` auto-generates field markup from a struct definition + builder call. A text field defined as:
```rust
.text_input(|c| c.named("phone").labeled("Phone").placeholder("+380..."))
```
replaces approximately:
```rust
<div class="field">
    <label class="field-label" for="phone-input">{label}</label>
    <input class="field-input" id="phone-input" type="tel" name="phone"
        placeholder="+380..." data-testid="phone-input" />
    {move || if validation_failed { view!{ <p class="field-error">...</p> } } else { ... }}
</div>
```

The generated markup uses the library's `GridFormStyle`. The project has a custom `@layer components` with `.field`, `.field-label`, `.field-input`, `.field-error` CSS classes. Using `leptos_form_tool` would require either:
1. Implementing a custom `FormStyle` trait to emit the project's CSS class structure, or
2. Accepting `GridFormStyle` output (which uses grid layout and its own SCSS, not the project's design system)

The library's `grid_form.scss` is separate from the project's Tailwind-based CSS architecture.

### Verdict (factual)

**`get_action_form()` path:** Yes, it works with `ActionForm`. HTML inputs have `name` attributes. On success, `ActionForm` reads `FormData` from the DOM. Playwright `.fill()` not firing event handlers does not break form submission — only client-side validation display.

**`get_form()` path:** Uses signal state + `action.dispatch()`. Playwright breaking `on:input`/`on:change` would cause empty data to be dispatched. This path is incompatible with the project's E2E test approach.

**Prior agent claim:** "sets `name` attributes on HTML inputs, submits via FormData, and uses on:input handlers purely for client-side validation signals" — **PARTIALLY CORRECT** for the `get_action_form()` path only. The `on:change` handler (default) is used, not `on:input`. The characterization that event handlers are "purely for client-side validation" is accurate for the `get_action_form()` path.

**Adoption cost:** Custom `FormStyle` implementation required to use the project's existing CSS design system (`.field`, `.field-label`, etc.). Without it, `GridFormStyle` output would not match the project's visual design.

---

## tw_merge

### Crate Metadata

- **Version:** 0.1.20
- **Total downloads:** 12,880
- **Repository:** https://github.com/rust-ui/tw_merge
- **Last published:** March 7, 2026
- **Description:** "Macros for merging Tailwind CSS v4 classes or creating variants."

### Leptos Dependency

From `Cargo.toml`:
```toml
# No Leptos dependency
nom = "7"
tw_merge_variants = { path = "../tw_merge_variants", version = "0.1.0", optional = true }
```

`tw_merge` is a standalone Rust crate with no Leptos dependency. It can be used in any Rust project.

### Tailwind v4 Support (VERIFIED from source)

From `src/core/merge/get_collision_id.rs`, the function `get_collision_id()` uses pattern matching on tokenized class arrays. It explicitly handles v4 utilities:

```rust
// 3D transforms (new in v4)
["scale", "z", ..] => Ok("scale-z")
["rotate", "x"] | ["rotate", "x", "reverse"] => Ok("rotate-x")
["translate", "z", ..] => Ok("translate-z")
// Text shadow (new in v4)
["text", "shadow"] => Ok("text-shadow")
["text", "shadow", "color", ..] => Ok("text-shadow-color")
// Inset shadow/ring (new in v4)
["inset", "shadow", ..] => Ok("inset-shadow")
["inset", "ring", ..] => Ok("inset-ring")
// Gradient naming (v4 renamed bg-gradient-to-* → bg-linear-*)
["bg", "linear", ..] => Ok("bg-image")
["bg", "conic", ..] => Ok("bg-image")
["bg", "radial", ..] => Ok("bg-image")
// Masks (new in v4)
["mask", "linear", ..] | ["mask", "radial", ..] | ["mask", "conic", ..] => Ok("mask-image")
```

From `src/core/merge/validators.rs`, the color validator explicitly recognizes `oklch`:
```rust
alt((tag("rgba"), tag("rgb"), tag("hsla"), tag("hsl"), ..., tag("oklch"), ...))
```

The validator accepts oklch values in arbitrary Tailwind classes like `text-[oklch(0.63_0.22_31)]`.

From `tests/merge.rs`, there is a comment: `"Tailwind v4 supports decimal fractions"` in the aspect_ratio test, confirming v4 awareness. However, the test suite does not have dedicated oklch or text-shadow tests.

### vs tailwind_fuse

| Attribute | tw_merge | tailwind_fuse |
|-----------|----------|---------------|
| Latest version | 0.1.20 | 0.3.2 |
| Last published | March 7, 2026 | January 19, 2025 |
| Total downloads | 12,880 | 47,731 (+ 29,689 v0.3.2) |
| Leptos dependency | None | None (but leptos-focused) |
| Tailwind v4 utilities | Explicit v4 class patterns in source | No evidence |
| Fork relationship | tw_merge's README states: "This is a fork of tailwind_fuse" | Original |
| Maintenance | Active (March 2026 release) | Last release January 2025 |

`tailwind_fuse` has more total downloads but has not been updated since January 2025. `tw_merge` explicitly targets Tailwind v4 in its crate description and has v4 class patterns in its collision detection.

### Project Need Assessment (factual)

The project's CSS architecture uses `@layer components` classes (`.btn`, `.badge`, `.field`) with `data-*` variant hooks. Class composition in Leptos components is done via static string literals:

```rust
<button class="btn" data-variant="secondary" ...>
<span class="badge" data-status="active" ...>
<div class="field-error" aria-live="assertive" ...>
```

From searching `src/**/*.rs`: no `format!()` calls constructing class names exist. All class strings are static literals.

The project does not currently compose classes dynamically. There is no case in the codebase where two strings of Tailwind utilities need to be merged with conflict resolution. The variant pattern uses CSS `data-*` attribute selectors rather than combining utility strings.

The scenario where `tw_merge` would be useful: a shared Leptos component that accepts a `class` prop for override utilities. Example:
```rust
// Without tw_merge: conflicts possible
<MyButton class="btn mt-4 mt-8" />  // mt-4 and mt-8 both present

// With tw_merge
let merged = tw_merge!("btn mt-4", class_prop);  // rightmost wins: mt-8
```

The project has no such prop-accepting components currently. All 19 components have fixed class strings.
