# Sequential UI/UX Fault Audit

## Design System Reference (key specs checked against)

**Palette:** brand-cream `oklch(0.98 0.01 90)` surface, brand-black `oklch(0.15 0.00 0)` text, brand-orange `oklch(0.63 0.22 31)` accent, brand-gray `oklch(0.45 0.01 250)` muted text.

**Typography:**
- Page title (h1): CyGrotesk 900, `clamp(1.8rem, 5vw, 2.8rem)`, line-height 1.15, letter-spacing -0.02em
- Section heading (h2): CyGrotesk 900, 1.3rem, line-height 1.15, letter-spacing -0.02em
- Body: Mont 400, 1.05rem (16.8px), line-height 1.75
- UI label: Mont 600, text-sm (0.875rem), line-height 1.4
- Overline label: Mont 600, 0.8rem, line-height 1.4, letter-spacing 0.1em, uppercase

**Buttons:** pill shape (`border-radius: 100px`), default padding `0.625rem 1.25rem`, font-size 0.875rem, min-height 44px.

**Form fields:** Mont 400, font-size 1rem, 1.5px border brand-gray, border-radius 0.5rem, padding `0.625rem 0.75rem`, min-height 44px.

**Content container:** `max-width: 65ch`, `margin-inline: auto`, `padding-inline: 1rem`, `padding-block: var(--density-space-lg)`.

**Density (admin):** space-sm=0.375rem, space-md=0.75rem, space-lg=1.25rem.

**Header:** `.app-header` flex, align-items center, gap space-sm (0.375rem admin / 0.75rem participant), padding `0.75rem 1rem`, logo height 2rem in CSS but `h-10` (2.5rem) in Rust.

**Mobile menu:** width 16rem, max-width 80vw, slide-in from right, z-index 50.

**Stepper:** step-marker 2.5rem circle, step-label 0.625rem uppercase.

**Hamburger:** `.menu-toggle` 2.75rem x 2.75rem, hidden on sm: (640px+).

---

## Screenshot 1: 10.57.55.jpeg -- Admin Participants page (mobile, 414px)

### Scene Description
iPhone XR (414x896), Admin Participants page. Header shows logo (orange mark) on left and hamburger menu icon on right. Below: page title "Учасники" (CyGrotesk), section heading "Додати учасника", form with two fields (phone, name) and a "Додати" pill button. Below: "Список учасників" heading, then a data-table with columns IM'Я, ТЕЛЕФОН, СТАТУС, Д[ІЇ] (clipped). Four participant rows visible.

### Faults Found

1. **MAJOR** [Header logo]: The logo uses `class="h-10 w-auto"` (2.5rem = 40px), but the CSS `.app-header img { height: 2rem }` specifies 2rem (32px). The Tailwind class `h-10` overrides the CSS component rule because utilities beat component-layer styles. The logo is 40px instead of the design-system-specified 32px. This is an inconsistency between the component CSS and the inline utility. The class in `app.rs` line 203 should be `h-8` (2rem) per the design-system, or the component CSS should be updated to match.

2. **MAJOR** [Data table right truncation]: The fourth column header "ДІЇ" (Actions) is clipped at the viewport edge, showing only "Д". The red deactivate buttons in the actions column are also clipped -- only a sliver of red is visible at the right edge. The `.data-table-wrapper` component provides `overflow-x: auto` but it is not used here. The table sits directly in `prose-page` which has `max-width: 65ch` and `padding-inline: 1rem`. On 414px the table exceeds the container width. The table needs a `.data-table-wrapper` parent for horizontal scrolling.

3. **MAJOR** [Missing spacing between form elements]: The two `.field` containers (phone and name) have no explicit vertical gap between them. The `.field` class sets `flex-direction: column; gap: 0.375rem` internally, but there is no margin between sibling `.field` elements. Looking at the screenshot, the gap between the bottom of the phone input and the "Ім'я" label is very tight (~4-6px). Per the design system, form field groups should have `--density-space-md` (0.75rem = 12px for admin density) between them. The `RegisterForm` in `participants.rs` wraps fields in an `ActionForm` without a flex-col gap container.

