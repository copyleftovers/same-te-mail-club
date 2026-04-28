# UI/UX Fault Audit

## Screenshot Inventory

| File | Page | Viewport | State |
|------|------|----------|-------|
| `10.57.55.jpeg` | Admin: Participants (`/admin/participants`) | iPhone XR (414x896) | Participant list with 4 entries, hamburger menu closed |
| `10.58.04.jpeg` | Admin: Participants (`/admin/participants`) | iPhone XR (414x896) | Mobile menu open (slide-in panel) |
| `10.58.17.jpeg` | Admin: Dashboard (`/admin`) | iPhone XR (414x896) | Cancelled season state, stepper with all completed, hamburger menu closed |
| `10.58.44.jpeg` | Participant: Home (`/`) | iPhone XR (414x896) | NoSeason state, logout button visible |
| `10.58.55.jpeg` | Participant: Home (`/`) | iPhone XR (414x896) | NoSeason state, mobile menu open with participant nav |
| `10.59.07.jpeg` | Login: Phone step (`/login`) | iPhone XR (414x896) | Phone input centered, logo visible |
| `10.59.17.jpeg` | Login: OTP step (`/login`) | iPhone XR (414x896) | OTP input centered, logo visible |
| `10.59.36.jpeg` | Admin: Season create (`/admin/season`) | iPhone XR (414x896) | Create season form with empty datetime-local inputs |
| `11.01.01.jpeg` | Admin: Dashboard (`/admin`) | Desktop (~1240px) | Cancelled season state, full admin nav bar, stepper visible |

---

## Critical Faults (breaks usability)

### C1. Participant table columns clipped on mobile (right side cut off)
- **Screenshot:** `10.57.55.jpeg`
- **Location:** Participant list table, rightmost column ("ДIЯ" / Actions column)
- **Observed:** The table extends beyond the viewport. The "ДIЯ" column header is truncated to "Д", and the destructive (deactivate) buttons in the action column are partially cut off (only a sliver of red visible). Users cannot tap the deactivate button.
- **Expected:** Per `design-system.md`, `.data-table-wrapper` should wrap the table with `overflow-x: auto` and `-webkit-overflow-scrolling: touch`. The table is missing this wrapper, so the actions column is inaccessible. Alternatively, the table should use a responsive card layout on mobile. This directly blocks admin functionality.

### C2. Hamburger menu icon is not recognizable as a hamburger icon
- **Screenshot:** `10.57.55.jpeg`, `10.58.17.jpeg`, `10.58.44.jpeg`, `10.59.07.jpeg`, `10.59.17.jpeg`, `10.59.36.jpeg`
- **Location:** Top-right corner of the header
- **Observed:** The hamburger icon renders as 3 horizontal lines stacked with `mt-1` spacing between `<span>` elements inside a flex container (`.menu-toggle` is `display: flex`). The flex container stacks the children horizontally by default, not vertically. The result is that the three lines appear as a flat horizontal dash cluster rather than a clearly recognizable 3-line hamburger. The lines appear bunched together as a single thick dash.
- **Expected:** The hamburger icon should render as three clearly separated horizontal lines stacked vertically. The `.menu-toggle` uses `display: flex; align-items: center; justify-content: center` but lacks `flex-direction: column`, so the three `<span>` lines are laid out in a row, not a column.

---

## Major Faults (degrades experience significantly)

### M1. No spacing between form fields and submit buttons
- **Screenshot:** `10.57.55.jpeg` (participants form), `10.59.36.jpeg` (season create form)
- **Location:** Form sections - between the last input field and the "Додати" / "Створити" button
- **Observed:** The submit button sits directly below the last input with no discernible spacing. The field group uses `.field` with `gap: 0.375rem` between label and input, but there is no margin/gap between consecutive `.field` elements or between the last field and the button.
- **Expected:** Per `design-system.md`, density spacing `--density-space-md` (0.75rem in admin context) should separate form groups. The `ActionForm` needs a gap or margin between fields and the submit button.

