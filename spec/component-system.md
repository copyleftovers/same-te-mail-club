# Component System -- Same Te Mail Club

Binding specification for all UI components. An implementation agent reads this document plus the codebase and builds every component without consulting research files.

**Stack:** Leptos 0.8, Axum, Postgres, Tailwind v4 standalone (no Node/npm). All CSS in `style/tailwind.css`.
**Constraint:** No external UI/component libraries. All components hand-rolled.

---

## Design Philosophy

1. **One action per screen.** Each page shows exactly one thing to do next. No dashboards-of-dashboards, no feature grids.
2. **Mobile IS the product.** Default styles target 375-428px. Desktop gets comfort via breakpoints, never requirements.
3. **Types encode states.** The `Phase` enum drives the entire UI. Invalid UI states do not compile.
4. **CSS components own variants.** `data-*` attributes + CSS custom property hooks (Lea Verou pseudo-private pattern). No `format!()` class names.
5. **Progressive enhancement.** HTML works before WASM. `<details>` for disclosure, HTML5 validation for forms, hydration gate for buttons.
6. **ActionForm is the form model.** `name` attributes, not signals. Playwright-compatible by design.
7. **Warmth through reduction.** Generous whitespace. One warm orange. Soft cream background. CyGrotesk display font. Mont body font. No corporate language.
8. **Accessibility is structural.** Semantic HTML, ARIA linkage, focus rings, reduced-motion support -- baked in, not bolted on.
9. **Celebrate sparingly.** Confetti for assignment reveal (first-time only). Toast for form success. Nothing else.
10. **No speculation.** Build what the spec defines.

---

## Component Catalog

### Layout

#### PageWrapper

**Purpose:** Root layout shell with dynamic viewport height and safe-area handling.

**HTML:**
```html
<div class="flex min-h-dvh flex-col">
  <header class="app-header"><!-- Header --></header>
  <main class="flex-1"><!-- Routes --></main>
</div>
```

**Leptos:** Not a standalone component -- applied in `app.rs` shell.

**Mobile:** `min-h-dvh` (not `100vh`) handles iOS Safari collapsing address bar.

**Priority:** P0 (modify existing shell).

---

#### ContentContainer (`.prose-page`)

**Purpose:** Centers content with readable line length. Already exists.

**CSS:** Already defined. `max-width: 65ch; margin-inline: auto; padding-inline: 1rem; padding-block: var(--density-space-lg);`

**Priority:** P0 (exists).

---

### Forms

#### Field

**Purpose:** Label + input + error container with ARIA linkage.

**HTML:**
```html
<div class="field">
  <label class="field-label" for="phone">Phone</label>
  <input class="field-input" id="phone" name="phone" type="tel"
         aria-invalid="true" aria-describedby="phone-error" />
  <p class="field-error" id="phone-error" role="alert" aria-live="assertive"
     data-testid="phone-error">Invalid phone number</p>
</div>
```

**CSS:** Already exists. `.field` (flex column, gap 0.375rem), `.field-label` (Mont 600, text-sm), `.field-input` (Mont 400, 16px, 1.5px border, padding `0.625rem 0.75rem`), `.field-error` (text-sm, `--color-error`).

**New CSS additions:**
```css
@layer components {
  .field-input {
    min-height: 44px; /* WCAG touch target */
  }

  select.field-input {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='8'%3E%3Cpath d='M1 1l5 5 5-5' stroke='%23565656' stroke-width='1.5' fill='none'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.75rem center;
    padding-right: 2.5rem;
  }

  textarea.field-input {
    min-height: 5.5rem;
    resize: vertical;
  }
}
```

**Leptos:**
```rust
#[component]
fn Field(
    #[prop(into)] label: String,
    #[prop(into)] field_id: String,
    #[prop(optional, into)] error: Option<Signal<Option<String>>>,
    children: Children,
) -> impl IntoView {
    let error_id = format!("{field_id}-error");
    view! {
        <div class="field">
            <label class="field-label" for=field_id.clone()>{label}</label>
            {children()}
            {move || error.and_then(|e| e.get()).map(|msg| view! {
                <p class="field-error" id=error_id.clone() role="alert"
                   aria-live="assertive" data-testid=format!("{}-error", field_id)>
                    {msg}
                </p>
            })}
        </div>
    }
}
```

**States:** Default (gray border), Focus (blue border + 3px glow), Error (red border + red glow), Disabled (50% opacity).

**Mobile:** `font-size: 16px` on `.field-input` prevents iOS auto-zoom. `min-height: 44px` meets touch target guidelines.

**A11y:** `<label for="id">` links label to input. `aria-invalid` announces error state. `aria-describedby` links error message. `role="alert"` + `aria-live="assertive"` on error container.

**Priority:** P0 (extract from existing inline patterns).

---

#### PhoneInput

**Purpose:** Phone number entry with tel keyboard.

**HTML:**
```html
<input class="field-input" id="phone" name="phone"
       type="tel" inputmode="tel"
       placeholder="+380 67 123 45 67"
       data-testid="phone-input" />
```

**CSS:** Inherits `.field-input`. Already has `font-variant-numeric: tabular-nums; letter-spacing: 0.1em` via `.field-input[type="tel"]`.

**Mobile:** `type="tel"` + `inputmode="tel"` triggers phone keyboard with +/- symbols.

**Priority:** P0 (exists, add `inputmode` attribute).

---

#### OtpInput (single field)

**Purpose:** 6-digit SMS verification code with autofill support.

**HTML:**
```html
<input class="field-input" id="otp-input" name="otp"
       type="text" inputmode="numeric"
       autocomplete="one-time-code"
       pattern="\d{6}" maxlength="6"
       placeholder="000000"
       aria-describedby="otp-hint"
       data-otp
       data-testid="otp-input" />
<p id="otp-hint" class="text-sm text-(--color-text-muted)">
  Enter the code we sent.
</p>
```

**CSS:** Already exists via `.field-input[data-otp]`: `font-variant-numeric: tabular-nums; letter-spacing: 0.1em`. Add:
```css
.field-input[data-otp] {
  text-align: center;
  font-size: 1.25rem;
}
```

**Mobile:** `inputmode="numeric"` triggers numeric keyboard. `autocomplete="one-time-code"` enables iOS/Android SMS autofill.

**A11y:** Single input (NOT 6 separate boxes -- breaks autofill, screen readers, paste). `aria-describedby` links to hint.

**BANNED:** Multiple `<input maxlength="1">` boxes.

**Priority:** P0 (exists).

---

#### NpCitySelect

**Purpose:** City selection for Nova Poshta branch. Native `<select>` -- triggers platform picker on mobile.

**HTML:**
```html
<select class="field-input" id="np-city" name="city" required
        data-testid="np-city-input">
  <option value="">Select city</option>
  <option value="kyiv">Kyiv</option>
  <option value="lviv">Lviv</option>
  <option value="odesa">Odesa</option>
  <option value="kharkiv">Kharkiv</option>
  <option value="dnipro">Dnipro</option>
</select>
```

**Decision: Native `<select>`, not text input, not autocomplete.** For 10-20 cities, native `<select>` is optimal. It triggers the iOS wheel picker and Android dropdown natively -- zero JS, perfect touch UX, fully accessible. An autocomplete widget would require building a custom dropdown, keyboard navigation, and screen reader announcements from scratch in a framework with no headless UI library. Not worth it for a short list.

**CSS:** `select.field-input` styles from the Field section above (appearance: none, custom chevron).

**Mobile:** Native platform picker. Touch target inherited from `.field-input` (44px min-height).

**A11y:** Native `<select>` is fully accessible out of the box.

**Priority:** P1 (planned NP field split).

---

#### NpNumberInput

**Purpose:** Nova Poshta branch number (numeric).

**HTML:**
```html
<input class="field-input" id="np-number" name="np_number"
       type="text" inputmode="numeric"
       placeholder="123" required
       data-testid="np-number-input" />
```

