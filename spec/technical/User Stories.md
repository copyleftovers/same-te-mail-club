Stories cover the app only — the offline ritual (creation, shipping, meetup) is outside the app's scope.

## Epic 1: Join the Community

Participants join through invite codes distributed by existing members; participants authenticate to access their account.

### Story 1.1: Join with an invite code

**Outcome:** A person with a valid invite code has an account and can participate in the club.

```
Given a person has a physical invitation with a unique code
When they enter their phone, verify via OTP, provide the code and their legal name
Then an account is created with the verified phone number and they proceed to onboarding
```

**AC:**
- The login page is the universal entry point for both new and returning participants
- After OTP verification, if no account exists for the phone, the system prompts for an invite code
- The OTP-verified phone number becomes the account phone — no re-entry or editing
- Valid code → name input → account created → redirect to onboarding
- Invalid, used, or revoked code → error message, unlimited retry, no rate limit on code attempts
- Codes are single-use: one code creates exactly one account
- The system records which distributor's code was redeemed (referral tracking)
- Legal name is required (matching government ID — needed for Nova Poshta parcel pickup)
- A phone number with a deactivated account cannot re-register — the unique constraint is the enforcement; re-entry requires a different phone number plus a new code
- Codes are word-based: 2-3 Ukrainian words joined by dashes (memorable, typeable from a physical card)
- Codes do not expire — a physical card is valid until used or revoked

### Story 1.2: Sign in with phone number

**Outcome:** The participant is authenticated and can access the app.

```
Given a registered participant
When they verify via SMS code
Then they have an authenticated session
```

**AC:**
- Phone number is the sole authentication method
- Sessions are long-living (weeks/months) to minimize SMS verification costs
- No password, no email, no social login
- Re-authentication only when the session expires or on a new device

### Story 1.3: Complete onboarding

**Outcome:** A first-time participant has provided everything needed to enroll in a season.

```
Given a participant signing in for the first time
When they complete the onboarding flow
Then their Nova Poshta branch is saved and they can enroll in seasons
```

**AC:**
- Onboarding is triggered on first sign-in, before access to the main app
- Collects preferred Nova Poshta branch (required)
- Participant can update their branch later (during season enrollment or in settings)
- Onboarding completes only once — returning participants skip it

### Story 1.5: Generate invite codes

**Outcome:** The organizer has invite codes to distribute, each trackable to a specific distributor.

```
Given the organizer wants to enable someone to invite a new participant
When they select a distributor (existing participant or themselves) and generate a code
Then a unique code is created and linked to that distributor
```

**AC:**
- Code generation lives in the participants section of the admin page (replaces the manual registration form)
- The organizer selects an existing participant or "self" as the distributor before generating
- Each generated code is immediately linked to its distributor — no unassigned codes exist
- The code string is displayed once for the organizer to copy or write onto a physical card
- The app produces the code string only — physical card production is entirely offline
- Generated codes appear in a list showing: code, distributor name, status (unused/used/revoked), redeemer name (if used)

### Story 1.6: Manage invite codes

**Outcome:** The organizer has visibility into the invitation pipeline and can revoke codes that should not be used.

```
Given invite codes have been generated
When the organizer views the code list
Then they see the status of every code and can revoke unused ones
```

**AC:**
- Code list shows: code string, distributor, status (unused / used / revoked), redeemer name and date (if used)
- The organizer can revoke any unused code — the physical card becomes dead
- Used codes cannot be revoked (the account already exists)
- Revoked codes show "revoked" status in the list, not removed
- The list is filterable or scannable — the organizer can find a specific code or see all codes from a specific distributor

### Story 1.4: Sign out

**Outcome:** The session is destroyed and the device is no longer authenticated.

```
Given an authenticated user (participant or admin)
When they click the logout button
Then their session is destroyed server-side and they are redirected to the login page
```

**AC:**
- Logout is available on every authenticated page (both participant and admin navigation)
- The server function deletes the session row from the database — server-side invalidation, not just cookie clearing
- After logout, the user is redirected to the login page
- Logout always succeeds — a corrupt or already-expired session never traps the user
- Both participant and admin share the same logout mechanism

---

## Epic 2: Participate in a Season

The core loop: enroll → confirm ready → receive assignment → confirm receipt.

### Story 2.1: Enroll in an upcoming season

**Outcome:** The participant has opted in for this specific season.

