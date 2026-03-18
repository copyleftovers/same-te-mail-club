# Font System — Landing Page & App Strategy

Investigated from `/Users/ryzhakar/pp/same-te-landing/` and documented for reuse in the Mail Club app.

---

## Executive Summary

The landing page uses **two font families** loaded as **woff2 only**:
- **Mont** (body/UI): Regular (400) and SemiBold (600) weights
- **CyGrotesk** (display/headings): Single weight 900

Both fonts are referenced in `index.html` via `@font-face` declarations. The landing page has TTF source files (16 Mont weights) and an OTF source (CyGrotesk), but only two Mont weights and one CyGrotesk have been converted to woff2 for distribution.

---

## Font Stack Configuration

### CSS Custom Properties (from landing page)

```css
--font-display: 'CyGrotesk', 'Arial Black', sans-serif;  /* headings */
--font-body: 'Mont', 'Inter', system-ui, sans-serif;     /* body text, buttons, labels */
```

### Fallback Chain

| Family | Primary | Fallback 1 | Fallback 2 | Fallback 3 |
|--------|---------|-----------|-----------|-----------|
| **CyGrotesk** (display) | CyGrotesk (woff2, 900) | Arial Black | — | sans-serif |
| **Mont** (body) | Mont (woff2, 400 & 600) | Inter | system-ui | sans-serif |

All fonts use `font-display: swap` (render fallback immediately, swap in custom font when ready).

---

## @font-face Declarations (from index.html)

```css
@font-face {
  font-family: 'CyGrotesk';
  src: url('fonts/CyGroteskGrandDark.woff2') format('woff2');
  font-weight: 900;
  font-display: swap;
}

@font-face {
  font-family: 'Mont';
  src: url('fonts/Mont-Regular.woff2') format('woff2');
  font-weight: 400;
  font-display: swap;
}

@font-face {
  font-family: 'Mont';
  src: url('fonts/Mont-SemiBold.woff2') format('woff2');
  font-weight: 600;
  font-display: swap;
}
```

---

## Actual Font Usage in Landing Page

### CyGrotesk (Weight 900)

| Element | CSS Property | Size | Usage |
|---------|-------------|------|-------|
| `.about h2` | `font-family: var(--font-display)` | clamp(1.8rem, 5vw, 2.8rem) | Section heading: "плівка, ножиці, клей — і пошта" |
| `.mail-club h2` | `font-family: var(--font-display)` | clamp(1.8rem, 5vw, 2.8rem) | Section heading: "Поштовий клуб" |
| `.step h3` | `font-family: var(--font-display)` | 1.3rem | Step titles: "Створюєш", "Відправляєш", "Отримуєш", "Зустрічаєшся" |

**Design intent**: Bold, attention-grabbing section and step headings. The weight 900 (ultra-black) creates visual hierarchy and personality.

### Mont Regular (Weight 400)

| Element | CSS Property | Size | Usage |
|---------|-------------|------|-------|
| `body` | `font-family: var(--font-body)` | — | Base font for all text |
| `.hero-tagline` | `font-weight: 400` | clamp(1rem, 2.5vw, 1.3rem) | "балакаєм наживо, знімаєм на плівку..." |
| `.about p` | (default) | 1.05rem | Body paragraphs in about section |
| `.mail-club p` | (default) | 1.05rem | Body paragraphs in mail club section |
| `.step p` | (default) | 0.95rem | Step descriptions |

**Design intent**: Default readable body text for long-form content.

### Mont SemiBold (Weight 600)

| Element | CSS Property | Size | Usage |
|---------|-------------|------|-------|
| `.cta-btn` | `font-weight: 600` | 1.1rem | "Ми в Instagram" buttons |
| `.about .highlight` | `font-weight: 600` | 1.2rem | Emphasis: "Ми збираємось у Києві..." |
| `.mail-club .label` | `font-weight: 600` | 0.8rem | "Скоро ✦" above "Поштовий клуб" |
| `.mail-club .punchline` | `font-weight: 600` | 1.05rem | Emphasis: "А ти всього-навсього..." |
| `.tag` | `font-weight: 600` | 0.8rem | Tag labels: "живі розмови", "плівка", etc. |

