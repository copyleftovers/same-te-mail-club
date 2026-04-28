# Component System Implementation Plan

Binding plan for implementing `spec/component-system.md`. Written for an opus-tier orchestrator who delegates aggressively and treats agent calls as nearly free.

---

## How to Read This Plan

Each phase is a **tracer bullet** — a thin vertical slice that touches CSS, Rust, and E2E, proving the full stack works before widening. You never implement "all CSS" then "all components" then "all tests." You implement one working feature at a time, verified end-to-end.

Your job as orchestrator is **decomposition, dispatch, and assembly.** You write zero lines of code yourself unless the edit is 3 lines and you already have the context. Everything else is delegated.

---

## Pre-Flight

### Swear manifestos
Correctness-by-construction, simple-made-easy, first-principles. Use `manifest-oath` skill. This is non-negotiable per CLAUDE.md.

### Orient yourself
Launch 3 parallel haiku agents (background, no conversation retention needed):

| Agent | Task | Reads |
|-------|------|-------|
| Spec digest | Summarize `spec/component-system.md` — list every component, its priority, its CSS class name | `spec/component-system.md` |
| Current state | List every `#[component]` in `src/`, every CSS class in `style/tailwind.css`, every POM method in `end2end/tests/fixtures/` | Source files |
| E2E baseline | Run `just e2e`, capture full output, report pass/fail per test | Shell |

Read their summaries. Now you know what exists, what's specified, and what's green.

### Invoke `/frontend-design` skill
Do this before any visual implementation work. The skill provides design quality guidance that complements the component spec.

---

## Phase 0: Foundation (P0) ✅ COMPLETE

**Tracer bullet:** After this phase, the app renders correctly on mobile Safari with safe areas, has PWA install capability, and all existing E2E tests still pass.

### 0.1 Viewport & Safe Areas

**Single sonnet agent (foreground, needs result before proceeding):**
- Read `spec/component-system.md` §Mobile Architecture
- Read `src/app.rs` (the shell component with `<Html>`, `<Head>`, `<Body>`)
- Add viewport meta: `<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">`
- Add theme-color meta: `<meta name="theme-color" content="#FAF9F6">`  + dark variant
- Update `.prose-page` in `style/tailwind.css`: add `padding-bottom: env(safe-area-inset-bottom)`
- Run `just check` to verify

**Why foreground:** This touches the app shell. Everything else depends on it rendering correctly.

### 0.2 PWA Manifest

**Haiku agent (background):**
- Read §Mobile Architecture for manifest spec
- Create `public/manifest.json` with name, icons, display: standalone, theme_color, background_color
- Add `<link rel="manifest" href="/manifest.json">` to app shell
- No service worker needed yet

### 0.3 Verify E2E Still Green

**Haiku agent (background, after 0.1 lands):**
- Run `just e2e > /tmp/e2e-phase0.log 2>&1`
- Report results
- If failures: DO NOT debug yourself. Delegate E2E debugging per `guidance/debugging-policy.md`

---

## Phase 1: Participant Forms — The Tracer Bullet (P0) ✅ COMPLETE

**Tracer bullet:** NP branch field split works end-to-end: separate city + number inputs → server fn parses cleanly → E2E tests pass with new selectors.

This is the most important phase because it changes a server function signature, form markup, AND E2E tests simultaneously. Get this right and the pattern repeats for every subsequent change.

### 1.0 Plan the field split (you, the orchestrator)

The change touches:
1. `src/pages/home.rs` — `enroll_in_season` server fn: replace `branch: String` with `branch_city: String, branch_number: i32`
2. `src/pages/home.rs` — `render_enrollment_open` view: replace single text input with `<select>` for city + `<input type="number">` for branch number
3. `src/pages/onboarding.rs` — same field split for onboarding form
4. `end2end/tests/fixtures/mail_club_page.ts` — POM methods that fill branch input
5. `end2end/tests/mail_club.spec.ts` — any test assertions on branch values
6. `spec/technical/User Stories.md` — update AC for enrollment story

