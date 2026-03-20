# Tier 1: Form Enhancement Landscape

## Search Methodology

1. **Awesome-Leptos index** — scanned all 98 entries for form-related libraries (none explicitly listed under a "Forms" category; found relevant entries under Components and Libraries)
2. **crates.io API searches** — queries: `leptos form`, `leptos validation`, `leptos input`, `leptos field`, `leptos error`, `leptos forms`, `leptos derive form`, `leptos validator`, `server fn validation`
3. **Direct GitHub inspection** — fetched Cargo.toml, README, and source files for every candidate to verify Leptos version and form submission pattern
4. **Dependency tracing** — checked workspace Cargo.toml files where individual crate files used `workspace = true` for Leptos version

Candidates evaluated: 14 distinct libraries (excluding subsidiary proc-macro crates that are implementation details of the same library).

---

## Candidates Found

### leptos_form_tool

- **crates.io:** https://crates.io/crates/leptos_form_tool
- **GitHub:** https://github.com/MitchellMarinoDev/leptos_form_tool
- **Leptos dep in Cargo.toml:** `leptos = "0.8"`
- **Category:** generation + validation + state
- **ActionForm compatible:** YES — `get_action_form()` wraps Leptos `<ActionForm>` natively; inputs carry `name` attributes; submission flows through `FormData` DOM reading. Evidence from `src/styles/grid_form.rs`: `name=control.data.name.clone()` on every HTML input element, and from `src/form_builder.rs`: `view! { <ActionForm action=action on:submit:capture=on_submit> {elements} </ActionForm> }`. The `on:input` handlers exist solely to update internal validation signals — they do NOT intercept form submission.
- **What it does:**
  - Declarative form building via builder pattern: `FormBuilder` with `.text_input()`, `.select()`, `.radio_buttons()`, `.stepper()`, `.slider()`, `.textarea()` methods
  - Pluggable `FormStyle` trait: implement once, applies rendering to all controls; built-in `GridFormStyle`
  - Two submission modes: `get_action_form()` (FormData/ActionForm, supports progressive enhancement) and `get_form()` (direct serialization via server function dispatch)
  - Client-side validation with cross-field rules: `ValidationBuilder` with per-control validators; `validate()` called on submit, `ev.prevent_default()` if any fail
  - `FromFormData` trait for struct construction from named form fields
  - Named field convention: nested structs use `data[field]` naming scheme for `FromFormData` reconstruction
  - Context-aware: pass read-only context to conditionally render controls
- **Last commit:** 2025-08-13
- **Total downloads:** 7,818
- **Notes:** Solo developer project. Leptos 0.8 support added in PR #27 merged 2025-05-10. The hybrid approach (name attrs + on:input) means ActionForm's DOM read and the library's internal signal state are both populated — no conflict with Playwright E2E since the ActionForm native submission still works. The only caveat: `get_form()` mode calls the server fn directly with serialized signal data and would bypass FormData entirely.

---

### leptos_form

- **crates.io:** https://crates.io/crates/leptos_form
- **GitHub:** https://github.com/tlowerison/leptos_form
- **Leptos dep in Cargo.toml:** `leptos = "0.6.5"` (workspace root; latest published crate is 0.2.0-rc1)
- **Category:** generation
- **ActionForm compatible:** UNKNOWN — signal-driven architecture inferred from README example which generates a component that accepts `initial: MyData` prop and manages state internally; the derive macro generates `<MyData initial=... />` components, not `<ActionForm>`-wrapped forms. No evidence of name attributes found in inspected source.
- **What it does:**
  - `#[derive(Form)]` macro generates a Leptos component from a struct
  - Component accepts `initial` prop for default values
  - Integrates with `#[server]` functions via `action = create_my_data(my_data)` attribute
  - `on_success` callback, `reset_on_success` attribute
  - Layout hints via `#[form(label(wrap(class = "...", rename_all = "Title Case")))]`
- **Last commit:** 2024-02-05
- **Total downloads:** 14,629 (highest in category, accumulated since 2023)
- **Notes:** Last updated February 2024. Latest version is 0.2.0-rc1 — a release candidate that never left RC. Targets Leptos 0.6.5, which is two major versions behind 0.8. **Not compatible with project's Leptos 0.8.** Abandoned or dormant.

---

### leptos-forms-rs