**Why `type="text"` not `type="number"`:** `type="number"` creates spinner controls (bad on mobile), strips leading zeros, and fires change events differently. `inputmode="numeric"` gives the numeric keyboard without the baggage.

**Priority:** P1 (planned NP field split).

---

#### Textarea

**Purpose:** Multi-line text input for receipt notes.

**HTML:**
```html
<textarea class="field-input" id="note" name="note"
          rows="4" placeholder="Describe the issue..."
          data-testid="receipt-note"></textarea>
```

**CSS:** `textarea.field-input` styles from the Field section above (min-height: 5.5rem, resize: vertical).

**Priority:** P1 (used in receipt confirmation).

---

### Actions

#### Button (`.btn`)

**Purpose:** Primary interactive element for all form submissions and actions. Already exists.

**CSS:** Already defined with three variants (`primary`, `secondary`, `destructive`), three sizes (`sm`, `default`, `lg`), pill shape. Add min-height:
```css
.btn { min-height: 44px; }
.btn[data-size="sm"] { min-height: 36px; }
.btn[data-size="lg"] { min-height: 48px; }
```

**Leptos (with loading state):**
```rust
#[component]
fn SubmitButton(
    #[prop(into)] label: String,
    #[prop(into)] loading_label: String,
    #[prop(into)] pending: Signal<bool>,
    #[prop(optional)] variant: Option<&'static str>,
    #[prop(optional)] testid: Option<&'static str>,
) -> impl IntoView {
    let hydrated = use_hydrated();
    view! {
        <button
            type="submit"
            class="btn"
            data-variant=variant.unwrap_or("primary")
            disabled=move || !hydrated.get() || pending.get()
            data-testid=testid.unwrap_or("submit-button")
        >
            {move || if pending.get() {
                view! {
                    <span class="inline-flex items-center gap-2">
                        <svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24"
                             fill="none" aria-hidden="true">
                            <circle cx="12" cy="12" r="10"
                                    stroke="currentColor" stroke-width="4"
                                    opacity="0.25" />
                            <path fill="currentColor" opacity="0.75"
                                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                        </svg>
                        {loading_label.clone()}
                    </span>
                }.into_any()
            } else {
                view! { <span>{label.clone()}</span> }.into_any()
            }}
        </button>
    }
}
```

**States:**
- **Pre-hydration:** disabled, opacity 0.45
- **Idle:** enabled, full opacity, label text
- **Pending:** disabled, spinner + loading text
- **Hover:** `filter: brightness(0.9)` (pointer devices only via `@media (hover: hover)`)
- **Focus-visible:** 2px outline ring

**Mobile:** 44px min-height. Full-width on mobile (`w-full` applied in context), natural width on desktop.

**Priority:** P0 (exists, add spinner integration).

---

#### LinkButton

**Purpose:** Navigation styled as button.

**HTML:**
```html
<a href="/" class="btn" data-variant="secondary" data-testid="back-home">
  Back to home
</a>
```

**CSS:** `.btn` already handles `<a>` elements (has `text-decoration: none` in existing styles). Add:
```css
a.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
```

**Priority:** P1.

---

### Feedback

#### Toast

**Purpose:** Non-blocking success feedback that auto-dismisses. NOT for errors (errors are inline).

**Decision: Bottom-center on mobile, bottom-right on desktop.** Bottom position doesn't obscure the content the user just interacted with. On mobile it uses full width with left/right margin. On desktop it parks right.

**HTML:**
```html
<div role="status" aria-live="polite" aria-atomic="true"
     class="fixed bottom-4 left-4 right-4 z-40 sm:left-auto sm:max-w-96">
  <div class="toast" data-testid="toast">
    <p class="toast-message">You're enrolled!</p>
  </div>
</div>
```

**CSS:**
```css
@layer components {
  .toast {
    --_border: var(--toast-border, var(--color-success));
    background: var(--color-surface-raised);
    border: 1px solid var(--_border);
    border-radius: var(--radius-md);
    padding: var(--density-space-md);
    box-shadow: 0 4px 12px oklch(from var(--color-brand-black) l c h / 0.15);
    animation: toast-in 200ms ease;
  }

  .toast[data-type="error"] {
    --toast-border: var(--color-error);
  }

  .toast-message {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text);
  }

  @keyframes toast-in {
    from { opacity: 0; translate: 0 1rem; }
    to { opacity: 1; translate: 0 0; }
  }
}
```

**Leptos architecture:** The live region container (`role="status"`) lives persistently in the app shell (always in DOM, content swapped). A `WriteSignal<Option<String>>` is provided via context.

```rust
// In app shell
let (toast_message, set_toast) = signal(None::<String>);
provide_context(set_toast);

view! {
    <div role="status" aria-live="polite" aria-atomic="true"
         class="fixed bottom-4 left-4 right-4 z-40 sm:left-auto sm:max-w-96">
        {move || toast_message.get().map(|msg| view! {
            <div class="toast" data-testid="toast">
                <p class="toast-message">{msg}</p>
            </div>
        })}
    </div>
}
```

```rust
// In any component after a successful action
let set_toast = expect_context::<WriteSignal<Option<String>>>();
Effect::new(move |_| {
    if let Some(Ok(_)) = action.value().get() {
        set_toast.set(Some("You're enrolled!".to_string()));
        set_timeout(move || set_toast.set(None), Duration::from_secs(5));
    }
});
```

**Timing:** 5s auto-dismiss. No separate error toast -- errors are always inline.

**Mobile:** Full-width with left/right margin. Safe-area padding: `padding-bottom: env(safe-area-inset-bottom)` on fixed container.

**A11y:** `role="status"` + `aria-live="polite"` -- screen reader announces without interrupting. The live region element exists persistently in DOM (content updated, not conditionally rendered).

**Priority:** P1.

---

#### Alert (`.alert`)

**Purpose:** Blocking error messages within page flow. Already exists.

**CSS:** Already defined. Red-tinted background, `--color-error` text, padding, rounded.

**Enhancement:** Add scroll-into-view behavior via Effect + `scrollIntoView({ behavior: "smooth", block: "center" })` on the error element when it renders.

**Priority:** P0 (exists, add scroll-into-view).

---

#### Skeleton

**Purpose:** Loading placeholder showing shape of incoming content.

**Decision: Generic pulse lines, not content-matched shapes.** Content-matched skeletons are more work per page than they're worth for a 10-page app with sub-second server responses. Generic lines communicating "loading" are sufficient. Each Suspense fallback uses 2-3 `skeleton-line` divs with varying widths.

**CSS:**
```css
@layer components {
  .skeleton-line {
    background: oklch(from var(--color-brand-gray) l c h / 0.15);
    border-radius: var(--radius-sm);
    animation: skeleton-pulse 2s ease-in-out infinite;
  }

  @keyframes skeleton-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
}
```

**Leptos:** Used as `Suspense` fallback:
```rust
<Suspense fallback=move || view! {
    <div class="space-y-3" aria-hidden="true">
        <div class="skeleton-line h-6 w-2/3"></div>
        <div class="skeleton-line h-4 w-full"></div>
        <div class="skeleton-line h-4 w-3/4"></div>
    </div>
}>
    {/* actual content */}
</Suspense>
```

**A11y:** `aria-hidden="true"` on skeleton container. `prefers-reduced-motion: reduce` disables pulse animation (static at 70% opacity).

**Priority:** P1 (replaces "Loading..." strings).

---

#### EmptyState

**Purpose:** Friendly guidance when no data exists, with optional CTA.

**HTML:**
```html
<div class="empty-state" data-testid="empty-state">
  <p class="empty-state-headline">No season running</p>
  <p class="empty-state-body">
    Sit tight -- we'll let you know when the next one starts.
  </p>
</div>
```