### 1.1 Server function + form markup

**Sonnet agent (foreground):**

Give it:
- `spec/component-system.md` §Form System (the NP field split design)
- `src/pages/home.rs` (read it)
- `src/pages/onboarding.rs` (read it)
- Clear instructions:
  1. In `enroll_in_season`: replace `branch: String` param with `branch_city: String, branch_number: String`. Remove the parsing logic (lines that extract number and city from combined string). Parse `branch_number` to i32 server-side.
  2. In `render_enrollment_open`: replace the single branch text input with two fields — a `<select name="branch_city">` with city options and an `<input type="number" name="branch_number" inputmode="numeric" min="1">`. Use the `.field` wrapper pattern for each. Add `data-testid="enroll-city-select"` and `data-testid="enroll-branch-input"`.
  3. In onboarding: same split pattern.
  4. Run `just clippy` to verify.
  5. Run `cargo sqlx prepare --workspace` if any query changes.

**Why foreground:** E2E agent needs to know the new testids.

### 1.2 E2E adaptation

**Sonnet agent (foreground, after 1.1):**

Give it:
- The testid changes from 1.1 (new `data-testid` values)
- `end2end/README.md` (binding E2E conventions)
- `end2end/tests/fixtures/mail_club_page.ts` (POM)
- `end2end/tests/mail_club.spec.ts` (tests)
- Instructions:
  1. Update POM methods that interact with branch input to use the new two-field pattern: select city, fill branch number.
  2. Update any test data that uses the combined "Відділення №5, Київ" format.
  3. Run `just e2e` to verify.

**Why foreground and sequential:** Must use the exact testids from 1.1.

### 1.3 New CSS component classes

**Sonnet agent (can run in parallel with 1.1, worktree recommended):**

Give it:
- `spec/component-system.md` §Form System + §CSS Additions
- `style/tailwind.css` (current CSS)
- `guidance/frontend-protocol.md` (CSS rules)
- Instructions:
  1. Add new form-related component classes from the spec to `@layer components` in `tailwind.css`
  2. Add any new `@theme` tokens specified
  3. Ensure no unlayered CSS, no hardcoded colors, no `@apply`
  4. Run `just check`

**Why worktree:** This modifies `tailwind.css` while 1.1 modifies `.rs` files. They won't conflict on files, but a worktree is cleaner if you want to validate independently. Merge the worktree branch after both pass.

### 1.4 Verify full stack

**Haiku agent (after 1.1 + 1.2 + 1.3 merged):**
- `just check && just e2e`
- Report results

---

## Phase 2: Feedback System (P0) ✅ COMPLETE

**Tracer bullet:** One page (home) has skeleton loading, button loading states, and toast feedback. Then roll out to all pages.

### 2.1 Skeleton + Button states (CSS)

**Sonnet agent (worktree):**
- Read `spec/component-system.md` §Feedback Strategy + §CSS Additions
- Add `.skeleton`, `.btn` loading state updates to `tailwind.css`
- Add `@utility skeleton-pulse` or equivalent
- `just check`

### 2.2 Toast architecture (Leptos)

**Sonnet agent (worktree, parallel with 2.1):**

This is the trickiest component — it needs a context provider pattern.
- Read `spec/component-system.md` §Feedback: Toast
- Create toast context: `src/components/toast.rs` (or wherever components live)
  - `ToastProvider` component that wraps the app
  - `use_toast()` hook that returns a dispatch function
  - Toast renders as a fixed-position `<div>` with `aria-live="polite"`
  - Auto-dismiss after 5s via `set_timeout`
- Wire into `src/app.rs`
- **Do NOT touch existing pages yet** — just establish the infrastructure
- `just clippy`

### 2.3 Apply to HomePage

