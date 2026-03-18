# Logo System Reference

Comprehensive guide to the Саме Те brand logo system. Source: `/Users/ryzhakar/pp/same-te-landing/Logo/SVG/`.

## Quick Start

For the mail club app, use:
- **Navigation header / favicon**: `same_te_mark_*` (icon only, compact)
- **Footer / dark background**: `same_te_mark_white.svg` or `same_te2_horizontal_white.svg`
- **Light background**: `same_te_mark_black.svg` or `same_te_mark_orange.svg`
- **Print / documentation**: `same_te_*` (full wordmark) or `same_te2_horizontal_*` (horizontal layout)

---

## Logo Categories (4 × 6 Variants = 24 SVGs)

### 1. Main Wordmark (`same_te_*.svg`)

**What it is**: Full vertical composition with complete "Саме Те" Ukrainian text wordmark + geometric mark. Used for primary branding, posters, and formal applications.

**ViewBox**: `0 0 1920 1080` (16:9 aspect ratio)

**Composition**:
- Black or colored geometric mark at top
- "Саме Те" text in sans-serif below
- Light accent/highlight area (e.g., nude/beige secondary shape in `_black` variant)

**Color variants**: black, blue, grey, nude, orange, white

**Best for**:
- Logo on landing page / brand pages
- Printed materials (letterhead, business cards)
- Brand identity presentations
- Web header/brand block (when space permits full height)

**Lighting**:
- Use `_black` on white/light backgrounds
- Use `_white` on dark/colored backgrounds
- Use `_orange` on white for warmth (primary brand color)

---

### 2. Mark Only (`same_te_mark_*.svg`)

**What it is**: Geometric icon/symbol alone, no text. Compact standalone mark suitable for small spaces.

**ViewBox**: `0 0 1920 1080` (16:9 aspect ratio, though mark occupies only ~30% of canvas)

**Composition**: Geometric abstract mark — a stylized shield or flowing organic shape, rendered in a single color

**Color variants**: black, blue, grey, nude, orange, white

**Best for**:
- Favicon/tab icon
- Navigation bar / app header logo
- App icon in sidebars
- Social media profile pictures
- Button icons (e.g., "home" button in nav)
- Small badge or stamp
- favicon.svg should match this category

**Lighting**:
- Use `_black` on white/light backgrounds
- Use `_white` on dark/colored backgrounds
- Use `_orange` on white for brand consistency

**Recommended sizes**: 16px–64px (stays readable down to favicon sizes)

---

### 3. Secondary Horizontal (`same_te2_horizontal_*.svg`)

**What it is**: Horizontal layout with mark on left, "Саме Те" text on right. Optimized for horizontal space constraints (e.g., web header, email signature, social media cover).

**ViewBox**: `0 0 1920 1080` (16:9 aspect ratio, but layout is very horizontal)

**Composition**:
- Geometric mark on left (~20% width)
- Text wordmark on right (~40% width)
- Whitespace on far right

**Color variants**: black, blue, grey, nude, orange, white

**Best for**:
- Web header / branding bar
- Email signature
- Social media cover image (crop to fit platform)
- Documents where horizontal space is abundant
- Presentation title slides

**Lighting**: Same as Main Wordmark

**Advantage over Main**: Reads better in constrained-height layouts (e.g., a 200px tall header)

---

### 4. Transparent/Knockout (`same_te_transparant_*.svg`)

**What it is**: Same composition as Main Wordmark, but designed to layer over photos, gradients, or complex backgrounds. The "transparent" variant typically has:
- Outlined/stroked shapes (not filled)
- Hollow letters allowing background to show through
- High contrast outline for readability on any background

**ViewBox**: `0 0 1920 1080` (16:9 aspect ratio)

**Composition**: Full wordmark with transparent/hollow interior

**Color variants**: black, blue, grey, nude, orange, white

**Best for**:
- Hero section overlays (mark + text over background image)
- Watermark on photos
- Transparent PNG export (though SVG is vector, so transparency is structural)
- Dark/light background adaptive use (single file handles both)
- Artistic / editorial layouts

**Lighting**:
- `_white` over dark images
- `_black` over light images
- `_orange` over neutral grey images for brand pop

---

## Root SVG Files (Used by Landing Page)

These three files are **directly referenced** by the landing site at `/Users/ryzhakar/pp/same-te-landing/`.

### `logo.svg`

**ViewBox**: `554 117 811 845` (tightly cropped around content)