- **crates.io:** https://crates.io/crates/leptos-forms-rs
- **GitHub:** https://github.com/cloud-shuttle/leptos-forms-rs
- **Leptos dep in Cargo.toml:** `leptos = "0.8"` (workspace.dependencies)
- **Category:** state + validation
- **ActionForm compatible:** NO — uses `use_form` hook pattern with signal-driven state. README example: `let (form, _submit_callback, _reset_callback) = use_form(LoginForm::default_values())`. Form submission dispatches via the hook's callback, not via `<ActionForm>` / FormData DOM reads. Form state lives in `RwSignal` fields, not DOM.
- **What it does:**
  - `use_form` hook returns reactive form state
  - `<FormField>`, `<FormSubmit>`, `<FormReset>` pre-built components
  - Derive macro (`leptos-forms-rs-macro`) for generating `Form` trait implementations
  - Field arrays and dynamic forms
  - Form persistence with localStorage
  - Type-safe compile-time validation
  - Accessibility-first with ARIA support
  - 144 tests across Chrome, Firefox, WebKit
- **Last commit:** 2025-09-20
- **Total downloads:** 2,907
- **Notes:** Actively maintained in 2025. Signal-driven architecture is fundamentally incompatible with Playwright's inability to reliably fire Leptos `on:input` handlers on hydrated elements. The `use_form` pattern is the React/hook equivalent; form values live in signals, not the DOM.

---

### borang

- **crates.io:** https://crates.io/crates/borang
- **GitHub:** https://github.com/jonsaw/borang
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8" }`
- **Category:** validation + state
- **ActionForm compatible:** NO — signal-driven. Evidence from `src/form.rs`: `pub struct FieldSignal { value: RwSignal<String> }`. Values retrieved with `field.value.get_untracked()` from signal, not DOM. Uses `#[validator(required, email)]` proc-macro attributes on struct fields.
- **What it does:**
  - `Form` struct manages field state as signals
  - `#[derive(Validation)]` proc-macro with `#[validator(required, email, min_length=N, max_length=N)]` field attributes
  - `form.validate()` validates all fields, returns bool
  - `form.data()` extracts data from signals
  - `<Field>` component wraps inputs; `<Input>` and `<Select>` components included
  - Real-time validation feedback as signals update
- **Last commit:** 2025-11-14
- **Total downloads:** 72
- **Notes:** Very early stage (v0.1.1, 72 downloads). Signal-driven architecture is incompatible with ActionForm pattern for the same reason as leptos-forms-rs.

---

### formidable

- **crates.io:** https://crates.io/crates/formidable
- **GitHub:** https://github.com/fabianboesiger/formidable
- **Leptos dep in Cargo.toml:** `leptos = "0.8.10"` (workspace.dependencies)
- **Category:** generation
- **ActionForm compatible:** NO — signal-driven. Evidence from `formidable/src/lib.rs`: `pub fn FormidableRwSignal<T>(#[prop(into)] value: RwSignal<T>) -> impl IntoView` — components accept `RwSignal<T>` directly. Submission uses a `Callback<Result<T, FormError>>` that calls `value.set(v)`. No `name` attributes or FormData in inspected code.
- **What it does:**
  - `#[derive(Formidable)]` generates form UI from structs and enums
  - `FormidableServerAction` component for server integration
  - Supports unit enums as radio buttons/selects; complex enum variants show conditional form sections
  - Dynamic repeating elements via `Vec<T>` fields
  - i18n integration with `leptos_i18n`
  - Pre-built type support: `time`, `url`, `color`, `bigdecimal`, email, phone
  - Placeholder support (added Nov 2025), pagination, colspan, Option<T> fields, empty structs
- **Last commit:** 2025-11-28
- **Total downloads:** 38
- **Notes:** Early stage (v0.1.0, 38 downloads). Actively developed through Nov 2025. Signal-driven architecture incompatible with ActionForm pattern.

---

### autoform

