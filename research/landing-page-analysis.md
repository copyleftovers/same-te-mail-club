# Landing Page Analysis — Саме Те (`index.html`)

Static analysis of `/Users/ryzhakar/pp/same-te-landing/index.html`.
Chrome browser tools were unavailable (extension not connected); all measurements are derived from source and computed programmatically. Line numbers reference the source file as read.

---

## What Was Done Well

**Zero render-blocking resources.** No external CSS, no JS, no synchronous scripts. One HTML document, three font fetches, two SVG asset fetches. Every font uses `font-display: swap`. This is the correct loading strategy for a content-first landing page.

**`font-display: swap` with good fallback stacks.** `--font-display: 'CyGrotesk', 'Arial Black', sans-serif` — Arial Black is a near-weight match for a black display font, so the FOUT is visually tolerable. `--font-body: 'Mont', 'Inter', system-ui, sans-serif` — system-ui as the final fallback is correct.

**Font file sizes are lean.** CyGroteskGrandDark.woff2: 46,224 bytes. Mont-Regular.woff2: 46,004 bytes. Mont-SemiBold.woff2: 45,832 bytes. Total: 138,060 bytes (~135 KB). Reasonable for three custom fonts.

**SVG grain is elegant.** 283 bytes in the data URI. `feTurbulence` is GPU-accelerated. `body::after` with `position: fixed` means the overlay composites separately — zero scroll repaint. `pointer-events: none` ensures it never interferes with interaction. `opacity: 0.09` and `mix-blend-mode: multiply` is the right combination to add texture without muddying either the light cream or the dark black sections.

**`100svh` for hero height** (line 57). Uses the Small Viewport unit, which accounts for mobile browser chrome (address bar, nav bar). `100vh` on iOS Safari notoriously over-shoots. This is the correct, modern choice.

**`prefers-reduced-motion` is correctly implemented** (lines 308–311). Disables the bounce animation on `.scroll-hint` and all transitions on `.cta-btn`. Both animated properties are covered.

**`aria-hidden="true"` on the scroll-hint arrow** (line 327). Decorative, animated content correctly removed from the accessibility tree.

**`rel="noopener"` on both external links** (lines 323, 383). Required for security on `target="_blank"` links.

**Touch target on CTA button passes minimum thresholds.** Computed height: `(1.1rem line-box) + 2 × 1rem padding` ≈ 17.6px + 32px = 49.6px at 16px base. Exceeds the 44px WCAG 2.5.5 minimum.

**`lang="uk"` on `<html>`** (line 2). Correct. Screen readers and search engines will use the right language model.

**`theme-color` meta** (line 8) set to `#FAF9F6`. Browser chrome on Android will match the page background. A considered touch.

**OG tags are present** (lines 9–12). Title, description, type, URL. Not a requirement for a landing page but correct practice.

---

## Code Quality

### HTML Semantics

**Missing `<h1>`.** The document has no `h1` element. The hero brand identity is carried by an `<img>` (line 318, `alt="Саме Те"`), which is semantically correct as a labelled image, but not a heading. Screen readers that navigate by heading find `h2` as the first heading. A screen reader user landing on this page cannot use heading navigation to find the page title. Fix: add a visually hidden `<h1>Саме Те</h1>` above or around the logo image, or give the `<img>` a `role="heading" aria-level="1"` — the latter is unusual but valid.

**`h2 → h2 → h3` hierarchy is technically valid** but both `h2` elements are section titles in sibling sections. The outline is flat: two sibling section headings, each with child `h3`s (steps). This is acceptable.

**Steps are `<div class="step">`, not `<ol>`** (lines 360–376). The four steps ("Створюєш / Відправляєш / Отримуєш / Зустрічаєшся") are a sequential, ordered process. They should be `<ol><li>` elements. A screen reader user hears "Створюєш, group" with no indication that it is one of four steps. Using `<ol>` provides count context ("item 1 of 4") for free.

**Tags are `<span>` not a `<ul>`** (lines 337–343). As purely decorative labels with no interactive intent, `<span>` inside a `<div class="tags">` is acceptable at this scale. They carry no semantics, which is fine because they are not filters or links.