### M2. Stepper labels clipped on mobile - leftmost label truncated
- **Screenshot:** `10.58.17.jpeg`
- **Location:** Phase stepper, leftmost step label
- **Observed:** The first step label reads "ЕCТРАЦIЯ" instead of "РЕЄСТРАЦIЯ" - the "Р" prefix and possibly other characters are cut off by the left edge of the container. Similarly on the right edge "ЗАВЕРШЕН" is cut to "ЗАВЕРШЕI" (partially visible).
- **Expected:** The stepper component uses `overflow-x: auto` on `.stepper`, but the labels are still clipping because the overall stepper width exceeds the viewport and the first/last items lack sufficient padding/margin. The `prose-page` padding-inline of `1rem` is not enough to prevent edge-clipping. The stepper should have padding-inline on the scrollable area, or the first/last labels should be visible.

### M3. Logout button positioned oddly in participant header
- **Screenshot:** `10.58.44.jpeg`
- **Location:** Header bar, between logo and hamburger menu
- **Observed:** The "Вийти" (Logout) button appears between the logo and the hamburger menu icon, creating a cramped three-element header. On the participant home page, the header shows: [Logo] [Вийти button] [Hamburger]. The logout button and hamburger compete for space in the right side of the header.
- **Expected:** The logout functionality should only appear in the mobile menu (it already does via `logout-button-mobile`). Having both a visible header logout button AND a hamburger menu icon is redundant and cluttered. On desktop (>640px), the hamburger is hidden and only the logout button shows, which is fine. But on mobile, both are visible simultaneously.

### M4. Mobile menu has no close button - only overlay tap to dismiss
- **Screenshot:** `10.58.04.jpeg`, `10.58.55.jpeg`
- **Location:** Mobile menu slide-in panel
- **Observed:** The mobile menu panel has no visible close/X button. The only way to dismiss it is to tap the semi-transparent overlay area or press Escape (which requires a physical keyboard). There is no visual affordance indicating how to close the menu.
- **Expected:** Mobile navigation patterns should include a visible close affordance (X button or similar) at the top of the slide-in panel. Relying solely on overlay taps is not discoverable for all users.

### M5. Participant home page NoSeason state has no visual hierarchy or engagement
- **Screenshot:** `10.58.44.jpeg`
- **Location:** Main content area
- **Observed:** The NoSeason state renders as a single paragraph of plain text: "Зараз немає активного сезону. Отримаєш SMS, коли відкриється наступний." with no heading, no icon, no visual structure. The content sits at the top-left of the `.prose-page` container with the body line-height.
- **Expected:** Per `design-system.md`, empty states should use the `.empty-state` component pattern with `.empty-state-headline` (CyGrotesk, 900 weight, 1.3rem) and `.empty-state-body` (0.875rem, muted color), centered vertically with `min-height: 12rem`. The current implementation is a bare `<p>` tag with no structural component.

### M6. Form spacing inconsistency between field label and input
- **Screenshot:** `10.57.55.jpeg`
- **Location:** "Номер телефону" and "Ім'я (для Нової Пошти)" labels
- **Observed:** The labels sit very close to the inputs below them (the `.field` gap of `0.375rem` is correct), but there is essentially no gap between the phone field and the name field. The two field groups blend together with the label of the second field appearing to be associated with the input above it.
- **Expected:** Consecutive `.field` groups should have `--density-space-md` gap between them (0.75rem in admin density). Currently the fields are sequential siblings with no containing flex/gap wrapper.

### M7. Login page excessively vertically centered with large empty top area
- **Screenshot:** `10.59.07.jpeg`, `10.59.17.jpeg`
- **Location:** Full page layout
- **Observed:** The login form is vertically centered using `min-h-[80svh] justify-center`, resulting in roughly 40% of the viewport being empty above the logo. On a mobile device, this pushes the form well below the natural reading position and the input may be pushed below the keyboard fold when tapped.
- **Expected:** The form should be positioned in the upper third of the viewport (roughly `padding-top: 20vh` or similar) rather than dead-center. Vertical centering works on desktop but on mobile it creates an awkward bottom-heavy layout that conflicts with keyboard interaction.