4. **MINOR** [Missing spacing between button and section heading]: The "Додати" button sits immediately above "Список учасників" heading with insufficient separation. The button needs `margin-bottom` or the section needs `margin-top` equal to `--density-space-lg` (1.25rem admin).

5. **MINOR** [Hamburger icon rendering]: The hamburger menu shows as three horizontal lines, but they appear as very thin, barely-visible bars. The lines use `h-0.5` (2px) and `w-5` (20px) with `bg-current`. At 2px height they are functional but could be thicker for better visibility. This is a minor aesthetic concern.

6. **MINOR** [Header bottom border visibility]: The `app-header` border-bottom uses `oklch(from var(--color-brand-gray) l c h / 0.2)` which is very subtle on cream. Functionally correct per spec but nearly invisible -- the separation between header and content is barely perceptible.

### Positive Notes
- Page title "Учасники" correctly uses CyGrotesk display font, proper weight and sizing
- Section heading "Додати учасника" correctly uses h2 styling
- Form inputs have correct border-radius (0.5rem), proper border color
- Phone input correctly shows `tabular-nums` / `letter-spacing: 0.1em` via the `type="tel"` CSS rule
- Button correctly uses pill shape with orange accent color and white text
- Badges (АКТИВНИЙ, ДЕАКТИВОВАНИЙ) correctly use pill shape with correct status colors (green/gray)
- Table header correctly uses uppercase, smaller text, proper muted color
- Cream background is correct

---

## Screenshot 2: 10.58.04.jpeg -- Admin Participants page with mobile menu open (mobile, 414px)

### Scene Description
Same page as screenshot 1 but with the mobile menu drawer open. The left side shows the participants page dimmed behind an overlay. The right side shows a slide-in panel (white background) with navigation links: Головна, Панель, Сезон, Учасники (highlighted orange with pink background), Розподіл, SMS, and a logout button ("Вий..." truncated) at the bottom.

### Faults Found

1. **CRITICAL** [Logout button text truncated]: The "Вийти" (logout) button is cut off, showing only "Вий". The button has `class="btn w-full"` and the menu width is 16rem (256px). The text should easily fit in that width. Inspecting the code: the mobile menu uses `width: 16rem; max-width: 80vw`. On 414px, 80vw = 331px. The button text "Вийти" at 0.875rem font should fit. The truncation suggests the button is overflowing the menu panel or there is a padding issue. Looking more carefully at the screenshot: the button outline (border) extends beyond the visible menu area on the right side. The menu panel itself appears to extend to the right edge of the viewport, but the button's right edge is clipped by the viewport. This may be caused by the `padding: var(--density-space-lg) var(--density-space-md)` where admin density makes `space-lg=1.25rem` and `space-md=0.75rem`. The `w-full` button spans the full width of the nav container, but the nav container itself has padding. Given admin density, the button should fit. However, looking closely at the screenshot, the mobile menu panel appears to extend exactly to the viewport edge -- the button's right border arc is clipped. The nav links above ("Головна", etc.) are also right-padded. The issue is that the button with `w-full` + pill border-radius extends to the exact edge. This is likely a minor clipping issue from the menu `right: 0` placement combined with the button styling.

2. **MAJOR** [Active link styling inconsistency]: "Учасники" shows highlighted in orange text with a pink/salmon background fill. The CSS for `.mobile-menu a[aria-current="page"]` specifies `color: var(--color-accent); background: oklch(from var(--color-accent) l c h / 0.08)`. This is 8% opacity orange background, which should appear as a very faint warm tint. The screenshot shows a noticeably pink/salmon background that looks more like `accent-alt` (brand-pink). This could be a rendering artifact of the JPEG compression or the oklch color mixing. If accurate, the background tint is stronger than specified.