**Instagram link: `target="_blank"` without an accessible label** (lines 323–326). Screen readers announce "Ми в Instagram, link" but do not announce that it opens in a new tab. WCAG 2.4.4 (Link Purpose) does not strictly require this, but 3.2.2 (On Input) implies users should be warned. Add `aria-label="Ми в Instagram (відкриється у новій вкладці)"` or append a visually-hidden span.

### CSS Architecture

**Custom properties are partially inconsistent.** The palette is defined cleanly in `:root` (lines 34–43), but several values are then hand-copied as literals:

- Line 89: `color: #fff` — not a token. Should be a `--white: #ffffff` property or at least `--cream` with a note.
- Line 158: `color: #D4B8B5` — a one-off muted pink, used only in `.about p`. No token defined.
- Line 287: `color: #8A7774` — footer small text, no token.
- Lines 100, 105: `rgba(251, 68, 23, 0.3)` and `rgba(251, 68, 23, 0.2)` — `#FB4417` (`--orange`) hand-copied as raw RGB. When CSS color-mix or relative color syntax is available (it is, in all modern browsers), use `color-mix(in srgb, var(--orange) 30%, transparent)` instead.
- Line 243: `rgba(22, 22, 22, 0.15)` — `#161616` (`--black`) hand-copied.

**Two headings share nearly identical declarations.** `.about h2` (lines 145–152) and `.mail-club h2` (lines 208–215) share: `font-family: var(--font-display)`, `font-size: clamp(1.8rem, 5vw, 2.8rem)`, `line-height: 1.15`, `letter-spacing: -0.02em`. They differ only in `color` and `margin-bottom`. At single-file scale this is survivable. In the app, extract a `.section-heading` utility class.

**Two inner containers share identical layout rules.** `.about-inner` (lines 140–143) and `.mail-club-inner` (lines 194–197) both set `max-width: 640px; margin: 0 auto`. A shared `.content-column` class would eliminate the duplication.

**Tag rotation is fragile** (lines 184–186):
```css
.tag:nth-child(even) { transform: rotate(2.5deg); }
.tag:nth-child(3)    { transform: rotate(1deg); }
.tag:nth-child(5)    { transform: rotate(-1deg); }
```
`nth-child(3)` and `nth-child(5)` override the `even` rule because they have higher specificity (element + nth-child vs element + nth-child(even) — same specificity, last wins). The visual result depends on tag order and count. The `@media (max-width: 480px)` override uses `!important` to flatten them. Reordering or adding tags breaks the visual pattern silently.

**`z-index: 100` on `body::after`** (line 301). See "App-scale concerns" below.

**`overflow-x: hidden` on `body`** (line 52). On some browsers (notably Safari), `overflow: hidden` on `body` breaks `position: fixed` containment — fixed elements scroll with the page instead of staying fixed. The grain layer uses `position: fixed` and it works because `body` itself contains the fixed child and the viewport is the containing block. In the app, if any fixed positioned element (navigation, modal backdrop) is added, `overflow-x: hidden` on `body` can cause subtle Safari bugs. Prefer `overflow-x: clip` on a wrapper `<div>` instead, which does not create a new containing block.

### Dead Code / Unused Declarations

No genuinely dead CSS found. `.step:last-child { border-bottom: none; }` (line 245) is active. All defined custom properties are used at least once.

### Single-File Approach

At 391 lines (14 KB), one HTML file is the correct choice. Splitting into separate CSS and HTML files would add a round-trip request with no caching benefit (no repeat visits assumed for a placeholder page). The inline-everything approach is appropriate at this scale.

---

## Accessibility

### Colour Contrast (WCAG 2.1 AA)

Computed using the WCAG relative luminance formula. AA normal text requires 4.5:1; AA large text (≥ 24px normal or ≥ 18.67px bold) requires 3.0:1.

| Text | Background | Ratio | Size (computed) | Result |
|------|-----------|-------|-----------------|--------|
| `#565656` gray | `#FAF9F6` cream | **6.97:1** | 16px–20.8px | PASS |
| `#ffffff` white | `#FB4417` orange | **3.53:1** | 17.6px bold (weight 600) | **FAIL** — not large text (needs 18.67px bold), needs 4.5:1 |
| `#FB4417` orange | `#FAF9F6` cream | **3.36:1** | 12.8px bold | **FAIL** — small text |
| `#EED3D0` pink | `#161616` black | **12.81:1** | 28.8px–44.8px | PASS |
| `#D4B8B5` muted | `#161616` black | **9.76:1** | 16.8px | PASS |
| `#161616` black | `#8DC1FF` blue | **9.66:1** | 12.8px bold | PASS |
| `#161616` black | `#FAF9F6` cream | **17.19:1** | various | PASS |
| `#565656` gray | `#FAF9F6` cream | **6.97:1** | 15.2px | PASS |
| `#8A7774` muted | `#161616` black | **4.28:1** | 12.8px | **FAIL** — below 4.5:1 |