**CSS:**
```css
@layer components {
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--density-space-lg) var(--density-space-md);
    text-align: center;
    min-height: 12rem;
  }

  .empty-state-headline {
    font-family: var(--font-display);
    font-weight: 900;
    font-size: 1.3rem;
    line-height: 1.15;
    color: var(--color-text);
    margin-bottom: 0.5rem;
  }

  .empty-state-body {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    max-width: 28rem;
  }
}
```

**Leptos:**
```rust
#[component]
fn EmptyState(
    #[prop(into)] headline: String,
    #[prop(into)] body: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="empty-state" data-testid="empty-state">
            <p class="empty-state-headline">{headline}</p>
            <p class="empty-state-body">{body}</p>
            {children.map(|c| c())}
        </div>
    }
}
```

**Empty state copy for each page:**

| Page | Headline | Body |
|------|----------|------|
| Home (no season) | "No season running" | "Sit tight -- we'll let you know when the next one starts." |
| Admin participants | "No participants yet" | "Register someone to get started." |
| Admin assignments | "Assignments not generated" | "All participants must confirm ready before generating." |
| Admin SMS | "No SMS sent yet" | "Trigger an SMS batch when ready." |

**Priority:** P0 (multiple pages show bare text for empty states).

---

### Display

#### Badge (`.badge`)

**Purpose:** Status pills for participants, seasons, assignments. Already exists.

**CSS:** Already defined. Five status variants (`active`, `pending`, `error`, `inactive`, `confirmed`). Mont 600, text-xs, uppercase, pill shape.

**Priority:** P0 (exists).

---

#### Card

**Purpose:** Raised surface for grouping related content.

**CSS:**
```css
@layer components {
  .card {
    background: var(--color-surface-raised);
    border: 1px solid oklch(from var(--color-brand-gray) l c h / 0.15);
    border-radius: var(--radius-md);
    padding: var(--density-space-md);
  }
}
```

**Priority:** P0 (wraps recipient details, deadline displays).

---

#### DescriptionList (`.info-list`)

**Purpose:** Key-value display for recipient details, season info.

**HTML:**
```html
<dl class="info-list">
  <div class="info-item">
    <dt class="info-label">Name</dt>
    <dd class="info-value">Oleksandr</dd>
  </div>
  <div class="info-item">
    <dt class="info-label">Phone</dt>
    <dd class="info-value">
      <a href="tel:+380671234567" class="info-link">+380 67 123 45 67</a>
    </dd>
  </div>
  <div class="info-item">
    <dt class="info-label">NP Branch</dt>
    <dd class="info-value">Kyiv #123</dd>
  </div>
</dl>
```

**CSS:**
```css
@layer components {
  .info-list {
    display: flex;
    flex-direction: column;
    gap: var(--density-space-sm);
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .info-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted);
  }

  .info-value {
    font-size: 1.05rem;
    color: var(--color-text);
  }

  .info-link {
    color: var(--color-accent);
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  @media (min-width: 640px) {
    .info-item {
      flex-direction: row;
      gap: var(--density-space-md);
    }
    .info-label {
      min-width: 8rem;
      text-align: right;
    }
  }
}
```

**Mobile:** Stacked layout (label above value) on mobile. Side-by-side on desktop.

**Priority:** P0 (replaces unstyled `<dl>` grids in `.prose-page`).

---

#### Deadline

**Purpose:** Time-sensitive information with visual urgency when approaching.

**HTML:**
```html
<div class="deadline" data-urgency="normal" data-testid="signup-deadline">
  <span class="deadline-label">Sign-up deadline</span>
  <time class="deadline-value" datetime="2026-04-21">21 April 2026</time>
</div>
```

**CSS:**
```css
@layer components {
  .deadline {
    --_accent: var(--deadline-accent, var(--color-brand-gray));
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding-left: var(--density-space-sm);
    border-left: 3px solid var(--_accent);
  }

  .deadline-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted);
  }

  .deadline-value {
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .deadline[data-urgency="soon"] {
    --deadline-accent: var(--color-accent);
  }

  .deadline[data-urgency="imminent"] {
    --deadline-accent: var(--color-error);
  }

  .deadline[data-urgency="imminent"] .deadline-value {
    color: var(--color-error);
  }
}
```

**Leptos:**
```rust
#[component]
fn Deadline(
    #[prop(into)] label: String,
    #[prop(into)] date: NaiveDate,
    #[prop(into)] testid: String,
) -> impl IntoView {
    let urgency = {
        let days_left = (date - Utc::now().date_naive()).num_days();
        if days_left <= 1 { "imminent" }
        else if days_left <= 7 { "soon" }
        else { "normal" }
    };

    view! {
        <div class="deadline" data-urgency=urgency data-testid=testid>
            <span class="deadline-label">{label}</span>
            <time class="deadline-value" datetime=date.to_string()>
                {format_date_uk(date)}
            </time>
        </div>
    }
}
```

**Urgency thresholds:** <=1 day = imminent (red), <=7 days = soon (orange), >7 days = normal (gray).

**Priority:** P0 (currently plain text, critical for participant awareness).

---

#### PhaseStepper

**Purpose:** Horizontal timeline showing season progress.

**Decision: Horizontal, for both participant and admin.** A single `PhaseStepper` component works everywhere. On participant pages it provides orientation ("where am I in this process?"). On admin pages it shows the same thing with operational context. Horizontal because the phases are a sequence with direction, and horizontal scrolling on mobile is natural for timeline-type UI.

**HTML:**
```html
<nav class="stepper" aria-label="Season progress" data-testid="phase-stepper">
  <div class="step" data-status="completed">
    <div class="step-marker" aria-hidden="true">&#x2713;</div>
    <div class="step-label">Enrollment</div>
  </div>
  <div class="step-connector" aria-hidden="true"></div>
  <div class="step" data-status="current" aria-current="step">
    <div class="step-marker" aria-hidden="true">2</div>
    <div class="step-label">Preparation</div>
  </div>
  <div class="step-connector" aria-hidden="true"></div>
  <div class="step" data-status="locked">
    <div class="step-marker" aria-hidden="true">3</div>
    <div class="step-label">Assignment</div>
  </div>
</nav>
```

**CSS:**
```css
@layer components {
  .stepper {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0;
    padding: var(--density-space-md) 0;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.375rem;
    flex-shrink: 0;
    min-width: 3.5rem;
  }

  .step-marker {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 50%;
    font-weight: 600;
    font-size: 0.75rem;
    background: oklch(from var(--color-brand-gray) l c h / 0.2);
    color: var(--color-text-muted);
    transition: background 120ms ease, box-shadow 120ms ease;
  }

  .step[data-status="completed"] .step-marker {
    background: var(--color-success);
    color: white;
  }

  .step[data-status="current"] .step-marker {
    background: var(--color-accent);
    color: white;
    box-shadow: 0 0 0 3px oklch(from var(--color-accent) l c h / 0.2);
  }

  .step[data-status="locked"] .step-marker {
    opacity: 0.4;
  }

  .step-label {
    font-weight: 600;
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted);
    text-align: center;
    max-width: 5rem;
  }

  .step[data-status="current"] .step-label {
    color: var(--color-text);
  }

  .step-connector {
    width: 1.5rem;
    height: 2px;
    background: oklch(from var(--color-brand-gray) l c h / 0.2);
    flex-shrink: 0;
    align-self: flex-start;
    margin-top: 1.25rem;
  }

  .step[data-status="completed"] + .step-connector {
    background: var(--color-success);
  }
}
```