### M8. Dashboard stat cards not visible on mobile in cancelled state
- **Screenshot:** `10.58.17.jpeg`
- **Location:** Below the stepper, content area
- **Observed:** In the cancelled season state, only a single small card showing "ФАЗА / СКАСОВАНО" is visible. The `grid grid-cols-2` layout renders this as a half-width card taking up a small portion of the screen. The rest of the page is empty.
- **Expected:** For terminal states (cancelled/complete), the layout should adapt. A single stat card in a 2-column grid looks orphaned. It should either span full width or be presented in a non-grid layout (e.g., as a banner or info-list item).

---

## Minor Faults (polish issues, inconsistencies)

### m1. Header logo size inconsistency between code and CSS
- **Screenshot:** All screenshots
- **Location:** Header, logo image
- **Observed:** The logo in `app.rs` uses `class="h-10 w-auto"` (40px height). The `.app-header img` CSS sets `height: 2rem` (32px). The Tailwind utility class `h-10` (2.5rem = 40px) overrides the component CSS, but the design system specifies the header logo at 2rem.
- **Expected:** Per `.app-header img { height: 2rem }`, the logo should be 32px. The inline class `h-10` overrides this to 40px. One of these should be removed to avoid confusion - preferably remove the inline class and let the component CSS rule, or update the component CSS to match.

### m2. "Новий сезон" button on dashboard has no spacing from stat card above
- **Screenshot:** `10.58.17.jpeg`
- **Location:** Below the ФАЗА/СКАСОВАНО card, before the "Новий сезон" button
- **Observed:** The "Новий сезон" button link sits below the stat card grid with only the `mb-6` from the grid container providing spacing. This is adequate but the button is wrapped in a `<p>` tag which adds default paragraph margins that are inconsistent with the density system.
- **Expected:** The button should use explicit spacing via `margin-top: var(--density-space-md)` or be in a flex container with gap, not relying on `<p>` default margins.

### m3. Mobile menu links have inconsistent visual weight
- **Screenshot:** `10.58.04.jpeg`
- **Location:** Mobile menu slide-in panel
- **Observed:** All menu links (Головна, Панель, Сезон, Учасники, Розподіл, SMS) use the same font-size and weight. The active link "Учасники" has an orange text + pink background highlight, which is correct per CSS (`.mobile-menu a[aria-current="page"]`). However, the "Вийти" button at the bottom is truncated/cut off horizontally - only "Вий" is visible due to the panel width.
- **Expected:** The logout button text should be fully visible. The `.mobile-menu` width is `16rem` with `max-width: 80vw`. On 414px viewport, 80vw = 331px which should accommodate "Вийти" text. The issue may be that the button has `w-full` class but the padding is eating into the text area. Also the button border creates a visual break that clips.

### m4. Season create form: datetime-local inputs lack placeholder styling
- **Screenshot:** `10.59.36.jpeg`
- **Location:** "Реєстрацiя до" and "Пiдтвердження до" datetime-local inputs
- **Observed:** The datetime-local inputs show browser-default placeholder text "dd.mm.yyyy, --:--" in a gray color. This is native browser behavior, but the visual treatment differs from the text input placeholder "Наприклад: Перший сезон" which follows the design system styling.
- **Expected:** Browser-native datetime-local inputs cannot be fully styled, but the visual inconsistency should be noted. No action required unless custom date inputs are desired.

### m5. Form field borders appear heavier/darker than specified
- **Screenshot:** `10.57.55.jpeg`, `10.59.07.jpeg`, `10.59.17.jpeg`, `10.59.36.jpeg`
- **Location:** All text input fields
- **Observed:** The input borders appear as a solid dark gray (~2px visual weight), which is visually heavier than what `1.5px solid var(--color-brand-gray)` should produce. The border color `--color-brand-gray` (`oklch(0.45 0.01 250)` / `#565656`) on the cream background creates a border that dominates the input visually.
- **Expected:** Per `design-system.md`, the border is `1.5px solid var(--color-brand-gray)`. This is correct by specification, but the visual result on the cream background is heavier than typical form inputs. Consider whether `oklch(from var(--color-brand-gray) l c h / 0.4)` would be more appropriate for inactive borders, reserving full `--color-brand-gray` for hover/active states.

