# Component Evaluation Framework

For single-project component evaluation. Not a reusable library standard.

---

## Principles

**1. The user sees a product, not components.** Every page must look intentional. Same elements must look the same everywhere. Drift between pages is the primary defect mode -- not missing API surface.

**2. The spec is the contract.** `design-system.md` and `frontend-protocol.md` define correct. If rendering diverges from spec, the rendering is wrong -- even if it "looks fine."

**3. Every state a user can reach must be styled.** Hover, focus, disabled, loading, error, empty. An unstyled state is a bug a user will encounter.

**4. Mobile is the real device.** 11-15 participants using phones. Touch targets, readable text, no horizontal scroll, thumb-reachable actions. Desktop is secondary.

---

## Criteria

### A. Visual Consistency

Does the same element look the same across every page?

| # | Criterion | Test |
|---|-----------|------|
| A1 | Zero hardcoded colors | `grep -rn '#[0-9a-fA-F]\{3,8\}' src/ style/` returns nothing outside `@theme` raw token block |
| A2 | Spacing from Tailwind scale only | No bare `px`/`rem` values outside `@theme` in `style/tokens.css` or `style/components.css` |
| A3 | One button definition | All pill-shaped clickable elements trace to `.btn`. `grep -rn 'border-radius.*100\|radius-pill' src/ style/` shows no independent declarations |
| A4 | One field definition | All `<input>` elements use `.field-input` or sit inside `.field` |
| A5 | One badge definition | All status indicators use `.badge[data-status]`. No ad-hoc colored pills |

### B. Spec Fidelity

Does the rendered output match `design-system.md`?

| # | Criterion | Test |
|---|-----------|------|
| B1 | Typography hierarchy matches | Each level in the spec table: family, weight, size, line-height, letter-spacing. Playwright computed-style extraction on representative pages |
| B2 | Semantic color aliases match | `:root` block values match spec's Semantic Aliases table exactly |
| B3 | Density tokens respect layout | `[data-layout="admin"]` overrides `--density-space-*` to compact values per spec |
| B4 | Grain overlay compliant | `body::after`: z-index 1, opacity 0.04, mix-blend-mode overlay, pointer-events none. Hidden under `prefers-reduced-motion: reduce` |

### C. Interactive States

Would a user hitting this state see something intentional?

| # | Criterion | Test |
|---|-----------|------|
| C1 | Buttons: all states present | `.btn` covers default, hover (behind `@media (hover:hover)`), focus-visible, disabled, loading |
| C2 | Fields: all states present | `.field-input` covers default, focus, error, disabled. Focus uses `--color-focus`, error uses `--color-error` |
| C3 | Hydration gate on every submit | Every `type="submit"` in `src/` has adjacent `disabled=move \|\| !hydrated.get()`. Grep to verify |
| C4 | Focus ring on all interactive elements | Base `:focus-visible` rule exists. No `outline: none` without replacement |

### D. Accessibility

Would a screen reader user or low-vision user be blocked?

| # | Criterion | Test |
|---|-----------|------|
| D1 | Accent contrast passes AA | `--color-brand-orange` is oklch(0.63 0.22 31), not original #FB4417. Grep to confirm |
| D2 | Error fields have ARIA | `field-error` elements have `aria-live="assertive"`. Error inputs have `aria-invalid`. Grep `src/` |
| D3 | Reduced motion respected | `prefers-reduced-motion: reduce` disables transitions and hides grain |

### E. Mobile Readiness

Would a participant on a phone complete the flow without frustration?

| # | Criterion | Test |
|---|-----------|------|
| E1 | No horizontal scroll | Playwright mobile viewport (375px): no element exceeds viewport width. `document.documentElement.scrollWidth <= window.innerWidth` |
| E2 | Touch targets >= 44px | All buttons and interactive elements meet 44x44px minimum. Playwright bounding-box check on mobile viewport |
| E3 | Text readable without zoom | Body text >= 16px computed on mobile. No text below 12px computed |
| E4 | Tables degrade on small screens | Admin data tables remain usable at 375px (horizontal scroll within table container, not page-level) |

---

## Running an Evaluation

1. **Grep pass.** Run every grep-based test. Report criterion ID + pass/fail + matching lines.
2. **Screenshot pass.** Playwright captures each major page at desktop (1280px) and mobile (375px). Compare against typography and color specs.
3. **Hydration audit.** Grep `type="submit"` across `src/`, verify hydration gate adjacency.

Output: criterion ID, pass/fail, evidence. No prose.