**Leptos:**
```rust
#[component]
fn PhaseStepper(current: Phase) -> impl IntoView {
    let phases = [
        (Phase::Enrollment, "Enrollment"),
        (Phase::Preparation, "Preparation"),
        (Phase::Assignment, "Assignment"),
        (Phase::Delivery, "Delivery"),
        (Phase::Complete, "Complete"),
    ];

    view! {
        <nav class="stepper" aria-label="Season progress" data-testid="phase-stepper">
            {phases
                .into_iter()
                .enumerate()
                .flat_map(|(idx, (phase, label))| {
                    let status = if phase < current {
                        "completed"
                    } else if phase == current {
                        "current"
                    } else {
                        "locked"
                    };

                    let connector = (idx > 0).then(|| view! {
                        <div class="step-connector" aria-hidden="true"></div>
                    });

                    let marker_content = if status == "completed" {
                        "\u{2713}".to_string()
                    } else {
                        (idx + 1).to_string()
                    };

                    let step = view! {
                        <div class="step" data-status=status
                             aria-current=if status == "current" { Some("step") } else { None }>
                            <div class="step-marker" aria-hidden="true">{marker_content}</div>
                            <div class="step-label">{label}</div>
                        </div>
                    };

                    [connector.map(|c| c.into_any()), Some(step.into_any())]
                })
                .flatten()
                .collect_view()}
        </nav>
    }
}
```

**Mobile:** Horizontal scroll on narrow screens (`overflow-x: auto`). Steps shrink to 3.5rem min-width.

**A11y:** `aria-label="Season progress"` on container. `aria-current="step"` on active step. Markers are `aria-hidden` (decorative). Color + icon + text (not color alone).

**Priority:** P0 (no progress indicator exists; critical for user orientation).

---

### Navigation

#### Header (`.app-header`)

**Purpose:** App-level header with logo and navigation. Already exists.

**Decision: Hamburger menu, not bottom nav.** Bottom nav wastes 60-80px of vertical space permanently for a 5-page app. The hamburger menu hides behind a single tap, freeing the viewport for content. On desktop (640px+), the hamburger is hidden and nav links display inline.

**Mobile enhancement -- hamburger menu toggle:**

**CSS:**
```css
@layer components {
  .menu-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.75rem;
    height: 2.75rem;
    border: none;
    background: none;
    color: var(--color-text);
    cursor: pointer;
    border-radius: var(--radius-md);
  }

  .menu-toggle:focus-visible {
    outline: 2px solid var(--color-focus);
    outline-offset: 2px;
  }

  @media (min-width: 640px) {
    .menu-toggle { display: none; }
  }
}
```

**Priority:** P1 (current nav works but lacks mobile hamburger).

---

#### MobileMenu

**Purpose:** Slide-in navigation panel for mobile.

**CSS:**
```css
@layer components {
  .mobile-menu-overlay {
    position: fixed;
    inset: 0;
    background: oklch(from var(--color-brand-black) l c h / 0.4);
    z-index: 40;
  }

  .mobile-menu {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 16rem;
    max-width: 80vw;
    background: var(--color-surface-raised);
    z-index: 50;
    padding: var(--density-space-lg) var(--density-space-md);
    display: flex;
    flex-direction: column;
    gap: var(--density-space-sm);
    box-shadow: -4px 0 16px oklch(from var(--color-brand-black) l c h / 0.1);
    animation: slide-in-right 200ms ease;
  }

  .mobile-menu a {
    display: block;
    padding: var(--density-space-sm) var(--density-space-md);
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text);
    border-radius: var(--radius-md);
    text-decoration: none;
  }

  .mobile-menu a:hover {
    background: oklch(from var(--color-brand-gray) l c h / 0.1);
  }

  .mobile-menu a[aria-current="page"] {
    color: var(--color-accent);
    background: oklch(from var(--color-accent) l c h / 0.08);
  }

  @keyframes slide-in-right {
    from { translate: 100% 0; }
    to { translate: 0 0; }
  }
}
```

**Leptos:**
```rust
#[component]
fn MobileMenu(on_close: Callback<()>) -> impl IntoView {
    view! {
        <div class="mobile-menu-overlay" on:click=move |_| on_close.call(())></div>
        <nav class="mobile-menu">
            <button
                on:click=move |_| on_close.call(())
                aria-label="Close menu"
                class="self-end text-2xl p-2"
            >
                "\u{2715}"
            </button>
            <a href="/">"Home"</a>
            <a href="/admin">"Admin"</a>
            <a href="/logout">"Logout"</a>
        </nav>
    }
}
```

**A11y:** `aria-expanded` on toggle button. `aria-current="page"` on active link. Overlay click closes menu. Escape key closes menu.

**Priority:** P1.

---

#### AdminNav (`.admin-nav`)

**Purpose:** Sub-navigation for admin pages. Already exists.

**Priority:** P0 (exists).

---

### Admin

#### StatCard

**Purpose:** Single metric display for admin dashboard.

**HTML:**
```html
<div class="stat-card" data-testid="enrolled-count">
  <div class="stat-label">Enrolled</div>
  <div class="stat-value">18</div>
</div>
```

**CSS:**
```css
@layer components {
  .stat-card {
    --_border: var(--stat-border, oklch(from var(--color-brand-gray) l c h / 0.15));
    background: var(--color-surface-raised);
    border: 1px solid var(--_border);
    border-radius: var(--radius-md);
    padding: var(--density-space-md);
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .stat-card[data-status="alert"] {
    --stat-border: var(--color-error);
  }

  .stat-card[data-status="success"] {
    --stat-border: var(--color-success);
  }

  .stat-value {
    font-family: var(--font-display);
    font-weight: 900;
    font-size: 2rem;
    line-height: 1;
    color: var(--color-text);
  }

  .stat-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted);
  }
}
```

**Leptos:**
```rust
#[component]
fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional, into)] status: Option<&'static str>,
    #[prop(into)] testid: String,
) -> impl IntoView {
    view! {
        <div class="stat-card" data-status=status data-testid=testid>
            <div class="stat-label">{label}</div>
            <div class="stat-value">{value}</div>
        </div>
    }
}
```

**Mobile:** Full-width stacked on mobile. 2-3 column grid on desktop (parent layout, not stat-card itself): `class="grid gap-4 sm:grid-cols-3"`.

**Priority:** P1 (admin dashboard enhancement).

---

#### ActionPanel

**Purpose:** Highlighted section showing the next admin action with context.

**CSS:**
```css
@layer components {
  .action-panel {
    background: var(--color-surface-raised);
    border: 2px solid var(--color-accent);
    border-radius: var(--radius-lg);
    padding: var(--density-space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--density-space-md);
  }

  .action-panel-title {
    font-family: var(--font-display);
    font-weight: 900;
    font-size: 1.3rem;
    line-height: 1.15;
    color: var(--color-text);
  }
}
```

**Priority:** P1.

---

#### DangerZone

**Purpose:** Visually separated area for destructive actions.

**CSS:**
```css
@layer components {
  .danger-zone {
    background: oklch(from var(--color-error) l c h / 0.05);
    border: 1px solid oklch(from var(--color-error) l c h / 0.2);
    border-radius: var(--radius-md);
    padding: var(--density-space-md);
  }

  .danger-zone-title {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-error);
    margin-bottom: 0.5rem;
  }
}
```

**Priority:** P1.

---

#### DataTable Wrapper

**Purpose:** Horizontal scroll container for admin tables. Already exists as `.data-table`.

**New CSS:**
```css
@layer components {
  .data-table-wrapper {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
    border: 1px solid oklch(from var(--color-brand-gray) l c h / 0.15);
    border-radius: var(--radius-md);
  }
}
```

**Mobile:** Table scrolls sideways. Columns don't collapse. Acceptable for admin (10-30 rows, known content).

**Priority:** P0 (add scroll wrapper around existing tables).

---

#### ConfirmDialog

**Purpose:** "Are you sure?" barrier for destructive/irreversible actions.

**CSS:**
```css
@layer components {
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: oklch(from var(--color-brand-black) l c h / 0.6);
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--density-space-md);
  }

  .modal {
    background: var(--color-surface-raised);
    border: none;
    border-radius: var(--radius-lg);
    padding: var(--density-space-lg);
    max-width: 28rem;
    width: 100%;
    box-shadow: 0 8px 24px oklch(from var(--color-brand-black) l c h / 0.2);
  }

  .modal-title {
    font-family: var(--font-display);
    font-weight: 900;
    font-size: 1.3rem;
    line-height: 1.15;
    margin-bottom: var(--density-space-sm);
  }

  .modal-actions {
    display: flex;
    gap: var(--density-space-sm);
    justify-content: flex-end;
    margin-top: var(--density-space-md);
  }
}
```