### m6. OTP input field styling inconsistency
- **Screenshot:** `10.59.17.jpeg`
- **Location:** OTP code input field
- **Observed:** The OTP input has `data-otp` which triggers `text-align: center; font-size: 1.25rem; letter-spacing: 0.1em; font-variant-numeric: tabular-nums`. The placeholder "000000" renders centered and larger. The input itself lacks the `data-otp` attribute in the rendered HTML because `data-otp` is set as a boolean attribute in the view macro but may not render correctly.
- **Expected:** Verify `data-otp` renders in the DOM. The OTP input styling appears to be working (centered text, monospace numerics), but should be verified. The field height appears taller than other inputs due to the `font-size: 1.25rem` override.

### m7. Login buttons use inconsistent sizing
- **Screenshot:** `10.59.07.jpeg`, `10.59.17.jpeg`
- **Location:** "Надіслати код" and "Підтвердити" buttons
- **Observed:** Both buttons use the default `.btn` size (no `data-size` attribute). They render as pill-shaped orange buttons with white text, which is correct. However, compared to the phone input above them, the buttons appear noticeably smaller/narrower. The button width is determined by text content, creating buttons of different widths between the two steps.
- **Expected:** Per `design-system.md`, the primary CTA button should use the default size with `padding: 0.625rem 1.25rem`. The buttons look correct per spec, but on mobile they feel undersized relative to the full-width input above them. Consider using `w-full` or `data-size="lg"` for the login flow's primary CTA.

### m8. No "back to phone step" option in OTP step
- **Screenshot:** `10.59.17.jpeg`
- **Location:** OTP step of login page
- **Observed:** Once the OTP step activates, the phone step is hidden via `style:display="none"`. There is no way for the user to go back and re-enter their phone number if they made a mistake.
- **Expected:** A "try different number" link or back button should be available to return to the phone input step. This is a UX flow issue, not strictly a visual fault, but the absence of any navigation affordance in this state is a visible gap.

### m9. Desktop admin nav logout button misaligned with nav links
- **Screenshot:** `11.01.01.jpeg`
- **Location:** Header nav bar, rightmost element
- **Observed:** The "Вийти" button is a `.btn` with `data-variant="secondary"` and `data-size="sm"` inside the `.admin-nav` flex container. It has a visible border and pill shape, while the nav links are plain text. The button's visual weight (border + padding) makes it appear misaligned vertically with the text links, and the button appears slightly lower or higher than the link text baseline.
- **Expected:** The logout button should align its text baseline with the nav link text. The `min-height: 36px` on `data-size="sm"` combined with the nav link `min-height: 44px` creates a height mismatch. Both should vertically center-align within the flex container.

### m10. Stepper connector color inconsistency
- **Screenshot:** `10.58.17.jpeg`, `11.01.01.jpeg`
- **Location:** Stepper connectors between step circles
- **Observed:** All step markers show completed (green circles with checkmarks), but the connectors between them remain the default gray color (`oklch(from var(--color-brand-gray) l c h / 0.2)`). The CSS rule `.step[data-status="completed"] + .step-connector` should turn completed connectors green, but the `+` adjacent sibling combinator only works if `.step-connector` directly follows `.step` in the DOM.
- **Expected:** When all steps are completed, connectors should also be green (matching `--color-success`). If the DOM order is `step, connector, step, connector, ...` then the CSS `step[completed] + connector` should work. If connectors are rendered as separate elements between steps, the selector may not match correctly.

### m11. Dashboard page title "Панель" uses CyGrotesk display font at excessively large size on mobile
- **Screenshot:** `10.58.17.jpeg`
- **Location:** Page heading "Панель"
- **Observed:** The h1 heading "Панель" uses `clamp(1.8rem, 5vw, 2.8rem)`. At 414px viewport, `5vw = 20.7px = 1.29rem`, which is below the clamp minimum, so it renders at `1.8rem` (28.8px). This is appropriate.
- **Expected:** Size is correct. No fault here - confirming the clamp works as intended.