**Colors**: Orange (#e84825) primary, nude/beige (#eed2cf) secondary accent

**Content**: Full wordmark with two-color treatment. Matches `same_te_orange.svg` from Main category.

**Used for**: Primary brand logo on landing page (hero, header)

**How to match**: This is a **copy of `Logo/SVG/Main/same_te_orange.svg`** with viewBox adjusted for tighter crop.

---

### `logo-white.svg`

**ViewBox**: `554 117 811 845` (tightly cropped, same crop as `logo.svg`)

**Colors**: Off-white (#faf9f6) text, no colored accent

**Content**: Full wordmark, white/light color only

**Used for**: Dark background contexts (e.g., dark footer, dark hero section with overlay)

**How to match**: This is a **copy of `Logo/SVG/Main/same_te_white.svg`** with viewBox adjusted for tighter crop.

---

### `favicon.svg`

**ViewBox**: `605 171 709 738` (tightly cropped around mark)

**Colors**: Orange (#e84825) single color

**Content**: Geometric mark only, no text. Minimal line work suitable for small icon rendering.

**Used for**: Browser tab favicon

**How to match**: This is a **copy of `Logo/SVG/Mark/same_te_mark_orange.svg`** with viewBox adjusted for tighter crop and possible line weight adjustment for 16px readability.

---

## Variant System Summary

| Category | Layout | Content | ViewBox | Use Case |
|----------|--------|---------|---------|----------|
| **Main** | Vertical | Wordmark + mark | 0 0 1920 1080 | Primary branding, posters |
| **Mark** | Compact | Icon only | 0 0 1920 1080 | Favicon, nav, small spaces |
| **Secondary (Horizontal)** | Horizontal | Wordmark + mark side-by-side | 0 0 1920 1080 | Web header, email signature |
| **Transparent** | Vertical | Wordmark + mark (outlined) | 0 0 1920 1080 | Overlay on photos, watermark |

**Color Axis (all categories)**:
- `_black` → Use on light backgrounds
- `_white` → Use on dark backgrounds
- `_orange` → Use on white/neutral for brand accent
- `_blue` → Use on light backgrounds (secondary brand color)
- `_grey` → Neutral option for monochrome contexts
- `_nude` → Warm neutral, use on light backgrounds

---

## SVG Technical Details

All SVG files use:
- **Codec**: UTF-8 XML
- **Units**: Coordinate-based (no physical dimensions; scale with viewBox)
- **Structure**: Path/polygon elements with CSS class styling
- **No embedded raster**: Pure vector (scalable to any size)

To use in HTML/CSS:
```html
<!-- Direct embedding -->
<img src="logo.svg" alt="Саме Те" />

<!-- As SVG object (allows CSS interaction) -->
<object data="logo.svg" type="image/svg+xml"></object>

<!-- As background image -->
<div style="background-image: url(logo.svg)"></div>

<!-- Inline (copy SVG content into HTML) -->
<svg viewBox="..."><!-- content --></svg>
```

---

## Unused Variants (Available for Future Use)

**Secondary Horizontal variants**: All 6 colors exist. Use if horizontal layout becomes primary branding need.

**Transparent variants**: All 6 colors exist. Use if overlay/watermark becomes frequent.

**Alternative colors**: Grey and nude variants exist as neutral options for monochrome printing or alternate brand contexts.

---

## Recommendations for Mail Club App

1. **Navigation logo** (top-left corner):
   - File: `Logo/SVG/Mark/same_te_mark_black.svg` (or `_white` if dark nav)
   - Size: 40–48px height
   - Purpose: Compact, recognizable, fast load

2. **Footer logo**:
   - File: `Logo/SVG/Mark/same_te_mark_white.svg` (if dark footer) or `_black.svg` (if light footer)
   - Size: 32–40px height
   - Purpose: Consistent, unobtrusive footer branding

3. **Favicon**:
   - File: `Logo/SVG/Mark/same_te_mark_orange.svg` (current: `/Users/ryzhakar/pp/same-te-landing/favicon.svg`)
   - Size: 16px, 32px, 64px exports
   - Purpose: Tab icon, bookmark icon

4. **Auth page hero / brand block**:
   - File: `Logo/SVG/Main/same_te_orange.svg`
   - Size: 200px–300px height
   - Purpose: Visual anchor, brand presence on onboarding

5. **Email templates / password reset links**:
   - File: `Logo/SVG/Mark/same_te_mark_orange.svg`
   - Size: 60px height
   - Purpose: Lightweight email header

---

## Files to Exclude from App

- **Desktop-oriented Main variants**: Not needed if app is mobile-first. Mark + horizontal variants cover all UI needs.
- **Secondary Horizontal variants (other colors)**: Only `_black` and `_white` needed for standard light/dark backgrounds.

To reduce asset size in production: copy only used SVG files into `/app/public/assets/logos/` and reference by filename.

---

## Related Files

- Source logos: `/Users/ryzhakar/pp/same-te-landing/Logo/SVG/`
- Currently used: `/Users/ryzhakar/pp/same-te-landing/{logo.svg, logo-white.svg, favicon.svg}`
- App integration point: `/Users/ryzhakar/pp/same-te-mail-club/public/` (for web assets)

---

## Brand Color Reference

From logo files:
- **Orange (Primary)**: `#e84825`
- **Black**: `#161616`
- **Nude/Beige (Accent)**: `#eed2cf`
- **White**: `#faf9f6`

Use these hex codes when styling buttons, links, or accents that echo the logo.