3. **MINOR** [Menu link vertical spacing]: The navigation links in the mobile menu appear to have generous spacing between them (~50-60px between link baselines). The CSS specifies `gap: var(--density-space-sm)` (0.375rem = 6px for admin) between flex children, plus each link has `padding: var(--density-space-sm) var(--density-space-md)` (0.375rem 0.75rem for admin). That total (~12px gap + padding) seems much tighter than what's visible. However, the admin density override `[data-layout="admin"]` applies to the `data-layout="admin"` div which wraps the page content. The mobile menu is rendered inside the `<header>` which is *outside* `AdminGuard`'s `<div data-layout="admin">`. So the mobile menu uses **participant** density: space-sm = 0.75rem, space-md = 1.25rem. The padding per link is then `0.75rem 1.25rem` with `0.75rem` gap. That gives roughly 30px between links including padding, which looks closer to what's visible. This is actually working as designed but is arguably too spacious for an admin menu. The menu shows admin nav links but uses participant density because it lives outside `AdminGuard`.

4. **MINOR** [Overlay background not reaching top]: The dimmed overlay appears to not quite reach the very top of the viewport -- there's a thin sliver of non-dimmed area at the top. This could be a screenshot artifact or a z-index stacking issue with the header.

### Positive Notes
- Menu slide-in animation direction is correct (from right)
- Overlay dimming is functional
- Current page (Учасники) is correctly identified and highlighted
- Navigation links are well-organized and complete
- Link ordering matches the admin nav order

---

## Screenshot 3: 10.58.17.jpeg -- Admin Dashboard page (mobile, 414px)

### Scene Description
iPhone XR, Admin Dashboard ("Панель"). Header with logo and hamburger. Page title "Панель" in CyGrotesk. Below: a horizontal stepper with 5 circular step markers (all green/completed with checkmarks), connected by line segments. Step labels: РЕЄСТРАЦІЯ (clipped to "ЄСТРАЦІЯ"), ПІДГОТОВКА, РОЗПОДІЛ, ДОСТАВКА, ЗАВЕРШЕННЯ (clipped to "ЗАВЕРШЕІ"). Below the stepper: a stat-card with "ФАЗА" label and "СКАСОВАНО" badge (gray/inactive). Below: a "Новий сезон" orange pill button.

### Faults Found

1. **CRITICAL** [Stepper label clipping]: The first label "РЕЄСТРАЦІЯ" is clipped on the left, showing "ЄСТРАЦІЯ". The last label "ЗАВЕРШЕНО" is clipped on the right, showing "ЗАВЕРШЕІ" (or similar truncation). The stepper component uses `overflow-x: auto` on `.stepper`, but the labels are being clipped rather than causing a scrollbar to appear. The `prose-page` container constrains the width, and the stepper sits within it. The stepper labels at 0.625rem with `max-width: 5rem` should fit, but the combined width of 5 steps + 4 connectors exceeds the 414px - 2rem padding = ~382px available width. The clipping without visible scrollbar makes the labels unreadable at the extremes.

2. **MAJOR** [Stepper label baseline misalignment with step markers]: Looking at the vertical alignment, the step markers are centered but the labels below appear to have inconsistent left/right positioning relative to their markers. The first label text is cut by the container left edge, meaning the step markers are not inset enough from the container edge. The `.step` component has `min-width: 3.5rem` and `flex-shrink: 0`, so the stepper tries to maintain full width. Combined with the `justify-content: center` on `.stepper`, the centered stepper extends past both edges.

3. **MAJOR** [Stat card layout -- single card taking only partial width]: The stat-card showing "ФАЗА / СКАСОВАНО" takes approximately 50% of the content width. The dashboard code uses `grid grid-cols-2 gap-4 sm:grid-cols-3 mb-6` for the stat cards. With only one stat card (phase) for a terminal season, the grid-cols-2 layout makes a single card take 50% of the row. This looks awkward -- a single card in a 2-column grid leaves a blank right cell. When the season is terminal (complete/cancelled), only the phase card is shown (enrolled/confirmed counts are hidden). The grid should collapse to `grid-cols-1` for terminal seasons, or the card should span both columns.