**Sonnet agent (after 2.1 + 2.2 merged):**
- Replace `<Suspense fallback=|| "Loading...">` with skeleton markup
- Replace inline error `<p class="alert">` with toast dispatch for action errors
- Add button loading states (pending signal → spinner class)
- `just check`

### 2.4 Roll out to remaining pages

**Fan-out: 4 sonnet agents in parallel, one per page group (worktrees):**

| Agent | Pages |
|-------|-------|
| A | `login.rs`, `onboarding.rs` |
| B | `admin/dashboard.rs`, `admin/season.rs` |
| C | `admin/participants.rs`, `admin/assignments.rs` |
| D | `admin/sms.rs` |

Each agent:
- Reads the toast context API from 2.2
- Reads the skeleton pattern from 2.3 (HomePage as reference)
- Applies same patterns to their pages
- `just clippy` in their worktree

Merge all worktrees. Run `just e2e`.

---

## Phase 3: The Assignment Reveal (P0) ✅ COMPLETE

**Tracer bullet:** Participant sees their assignment with envelope animation on mobile. E2E test verifies the reveal renders.

### 3.1 CSS for envelope + confetti

**Sonnet agent (worktree):**
- Read `spec/component-system.md` §The Assignment Reveal (full CSS)
- Add `.envelope`, `.envelope-flap`, `.envelope-card`, `.confetti` classes to `tailwind.css`
- Add `@media (prefers-reduced-motion: reduce)` fallback
- `just check`

### 3.2 Leptos reveal component

**Sonnet agent (worktree, parallel with 3.1):**
- Read §The Assignment Reveal for the Leptos component design
- Read `src/pages/home.rs` — the `HomeState::Assigned` arm and `render_assigned_view`
- Replace the plain `<dl>` with the envelope reveal component
- Use `localStorage` to track first-time-only confetti (or a signal)
- Information reveal order: name → branch → phone
- Keep all existing `data-testid` attributes on the revealed content
- `just clippy`

### 3.3 E2E verification

**Sonnet agent (after 3.1 + 3.2 merged):**
- E2E tests should still pass — the `data-testid` attributes are preserved
- Run `just e2e`
- If the envelope animation causes timing issues (elements hidden during animation), add a wait for the testid element to become visible

---

## Phase 4: Admin Dashboard (P1) ✅ COMPLETE

### 4.1 Phase Stepper component

**Sonnet agent (worktree):**
- Read `spec/component-system.md` §Display: Phase Stepper
- Create stepper component in `src/components/` (or inline in dashboard)
- CSS in `tailwind.css`
- Apply to `admin/dashboard.rs` — replace the phase badge with the stepper
- Apply to participant home — show phase indicator (simplified stepper)
- `just clippy`

### 4.2 Stat Cards + Dashboard layout

**Sonnet agent (worktree, parallel with 4.1):**
- Read §Admin: Stat Card
- Refactor dashboard to use stat cards for enrolled/confirmed/not-received counts
- CSS in `tailwind.css`
- `just clippy`

### 4.3 Cycle Visualization upgrade

**Sonnet agent (worktree, parallel with 4.1 and 4.2):**

This is the most complex admin component. The agent should:
- Read §Admin: Cycle Visualization
- Read `src/admin/assignments.rs` — current `render_cycle_visualization`
- Replace `<ol>` chain display with SVG ring visualization
- Nodes on circle perimeter, arrows showing direction
- `just clippy`

### 4.4 Merge + E2E

Merge all three worktrees. Run `just e2e`. Delegate failures.

---

## Phase 5: Mobile Navigation (P1) ✅ COMPLETE

### 5.1 Hamburger menu

**Sonnet agent:**
- Read §Navigation: Header, Mobile Menu
- Refactor `Header` / `HeaderNav` in `src/app.rs`
- CSS-only hamburger (checkbox hack or `<details>`) for pre-hydration
- Enhance with Leptos signals after hydration
- Test on mobile viewport (320px-428px)
- `just check`

### 5.2 Admin nav update

