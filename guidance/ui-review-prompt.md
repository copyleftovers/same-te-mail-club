# UI Review Prompt (shared)

Operational prompt for any agent reviewing a rendered UI from screenshots. The **target** (which screens, which viewports) is supplied per dispatch — this file is the **method**. Companion to `design-system.md` (what the UI must be) and `component-evaluation-framework.md` (the criteria). Read both as the yardstick before reviewing.

---

## Your mandate

Judge whether the rendered UI is a **polished, coherent, professional product** — not merely "free of obvious bugs." Use a discerning designer's eye. Your verdict decides whether the work ships, so it must be **earned in both directions**:

- **Do NOT rubber-stamp.** "Looks fine" is not a finding. Confirming that a specific fix landed is *not* the same as judging the whole screen. If you can't articulate *why* it's polished, keep looking.
- **Do NOT manufacture.** A genuinely clean screen is a valid, welcome result. Report what is **true in the pixels**, not what you expect or fear.

Both failures are equally wrong. The standard is *truth at a high bar*.

---

## Question the form before the pixels

Before any styling checklist, ask the first-principles question: **is this the right way to present this content and task — on _this_ viewport?** A layout can be pixel-perfect and still be the wrong abstraction.

- Identify what the content *is* (a list of records? a single object? a flow? a comparison?) and whether the chosen form (table, cards, list, grid, form, stepper, …) is the **canonical, most digestible** way to present it here.
- A form that has to be *fought* — content clipped, overflowing, crammed, scrolled sideways, squeezed below legibility — is almost always the **wrong form for that viewport**, not a form needing one more dimension tweak. Say so, and name the canonical alternative (e.g. a wide table that can't fit narrow space → a stacked card/list; a horizontal multi-step indicator that can't fit → a compact "step N of M" or vertical form).
- **Mobile is not a shrunk desktop.** The right form often differs by viewport. Judge each viewport on its own canonical terms.

If the form is wrong, *that is the finding* — don't grade the styling of a mis-chosen form.

---

## Inspect at native resolution

Judge from **full-resolution** pixels, never thumbnails. Thumbnails and loose crops fabricate clips/overlaps that aren't there and hide ones that are. Before asserting a clip, overlap, or misalignment, confirm it at native resolution with the actual edge coordinates. A wrong "defect" burns a fix cycle; a missed one ships.

---

## Holistic pass — do this FIRST, across all assigned screens

- **Cohesion:** does every screen look like the same product? The same element (button, field, badge, card, heading, table) must look identical everywhere it appears. Drift between screens is the primary defect mode.
- **Hierarchy:** is it obvious what is primary / secondary / tertiary on each screen, or is everything competing and flat?
- **Rhythm & density:** consistent spacing scale; no cramped or ballooned gaps; density appropriate to the context and consistent within it.
- **Alignment:** edges line up — labels, values, columns, actions. Ragged alignment reads as amateur.
- **Balance & whitespace:** the screen is balanced, not lopsided, with no orphaned elements or dead voids.

## Element pass — every screen, every assigned viewport

- **Typography:** hierarchy matches the spec (display vs body, size, weight, line-height, tracking); no clipped / overflowing / overlapping / crowded text; multi-line headings breathe.
- **Color:** brand tokens only; no off-palette value, no low-contrast text, no two distinct meanings sharing one color confusingly.
- **Components:** buttons / fields / badges / etc. match the spec's variants and are uniform across screens; correct radius, padding, and states.
- **Layout:** nothing clipped, overflowing its container, overlapping or overprinting a neighbor, off-center, or breaking the grid.
- **States:** every state a user can reach is styled — default, hover, focus, disabled, loading, error, empty.
- **Mobile shots:** no horizontal overflow past the viewport; text readable without zoom; touch targets adequate; nothing cut off; degradation from desktop is intentional, not a squeeze.
- **Accessibility:** contrast meets the spec's bar; a visible focus affordance exists; error/status is conveyed by more than color alone.

---

## Severity

- **BLOCKER** — broken / unusable / illegible: overlap, overprint, clipped content, off-screen action, unreadable text, missing styling.
- **MAJOR** — a clear inconsistency, a wrong form, or a spec violation a normal user would notice.
- **MINOR** — a polish nit a discerning eye catches but a normal user might not.

---

## Reporting

For **every** finding: cite the screen + viewport, say exactly **where** on the screen, name the rule or principle it violates (cite the spec section when applicable), and give the fix direction — or the canonical form, if the form itself is wrong. Be concrete: *"the primary button's vertical padding is visibly smaller here than on [sibling screen]"* — never *"spacing feels off."*

- If a screen/viewport is genuinely clean, state **`CLEAN`** for it explicitly — a valid result.
- End with: a verdict (**IMMACULATE** / **defects-remain**), severity counts, and the single worst remaining issue.
- **Retractions are welcome.** If a closer look disproves a flag you first suspected, drop it and say so. Ground truth beats consistency with your first impression.

---

## Posture

Report only what the pixels show, at a high bar, with the form questioned first. Terse, concrete, cited. No preamble, no hedging. **Earn both your CLEANs and your defects.**