**Leptos:**
```rust
#[component]
fn ConfirmDialog(
    #[prop(into)] title: String,
    #[prop(into)] body: String,
    #[prop(into)] confirm_label: String,
    #[prop(optional)] destructive: Option<bool>,
    on_confirm: impl Fn() + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let variant = if destructive.unwrap_or(false) { "destructive" } else { "primary" };
    view! {
        <div class="modal-overlay" data-testid="confirm-dialog"
             on:click=move |_| on_cancel()>
            <dialog class="modal" open aria-modal="true" aria-labelledby="modal-title"
                    on:click=|ev| ev.stop_propagation()>
                <h2 class="modal-title" id="modal-title">{title}</h2>
                <p class="text-sm text-(--color-text-muted)">{body}</p>
                <div class="modal-actions">
                    <button class="btn" data-variant="secondary"
                            data-testid="modal-cancel"
                            on:click=move |_| on_cancel()>
                        "Cancel"
                    </button>
                    <button class="btn" data-variant=variant
                            data-testid="modal-confirm"
                            on:click=move |_| on_confirm()>
                        {confirm_label}
                    </button>
                </div>
            </dialog>
        </div>
    }
}
```

**Mobile:** Dialog fills 100% width minus margin. Buttons stay side-by-side (labels are short).

**A11y:** `aria-modal="true"`, `aria-labelledby` links to title. Overlay click dismisses. Focus trapped inside dialog (implement via Effect setting focus on first button).

**Priority:** P1 (needed for cancel season, deactivate participant, release assignments).

---

#### CycleVisualization

**Purpose:** SVG ring graph showing assignment cycle topology.

**Decision: Ring/circle layout, not list.** The cycle is the defining structure of the assignment algorithm. A circular graph makes the "everyone sends to the next person in the ring" topology immediately visible. A list would hide the cyclic nature.

**CSS:**
```css
@layer components {
  .cycle-viz-container {
    display: flex;
    justify-content: center;
    padding: var(--density-space-md);
  }

  .cycle-viz {
    width: 100%;
    max-width: 400px;
    aspect-ratio: 1;
  }
}
```

**Leptos:**
```rust
#[component]
fn CycleVisualization(
    participants: Vec<CycleNode>,
) -> impl IntoView {
    let total = participants.len();
    let radius = 160.0_f64;

    view! {
        <figure class="cycle-viz-container" data-testid="cycle-visualization">
            <figcaption class="sr-only">
                "Assignment cycle: " {total} " participants"
            </figcaption>
            <svg viewBox="0 0 400 400" class="cycle-viz" role="img"
                 aria-label=format!("Assignment cycle: {total} participants")>
                <defs>
                    <marker id="arrowhead" viewBox="0 0 10 7" refX="10" refY="3.5"
                            markerWidth="8" markerHeight="6" orient="auto-start-reverse">
                        <polygon points="0 0, 10 3.5, 0 7" fill="var(--color-text-muted)" />
                    </marker>
                </defs>
                <g transform="translate(200, 200)">
                    {participants.iter().enumerate().map(|(i, node)| {
                        let angle = (i as f64 / total as f64) * std::f64::consts::TAU
                                    - std::f64::consts::FRAC_PI_2;
                        let x = radius * angle.cos();
                        let y = radius * angle.sin();
                        let next_i = (i + 1) % total;
                        let next_angle = (next_i as f64 / total as f64) * std::f64::consts::TAU
                                         - std::f64::consts::FRAC_PI_2;
                        let nx = radius * next_angle.cos();
                        let ny = radius * next_angle.sin();

                        view! {
                            <line x1=x y1=y x2=nx y2=ny
                                  stroke="var(--color-text-muted)"
                                  stroke-width="1.5"
                                  marker-end="url(#arrowhead)"
                                  opacity="0.5" />
                            <circle cx=x cy=y r="14" fill=node.color()
                                    data-testid=format!("node-{}", node.id) />
                            <text x=x y={y + 24.0} text-anchor="middle"
                                  font-size="10" fill="var(--color-text)">
                                {node.short_name.clone()}
                            </text>
                        }
                    }).collect_view()}
                </g>
            </svg>
        </figure>
    }
}
```

**Mobile:** SVG scales to container width. Labels readable at 320px viewport.

**A11y:** `role="img"` on SVG, `aria-label` with participant count, `<figcaption class="sr-only">` for screen readers.

**Priority:** P1.

---

## The Assignment Reveal

This is the emotional climax of the entire season. The participant has been creating something -- a letter, a drawing, a small gift -- without knowing who will receive it. Now they learn.

### Design Principles

1. **Ceremonial pace.** 1.4 seconds total. Not instantaneous, not sluggish.
2. **Information hierarchy.** Name first (who), then branch (where), then phone (how to coordinate).
3. **One-time magic.** First view: full animation + confetti. Re-visits: content visible immediately.
4. **Mobile-native.** Full-width, thumb-triggerable, no pinch/zoom needed.
5. **Graceful degradation.** Reduced motion: instant reveal, no confetti.

### Decision: CSS 3D flap rotation

Not a slide-up (too generic, every app does it). Not a fade (too subtle for the emotional weight). The envelope flap rotates open in 3D space using CSS `perspective` + `rotateX`. It is physical, tactile, and references the literal act of opening mail. The `cubic-bezier(0.68, -0.55, 0.27, 1.55)` easing creates a slight overshoot bounce that feels satisfying.

### Decision: Confetti yes, first-time only

Confetti fires on the first reveal. Re-visits show content immediately with no animation. The `already_seen` flag comes from the server (stored as a boolean on the assignment record). This prevents confetti on page refresh.

### HTML Structure

```html
<article class="reveal-envelope" data-testid="reveal-envelope"
         role="button" tabindex="0"
         aria-expanded="false" aria-label="Tap to reveal your recipient">
  <div class="envelope-body" aria-hidden="true">
    <div class="envelope-flap"></div>
  </div>
  <div class="envelope-card">
    <h2 class="envelope-recipient" data-testid="recipient-name">Oleksandr</h2>
    <dl class="info-list">
      <div class="info-item">
        <dt class="info-label">Nova Poshta</dt>
        <dd class="info-value">Kyiv #123</dd>
      </div>
      <div class="info-item">
        <dt class="info-label">Phone</dt>
        <dd class="info-value">
          <a href="tel:+380671234567" class="info-link">+380 67 123 45 67</a>
        </dd>
      </div>
    </dl>
  </div>
</article>

<div class="confetti" aria-hidden="true"></div>
```

### Full CSS

