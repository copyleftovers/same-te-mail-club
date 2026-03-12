Stories cover the app only — the offline ritual (creation, shipping, meetup) is outside the app's scope.

## Epic 1: Join the Community

The organizer registers new participants; participants authenticate to access their account.

### Story 1.1: Organizer registers a new participant

**Outcome:** A new person exists in the system, ready to sign in.

```
Given the organizer has met someone in person and invited them
When the organizer enters their phone number and real name into the system
Then an account is created and the participant can sign in
```

**AC:**
- Only the organizer can create new accounts
- Minimum data at registration: phone number + real name (legal name matching government ID — required for Nova Poshta parcel pickup)

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
- Assignments are not released to participants until the organizer confirms

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
- Re-activation requires the organizer to create a new account (full re-invitation cycle)

---

## Story Dependencies

```
1.1 (register) → 1.2 (sign in) → 1.3 (onboarding) → 2.1 (enroll) → 5.4 (pre-deadline SMS) → 2.2 (ready-confirm) → 3.1 (generate)
                                                                                                                               ↓
                                                                                                                         3.2 (social-aware, reads DB directly)
                                                                                                                               ↓
                                                                                                                         3.3 (override)
                                                                                                                               ↓
                                                                                                                         2.3 (receive assignment) → 5.1 (SMS)
                                                                                                                               ↓
                                                                                                                         2.4 (receive-confirm) → 5.2 (SMS)

4.1 (create season) → 4.2 (launch season) → 5.3 (season-open SMS) → 2.1 (enroll)
```

## Deferred Stories

- **In-app social graph management:** Organizer edits database directly in season 1; in-app UI for defining/editing known groups deferred until team grows
- **Season status view:** Participant sees where they are in the current season (enrolled, confirmed, assigned, received). Currently implied by which actions are available, but no explicit "here's what's happening" screen
- **Organizer dashboard:** Season health at a glance — enrolled count, confirmed count, received count, outstanding actions. Organizer queries the database directly in season 1
- **Season history view:** Participant sees their past seasons, who they sent to, who sent to them → deferred until multi-season data exists
- **Meetup scheduling/RSVP:** Handled manually in season 1
- **Participant profiles:** No profiles — identity is revealed through mail and meetups, not through an app
- **Automated social graph inference from past pairings:** Explicitly rejected — organizer curates manually
- **In-app messaging between participants:** Not building this — organic connections happen via personal channels
- **Data deletion self-service:** Participant requests deletion by contacting organizer in season 1; self-service deletion deferred