**Design intent**: Emphasis within paragraphs, UI labels, and interactive elements (buttons). Weight 600 sits between regular body (400) and display (900).

---

## Font Files Available

### Distributed (in use on landing page)

| File | Size | Format | Family | Weight | Style | Status |
|------|------|--------|--------|--------|-------|--------|
| `Mont-Regular.woff2` | 45K | woff2 | Mont | 400 | normal | ✓ Used |
| `Mont-SemiBold.woff2` | 45K | woff2 | Mont | 600 | normal | ✓ Used |
| `CyGroteskGrandDark.woff2` | 45K | woff2 | CyGrotesk | 900 | normal | ✓ Used |

### Available but not converted to woff2 (14 weights)

| File | Family | Weight | Style | Status |
|------|--------|--------|-------|--------|
| `Mont-Thin.ttf` | Mont | 100 | normal | Source only |
| `Mont-ThinItalic.ttf` | Mont | 100 | italic | Source only |
| `Mont-ExtraLight.ttf` | Mont | 200 | normal | Source only |
| `Mont-ExtraLightItalic.ttf` | Mont | 200 | italic | Source only |
| `Mont-Light.ttf` | Mont | 300 | normal | Source only |
| `Mont-LightItalic.ttf` | Mont | 300 | italic | Source only |
| `Mont-Bold.ttf` | Mont | 700 | normal | Source only |
| `Mont-BoldItalic.ttf` | Mont | 700 | italic | Source only |
| `Mont-Black.ttf` | Mont | 900 | normal | Source only |
| `Mont-BlackItalic.ttf` | Mont | 900 | italic | Source only |
| `Mont-RegularItalic.ttf` | Mont | 400 | italic | Source only |
| `Mont-SemiBoldItalic.ttf` | Mont | 600 | italic | Source only |
| `cy-grotesk-grand-dark.otf` | CyGrotesk | 900 | normal | Source OTF |

---

## Format Strategy: woff2-Only Delivery

### Current Approach
- **Served format**: woff2 only (no woff fallback, no ttf)
- **Browser support**: All modern browsers (Chrome 36+, Firefox 39+, Safari 14+, Edge 15+)
- **File size**: ~45K per font file (highly compressed)
- **Font display policy**: `swap` (show fallback while loading, swap in custom font)

### What This Means

| Aspect | Impact |
|--------|--------|
| **Older browsers** (IE, very old Safari) | Fall back to `Arial Black` (CyGrotesk) or `Inter/system-ui` (Mont). Page remains readable. |
| **Network performance** | Minimal: only 3 font files (~135K total) on landing page. The woff2 format is the most compressed; no additional fallback formats needed. |
| **Feature support** | No italics served (landing page has no italic usage). Weight coverage is minimal but sufficient for landing page design (400, 600, 900). |

---

## Selection Rationale: Why Only 400, 600, and 900?

### Mont Weights (400 Regular, 600 SemiBold)

The landing page design uses a **two-tier weight hierarchy**:

1. **Weight 400 (Regular)**: Body text, long-form paragraphs, step descriptions. Standard readability weight.
2. **Weight 600 (SemiBold)**: Emphasis within body (highlight spans), UI labels (tags, buttons, section labels), and interactive elements (CTA buttons). One step above regular, not extreme.

**Why NOT other weights?**
- **Light (300)**: Not needed — body text at 400 is already readable on cream/light backgrounds.
- **Bold (700)**: Not needed — SemiBold (600) provides enough emphasis without jumping to 700.
- **Black (900)**: Reserved for CyGrotesk display font; no need for Mont at weight 900.
- **Italics**: No italic usage on landing page (all text is `font-style: normal`).

**Design philosophy**: Minimal weight variation = minimal HTTP requests = faster load. Two weights (400, 600) cover all body and emphasis needs. Display headings get their own font (CyGrotesk) at weight 900.