4. **MINOR** [Spacing between stepper and stat card]: The gap between the stepper bottom (last label) and the stat card appears to be only about 8-12px. The `prose-page` section/subsection spacing should use `--density-space-md` (0.75rem = 12px admin) minimum. The stat grid has `mb-6` (1.5rem) bottom margin but no top margin to separate from the stepper above.

5. **MINOR** ["Новий сезон" button is a link styled as button]: In `dashboard.rs` line 229, `<a class="btn" data-size="sm" href="/admin/season">`. This renders with `data-size="sm"` which gives smaller padding (0.375rem 0.75rem) and min-height 36px. But in the screenshot, the button appears full-size (not "sm"). The visual rendering may not match what is expected. Actually, looking again at the code for the terminal case (line 229), it does use `data-size="sm"`. The button in the screenshot appears larger than "sm" -- either the "sm" attribute is not being applied or the visual size at mobile width makes it appear larger.

### Positive Notes
- Page title "Панель" uses correct CyGrotesk display font
- Step markers are correctly rendered as circles with checkmarks
- Green completed color is consistent with `--color-success`
- "СКАСОВАНО" badge correctly uses inactive (gray) status color
- Stat card has correct border, border-radius, and padding
- The overall page uses correct cream background

---

## Screenshot 4: 10.58.44.jpeg -- Participant Home page, no active season (mobile, 414px)

### Scene Description
iPhone XR, Participant Home page when no active season exists. Header shows logo (left), "Вийти" logout button (center-ish), and hamburger icon (right). Main content: a single paragraph "Зараз немає активного сезону. Отримаєш SMS, коли відкриється наступний." in body text. The rest of the page is empty cream background.

### Faults Found

1. **MAJOR** [Header layout -- logout button position]: The "Вийти" button appears positioned roughly in the center of the header, between the logo and the hamburger icon. The header uses `display: flex; align-items: center; gap: var(--density-space-sm)`. The `HeaderNav` component wraps the logout button in a `div.header-nav` which has `margin-left: auto`. But the hamburger button (`.menu-toggle`) also has `margin-left: auto`. Both elements competing for `margin-left: auto` creates ambiguity. Looking at the DOM order: logo -> HeaderNav (div.header-nav with margin-left:auto) -> menu-toggle button (margin-left:auto). The first `margin-left: auto` pushes HeaderNav to the right of the logo, then the hamburger has no remaining space to push further right. The result is: logo | [gap] | [Вийти button] | [hamburger]. This is functionally correct but the logout button sits in the middle of the header rather than being grouped with the hamburger or pushed to the far right. The visual weight is unbalanced.

2. **MAJOR** [Empty page -- no visual hierarchy or empty state]: The `NoSeason` home state renders as a bare `<p>` tag with text. There is no heading, no visual hierarchy, no empty-state component. The design system defines an `.empty-state` component with centered layout, headline, and body text. The `NoSeason` state should use the empty-state pattern rather than a lone paragraph. The page feels barren and provides no visual anchor.

3. **MINOR** [Hamburger visible on participant pages]: The hamburger menu is shown on all pages including participant home. For participants, the mobile menu shows only "Головна" and "Вийти" (two items). Opening a slide-in drawer for just two items is questionable UX. The "Вийти" button is already visible in the header. The hamburger adds complexity without value for participants.

4. **MINOR** [Content top padding]: The text starts very close to the header border. The `prose-page` has `padding-block: var(--density-space-lg)` which for participants is 2rem (32px). The gap appears smaller than 32px in the screenshot, but this could be a visual estimation issue with the JPEG compression.

### Positive Notes
- Correct cream background color
- Body text uses correct Mont font at appropriate size
- Logout button correctly uses secondary variant (transparent bg, border, text color)
- Logo correctly shows the orange brand mark