```css
@layer components {
  .reveal-envelope {
    perspective: 1000px;
    width: 100%;
    max-width: 24rem;
    aspect-ratio: 5 / 3;
    position: relative;
    cursor: pointer;
    margin-inline: auto;
  }

  .envelope-body {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      135deg,
      var(--color-accent) 0%,
      oklch(from var(--color-accent) calc(l - 0.08) c h) 100%
    );
    border-radius: var(--radius-lg);
    box-shadow: 0 4px 16px oklch(from var(--color-brand-black) l c h / 0.15);
  }

  .envelope-flap {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 50%;
    background: linear-gradient(
      135deg,
      oklch(from var(--color-accent) calc(l - 0.04) c h) 0%,
      oklch(from var(--color-accent) calc(l - 0.12) c h) 100%
    );
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    transform-origin: top center;
    transform-style: preserve-3d;
    transition: transform 600ms cubic-bezier(0.68, -0.55, 0.27, 1.55);
    z-index: 2;
  }

  .reveal-envelope[aria-expanded="true"] .envelope-flap {
    transform: rotateX(-170deg);
  }

  .envelope-card {
    position: absolute;
    inset: 0;
    background: var(--color-surface-raised);
    border-radius: var(--radius-lg);
    padding: var(--density-space-lg);
    display: flex;
    flex-direction: column;
    justify-content: center;
    opacity: 0;
    transform: translateY(1rem);
    transition: opacity 600ms ease 200ms, transform 600ms ease 200ms;
    z-index: 1;
  }

  .reveal-envelope[aria-expanded="true"] .envelope-card {
    opacity: 1;
    transform: translateY(0);
  }

  .reveal-envelope[aria-expanded="true"] .envelope-body {
    pointer-events: none;
  }

  .envelope-recipient {
    font-family: var(--font-display);
    font-weight: 900;
    font-size: clamp(1.3rem, 5vw, 1.8rem);
    line-height: 1.15;
    color: var(--color-text);
    margin-bottom: var(--density-space-md);
  }

  /* Confetti */
  .confetti {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 60;
    opacity: 0;
  }

  .confetti[data-active="true"] {
    animation: confetti-burst 1.2s ease-out forwards;
    background-image:
      radial-gradient(circle, var(--color-brand-orange) 20%, transparent 20%),
      radial-gradient(circle, var(--color-brand-pink) 15%, transparent 15%),
      radial-gradient(circle, var(--color-brand-blue) 18%, transparent 18%),
      radial-gradient(circle, var(--color-brand-orange) 12%, transparent 12%),
      radial-gradient(circle, var(--color-brand-pink) 20%, transparent 20%);
    background-size: 8px 8px, 6px 6px, 10px 10px, 5px 5px, 7px 7px;
  }

  @keyframes confetti-burst {
    0% {
      opacity: 1;
      background-position:
        50% 50%, 30% 40%, 70% 45%, 40% 55%, 60% 35%;
    }
    100% {
      opacity: 0;
      background-position:
        50% 120%, 10% 110%, 90% 115%, 20% 125%, 80% 105%;
    }
  }
}
```

### Leptos Component

```rust
#[component]
fn AssignmentReveal(
    recipient_name: String,
    recipient_branch: String,
    recipient_phone: String,
    /// If true, skip animation (re-visit)
    #[prop(optional)]
    already_seen: bool,
) -> impl IntoView {
    let (revealed, set_revealed) = signal(already_seen);
    let (confetti_active, set_confetti_active) = signal(false);

    let on_reveal = move |_| {
        if !revealed.get_untracked() {
            set_revealed.set(true);
            if !already_seen {
                set_confetti_active.set(true);
                set_timeout(
                    move || set_confetti_active.set(false),
                    Duration::from_millis(1500),
                );
            }
        }
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" || ev.key() == " " {
            ev.prevent_default();
            on_reveal(());
        }
    };

    view! {
        <article
            class="reveal-envelope"
            data-testid="reveal-envelope"
            role="button"
            tabindex="0"
            aria-expanded=move || revealed.get()
            aria-label="Tap to reveal your recipient"
            on:click=move |_| on_reveal(())
            on:keydown=on_keydown
        >
            <div class="envelope-body" aria-hidden="true">
                <div class="envelope-flap"></div>
            </div>
            <div class="envelope-card">
                <h2 class="envelope-recipient" data-testid="recipient-name">
                    {recipient_name}
                </h2>
                <dl class="info-list">
                    <div class="info-item">
                        <dt class="info-label">"Nova Poshta"</dt>
                        <dd class="info-value">{recipient_branch}</dd>
                    </div>
                    <div class="info-item">
                        <dt class="info-label">"Phone"</dt>
                        <dd class="info-value">
                            <a href=format!("tel:{}", recipient_phone) class="info-link">
                                {recipient_phone}
                            </a>
                        </dd>
                    </div>
                </dl>
            </div>
        </article>

        <div class="confetti"
             data-active=move || confetti_active.get()
             aria-hidden="true">
        </div>
    }
}
```

### Timing Sequence

```
0ms      -- User taps envelope
0-600ms  -- Flap rotates open (cubic-bezier bounce)
200ms    -- Card begins fade-in + slide-up
800ms    -- Card fully visible
200ms    -- Confetti burst begins
1200ms   -- Confetti fades out
1400ms   -- Sequence complete
```

### Re-visit Behavior

When `already_seen` is true: `revealed` signal starts as `true`. No animation plays (CSS transitions already in end state). No confetti. Content immediately visible.

### Reduced Motion

With `prefers-reduced-motion: reduce`: `transition: none` on flap and card -- state changes are instant. `.confetti { display: none }` -- no particle animation. Content is still revealed on tap, just without motion.

---

## Form System

### ActionForm Pattern

All mutations use `ActionForm`. This is non-negotiable.

```rust
let action = ServerAction::<EnrollInSeason>::new();
let pending = action.pending();
let hydrated = use_hydrated();

view! {
    <ActionForm action=action>
        <div class="field">
            <label class="field-label" for="branch">"NP Branch"</label>
            <input class="field-input" type="text" id="branch" name="branch"
                   required data-testid="branch-input" />
        </div>
        <button type="submit" class="btn"
                disabled=move || !hydrated.get() || pending.get()
                data-testid="enroll-button">
            {move || if pending.get() { "Enrolling..." } else { "Enroll" }}
        </button>
    </ActionForm>

    {move || action.value().get().and_then(|r| r.err()).map(|e| view! {
        <div class="alert" role="alert" aria-live="assertive"
             data-testid="action-error">
            {e.to_string()}
        </div>
    })}
}
```

### Key rules

1. **`name` attributes match server function parameters.** ActionForm reads DOM via FormData.
2. **No `on:input` signals for form values.** Playwright `.fill()` does not fire Leptos event handlers.
3. **Hydration gate on submit button.** `disabled=move || !hydrated.get()` prevents pre-hydration clicks.
4. **Pending gate on submit button.** `disabled=move || pending.get()` prevents double-submit.
5. **Error display via `action.value()`.** Check for `Err` variant, render as `.alert`.

### NP Field Split

Currently a single free-form text field. Planned split into two fields:

```rust
<div class="flex flex-col gap-(--density-space-md) sm:flex-row sm:gap-(--density-space-sm)">
    <div class="field sm:w-1/2">
        <label class="field-label" for="np-city">"City"</label>
        <select class="field-input" id="np-city" name="city" required
                data-testid="np-city-input">
            <option value="">"Select city"</option>
            <option value="kyiv">"Kyiv"</option>
            <option value="lviv">"Lviv"</option>
            <option value="odesa">"Odesa"</option>
            <option value="kharkiv">"Kharkiv"</option>
            <option value="dnipro">"Dnipro"</option>
        </select>
    </div>
    <div class="field sm:w-1/2">
        <label class="field-label" for="np-number">"Branch number"</label>
        <input class="field-input" type="text" id="np-number" name="np_number"
               inputmode="numeric" placeholder="123" required
               data-testid="np-number-input" />
    </div>
</div>
```

Server function receives `city: String` and `np_number: String` separately. Validation is clearer: "Invalid city" vs "Invalid branch number" instead of "Invalid branch format."

### Error Display Strategy

- **Field-level errors:** Inline `.field-error` below the offending field with `aria-describedby` linkage.
- **Form-level errors:** `.alert` block below the form with `role="alert"`.
- **Scroll into view:** When an error renders, use an Effect to call `scrollIntoView({ behavior: "smooth", block: "center" })` on the error element.
- **No toast for errors.** Errors are inline so the user sees error + input simultaneously, especially on mobile where the keyboard is visible.

### Touch Targets

| Element | Min Height | Notes |
|---------|-----------|-------|
| `.field-input` | 44px | `font-size: 16px` prevents iOS zoom |
| `.btn` (default) | 44px | |
| `.btn[data-size="sm"]` | 36px | Admin only |
| `.btn[data-size="lg"]` | 48px | |