- **crates.io:** https://crates.io/crates/autoform
- **GitHub:** none listed
- **Leptos dep in Cargo.toml:** No direct `leptos` dependency — it is a proc-macro crate with only `darling`, `proc-macro2`, `quote`, `syn` dependencies. Generates code that references `crate::registry::hooks::use_form::Form<Self>`.
- **Category:** generation
- **ActionForm compatible:** NO — generates code that references an internal `use_form::Form<Self>` hook and `<FormField>`, `<FormInput>` component types from `crate::registry::ui`. This is a companion macro for a specific application's component registry, not a standalone library. No Leptos crate version constraint found.
- **What it does:**
  - `#[derive(AutoForm, Validate, Serialize, Deserialize)]` macro
  - Generates `render_fields()` method outputting Leptos `view!` code
  - Field type mapping: textarea → `<FormTextarea>`, checkbox → `<FormCheckbox>`, number → `<FormInput attr:r#type="number">`
  - Attributes: `#[autoform(label = "...", placeholder = "...", field_type = "textarea")]`
  - Integrates with `validator` crate for `#[validate(...)]` attributes
- **Last commit:** 2026-02-02
- **Total downloads:** 33
- **Notes:** Published 2026-02-02. No repository URL. References `crate::registry` internals — this is a macro designed to work within a specific project's code structure, not a general-purpose standalone library. Unpublished source. Near-zero traction (33 downloads).

---

### urlap

- **crates.io:** https://crates.io/crates/urlap
- **GitHub:** https://github.com/LeoBorai/urlap
- **Leptos dep in Cargo.toml:** `leptos = "0.7"` (Cargo.toml in root)
- **Category:** unknown (described as "Leptos Form Helpers")
- **ActionForm compatible:** UNKNOWN — insufficient documentation. Repository has only 6 commits total, no meaningful README content beyond the title.
- **What it does:** Unknown — no documentation available. Name is Hungarian for "form" (űrlap).
- **Last commit:** 2025-03-09 (repository creation date; 6 total commits)
- **Total downloads:** 720
- **Notes:** Targets Leptos 0.7, which is incompatible with this project's Leptos 0.8. Alpha version (0.1.0-alpha.1). No documentation. Not viable.

---

### vld / vld-leptos

- **crates.io (vld):** https://crates.io/crates/vld
- **crates.io (vld-leptos):** https://crates.io/crates/vld-leptos
- **GitHub:** https://github.com/s00d/vld
- **Leptos dep in Cargo.toml:** No direct Leptos dependency in `crates/vld-leptos/Cargo.toml` — only `vld`, `serde`, `serde_json` (workspace deps). No Leptos version constraint.
- **Category:** validation (server-side + shared)
- **ActionForm compatible:** YES — designed to work inside `#[server]` functions, not to replace form rendering. Evidence from README: `validate_args! { name => shared::name_schema(), email => shared::email_schema() }` macro used inside `#[server]` function body. Also supports client-side WASM validation via `check_field()` in Leptos memos.
- **What it does:**
  - `validate_args!` macro for validating server function arguments inline
  - Define validation schemas once in a shared module; compile for both server and WASM targets
  - `VldServerError` for structured error transport as JSON from server to client
  - `check_field()` for reactive client-side field validation in Leptos memos
  - Schemas: string length, email format, numeric ranges, custom validators
  - Zod-inspired API: `string().min(3).max(50).email()` schema chaining
- **Last commit:** 2026-03-19
- **Total downloads (vld-leptos):** 15
- **Notes:** Released today (2026-03-19). Extremely low traction (15 downloads for vld-leptos). The `vld-leptos` crate has no direct Leptos dependency — it works at the validation layer only, leaving form rendering entirely to the developer. ActionForm-compatible by design since it hooks into server functions, not form rendering.

---

### input-rs (opensass)

- **crates.io:** https://crates.io/crates/input-rs
- **GitHub:** https://github.com/opensass/input-rs
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.7.7", optional = true }`
- **Category:** state (single-field component)
- **ActionForm compatible:** NO — signal-driven. Evidence from LEPTOS.md: `let email_handle = signal(String::default())` passed to `<Input handle={email_handle} />`. Form submission uses `on:submit` with `ev.prevent_default()` and reads signal values directly.
- **What it does:**
  - Standalone `<Input>` component for WASM frameworks (Yew, Dioxus, Leptos)
  - Props: `handle` (signal tuple), `valid_handle` (signal tuple), `validate_function` (fn pointer), `error_message`
  - Built-in password visibility toggle (`eye_active`, `eye_disabled` props)
  - Per-field validation callback