### m12. "Список учасникiв" heading spacing from table
- **Screenshot:** `10.57.55.jpeg`
- **Location:** Between section heading "Список учасникiв" and the table header row
- **Observed:** The h2 heading "Список учасникiв" has `margin-bottom: var(--density-space-sm)` (0.375rem in admin density), which creates only a thin gap before the table header row (ІМ'Я / ТЕЛЕФОН / СТАТУС / ДIЯ). The heading and table header visually merge.
- **Expected:** The gap between a section heading and its content should be `--density-space-md` for visual separation. The current `--density-space-sm` is too tight for admin density.

### m13. Participant mobile menu shows truncated logout button
- **Screenshot:** `10.58.55.jpeg`
- **Location:** Mobile menu, bottom element
- **Observed:** The "Вийти" (logout) button at the bottom of the mobile menu renders as a full-width secondary button with a visible border. The button text and border are fully visible in this screenshot, but the button's visual style (bordered pill) is inconsistent with the plain text links above it.
- **Expected:** The logout action in the mobile menu could be styled as a plain text link (matching the other menu items) rather than a bordered button, since it's inside the menu context. Alternatively, the button is intentionally differentiated to signal a destructive/important action, which is acceptable but visually jarring.

### m14. Empty space below content on most mobile pages
- **Screenshot:** `10.58.17.jpeg`, `10.58.44.jpeg`, `10.59.07.jpeg`, `10.59.17.jpeg`, `10.59.36.jpeg`
- **Location:** Lower half of the viewport
- **Observed:** Most pages have significant empty space below the content. The dashboard cancelled state, the NoSeason home, the login pages, and the season create form all show content that occupies only the top 30-50% of the viewport, leaving the bottom half as empty cream-colored space.
- **Expected:** This is partially intentional (content-sparse pages), but for states like NoSeason and the dashboard's cancelled state, the content should be vertically distributed more intentionally. The empty states should use the `.empty-state` component with `min-height: 12rem` and centered content to fill the viewport more naturally.

### m15. Prose page h2 heading uses CyGrotesk but body text uses Mont - transition is abrupt
- **Screenshot:** `10.57.55.jpeg`
- **Location:** "Додати учасника" heading followed immediately by "Номер телефону" label
- **Observed:** The h2 "Додати учасника" is CyGrotesk 900 at 1.3rem, followed immediately by "Номер телефону" in Mont 600 at 0.875rem. The dramatic font change (from a dark geometric display face to a lighter geometric body face) with no spacing or visual separator between them creates an abrupt visual transition.
- **Expected:** Per `design-system.md`, h2 has `margin-bottom: var(--density-space-sm)`. The spacing exists but is minimal in admin density (0.375rem). A slightly larger gap or a subtle border/divider between the heading and form would improve readability.

### m16. Badge text contrast issue on confirmed badge
- **Screenshot:** `10.57.55.jpeg`
- **Location:** Badge elements in the participant list (АКТИВНИЙ badges)
- **Observed:** The "АКТИВНИЙ" badges use `data-status="active"` with `background: var(--color-success)` (`oklch(0.58 0.16 160)`) and `color: white`. The green background with white text appears readable but the specific oklch value may need verification for WCAG AA compliance at 0.75rem (12px) text size.
- **Expected:** The `--color-success` at L=0.58 against white text needs contrast ratio verification. At small text sizes (0.75rem), WCAG AA requires 4.5:1. The success green may be borderline.

### m17. Header border-bottom opacity creates inconsistent divider visibility
- **Screenshot:** All mobile screenshots
- **Location:** Header bottom edge
- **Observed:** The `.app-header` border-bottom uses `1px solid oklch(from var(--color-brand-gray) l c h / 0.2)`. This produces a very faint line that is barely visible on the cream background, especially at the JPEG compression level.
- **Expected:** The border is intentionally subtle, but it may be too subtle to serve as a clear section divider between header and content. Consider increasing opacity to 0.3 or 0.4 for better definition.

### m18. Desktop dashboard stepper and stat card layout feels sparse
- **Screenshot:** `11.01.01.jpeg`
- **Location:** Main content area below heading
- **Observed:** On desktop, the stepper and stat card (ФАЗА/СКАСОВАНО) are left-aligned within the `prose-page` container (max-width: 65ch), creating a large empty right portion of the screen. The stat card takes up roughly 1/3 of the prose-page width, leaving substantial whitespace.
- **Expected:** The 65ch max-width is specified in the design system and is correct for prose content. For dashboard-style content with cards and grids, a wider container or centered card layout might be more appropriate, but this is a design decision rather than an implementation fault.

### m19. Login page form elements not constrained to a max-width
- **Screenshot:** `10.59.07.jpeg`, `10.59.17.jpeg`
- **Location:** Phone input and OTP input
- **Observed:** The form inputs span roughly 60% of the viewport width, which looks appropriate on mobile. The parent container uses `items-center text-center` but the input itself does not have an explicit max-width, so on wider viewports it could stretch.
- **Expected:** The `prose-page` container applies `max-width: 65ch` which constrains the form. On mobile this results in full-width inputs which is correct. The centered `text-center` class means labels are centered but the input is left-aligned within its field container, creating a slight visual misalignment between the centered label and the left-anchored input.

### m20. Admin page heading "Управлiння сезоном" line breaks awkwardly on mobile
- **Screenshot:** `10.59.36.jpeg`
- **Location:** h1 heading at top of page
- **Observed:** The heading "Управління сезоном" breaks across two lines as "Управління" / "сезоном" at the mobile viewport width. The clamp formula produces `1.8rem` at 414px viewport. The line break itself is natural, but the two-line heading takes up substantial vertical space.
- **Expected:** This is acceptable behavior for responsive typography. The line break point is natural for the Ukrainian text. No change needed, but noted for completeness.

---

## Cross-Page Consistency Issues

### X1. Header layout differs between admin pages and participant pages
- **Admin pages (10.57.55, 10.58.17, 10.59.36):** Logo left, hamburger right, admin nav hidden on mobile.
- **Participant pages (10.58.44):** Logo left, logout button center, hamburger right.
- **Login pages (10.59.07, 10.59.17):** Logo left, hamburger right, no logout (correct).
- The participant pages show a logout button in the header bar AND a hamburger icon simultaneously, while admin pages only show the hamburger. This creates an inconsistent header pattern across the app.

### X2. Form spacing differs between admin and participant forms
- **Admin forms (10.57.55 participants, 10.59.36 season):** Fields stack vertically with minimal spacing.
- **Participant forms (onboarding, enrollment):** Use `flex flex-col gap-(--density-space-md) sm:flex-row` which provides spacing.
- The admin forms in the participants page have no wrapper flex container with gap between fields, resulting in tighter-than-intended field spacing.

### X3. Button sizing inconsistency across pages
- **Admin pages:** Buttons use default size (0.625rem 1.25rem padding).
- **Login page:** Buttons use default size but appear smaller relative to inputs.
- **Dashboard "Новий сезон":** Uses `data-size="sm"` which is smaller than other CTAs.
- Primary CTAs should be visually consistent across the app. The login flow's primary action should feel as prominent as admin action buttons.

### X4. Empty/placeholder states use different patterns
- **Participant home NoSeason:** Plain `<p>` text, top-left aligned.
- **Admin participants empty:** `.empty-state` component, centered with headline + body.
- Both are "nothing to show" states but they use completely different visual treatments.

### X5. Page title typography is consistent (good)
All pages use `.prose-page h1` with CyGrotesk, which is consistent across admin ("Учасники", "Панель", "Управління сезоном") and login (none, but logo serves as hero). This is working correctly.

---

## Summary Statistics

- **Total faults: 31**
- **Critical: 2** (C1 table overflow, C2 hamburger icon rendering)
- **Major: 8** (M1-M8)
- **Minor: 20** (m1-m20)
- **Cross-page: 5** (X1-X5, 4 issues + 1 positive confirmation)

### Priority Recommendations

1. **Immediate:** Fix C1 (wrap data-table in `.data-table-wrapper` for horizontal scroll) and C2 (add `flex-direction: column` to `.menu-toggle` or restructure the hamburger icon markup).
2. **High:** Fix M1 (form field spacing), M3 (remove duplicate logout in participant header on mobile), M5 (use `.empty-state` for NoSeason).
3. **Medium:** Fix M2 (stepper edge clipping), M4 (add close button to mobile menu), M6 (field group spacing), M7 (login vertical positioning), M8 (single stat card in grid).
4. **Low:** Address minor polish items and cross-page consistency issues.