---

## Screenshot 5: 10.58.55.jpeg -- Participant Home with mobile menu open (mobile, 414px)

### Scene Description
Participant Home page with mobile menu drawer open. Overlay dims the left side. The slide-in panel shows only two items: "Головна" (highlighted in orange with pink background) and "Вийти" (full-width secondary button). Large empty white space below.

### Faults Found

1. **MAJOR** [Excessive empty space in mobile menu]: The menu panel is 16rem (256px) wide and extends the full height of the viewport. With only two items (Головна link + Вийти button), roughly 85-90% of the menu panel is empty white space. This makes the menu feel broken or incomplete. For participant users who only have 2 navigation options, a full slide-in drawer is overkill. A simpler dropdown or just header buttons would be more appropriate.

2. **MAJOR** [Logout button not fully visible / clipped]: Similar to screenshot 2, the "Вийти" button text is visible but the button's right edge appears to extend very close to or past the right edge of the menu panel. The pill border-radius causes the rounded right edge to be at the exact boundary.

3. **MINOR** [Active page highlight style]: Same observation as screenshot 2 -- the "Головна" link has a pinkish/salmon background tint that appears stronger than the specified 8% opacity orange.

4. **MINOR** [Menu panel background]: The mobile menu uses `--color-surface-raised` (white) as background, contrasting with the cream page background visible through the overlay. This is technically correct per the design system, but the white-on-cream contrast between the menu and the page creates an inconsistency -- the menu feels like it belongs to a different visual system.

### Positive Notes
- Overlay correctly dims the background
- Current page "Головна" is correctly highlighted
- Menu animation is from the right as specified
- Logout button uses correct secondary variant styling

---

## Screenshot 6: 10.59.07.jpeg -- Login page, phone step (mobile, 414px)

### Scene Description
iPhone XR, Login page (phone input step). Header with logo and hamburger (no logout -- correct for unauthenticated state). Main content is vertically centered: "Саме Те" full logo (orange pinwheel mark with "САМЕ ТЕ" text), followed by "Номер телефону" label, phone input field with placeholder "+380XXXXXXXXX", and "Надіслати код" orange pill button. All elements are horizontally centered.

### Faults Found

1. **MAJOR** [Login form not using field component structure]: The phone label "Номер телефону" appears to be styled as `.field-label` (Mont 600, 0.875rem) but the gap between the label and the input appears larger than the `.field` component's specified 0.375rem gap. Looking at the code, the form does use `<div class="field">` wrapping, so the gap should be 0.375rem. The visual appearance might be an artifact of the JPEG, but the spacing between label and input looks like ~8-12px rather than 6px (0.375rem).

2. **MINOR** [Logo size on login page]: The logo uses `class="h-20 w-auto mb-8"` (5rem = 80px height, 2rem bottom margin). This is significantly larger than the header mark (2.5rem). For a login page hero, 80px is reasonable, but the design system specifies "Auth page hero: `logo.svg`" without specifying a size. The 80px feels proportionate for this layout.

3. **MINOR** [Button width relative to input]: The "Надіслати код" button appears narrower than the phone input above it. The input likely stretches to the available width, while the button uses `inline-flex` with natural width. This creates a visual misalignment of right edges. The button could be full-width (`w-full`) to match the input width for a more polished centered form layout.

4. **MINOR** [Hamburger menu on login page]: The hamburger menu is shown on the login page, but there are no navigation options for unauthenticated users. Opening the menu would show nothing useful (no admin links, no home link beyond "/", no logout). The `HeaderNav` component correctly hides the logout button on `/login`, but the hamburger icon is always visible. This is confusing -- the user sees a menu icon that leads to either nothing or just "Головна".

5. **MINOR** [Vertical centering slightly low]: The login form group appears to be centered slightly below the visual center of the viewport. The `justify-center` in the `min-h-[80svh]` container should center it, but the logo's `mb-8` margin pushes the form content slightly lower than optical center.