```
Given a season with an open sign-up window
When an eligible participant enrolls
Then they are in the pool for this season's cohort formation
```

**AC:**
- Enrollment is per-season (no auto-enrollment)
- Participant can update their Nova Poshta branch during enrollment
- Enrollment closes at the sign-up deadline
- Participant sees the season timeline (creation deadline, expected meetup window)
- Content guidelines are displayed during enrollment (participant sees them before opting in)
- No explicit withdrawal — a participant who enrolls but cannot participate simply does not confirm ready

### Story 2.2: Confirm mail is ready

**Outcome:** The participant is committed and enters the assignment algorithm.

```
Given an enrolled participant before the ready-confirm deadline
When they confirm their mail is ready to send
Then they are included in the assignment graph
```

**AC:**
- Confirmation is a deliberate action (button/checkbox), not automatic
- The confirm button is no longer shown once the ready-confirm deadline passes
- Unconfirmed participants are excluded from the graph — no penalty, no post-exclusion notification
- Confirmation is irreversible (you can't un-ready yourself after confirming)
- The ready-confirm deadline is clearly visible with countdown

### Story 2.3: Receive assignment

**Outcome:** The participant knows who to send their mail to and how.

```
Given a confirmed participant after the assignment algorithm runs
When they open the app
Then they see their recipient's real name, phone number, and Nova Poshta branch
```

**AC:**
- Assignment is visible in-app: recipient's real name, phone number, and Nova Poshta branch
- The phone number is necessary for Nova Poshta document-tier shipping (recipient gets a pickup SMS)
- An SMS nudge is sent when assignment is available
- The participant sees ONLY their own recipient — no other cohort information
- No information about who is sending TO them
- Assignment details are initially hidden inside an interactive envelope element
- The participant must click, tap, or press Enter/Space to reveal their recipient
- A brief celebratory animation plays on first reveal (suppressed under `prefers-reduced-motion: reduce`)
- The reveal state is persisted per season — each new season delivers a fresh reveal experience
- On subsequent visits within the same season, details are shown immediately without animation
- The envelope is keyboard-accessible (`role="button"`, `tabindex="0"`, `aria-expanded`)

### Story 2.4: Confirm mail received

**Outcome:** The system knows whether the chain is intact at this link.

```
Given a participant 5 days after assignment
When prompted by the app
Then they confirm whether they have received mail
```

**AC:**
- Prompt appears in-app; SMS nudge sent if no response
- Binary response: received / not received
- Optional free-text note: "Anything the organizer should know?" (surfaces edge cases — damaged mail, wrong package, concerns — without complicating the primary flow)
- A "not received" triggers organizer notification for forwarding protocol
- Participants are not asked to describe or evaluate what they received — the note is for the organizer, not a review

---

## Epic 3: Assignment Algorithm

The system generates the Hamiltonian cycle with social-awareness constraints.

### Story 3.1: Generate cohort assignments

**Outcome:** All confirmed participants are split into cohorts, each forming a single loop.

```
Given N confirmed participants (N ≥ 3)
When the organizer triggers assignment generation
Then the system produces cohorts, each a Hamiltonian cycle
```

**AC:**
- The organizer sees the confirmed count and decides whether to proceed or cancel before triggering generation
- Every participant appears in exactly one cohort
- Each cohort is a single cycle (no subgraphs, no disconnected nodes)
- Target cohort size: 11–15. The system supports any N ≥ 3 — the organizer decides whether small cohorts are viable
- If N is not evenly divisible into 11–15, the system finds the best split (e.g., 25 → 13+12, not 15+10)

### Story 3.2: Apply social-awareness constraints

**Outcome:** The algorithm minimizes pairings between people who already know each other.

```
Given a social graph with known-group data entered by the organizer (directly in the database)
When generating the cycle
Then pairings between members of the same known group are minimized
```

**AC:**
- Known groups are weighted: team members > other declared groups
- The constraint is soft — minimize, not forbid
- The algorithm scores candidate cycles and picks the lowest-connection-score option

### Story 3.3: Override assignments

**Outcome:** The organizer can manually adjust before releasing assignments.

```
Given a generated assignment set
When the organizer reviews it
Then they can swap individual sender→recipient pairings while maintaining cycle integrity
```

**AC:**
- Swaps must preserve the single-loop topology (the app validates this)
- The organizer sees the full graph for each cohort
- Assignments become visible to participants when the organizer advances to Delivery phase (advance is the release)

---

## Epic 4: Season Management (Organizer)

The organizer creates and controls each season.

### Story 4.1: Create a new season

**Outcome:** A season exists with a defined timeline, ready for enrollment.

```
Given the organizer decides to run a new season
When they set the sign-up deadline, ready-confirm deadline, and optional theme
Then a season is created and visible to participants
```

**AC:**
- Only the organizer can create seasons
- Required: sign-up deadline, ready-confirm deadline
- Optional: season theme (displayed to participants during enrollment and creation period)
- Season is not open for enrollment until the organizer explicitly launches it

### Story 4.2: Launch a season

**Outcome:** Participants can enroll and the season-open SMS fires.

```
Given a created season
When the organizer launches it
Then enrollment opens and all pool members receive an SMS notification
```

**AC:**
- Launch triggers SMS notification to all registered participants (Story 5.3)
- Enrollment opens immediately upon launch
- The organizer can preview before launching but cannot un-launch

### Story 4.3: Cancel a season

**Outcome:** The system returns to a clean state when proceeding is infeasible.

```
Given an active season in any non-terminal phase
When the organizer cancels the season
Then the season transitions to 'cancelled' and participants see that the season was cancelled
```

**AC:**
- Only the organizer can cancel — admin role required
- Cancellation succeeds from any non-terminal phase (enrollment, preparation, assignment, delivery)
- Terminal phases (complete, cancelled) cannot be cancelled again
- Cancellation requires a confirmation step before submitting (destructive action affecting enrolled participants)
- After cancellation, participants see a distinct "season cancelled" state, not the generic "no active season" message
- No SMS notification on cancellation — participants discover the state change passively
- The cancel button uses destructive styling to signal severity
- Cancel is only available in the UI after launch (unlaunched seasons are recreated, not cancelled)

### Story 4.4: See SMS target counts before sending

**Outcome:** The organizer knows exactly how many people will receive an SMS before triggering the send.

```
Given the organizer is about to send an SMS batch
When they look at the send button
Then they see the target count adjacent to it
```

**AC:**
- Season-open SMS shows "→ N active users" before the button
- Assignment SMS shows "→ N senders (M not yet notified)" before the button
- Confirm nudge shows "→ N unconfirmed enrolled" before the button
- Receipt nudge shows "→ N recipients with no response" before the button
- Counts update after each send (reflecting the new state)

### Story 4.5: Manage everything from one admin page

**Outcome:** The organizer completes any season workflow without navigating between pages.

```
Given the organizer needs to check data and take action
When they open /admin
Then all relevant data and actions for the current phase are on one page
```

**AC:**
- Single page at `/admin` with two sections: season (phase-aware) and participants
- Season section content morphs by phase — enrollment shows enrolled count + advance button, assignment shows cycle visualization + generate, etc.
- Participant registration, listing, and deactivation are always visible below
- When no season exists, the create-season form appears in the season section
- Terminal phases (complete, cancelled) show a summary with "create new season" option
- All existing `data-testid` values preserved — E2E suite passes without POM changes

### Story 4.6: See only relevant SMS actions per phase

**Outcome:** The organizer cannot accidentally trigger an SMS that is irrelevant to the current phase.

```
Given the organizer is in a specific season phase
When they look for SMS actions
Then only the SMS actions relevant to that phase are visible
```

**AC:**
- Enrollment phase: only season-open SMS visible
- Preparation phase: only confirm nudge SMS visible
- Delivery phase: assignment SMS and receipt nudge SMS visible
- Other phases: no SMS actions shown
- Irrelevant SMS buttons are hidden, not disabled — the organizer doesn't see options that don't apply

### Story 4.7: Advance phase only when prerequisites are met

**Outcome:** The organizer cannot accidentally skip a prerequisite step when advancing the season phase.

```
Given the organizer wants to advance to the next phase
When the prerequisite for that transition has not been completed
Then the advance button is disabled with a reason
```

**AC:**
- Assignment → Delivery: advance disabled until assignments are generated
- Other transitions: advance enabled when the phase is reachable (existing server-side validation remains the final guard)

### Story 4.8: Swap assignments by name, not UUID

**Outcome:** The organizer selects participants by name when swapping assignments, instead of typing raw UUIDs.

```
Given the organizer wants to swap two participants' assignments
When they open the swap form
Then they select from dropdowns populated with participant names from the cycle
```

**AC:**
- Swap form uses two `<select>` dropdowns populated from the cycle visualization
- Each option shows the participant name with the UUID as the hidden value
- The swap form is only visible after assignments are generated

---

## Epic 5: SMS Notifications

The app sends carrier SMS at key moments.

### Story 5.1: Send assignment notification

**Outcome:** Participants are nudged to check the app when their recipient is assigned.

```
Given assignments are released
When a participant's assignment is available
Then they receive an SMS directing them to the app
```

**AC:**
- SMS contains no recipient details (privacy) — just a prompt to open the app
- Sent once, not repeated

### Story 5.2: Send receive-confirm nudge

**Outcome:** Participants who haven't confirmed receipt are reminded.

```
Given 5 days have passed since assignment
When a participant has not confirmed receipt
Then they receive an SMS asking them to confirm in the app
```

**AC:**
- Sent only to participants who haven't responded yet
- Single reminder, not repeated spam

### Story 5.3: Send season-open notification

**Outcome:** Pool members know a new season is starting.

```
Given a new season's sign-up window opens
When the organizer launches the season
Then all pool members receive an SMS about enrollment
```

**AC:**
- Sent to all registered participants (with active accounts), not just last season's cohort
- Contains no pressure — informational only

### Story 5.4: Send pre-deadline nudge to non-confirmers

**Outcome:** Enrolled participants who haven't confirmed are reminded before the window closes.

```
Given an enrolled participant who has not yet confirmed their mail is ready
When approximately one hour remains before the ready-confirm deadline
Then they receive an SMS prompting them to confirm before the deadline
```

**AC:**
- Sent only to enrolled participants who have not yet confirmed
- Single message, not repeated
- Timed ~1 hour before the ready-confirm deadline
- No nudge is sent to participants who have already confirmed

---

## Epic 6: Account Management (Organizer)

### Story 6.1: Deactivate a participant's account

**Outcome:** A participant who has repeatedly failed is excluded from future seasons.

```
Given a participant who has failed to send or repeatedly no-showed
When the organizer deactivates their account
Then the participant cannot enroll in future seasons and does not receive season-open SMS
```

**AC:**
- Only the organizer can deactivate accounts
- Deactivated accounts cannot sign in or enroll
- Deactivated participants are excluded from season-open SMS notifications
- Re-entry requires a new invite code and a different phone number — the deactivated phone is permanently burned; the unique constraint enforces this without additional logic

---

## Story Dependencies

```
1.5 (generate codes) → 1.1 (self-register with code) → 1.2 (sign in) → 1.3 (onboarding) → 2.1 (enroll) → 5.4 (pre-deadline SMS) → 2.2 (ready-confirm) → 3.1 (generate)
                                                                                                                               ↓
                                                                                                                         3.2 (social-aware, reads DB directly)
                                                                                                                               ↓
                                                                                                                         3.3 (override)
                                                                                                                               ↓
                                                                                                                         2.3 (receive assignment) → 5.1 (SMS)
                                                                                                                               ↓
                                                                                                                         2.4 (receive-confirm) → 5.2 (SMS)

4.1 (create season) → 4.2 (launch season) → 5.3 (season-open SMS) → 2.1 (enroll)

1.4 (sign out) — available from any authenticated state, no dependency chain
1.6 (manage codes) — depends on 1.5; available at any time after codes exist
4.3 (cancel season) — branches from any non-terminal phase after 4.2; alternative to proceeding
4.4 (dashboard) — depends on 4.1 (season must exist); read-only view
```

## Deferred Stories

- **In-app social graph management:** Organizer edits database directly in season 1; in-app UI for defining/editing known groups deferred until team grows
- **Season status view:** Participant sees where they are in the current season (enrolled, confirmed, assigned, received). Currently implied by which actions are available, but no explicit "here's what's happening" screen
- **Season history view:** Participant sees their past seasons, who they sent to, who sent to them → deferred until multi-season data exists
- **Meetup scheduling/RSVP:** Handled manually in season 1
- **Participant profiles:** No profiles — identity is revealed through mail and meetups, not through an app
- **Automated social graph inference from past pairings:** Explicitly rejected — organizer curates manually
- **In-app messaging between participants:** Not building this — organic connections happen via personal channels
- **Data deletion self-service:** Participant requests deletion by contacting organizer in season 1; self-service deletion deferred