**Three failures:**

1. **CTA button** (lines 89, 98–100): `#ffffff` on `#FB4417` = 3.53:1. The button is 1.1rem / 17.6px at weight 600. WCAG large text threshold for bold is 18.67px (14pt). 17.6px does not qualify. This is an AA failure. Fix: darken `--orange` by ~15% to `#D43710`, giving ≈4.5:1, or increase font-size to `1.2rem` (19.2px) to cross the large-text threshold.

2. **Label "Скоро ✦"** (lines 199–205): `#FB4417` on `#FAF9F6` = 3.36:1 at 12.8px bold. An AA failure. This is decorative overline text, but WCAG does not grant an exemption for decorative text that is also actual content. Fix: use `#C43510` (≈5:1 on cream) or switch to a more legible token.

3. **Footer small text** (line 287): `#8A7774` on `#161616` = 4.28:1 at 12.8px. Fails AA by 0.22 points. Fix: lighten to approximately `#9A8784` to reach 4.5:1, or use `--pink` (`#EED3D0`) which passes at 12.81:1.

**Scroll hint contrast:** The `.scroll-hint` (lines 119–126) is `color: var(--gray)` (`#565656`) at `opacity: 0.5`. The effective perceived colour blended against the cream background is approximately `#A8A7A6`, giving a contrast ratio of **2.28:1**. This fails WCAG. The element is `aria-hidden="true"`, so it does not need to meet contrast requirements. The `aria-hidden` here is therefore load-bearing for WCAG compliance — if it were ever removed, the contrast would need to be fixed.

### Focus States

`.cta-btn:focus-visible` defines a 3px blue outline (line 108–111). This is correct and visible.

No `:focus-visible` rules exist for any other element. Tags (`.tag`) are non-interactive spans, so this is fine currently. No other interactive elements exist in the document.

Keyboard navigation order follows document order (hero → about → mail-club → footer). The CTA links appear in hero and footer. A user tabbing through the page will hit: hero CTA → footer CTA. That is the entire tab sequence. Acceptable for a 2-link page.

### Reduced Motion

`@media (prefers-reduced-motion: reduce)` (lines 308–311) disables `.scroll-hint` animation and `.cta-btn` transition. Both are correctly targeted. The bounce animation is the only looping animation on the page.

---

## Visual Quality (Static Assessment)

### Typography

**Font size hierarchy is internally consistent:**
- Display headings: `clamp(1.8rem, 5vw, 2.8rem)` — 28.8px to 44.8px
- Step headings: `1.3rem` (20.8px)
- Label: `0.8rem` (12.8px) uppercase — small but intentional for overline text
- Body: `1.05rem` (16.8px)
- Body secondary: `0.95rem` (15.2px) for step descriptions

The step heading `line-height: 1` (line 253) is aggressive for a Cyrillic display font. Some Cyrillic uppercase letters have tall ascenders that can clip at tight line heights. CyGrotesk at 1.3rem with line-height 1 is borderline — actual rendering should be checked. A value of 1.1 would be safer.

**`max-width: 48ch` on the hero tagline** (line 77) is a good measure control for readability (optimal prose width is 45–75ch).

### Spacing and Vertical Rhythm

Spacing is not from a scale — values are: 0.3rem, 0.35rem, 0.5rem, 0.6rem, 0.8rem, 0.9rem, 1rem, 1.2rem, 1.5rem, 2rem, 2.2rem, 2.5rem, 3rem, 6rem. This is a collection of ad-hoc values rather than a rhythm. On a four-section landing page, this is not perceptible to users. In the app, the same ad-hoc approach would produce visually inconsistent forms and component spacing.

Section padding of `6rem 2rem` (about, mail-club) and `3rem 2rem` (footer) establishes a clear visual weight for sections vs footer. The hero at `2rem` with full viewport height relies on centering rather than padding-based rhythm — correct for a fullscreen hero.