**Sonnet agent (parallel with 5.1, worktree):**
- Read §Navigation: Admin Nav
- Update `.admin-nav` for mobile (collapsible, touch targets)
- `just check`

---

## Phase 6: Polish (P2) ✅ COMPLETE

Launch a **swarm of parallel haiku audit agents**, each checking one concern:

| Agent | Audit |
|-------|-------|
| 1 | Color contrast — verify all text/bg combinations meet WCAG AA 4.5:1 |
| 2 | Touch targets — grep all `<button>`, `<a>`, `<input>` for minimum 44px/48px |
| 3 | aria attributes — verify all forms have aria-describedby, aria-invalid, aria-live |
| 4 | data-testid — verify every interactive element has one |
| 5 | Dark mode — check all semantic tokens reassign correctly |
| 6 | Reduced motion — verify grain overlay hides, animations disable |
| 7 | Empty states — check every Resource/Suspense has a meaningful empty state |
| 8 | Keyboard nav — verify tab order, focus rings, escape closes modals |

Each writes a report. Fix findings with targeted sonnet agents.

---

## Verification Harness Adaptation

### The problem

The E2E tests are serial and share DB state. Changing form fields, adding new components, or altering page structure can break selectors. The harness must evolve WITH the implementation, not after.

### The rule

**Every phase that changes markup must include E2E adaptation in the same phase.** Never merge a markup change without updating the POM. The tracer bullet principle: CSS + Rust + E2E land together.

### Practical pattern

1. **Before touching markup:** Read the POM (`end2end/tests/fixtures/mail_club_page.ts`) to know which testids are used
2. **Preserve testids:** New components must keep existing `data-testid` attributes. Add new ones, never remove without updating tests.
3. **POM-first for new interactions:** If adding a new interactive element (toast dismiss, envelope click, hamburger toggle), add the POM method FIRST (it will fail), then implement the component (it will pass). This is TDD at the E2E level.
4. **Delegate E2E debugging:** Per `guidance/debugging-policy.md`, NEVER debug E2E in orchestrator context. Always delegate to a sonnet agent with: failure output, screenshot path, relevant source files.

### When to run E2E

- After every phase merge (mandatory)
- After any markup change that touches `data-testid` elements
- NOT after CSS-only changes (CSS doesn't break E2E selectors)

---

## Worktree Strategy

### When to use worktrees

- Two agents modify the same file (e.g., both touch `tailwind.css`)
- You want to validate a change independently before merging
- A risky change (reveal animation, toast architecture) that you might revert

### When NOT to use worktrees

- Sequential work where agent B depends on agent A's output
- Single-file changes that can't conflict
- E2E runs (always run on main after merge)

### Merge protocol

1. Agent completes in worktree, reports success
2. You (orchestrator) merge the worktree branch into main: `git merge --no-ff worktree-branch`
3. Run `just check` on main
4. If conflict: delegate resolution to a sonnet agent with both branches

---

## Delegation Cheat Sheet

| Task type | Model | Foreground/Background | Worktree? |
|-----------|-------|----------------------|-----------|
| Read files + report | haiku | background | no |
| CSS additions | sonnet | background | yes (if parallel) |
| Leptos component | sonnet | foreground (if blocking) | yes (if parallel) |
| E2E adaptation | sonnet | foreground (sequential) | no |
| E2E debugging | sonnet | foreground | no |
| `just check` / `just e2e` | haiku | background | no |
| Audit (a11y, contrast, etc.) | haiku | background | no |
| Architecture decisions | opus | foreground | no |
| Spec interpretation disputes | opus | foreground | no |

### The swarm default

When you have N independent tasks, launch N agents. Don't batch. Don't serialize. The cost of 10 parallel haiku agents is less than you reading one file. The cost of 4 parallel sonnet agents in worktrees is less than you implementing one component.

### The one thing you do yourself

**Merge decisions.** You read agent summaries, decide merge order, resolve conflicts, and run the final verification. That's your job. Everything else is delegated.