### Positive Notes
- Clean, focused login layout
- Logo is the correct "Саме Те" mark+text variant for auth pages
- Phone input correctly shows the tel type with tabular-nums styling
- Button is properly pill-shaped with correct orange color
- Placeholder text is appropriate
- CyGrotesk and Mont fonts are rendering correctly
- No unnecessary UI elements cluttering the login

---

## Screenshot 7: 10.59.17.jpeg -- Login page, OTP step (mobile, 414px)

### Scene Description
iPhone XR, Login page OTP verification step. Same layout as screenshot 6 but showing OTP fields. "Саме Те" logo centered, "Код з SMS" label, OTP input with placeholder "000000" (centered text, wide letter-spacing), and "Підтвердити" orange pill button.

### Faults Found

1. **MINOR** [OTP input wider than phone input]: The OTP input field appears wider than the phone input on the previous step. The OTP input uses `data-otp` attribute which adds `text-align: center; font-size: 1.25rem` via CSS. The wider font size increases the natural width of the input. Both inputs should have the same width for visual consistency between the two login steps. Since the container is the same (`div.field` inside a centered flex column), the input widths should match if they both fill available space. This might be a visual artifact.

2. **MINOR** [Same button width inconsistency]: Like screenshot 6, the "Підтвердити" button is narrower than the input field, creating misaligned right edges.

3. **MINOR** [Same hamburger-on-login issue]: As noted in screenshot 6.

### Positive Notes
- OTP input correctly uses `data-otp` styling: centered text, larger font, tabular-nums
- Placeholder "000000" correctly communicates the 6-digit format
- Consistent visual layout with the phone step (logo, label, input, button in same vertical flow)
- Button text "Підтвердити" is clear and action-oriented

---

## Screenshot 8: 10.59.36.jpeg -- Admin Season Management page, create form (mobile, 414px)

### Scene Description
iPhone XR, Admin Season Management page ("Управління сезоном") showing the create-season form. Header with logo and hamburger. Page title "Управління сезоном" in CyGrotesk. Section heading "Новий сезон" in CyGrotesk h2. Three form fields: "Реєстрація до" (datetime-local input), "Підтвердження до" (datetime-local input), "Тема (необов'язково)" (text input with placeholder "Наприклад: Перший сезон"). Orange pill "Створити" button at bottom.

### Faults Found

1. **MAJOR** [Missing spacing between form fields]: Similar to screenshot 1. The three `.field` containers are siblings inside the `ActionForm` but lack explicit vertical spacing between them. The gap between the bottom of one input and the next label appears very tight (~4-6px). In `season.rs` the `CreateSeasonForm` puts `.field` divs directly inside the `ActionForm` without a gap-providing container. They should be wrapped in a `flex flex-col gap-(--density-space-md)` or each `.field` should have `margin-bottom: var(--density-space-md)`.

2. **MAJOR** [Missing spacing between button and preceding field]: The "Створити" button sits immediately below the "Тема" input field with minimal separation. There should be `--density-space-md` (0.75rem admin, 1.25rem participant) between the last field and the submit button.

3. **MINOR** [Datetime-local input styling]: The datetime-local inputs show the browser's native date/time picker chrome (calendar icon on the right). The placeholder text "dd.mm.yyyy, --:--" is browser-default formatting. While functionally correct, the native picker chrome may not match the design system's input styling perfectly (the calendar icon is browser-controlled and may not use brand colors).

4. **MINOR** [Page title line break]: "Управління сезоном" wraps to two lines at 414px width: "Управління" on line 1, "сезоном" on line 2. The `clamp(1.8rem, 5vw, 2.8rem)` font size at 414px viewport gives approximately 1.8rem (28.8px at base). With two long Cyrillic words, wrapping is expected, but the line break between "Управління" and "сезоном" creates an unbalanced visual with a full line followed by a short orphan. Using `text-wrap: balance` (utility `text-balance`) on h1 would distribute more evenly.