### Virtual Keyboard Triggers

| Input | Attributes | Keyboard |
|-------|-----------|----------|
| Phone | `type="tel"` + `inputmode="tel"` | Numeric + symbols (+, -, etc.) |
| OTP | `type="text"` + `inputmode="numeric"` | Numeric only |
| Branch number | `type="text"` + `inputmode="numeric"` | Numeric only |
| Text (name, theme) | `type="text"` | Standard QWERTY |

---

## Feedback Strategy

### Skeleton Screens

Replace all `"Loading..."` / `"Завантаження..."` text strings with generic pulse lines:

```rust
<Suspense fallback=move || view! {
    <div class="space-y-3" aria-hidden="true">
        <div class="skeleton-line h-6 w-2/3"></div>
        <div class="skeleton-line h-4 w-full"></div>
        <div class="skeleton-line h-4 w-3/4"></div>
    </div>
}>
```

### Button States

Three states driven by hydration gate + action pending:

```
1. Pre-hydration: disabled, opacity 0.45
2. Idle:          enabled, full opacity, label text
3. Pending:       disabled, spinner + loading text
```

Error is not a button state -- errors are shown inline via `.alert`.

### Toast

Toasts for non-blocking success feedback only. Architecture described in the Toast component section above. Key pattern: persistent live region in app shell, context-provided signal, 5s auto-dismiss.

### Empty States

Each page that can show empty data gets a contextual `<EmptyState>` (see empty state copy table in the EmptyState component section).

---

## Mobile Architecture

### Viewport Meta

```html
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover, user-scalable=yes" />
```

- `viewport-fit=cover` -- extends into notch/safe areas
- `user-scalable=yes` -- WCAG requires zoom capability. Never disable.

### Dynamic Viewport Height

```css
@layer base {
  body {
    min-height: 100dvh;
    padding-left: env(safe-area-inset-left);
    padding-right: env(safe-area-inset-right);
  }
}
```

`dvh` handles iOS Safari's collapsing address bar. The `100vh` already in the app shell `min-h-dvh` class handles this at component level.

### Safe Area Insets

Fixed bottom elements (toast) must account for home indicator:
```css
.toast {
  padding-bottom: env(safe-area-inset-bottom);
}
```

### iOS Safari Specifics

1. **Input font-size: 16px minimum** -- prevents auto-zoom on focus. Already set on `.field-input`.
2. **No `overflow-x: hidden` on body** -- breaks `position: fixed`. Use `overflow-x: clip` (already set).
3. **`dvh` instead of `vh`** -- handles collapsing address bar.

### CTA Placement

Primary action buttons ("Enroll", "Confirm Ready", "See Assignment") placed in center of content flow. NOT fixed to bottom of viewport (avoids conflict with iOS home indicator and keyboard). Center of screen is the most-viewed and most-touched area on phones.

### PWA Minimum

`public/manifest.json`:
```json
{
  "name": "Same Te Mail Club",
  "short_name": "Mail Club",
  "start_url": "/",
  "scope": "/",
  "display": "standalone",
  "orientation": "portrait-primary",
  "background_color": "#FAF9F6",
  "theme_color": "#D93A12",
  "icons": [
    { "src": "/favicon-192.png", "sizes": "192x192", "type": "image/png" },
    { "src": "/favicon-512.png", "sizes": "512x512", "type": "image/png" }
  ]
}
```

In HTML head:
```html
<link rel="manifest" href="/manifest.json" />
<meta name="theme-color" content="#D93A12" />
<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
```

Icons: Generate 192x192 and 512x512 PNGs from the orange mark SVG.

---

## CSS Additions

All additions go into `style/tailwind.css`. Current file is 425 lines. All new CSS is provided inline with each component above. This section collects the complete additions for easy copy-paste.

### New `@layer base` additions

Add to the existing `@layer base` block:

```css
body {
  min-height: 100dvh;
  padding-left: env(safe-area-inset-left);
  padding-right: env(safe-area-inset-right);
}
```

### New `@layer components` additions

Append to the existing `@layer components` block. The full CSS for each class is defined in the component catalog above. Summary of new classes:

| Class Group | Classes | Component |
|-------------|---------|-----------|
| Field enhancements | `.field-input` min-height, `select.field-input`, `textarea.field-input`, `.field-input[data-otp]` text-align | Field |
| Button enhancements | `.btn` min-height, `a.btn` | Button, LinkButton |
| Card | `.card` | Card |
| Info list | `.info-list`, `.info-item`, `.info-label`, `.info-value`, `.info-link` | DescriptionList |
| Deadline | `.deadline`, `.deadline-label`, `.deadline-value` | Deadline |
| Empty state | `.empty-state`, `.empty-state-headline`, `.empty-state-body` | EmptyState |
| Skeleton | `.skeleton-line`, `@keyframes skeleton-pulse` | Skeleton |
| Toast | `.toast`, `.toast-message`, `@keyframes toast-in` | Toast |
| Stepper | `.stepper`, `.step`, `.step-marker`, `.step-label`, `.step-connector` | PhaseStepper |
| Stat card | `.stat-card`, `.stat-value`, `.stat-label` | StatCard |
| Action panel | `.action-panel`, `.action-panel-title` | ActionPanel |
| Danger zone | `.danger-zone`, `.danger-zone-title` | DangerZone |
| Data table wrapper | `.data-table-wrapper` | DataTable |
| Modal | `.modal-overlay`, `.modal`, `.modal-title`, `.modal-actions` | ConfirmDialog |
| Menu | `.menu-toggle`, `.mobile-menu-overlay`, `.mobile-menu`, `@keyframes slide-in-right` | Header, MobileMenu |
| Envelope | `.reveal-envelope`, `.envelope-body`, `.envelope-flap`, `.envelope-card`, `.envelope-recipient` | AssignmentReveal |
| Confetti | `.confetti`, `@keyframes confetti-burst` | AssignmentReveal |
| Cycle viz | `.cycle-viz-container`, `.cycle-viz` | CycleVisualization |

### Reduced motion additions

Add to the existing `@media (prefers-reduced-motion: reduce)` block in `@layer base`:

```css
@media (prefers-reduced-motion: reduce) {
  .skeleton-line { animation: none; opacity: 0.7; }
  .toast { animation: none; }
  .step-marker { transition: none; }
  .envelope-flap { transition: none; }
  .envelope-card { transition: none; }
  .confetti { display: none; }
  .mobile-menu { animation: none; }
}
```

Note: The existing blanket rule `*, *::before, *::after { transition: none !important; animation: none !important; }` inside `@media (prefers-reduced-motion: reduce)` already handles most of this. The explicit rules above serve as documentation of which components have animations.

### Dark mode

No additions needed. All new components use semantic tokens (`--color-surface`, `--color-surface-raised`, `--color-text`, `--color-text-muted`) which adapt automatically via the existing `@media (prefers-color-scheme: dark)` block.

**Decision: System-only dark mode, no toggle.** For 10-30 users in a community app, a manual toggle is an unnecessary feature. System preference respects user intent automatically.

### Z-index documentation

| Layer | Z-index | Element |
|-------|---------|---------|
| Grain overlay | 1 | `body::after` |
| Toast | 40 | `.toast` container |
| Mobile menu overlay | 40 | `.mobile-menu-overlay` |
| Mobile menu panel | 50 | `.mobile-menu` |
| Modal overlay | 50 | `.modal-overlay` |
| Confetti | 60 | `.confetti` |

---

## Implementation Roadmap

### Phase 1: Foundations (P0) -- 1 day