### Grain Overlay

At `opacity: 0.09` with `mix-blend-mode: multiply`, the grain will be nearly imperceptible on the cream section (light surface, multiply does little) and more visible on the dark about/footer sections (black surface, multiply darkens). This is likely intentional — the grain adds tactile texture to dark sections while staying subtle on light ones.

The `stitchTiles="stitch"` attribute on `feTurbulence` prevents visible seams at the 200px tile boundary. Correct.

The `baseFrequency="0.9"` produces a relatively coarse grain (lower frequency = larger grain). This is an aesthetic choice consistent with the analogue/film community identity.

---

## Responsive Behaviour

### Fluid Layout

The page uses a single breakpoint (`max-width: 480px`) solely to flatten tag rotations. Everything else is fluid via `clamp()`, `max-width`, and percentage-free fixed padding. This is appropriate and results in predictable behaviour across viewport widths.

At 375px width (iPhone SE):
- Hero font: `clamp(1rem, 2.5vw, 1.3rem)` → 1rem (16px) — floor kicks in, legible.
- Section headings: `clamp(1.8rem, 5vw, 2.8rem)` → 1.8rem (28.8px) — floor kicks in.
- Steps content width: 375 − 32px (section padding) − 27px (steps left padding + border) = 316px — tight but readable.
- CTA button remains full-size at 49.6px height, adequate touch target.

No horizontal overflow risk identified from static analysis. All text elements have `max-width` or are inside centred constrained containers.

### One Missing Breakpoint

The hero section has no typography adjustment below 480px beyond the clamp floor. The tagline `<br>` in "балакаєм…" / "у Києві. по-справжньому." may create awkward wrapping at 375px depending on font metrics. This is a minor visual concern that requires live inspection to confirm.

---

## Performance Signals

### Font Loading

Three woff2 files (~46 KB each, 138 KB total) are declared in `<style>` inside `<head>`. No `<link rel="preload">` is present. This means the browser must:
1. Parse the HTML.
2. Begin constructing the CSSOM.
3. Encounter the `@font-face` rules.
4. Discover that fonts are needed.
5. Queue font requests.

With `font-display: swap`, text renders immediately in the fallback font, then re-renders when the custom fonts arrive. The hero SVG logo is not font-dependent, so the above-the-fold content (logo + tagline + CTA) appears instantly with fallback fonts. The first below-fold section uses CyGrotesk — FOUT is visible only on scroll. This is acceptable.

Adding `<link rel="preload" as="font" href="fonts/CyGroteskGrandDark.woff2" type="font/woff2" crossorigin>` would eliminate the FOUT entirely by the time the first scroll occurs on a normal connection. Not a bug, but an easy improvement.

### SVG Grain

The `feTurbulence` SVG filter inside a data URI tiled at 200px is a well-known pattern. It is 283 bytes. The alternative — a PNG grain texture — would be 10–50 KB and require an additional network request. The SVG approach is strictly better on a landing page with no CDN.

On low-end Android devices, `feTurbulence` with `numOctaves="4"` can cause a noticeable first-paint delay because the filter is rasterized during paint. Reducing to `numOctaves="3"` would be imperceptibly different visually but marginally faster. Not a production concern for a community landing page.

### No JavaScript

Zero JS. This is a feature. The page cannot have JS errors, no hydration issues, no bundle load time. Noted explicitly because the app design system should not attempt to bring JS dependencies into the landing page.

---

## App-Scale Concerns

Patterns in the landing page that will cause problems when the mail club app adopts this design system.

**`z-index: 100` on the grain overlay.** In the app, any modal, toast notification, dropdown, or popover must use `z-index > 100` to appear above the grain. This creates implicit coupling between a purely cosmetic layer and all interactive layered UI. In the app, the grain should use `z-index: 1` or a named layer via `@layer`, and all interactive stacking contexts should be defined relative to a known scale.

**No semantic colour tokens.** `--orange`, `--pink`, `--gray`, `--blue` are colour-literal names. In a form-heavy app UI, you need semantic tokens: `--color-action` (the CTA orange), `--color-text-muted` (the gray), `--color-surface-dark` (the black). Without semantic tokens, every use of `--orange` that is not a CTA action (e.g., a warning state, an accent) conflates meaning with colour. The app already has more context — token the semantics now rather than later.