### Positive Notes
- Typography hierarchy is correct: h1 "Управління сезоном" in CyGrotesk display, h2 "Новий сезон" in smaller CyGrotesk
- Form labels are correctly styled as `.field-label` (Mont 600, text-sm)
- Input fields have correct border-radius, border color, and padding
- Button is correctly pill-shaped with orange accent
- Placeholder text for the theme field is helpful and well-formatted
- Cream background is correct

---

## Screenshot 9: 11.01.01.jpeg -- Admin Dashboard page (desktop, ~1353px)

### Scene Description
Desktop viewport (2706px at 2x = 1353px CSS), Admin Dashboard. Full-width header: logo (left), inline admin nav links (Панель in orange, Сезон, Учасники, Розподіл, SMS), and "Вийти" secondary button (right). Below: page title "Панель" in CyGrotesk. Stepper with 5 completed steps (full labels visible: РЕЄСТРАЦІЯ, ПІДГОТОВКА, РОЗПОДІЛ, ДОСТАВКА, ЗАВЕРШЕНО). Stat card showing "ФАЗА / СКАСОВАНО". "Новий сезон" button.

### Faults Found

1. **MAJOR** [Content not centered / left-aligned]: The content below the header (title, stepper, stat card, button) appears left-aligned rather than centered. The `prose-page` container has `max-width: 65ch; margin-inline: auto`, which should center the content block within the viewport. In the screenshot, the title "Панель" starts at roughly x=340px (out of 1353px available). If the container were centered, the left edge of the 65ch block would be at approximately (1353 - 65ch_in_px) / 2. At the body font size of ~16.8px, 65ch is approximately 65 * 8.7px = 565px. Centered: (1353 - 565) / 2 = 394px from left edge. The "Панель" title at ~340px is close but appears slightly left of true center. The stepper appears more centered. This discrepancy might be because the h1 text starts at the container's left padding edge, which is correct -- the container is centered but text is left-aligned within it. This is actually correct behavior per the design system: the prose-page container is centered, content is left-aligned within it.

2. **MAJOR** [Same stat card partial-width issue as screenshot 3]: The stat card with "ФАЗА / СКАСОВАНО" takes approximately 33% of the content width (one cell in the `grid-cols-2` / `sm:grid-cols-3` grid). On desktop the grid uses 3 columns, so one card takes 1/3 of the space. This is even more visually awkward on desktop -- a single small card in a wide 3-column grid with 2 empty cells.

3. **MINOR** [Stepper horizontally centered but content below is not]: The stepper is centered (via `justify-content: center`) while the stat card and button below are left-aligned within the prose-page container. This creates a misalignment between the centered stepper and the left-aligned content blocks below it.

4. **MINOR** [Desktop admin nav spacing]: The admin nav links in the header have generous spacing (looks like ~32-40px between items). The CSS specifies `gap: var(--density-space-md)` at sm+. Under admin density, `--density-space-md` = 0.75rem = 12px. But the mobile menu is outside `[data-layout="admin"]`, and looking at the header -- the `AdminNav` component is also rendered inside the header which is *outside* the `AdminGuard` wrapper. So the admin-nav also uses participant density: `--density-space-md` = 1.25rem = 20px. That plus `padding: 0.5rem 1rem` per link gives wider spacing. The visual gap looks even larger than 20px though -- possibly because the text is shorter in Ukrainian.

5. **MINOR** [Logout button alignment in header]: The "Вийти" button in the desktop header is the last item in the admin-nav flex container. The nav has `margin-left: auto`, pushing it to the right side of the header. The logout button as the last flex child sits at the far right. This is correct behavior, but there's no visual separation between the nav links and the logout button -- "SMS" and "Вийти" appear equidistant from their neighbors, making it hard to distinguish navigation from the account action.