- **Last commit:** 2025-04-17
- **Total downloads:** 6,203
- **Notes:** Targets Leptos 0.7.7, incompatible with project's 0.8. Signal-driven architecture. Multi-framework crate — Leptos is one of three supported frameworks. Not a form library; single input component only.

---

### leptos-shadcn-form (cloud-shuttle)

- **crates.io:** https://crates.io/crates/leptos-shadcn-form
- **GitHub:** https://github.com/cloud-shuttle/leptos-shadcn-ui
- **Leptos dep in Cargo.toml:** `leptos = "0.8.9"` (workspace.dependencies)
- **Category:** error-display + state (form component wrapper)
- **ActionForm compatible:** UNKNOWN — the cloud-shuttle ecosystem targets Leptos 0.8 and claims "production-ready" components; however, the README emphasizes "real-time validation" and "reactive state management," which indicates signal-driven inputs. Leptos-shadcn-form is a port of shadcn/ui's Form component, which in the React original wraps react-hook-form (signal-driven). No evidence of ActionForm usage found.
- **What it does:**
  - Leptos port of shadcn/ui Form component
  - Structured form field layout: `<FormField>`, `<FormLabel>`, `<FormControl>`, `<FormDescription>`, `<FormMessage>`
  - Field-level error display via `<FormMessage>`
  - Part of larger leptos-shadcn-ui component suite (46+ components)
  - WCAG 2.1 AA compliance claims
- **Last commit:** December 2024 (from repository metadata; 261 total commits)
- **Total downloads:** Not separately tracked (part of leptos-shadcn-ui bundle)
- **Notes:** The cloud-shuttle organization publishes an unusually large number of Leptos-adjacent crates (leptos-forms-rs, radix-leptos, leptos-shadcn-ui, leptos-ws-pro, etc.) with high version numbers (0.9.0, 1.3.0) but low download counts, suggesting aggressive publishing cadence rather than organic adoption.

---

## Landscape Summary

### By ActionForm Compatibility

**Compatible with ActionForm (work alongside existing pattern):**
- `leptos_form_tool` — uses `<ActionForm>` natively in `get_action_form()` mode; adds declarative form building, client-side validation, and pluggable styling on top. The only library in the ecosystem that explicitly supports ActionForm as its primary submission mechanism.
- `vld-leptos` — pure server-side / shared validation macro; completely orthogonal to form rendering.

**Incompatible with ActionForm (replace ActionForm with signals):**
- `leptos-forms-rs` — `use_form` hook, `<FormField>` components, signal state
- `borang` — `Form` struct with `RwSignal<String>` fields
- `formidable` — `RwSignal<T>` props, `Callback<Result<T, FormError>>`
- `input-rs` — `handle` signal tuple props
- `leptos-shadcn-form` — shadcn/ui Form port, likely signal-driven (unconfirmed)

**Not applicable (abandoned, wrong version, or undocumented):**
- `leptos_form` — targets Leptos 0.6.5, last commit 2024-02-05, RC never released
- `urlap` — targets Leptos 0.7, no documentation, 6 commits
- `input-rs` — targets Leptos 0.7.7
- `autoform` — no repository, references internal `crate::registry`, not standalone

### By Leptos Version

| Library | Leptos version | Status |
|---------|---------------|--------|
| `leptos_form_tool` | `"0.8"` | Active |
| `leptos-forms-rs` | `"0.8"` | Active |
| `formidable` | `"0.8.10"` | Active |
| `borang` | `"0.8"` | Active |
| `leptos-shadcn-form` | `"0.8.9"` | Active |
| `vld-leptos` | none (no direct dep) | Active |
| `autoform` | none (proc-macro only) | Active |
| `input-rs` | `"0.7.7"` | Incompatible |
| `urlap` | `"0.7"` | Incompatible |
| `leptos_form` | `"0.6.5"` | Abandoned |

### Gaps in the Ecosystem

No library found that provides:
- **Structured field-level error display from `ServerFnError`** — every library either handles errors internally (signal-driven) or leaves error display entirely to the developer
- **Hydration gate helpers** — no library abstracts the `signal(false) + Effect::new` pattern
- **Server function argument validation with structured field errors** — `vld-leptos` validates args inside the server fn but transports errors as a single string via `ServerFnError::new(e.to_string())`; no library maps errors back to specific form fields in a type-safe way