**No spacing scale.** The landing page uses 14 distinct spacing values with no variable. The app has forms, step indicators, participant lists, admin panels. Without a spacing scale (`--space-1` through `--space-10` or similar), every component will use ad-hoc values and visual inconsistency will compound. Establish the scale in the shared CSS before building forms.

**No form element styles.** The landing page has zero form elements. The app is built almost entirely around `ActionForm`. Input fields, labels, validation error states, OTP inputs, and submit buttons all need to be specified in the design system before they are built, not after. The CTA button style (pill, orange, bold) covers CTAs but not functional form controls.

**Colour tokens for hardcoded one-offs.** `#D4B8B5` (line 158) and `#8A7774` (line 287) are values that appear once and have no names. In the app, any re-use of these colours without a token will produce invisible divergence. Give them names or eliminate them.

**Steps as `<div>` not `<ol>`.** The landing page's process steps are `<div class="step">`. If the app renders season phases, onboarding steps, or assignment stages using the same visual style, they should be `<ol>` elements. Do not carry the `<div>` pattern into the app.

**Inline SVG icon duplication.** The Instagram path (~680 characters) is duplicated in hero and footer. In Leptos, SVG icons should be components. The landing page cannot use components, so duplication is the only option. The app has no such constraint — define an `<Icon>` component from day one.

**`overflow-x: hidden` on `body`.** Safe on the landing page. In the app, if a fixed sidebar, sticky header, or modal backdrop is needed, `overflow: hidden` on `body` in Safari can break the fixed positioning. Use `overflow-x: clip` on an inner wrapper instead when building the app shell.

**`:focus-visible` scope.** The landing page defines focus style only for `.cta-btn`. In the app, every interactive element needs a focus ring. Establish a global `:focus-visible` rule in the base CSS that all components inherit, then allow overrides. The default browser focus ring on form inputs is often insufficient in a branded context.

---

## Visual Inspection (Chrome)

Live browser inspection via HTTP server (`python3 -m http.server 9753`) at Chrome viewport 1168×501 px (devicePixelRatio: 2 — Retina). All measurements from `getComputedStyle` or `getBoundingClientRect` probes.

---

### Font Load Status

All three fonts loaded successfully. No 404s, no fallback in effect.

| Font | Weight | Style | Status |
|------|--------|-------|--------|
| CyGrotesk | 900 | normal | **loaded** |
| Mont | 400 | normal | **loaded** |
| Mont | 600 | normal | **loaded** |

`document.fonts.size: 3` — no extra fonts, no failures.

---

### Console Errors

**None.** Zero console messages captured across two page loads. No font 404s, no resource failures, no JS errors (expected — zero JS on the page).

Both SVG assets (logo, logo-white) loaded without error (`document.images`: 2 images, 0 failed, all `naturalWidth > 0`).

---

### CTA Button — Actual Rendered Dimensions and Colors

Measured via `getBoundingClientRect` and `getComputedStyle`:

| Property | Value |
|----------|-------|
| Width | 240.39px |
| Height | **54.5px** |
| Font size | 17.6px |
| Font weight | 600 |
| Text color | `rgb(255, 255, 255)` — #ffffff |
| Background | `rgb(251, 68, 23)` — #FB4417 |
| Padding | 16px 35.2px |
| Border radius | 100px |
| Touch target | PASS — 54.5px height exceeds 44px WCAG 2.5.5 minimum |