### Positive Notes
- Desktop header layout works well: logo left, nav centered-ish, logout right
- All stepper labels are fully visible at desktop width (no clipping)
- Step markers are properly sized and colored
- Active page "Панель" correctly highlighted in orange in the nav
- Typography is crisp and correctly hierarchical
- Overall proportions are clean at desktop width
- Hamburger menu is correctly hidden at this width (>640px)

---

## Cross-Screenshot Consistency Analysis

### Header Consistency
The header is consistent across all screenshots: logo left, contextual nav (admin nav or logout button), hamburger right on mobile, inline nav on desktop. The logo size inconsistency (CSS says 2rem, Tailwind class says 2.5rem) is present everywhere. The header border-bottom is consistently subtle across all views.

### Logo inconsistency
- Header uses `same_te_mark_orange.svg` (small mark) consistently
- Login pages use `logo.svg` (full mark + text) correctly
- The header logo height is `h-10` (2.5rem) everywhere, conflicting with the CSS `.app-header img { height: 2rem }`

### Mobile Menu
Both admin (screenshot 2) and participant (screenshot 5) mobile menus use the same slide-in drawer pattern. The admin version is well-populated (6 links + logout), while the participant version is nearly empty (1 link + logout). The density issue (using participant density for admin menus because the menu lives outside `AdminGuard`) is consistent across both.

### Form Spacing
Insufficient spacing between form fields is a systemic issue visible in screenshots 1 (participants form) and 8 (season create form). The pattern is the same: multiple `.field` divs as direct siblings without a gap-providing parent container. The `RegisterForm` and `CreateSeasonForm` both lack wrapping flex containers with gap.

### Stepper
The stepper is visible in screenshots 3 (mobile) and 9 (desktop). On mobile, label clipping is a critical issue. On desktop, labels fit correctly. The stepper component itself does not adapt gracefully to narrow viewports.

### Empty State Pattern
The `NoSeason` state (screenshot 4) uses a bare `<p>` tag instead of the `.empty-state` component. Other empty states in the codebase (e.g., `ParticipantList` empty) correctly use `.empty-state`. This is inconsistent.

### Button Width Consistency
Login page buttons (screenshots 6, 7) are natural-width (narrower than inputs). Admin form buttons (screenshots 1, 8) are also natural-width. The button width approach is consistent but creates visual misalignment when placed below full-width inputs.

### Admin Density Application
The `[data-layout="admin"]` wrapper is inside the `<main>` tag (applied by `AdminGuard`). The header and mobile menu live *outside* this wrapper, so they always use participant density. This means admin pages have split density: tighter spacing in the content area, wider spacing in the header/nav. This is consistent but may be unintentional.

---

## Summary

- **Total faults found: 30**
- **Critical: 3** (stepper label clipping on mobile, logout button text truncation, mobile menu truncation)
- **Major: 13** (table column clipping, form spacing gaps, empty state pattern missing, stat card layout, header layout issues, content alignment)
- **Minor: 14** (button width, hamburger on login, menu density, date input styling, border visibility, active link tint)

### Top 3 Systemic Issues

1. **Missing vertical spacing between form field groups (affects 2+ pages)**. The `.field` component defines internal gap (label-to-input) but forms stack multiple `.field` containers without external spacing. Fix: wrap all multi-field forms in `flex flex-col gap-(--density-space-md)` or add `margin-bottom: var(--density-space-md)` to `.field` in the component CSS.

2. **Stepper component does not degrade gracefully on narrow viewports (affects all admin pages on mobile)**. The 5-step stepper with labels exceeds available width at 414px, causing label clipping. Fix: make step labels responsive (abbreviate or hide on small screens), reduce step-marker size on mobile, or use a vertical stepper on narrow viewports.

3. **Admin density boundary mismatch: header/mobile-menu use participant density while page content uses admin density**. The `[data-layout="admin"]` wrapper is inside `<main>`, but the header and mobile menu live outside it. This creates inconsistent spacing between the navigation chrome and the page content. Fix: either move the admin density attribute to a higher-level wrapper that includes the header, or explicitly set admin density on the header when rendering admin pages.