**Deliverables:**
1. Add all new CSS classes to `style/tailwind.css` (all from this spec)
2. Add viewport meta tag with `viewport-fit=cover` and `user-scalable=yes`
3. Add `min-height: 100dvh` and safe-area padding to body in `@layer base`
4. Add `manifest.json` to `public/` and link in HTML head
5. Add `.field-input` min-height (44px)
6. Add `.data-table-wrapper` for horizontal scroll on admin tables
7. Add `.card` class and wrap recipient details in cards
8. Add `.info-list` / `.info-item` classes and replace unstyled `<dl>` grids
9. Add `.deadline` styling and replace plain text deadlines
10. Add `<EmptyState>` component and replace bare text empty states across all pages

**Verification:** `just check` passes. Visual spot-check: deadline urgency colors, empty states centered, recipient card elevated, tables scroll horizontally on mobile.

### Phase 2: Participant Journey (P0-P1) -- 2 days

**Deliverables:**
1. Extract reusable `<Field>` component wrapping label + input + error with ARIA linkage
2. `<PhaseStepper>` component on participant home page
3. `<AssignmentReveal>` envelope component (full animation + confetti)
4. Button loading states with spinner SVG across all ActionForms
5. Skeleton screens replacing "Loading..." in `<Suspense>` fallbacks
6. Error scroll-into-view behavior (Effect + `scrollIntoView`)
7. Toast system: context provider in app shell, persistent live region, auto-dismiss

**Verification:** Walk through full participant journey manually. Assignment reveal animates on first view, instant on revisit. Reduced motion disables all animation. Skeleton screens appear during load. Toast appears on enrollment success.

### Phase 3: Admin Dashboard (P1) -- 2 days

**Deliverables:**
1. `<StatCard>` component on admin dashboard
2. `<ActionPanel>` for next-action section
3. `<DangerZone>` for destructive actions
4. `<PhaseStepper>` on admin dashboard (reuse participant component)
5. `<CycleVisualization>` SVG ring graph on assignments page
6. `<ConfirmDialog>` for cancel season, deactivate participant, release assignments
7. SMS trigger loading state and inline report improvements

**Verification:** Admin dashboard shows stat cards, stepper, action panel, danger zone. Cycle viz renders for 3-20 participant rings correctly. Confirm dialog blocks destructive actions.

### Phase 4: Mobile Navigation + Forms (P1) -- 1 day

**Deliverables:**
1. Hamburger menu toggle on header (mobile only, hidden on desktop)
2. Mobile menu slide-in panel with overlay
3. Admin nav integrated into mobile menu
4. `aria-expanded`, `aria-current="page"`, Escape key handling
5. NP field split: city select + branch number input (server function updated to receive `city` + `np_number`)

**Verification:** Menu opens/closes on mobile. Desktop shows inline nav. Active page highlighted. Escape closes menu. NP fields show native picker for city, numeric keyboard for branch.

### Phase 5: Polish (P2) -- 1 day

**Deliverables:**
1. Dark mode verification for all new components (semantic tokens reassign)
2. Accessibility audit: focus rings, ARIA attributes, keyboard navigation, contrast
3. Generate PWA icons (192x192, 512x512 PNG from orange mark)
4. Receipt confirmation button hierarchy (Received = primary, Did not receive = secondary)
5. `<LinkButton>` styling for navigation CTAs

**Verification:** `just check` passes. `just e2e` passes. All new components work in dark mode. Tab navigation reaches all interactive elements. Screen reader announces states correctly.

---

## Key Decisions Summary

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| 1 | Navigation pattern | Hamburger menu | Bottom nav wastes 60-80px permanently for 5 pages. Hamburger frees the viewport. |
| 2 | NP city input | Native `<select>` | 10-20 cities. Native picker is perfect touch UX with zero JS. Autocomplete not worth building from scratch. |
| 3 | OTP input | Single input | Multi-box breaks autofill, screen readers, and paste. Single field with `autocomplete="one-time-code"` works natively. |
| 4 | Toast position | Bottom | Doesn't obscure content the user just interacted with. Full-width on mobile, bottom-right on desktop. |
| 5 | Confetti on reveal | Yes, first-time only | The reveal is THE emotional peak. Confetti marks the occasion. Re-visits skip it. |
| 6 | Skeleton shape | Generic pulse lines | Not worth per-page content-matching for a 10-page app with sub-second responses. |
| 7 | Phase stepper | Horizontal, both participant and admin | Phases are a sequence with direction. Reusing one component avoids duplication. Horizontal scroll on mobile is natural for timelines. |
| 8 | Envelope animation | CSS 3D flap rotation | Physical, tactile, references opening real mail. Cubic-bezier bounce feels satisfying. Not a generic slide/fade. |
| 9 | Admin cycle viz | Ring/circle | The cycle IS a ring. Circular layout makes the topology immediately visible. A list would hide the cyclic structure. |
| 10 | Dark mode | System-only, no toggle | 10-30 users, community app. System preference respects user intent. No toggle to build or maintain. |

---

## Component Priority Summary

| Component | Priority | Status |
|-----------|----------|--------|
| Field (wrapper) | P0 | Extract from existing |
| Phone input | P0 | Exists, add `inputmode` |
| OTP input | P0 | Exists |
| Button (with spinner) | P0 | Exists, add spinner |
| Alert | P0 | Exists, add scroll-into-view |
| Badge | P0 | Exists |
| EmptyState | P0 | New |
| Card | P0 | New |
| DescriptionList | P0 | New (replaces unstyled dl) |
| Deadline | P0 | New (replaces unstyled text) |
| PhaseStepper | P0 | New |
| DataTable wrapper | P0 | Enhance existing |
| AssignmentReveal | P0 | New |
| AdminNav | P0 | Exists |
| Skeleton | P1 | New |
| Toast | P1 | New |
| NpCitySelect | P1 | New (planned feature) |
| NpNumberInput | P1 | New (planned feature) |
| Textarea | P1 | Add CSS for existing |
| StatCard | P1 | New |
| ActionPanel | P1 | New |
| DangerZone | P1 | New |
| CycleVisualization | P1 | New |
| ConfirmDialog | P1 | New |
| Header hamburger | P1 | Enhance existing |
| MobileMenu | P1 | New |
| LinkButton | P1 | Add CSS for a.btn |

---

## Banned Patterns

**CSS:**
- `@apply` to build component classes (use native CSS properties)
- Hardcoded color values (always `var(--color-*)`)
- Unlayered CSS (everything inside `@layer`)
- `!important` (fix specificity instead)
- `z-index` without documenting in the table above

**Tailwind in Rust:**
- `format!()` to construct class names (scanner cannot detect)
- `class:` directive with computed names (literal only)

**HTML:**
- Steps/sequences as `<div>` (use `<ol><li>`)
- Missing `data-testid` on interactive/assertable elements
- Form inputs without ARIA linkage (`aria-describedby`, `aria-invalid`)
- Multiple `<input maxlength="1">` for OTP (use single field)

**Leptos:**
- Signal-driven form inputs for ActionForm submission (use `name` attributes)
- `on:input` signals for form values (Playwright cannot fire them)

---

## What Is NOT In This System

- No animation library. CSS transitions only, 120ms ease on interactive elements.
- No icon system beyond the logo mark. Add `leptos_icons` when first icon is needed.
- No dark mode toggle. System preference only.
- No responsive breakpoints beyond Tailwind defaults. Single-column at all sizes.
- No print styles.
- No component library dependency. Hand-rolled. Re-evaluate at Leptos 1.0.
- No form validation library. HTML5 + server-side.
- No complex interactive patterns beyond what is specified here. No drag-and-drop, no infinite scroll, no virtual lists.

---

## Ecosystem Decisions

- **No component libraries.** All hand-rolled. Leptos ecosystem is immature; every candidate has blockers (Tailwind v4 incompatibility, SSR issues, abandoned maintenance).
- **leptos-use 0.18.x:** Already a transitive dependency. Promote to direct dep. Use `use_media_query`, `use_debounce_fn` when needed.
- **leptos_icons:** Adopt when first icon needed. Feature-flag specific icon pack.
- **`use_hydrated()`:** Already extracted to `src/hooks.rs`. Single source of truth for hydration gate. All new components use it.