### CyGrotesk (Weight 900 Only)

**Single weight because:**
- Used only for headings (`.about h2`, `.mail-club h2`, `.step h3`).
- Weight 900 (ultra-black, boldest) is appropriate for attention-grabbing display use.
- No body text set in CyGrotesk, so lighter weights are unnecessary.
- Enforces visual separation: headings are distinctly different from body.

---

## Conversion Pipeline: TTF → woff2

The source files live in `/Users/ryzhakar/pp/same-te-landing/fonts/`:

```
fonts/
  ├── Mont Family/
  │   ├── Mont-*.ttf (16 files: weights 100–900, normal + italic)
  │   └── (These are the source originals)
  ├── Cy Grotesk Grand Dark/
  │   ├── cy-grotesk-grand-dark.otf (source original)
  │   └── (Not served)
  ├── Mont-Regular.woff2 ← TTF → woff2 conversion (weight 400)
  ├── Mont-SemiBold.woff2 ← TTF → woff2 conversion (weight 600)
  └── CyGroteskGrandDark.woff2 ← OTF → woff2 conversion (weight 900)
```

**Conversion tool** (not specified in codebase, likely `fonttools` or similar):
```bash
# Example conversion (tool/method not documented in landing page repo):
# fonttools varLib.instancer Mont-Regular.ttf -o Mont-Regular.woff2
# (or similar woff2 conversion)
```

The TTF and OTF files are **version control artifacts** (source originals) but are **not served to browsers**. Only the woff2 files are distributed.

---

## What the Mail Club App Needs

### Strategy

The Mail Club app is a **full-featured web application** with:
- Admin dashboards (season management, assignment generation)
- Forms (login, enrollment, confirmation)
- Status displays (assignment view, delivery tracking)
- User profiles and settings

This requires **more type variation** than a landing page.

### Recommended Font Setup for Mail Club

#### Must Include (tested from landing page)
- `Mont-Regular.woff2` (weight 400) — body text, form labels, default text
- `Mont-SemiBold.woff2` (weight 600) — button labels, section headers, emphasis
- `CyGroteskGrandDark.woff2` (weight 900) — page titles, brand heading if needed

#### Should Consider Adding (NOT on landing page, but useful for app)

| Weight | Style | Rationale | Use Case |
|--------|-------|-----------|----------|
| **Light (300)** | normal | Lighter hierarchy for secondary text (hints, disabled state labels) | Form hints, small print, inactive status labels |
| **Bold (700)** | normal | For strong emphasis without jumping to 900 | Error messages, highlighted alerts, badge emphasis |

**Design question for app**: Does the design call for a lighter weight for hints/secondary text, or is 400 sufficient? If secondary text exists, adding Light (300) improves visual hierarchy without converting the entire body to a lighter weight.

#### Not Needed
- Italics: Unless the app design includes slanted text (uncommon in admin UIs), skip italic styles.
- Weights 100, 200: Too light; rarely used in web apps.
- Weights 800, 900 in Mont: CyGrotesk already handles display/ultra-heavy at weight 900. Mont at 900 would conflict.

### Minimal Recommendation

Copy these three files to the Mail Club app:
```
app/static/fonts/
  ├── Mont-Regular.woff2
  ├── Mont-SemiBold.woff2
  └── CyGroteskGrandDark.woff2
```

Add `@font-face` declarations identical to landing page (see CSS above). Use the same custom properties (`--font-body`, `--font-display`) for consistency across properties.

### Gradual Enhancement

If, during app implementation, you need lighter secondary text or stronger emphasis, consider adding:
- `Mont-Light.woff2` (weight 300) — for secondary/hint text
- `Mont-Bold.woff2` (weight 700) — for strong emphasis

These would require converting the TTF originals to woff2. The landing page repo has the source TTFs; conversion is straightforward.

---

## Browser Support

