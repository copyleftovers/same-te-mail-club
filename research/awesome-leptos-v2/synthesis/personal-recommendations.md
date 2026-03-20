# Personal Recommendations — Component Strategy

Written for ryzhakar, 2026-03-20. Based on full ecosystem research.

---

## The honest assessment

The Leptos component ecosystem is 18–24 months behind React's. There is no shadcn/ui, no Radix, no Headless UI equivalent that actually works. The attempts exist but are either facades, archived, or version-locked. This is not going to change in the next 6 months — the bottleneck is Leptos's rapid 0.x version churn breaking every community library on every release.

You have two real options.

---

## Option A: Build your own tiny component kit (recommended)

You already have the hard parts: a coherent design system, oklch tokens, Tailwind v4, data-variant CSS pattern. What you're missing is the Rust abstraction layer on top of it.

**What to build:**

```
src/ui/
  mod.rs
  hydration.rs       ← use_hydrated() hook (exists nowhere in ecosystem)
  button.rs          ← <Button> with variant/size/disabled/loading props
  field.rs           ← <Field> with label/error/aria wiring
  badge.rs           ← <Badge> with status prop
  alert.rs           ← <Alert> for action errors
  table.rs           ← <DataTable> generic over row type
  card.rs            ← <Card> for content sections
```

This is ~300 lines of Rust total. You already wrote the CSS. The components are thin wrappers that:
- Accept typed props (variant enums, not strings)
- Wire ARIA attributes automatically
- Handle the hydration gate internally
- Render your existing CSS classes

**Why this wins:**
- Zero version conflicts forever
- SSR + hydration works because you control it
- ActionForm compatible because you designed it that way
- Tailwind v4 compatible because it's your CSS
- 2–3 hours of work, not a research project
- Every component is tested by your existing 56 E2E tests

**The key insight:** You don't need a component *library*. You need 7 components. A library solves the problem of providing 50+ components to unknown consumers with unknown design systems. You have 1 consumer and 1 design system. The abstraction you need is a thin typed wrapper, not a framework.

---

## Option B: Bet on biji-ui's future

biji-ui is the only real headless library. Its three blockers are all fixable upstream:
- leptos-use version bump (0.16 → 0.18)
- SSR guards on DOM calls
- `MaybeSignal<bool>` for reactive props

You could:
1. Fork biji-ui
2. Fix the three blockers
3. Submit PRs upstream
4. Use your fork as a git dependency until merged

**Why you might want this:** biji-ui has real focus trapping, keyboard navigation, and collision-aware positioning. If you're going to add dialogs, comboboxes, or tooltips, hand-rolling the accessibility for those is genuinely hard and error-prone.

**Why you probably don't want this right now:** You have zero modals, zero dropdowns, zero tooltips, zero comboboxes. Building accessibility for components you don't have is premature. If you need a dialog in 3 months, fork biji-ui's dialog component at that point.

---

## My recommendation

**Do Option A now. Keep Option B in your back pocket.**

Build the 7-component kit this week. It pays for itself immediately by eliminating the 150+ lines of boilerplate identified in the audit. When (if) you need a complex interactive primitive — dialog with focus trap, combobox with keyboard nav — fork the single biji-ui component you need and fix its blockers.

---

## Other advice

**Adopt immediately (30 minutes):**
- Promote `leptos-use` from transitive to direct dependency. You're already compiling it. `use_debounce_fn`, `use_media_query`, `use_preferred_dark` are free.
- Add `wasm-tracing` to `hydrate()`. One line. You currently have zero visibility into client-side execution.

**Don't adopt:**
- `leptos_form_tool` — its ActionForm path works but requires implementing a custom `FormStyle` trait to use your CSS. More work than just building `<Field>`.
- `tw_merge` — you have zero dynamic class composition. Solve a problem you have, not one you might have.
- `leptos-struct-table` — you have one table with no sorting. If you add a second complex table, reconsider then.
- `leptosfmt` — try it for 30 minutes. If it formats `view!` macros the way you like, keep it. If not, drop it. Zero risk either way.

**Watch the ecosystem, don't fight it:**
- Leptos 0.9 or 1.0 will stabilize the API surface. That's when component libraries will stop breaking on every release. That's when biji-ui (or its successor) will mature. Until then, owning your components is cheaper than chasing upstream.

**The meta-lesson from this research:**
The Leptos ecosystem's immaturity is real but narrowly scoped. The *framework* is excellent — SSR, hydration, signals, server functions all work well. The *library ecosystem around UI components* is where the gap is. Everything else — i18n, routing, meta, database, auth — you already have and it works. The component layer is the one place where you're on your own, and the right response to that is to own it cleanly rather than depend on something fragile.