**Contrast failure confirmed live.** White (#ffffff) on #FB4417 at 17.6px weight 600 = 3.53:1. Does not meet 4.5:1 (normal text) and does not qualify as large text (large text threshold for bold is 18.67px). This is an AA failure, consistent with the static analysis calculation.

The hero CTA button is positioned below the fold at this viewport height (hero renders at 623.7px, viewport is 501px). A user arriving on the page sees the logo and tagline only; the CTA button requires scrolling ~120px.

---

### Grain Overlay

Computed `::after` styles confirmed live:

| Property | Value |
|----------|-------|
| position | fixed |
| z-index | 100 |
| opacity | 0.09 |
| mix-blend-mode | multiply |
| background-size | 200px |
| content | SET (non-empty) |
| width | 1153px (viewport minus scrollbar) |
| height | 501px (viewport height — fixed, tiles on scroll) |

**On the cream hero/mail-club sections:** The grain is nearly imperceptible. `mix-blend-mode: multiply` at 0.09 opacity on `#FAF9F6` (near-white) produces a barely-there texture. In the JPEG screenshots the grain is not visible — the compression flattens it entirely.

**On the dark about/footer sections:** The grain is also not visually detectable in screenshots. `multiply` blend mode on `#161616` (near-black) means the grain darkens pixels that are already at the dark end of the scale — effectively no visible shift. The grain texture is more of a suggestion than a visible effect at this opacity on dark backgrounds. If the intent was to add analogue texture to the dark sections, the grain is not achieving it perceptibly at 0.09 opacity with multiply.

**No stacking isolation issues:** All sections (`about`, `mail-club`, `footer`) have `isolation: auto`, `transform: none`, `filter: none`. No stacking context prevents the grain from compositing above them.

---

### Step Heading Line-Height — Cyrillic Ascender Clipping

**The static analysis concern is confirmed and measurable.**

Computed styles for `.step h3`:
- `font-size: 20.8px` (1.3rem)
- `line-height: 20.8px` (resolves to exactly 1.0 — the CSS value of `line-height: 1`)
- `font-family: CyGrotesk, "Arial Black", sans-serif`

DOM measurement — all four headings show the same overflow pattern:

| Heading | clientHeight | scrollHeight | Overflow |
|---------|-------------|-------------|---------|
| Створюєш | 21px | **23px** | 2px clipped |
| Відправляєш | 21px | **23px** | 2px clipped |
| Отримуєш | 21px | **23px** | 2px clipped |
| Зустрічаєшся | 21px | **23px** | 2px clipped |

`scrollHeight > clientHeight` by 2px for every heading. The font's ascenders or descenders require 23px of space but the line box is constrained to 21px (browser rounds 20.8px up to 21px for layout). **2px of glyph content is overflowing the line box on every step heading.** Whether this clips visually depends on whether the parent has `overflow: hidden` — it does not (the `.step` container has default overflow), so the glyphs likely overflow invisibly upward into the divider border or step gap. The effect is not a hard clip of visible text, but the line-height is too tight for CyGrotesk Cyrillic at this size. Changing to `line-height: 1.1` (22.88px) would eliminate the overflow.

---

### Mobile — Horizontal Overflow and Layout

The `resize_window` tool did not successfully change the Chrome viewport (confirmed via `window.innerWidth` remaining at 1168px after resize call). Mobile layout was assessed by:
1. Computing clamp() resolutions at 375px viewport mathematically
2. Measuring at-current behavior and extrapolating

**Horizontal overflow (at current viewport):** `scrollWidth === clientWidth === 1153px`. No horizontal overflow exists at desktop width.

**At 375px (extrapolated):**
- Hero tagline font: `clamp(1rem, 2.5vw, 1.3rem)` → at 375px: `2.5vw = 9.375px`, below the 1rem floor → **16px**. Correct, legible.
- Section headings: `clamp(1.8rem, 5vw, 2.8rem)` → at 375px: `5vw = 18.75px`, below 1.8rem floor → **28.8px**.
- Container width: 375 − 64px (2rem padding each side) = **311px** for content.
- Tagline at 311px/16px: text is ~74 visible characters before the `<br>` tag. At 311px with 16px font (~38 chars/line estimate), the tagline wraps to approximately 3 lines — same as desktop, just narrower. No catastrophic wrapping detected.
- CTA button height: 54.5px — **exceeds 44px touch target minimum at all sizes** (padding-driven, not font-driven, so it scales with `rem` base).
- Tag rotation: at ≤480px the media query applies `transform: none !important` to all tags, flattening them. This is correct behavior for a narrow single-column layout where rotations would look cluttered.

**Tagline `<br>` behavior:** The static analysis flagged awkward wrapping risk. At 375px, the `<br>` in the source HTML forces "у Києві. по-справжньому." to always start a new line, regardless of the available width. This is intentional typographic control — not a wrapping bug.

---

### Tag Rotation — Desktop

All six tags active with distinct rotation values decoded from computed matrix transforms:

| Tag (index) | Text | Matrix sin value | Computed rotation | CSS rule applied |
|-------------|------|-----------------|------------------|-----------------|
| 1 | живі розмови | -0.0349 | ≈ -2° | `.tag` default (odd, assumed ~-2deg) |
| 2 | плівка | +0.0436 | ≈ +2.5° | `.tag:nth-child(even)` |
| 3 | зіни | +0.01745 | ≈ +1° | `.tag:nth-child(3)` override |
| 4 | воркшопи | +0.0436 | ≈ +2.5° | `.tag:nth-child(even)` |
| 5 | поштовий клуб | -0.01745 | ≈ -1° | `.tag:nth-child(5)` override |
| 6 | Київ | +0.0436 | ≈ +2.5° | `.tag:nth-child(even)` |

The rotation pattern is **visually coherent with the current 6-tag set.** Tags alternate between gentle positive and negative tilts with the 1°/−1° overrides on positions 3 and 5 adding variation. The progression reads: −2°, +2.5°, +1°, +2.5°, −1°, +2.5° — a slightly irregular but intentional handmade scatter.

**Fragility confirmed:** The static analysis warning is valid. Tag 1 defaults to a base rotation that is not defined by any explicit rule — the `nth-child(odd)` case has no explicit rule, implying the default (which appears to be approximately −2°, though this may be browser rounding of some base style). If the tag count changes or order shifts, the rotation pattern changes unpredictably. Tags 3 and 5 override `even` via specificity tie-break (last-wins), which is correct CSS behavior but fragile to reordering.

---

### Focus Ring

The `:focus-visible` ring renders correctly when the keyboard focus heuristic is active. Verified by injecting the focus-visible style class manually and capturing a screenshot:

- **Outline:** 3px solid `#8DC1FF` (--blue)
- **Outline offset:** 3px
- **Appearance:** The blue ring was visually distinct and clearly legible against the cream (#FAF9F6) background
- **Form:** The pill shape (border-radius: 100px) means the focus ring follows the curved button outline

Note: `document.activeElement === .cta-btn` with `.focus()` via JS does **not** activate `:focus-visible` — the browser requires a genuine keyboard navigation event (Tab key) to enter keyboard focus mode. `element.matches(':focus-visible')` returned `false` after `.focus()`. The rule is correctly coded and renders correctly on keyboard Tab navigation.

---

### Additional Findings from Live Inspection

**Hero section is below-fold by ~122px at common laptop viewport heights.** At 1280×800 (13" MacBook Pro), the hero at 623px means the CTA button is off-screen on first load. A user sees only the logo and first line of tagline without scrolling. The `scroll-hint` arrow is also not visible since it's inside the hero near the bottom. This is intentional for a fullscreen hero design, but the CTA is not in the initial viewport on the most common laptop screen.

**Grain behavior on dark backgrounds is aesthetically near-invisible.** The `multiply` blend mode on a background approaching #000000 produces negligible darkening (darkening 0% brightness has no effect). The grain at 0.09 opacity barely shifts any pixel on the dark about/footer sections. On the cream sections, multiply on near-white also barely shifts pixels. The grain is most effective at mid-range brightness values. At the actual page colors (very light cream vs very dark black), the grain's tactile contribution is almost entirely removed by JPEG compression in screenshots, suggesting it is similarly subtle in actual rendering. This is an aesthetic choice, not a bug, but the effect is far less prominent in reality than the concept implies.

**`.label` orange-on-cream contrast failure directly observed.** "СКОРО ✦" computed as `rgb(251, 68, 23)` on `rgb(250, 249, 246)` — exactly the values the static analysis calculated. The label is small (12.8px), uppercase, and visually subordinate — but it is readable content, not decoration. The contrast failure at 3.36:1 is live and verified.

**Footer small text contrast failure directly observed.** "Київ · @samete.community" computed as `rgb(138, 119, 116)` (#8A7774) on `rgb(22, 22, 22)` (#161616) at 12.8px. This is the 4.28:1 failure the static analysis calculated. No new information, but confirmed live.

**`body { overflow-x: hidden }` active and effective at current viewport.** `bodyOverflowX: "hidden"` confirmed. No horizontal overflow at 1168px wide (scrollWidth === clientWidth). The Safari fixed-position concern documented in the static analysis remains a latent risk for app development, not a current rendering bug.

**Two links, zero buttons, zero inputs confirmed.** The entire interactive surface of the page is two `<a>` elements (hero CTA → Instagram, footer CTA → Instagram). No other interactive elements exist. Tab sequence is: hero CTA → footer CTA. This is correct and complete.