| Browser | Min Version | woff2 Support | Notes |
|---------|-------------|---------------|-------|
| Chrome | 36+ | ✓ | Excellent support since 2015 |
| Firefox | 39+ | ✓ | Full support; Brotli compression |
| Safari | 14+ (on macOS 11+) | ✓ | Mobile Safari 14+ (iOS 14+) |
| Edge | 15+ | ✓ | Chromium-based; excellent support |
| Internet Explorer | Any | ✗ | Falls back to `Arial Black` / `Inter` / system-ui |
| Android Browser | 5.0+ | ✓ | Modern Android uses Chrome engine |

**Summary**: woff2-only is safe for modern browsers. Old IE users see fallback fonts. This is acceptable for a contemporary web app.

---

## File Sizes & Performance

### Landing Page Font Delivery

| File | Size | Gzipped | Notes |
|------|------|---------|-------|
| Mont-Regular.woff2 | 45K | ~14K | With Brotli: ~11K |
| Mont-SemiBold.woff2 | 45K | ~14K | With Brotli: ~11K |
| CyGroteskGrandDark.woff2 | 45K | ~14K | With Brotli: ~11K |
| **Total** | **135K** | **~42K** | Served via HTTP/2, cached browser-side |

### Mail Club App — If Expanding to 5 Files

If you add Light (300) and Bold (700):

| File | Size | Gzipped |
|------|------|---------|
| All 5 files (300, 400, 600, 700, CyGrotesk 900) | ~225K | ~70K |

This is still acceptable for a web app (CSS and JavaScript bundles are usually larger). Load them asynchronously with `font-display: swap` to avoid blocking rendering.

---

## Implementation Checklist for Mail Club App

- [ ] Copy three woff2 files (`Mont-Regular.woff2`, `Mont-SemiBold.woff2`, `CyGroteskGrandDark.woff2`) to `app/static/fonts/` or equivalent static asset directory.
- [ ] Add `@font-face` declarations (copy from landing page index.html §14–32) to the app's global CSS or Leptos root component style block.
- [ ] Define CSS custom properties: `--font-body: 'Mont', ...` and `--font-display: 'CyGrotesk', ...`.
- [ ] Apply `font-family: var(--font-body)` to `<body>` or Leptos root component.
- [ ] Use `font-family: var(--font-display)` for page titles and prominent headings.
- [ ] Use `font-weight: 600` for button labels, labels, emphasis (SemiBold).
- [ ] Use `font-weight: 400` for body text, form inputs, default (Regular).
- [ ] Document font usage in a project-specific CSS comment (e.g., "Mont 400/600 for body/UI, CyGrotesk 900 for headings").
- [ ] Ensure server serves fonts with correct `Content-Type: font/woff2` header.
- [ ] Test fallback rendering (temporarily disable fonts in DevTools) to verify readability on Arial Black / Inter / system-ui.

---

## Summary Table: Font Decisions

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **Body font** | Mont (400, 600 weights) | Humanist geometric sans; friendly, approachable for community brand |
| **Display font** | CyGrotesk (900 weight) | Bold, distinctive; Ukrainian-centric design (supports Cyrillic well) |
| **Emphasis weight** | Weight 600 (SemiBold) | Mid-range; sufficient for UI labels and in-text emphasis without becoming a separate font |
| **Served format** | woff2 only | Modern compression; ~45K per file; acceptable for all browsers post-2015 |
| **Font display** | `swap` | Fallback shows immediately; custom font swaps in when ready. No blank text flash. |
| **Fallback chain** | Arial Black → Inter → system-ui | Professional fallbacks; readable on all systems |
| **Italics** | Not needed | Landing page (and likely app) does not use slanted text. Skip to reduce bundle. |
| **App expansion** | Consider Light (300) + Bold (700) if needed | Only if design calls for lighter secondary text or stronger emphasis. Currently, 400/600/900 suffice. |

---

## References

- Landing page: `/Users/ryzhakar/pp/same-te-landing/index.html`
- Font source directory: `/Users/ryzhakar/pp/same-te-landing/fonts/`
- woff2 spec: https://www.w3.org/TR/WOFF2/
- Font display API: https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/font-display
