## Product Identity

A seasonal, offline-first self-expression ritual where participants create something mailable without knowing who will receive it, then send it to an algorithmically assigned recipient, then meet in person to share what they experienced.

**This is NOT:** a pen-pal service, a writing workshop, a Secret Santa, a social network, or a content platform.

**This IS:** a structured community protocol for manufacturing authentic intimacy through blind self-expression, physical mail, and in-person meetups.

## Core Mechanism

The fundamental act: creation without a known audience → assignment → physical delivery → reception → communal sharing.

The separation of creation from recipient knowledge is the load-bearing structural element. It forces self-expression (you can only write about/from yourself) and eliminates audience-optimization (you can't perform for someone you don't know).

## Geography

Kyiv first. The model should be portable to other cities later. Portability means the app, the process, and the ritual format can be replicated — not that a single cohort spans multiple cities.

## Cohort Model

- **Target size:** 11–15 participants per cohort
- **Minimum size:** No hard minimum — the organizer decides whether to run a season for any confirmed count. The system supports cycles of N ≥ 3. At very small sizes (3–5), sender anonymity degrades (process of elimination); the organizer weighs this against cancellation.
- **Membership:** Fluid — the entire participant pool is reshuffled into new cohorts each season
- **Formation:** Algorithm splits the pool into cohorts; organizer can override individual assignments
- **Parallel cohorts:** When the pool exceeds 15, multiple cohorts run simultaneously in the same season
- **Topology:** Each cohort forms a single Hamiltonian cycle (one loop, no subgraphs). Every participant sends to exactly one person and receives from exactly one person. No cliques, no in-groups.

## Entry

In-person invitation through the organizer only. Any existing participant can recommend someone new, but the organizer meets the person and makes the final call on registration. The organizer manually signs them up in the system. After that, the participant can sign in via SMS authentication. No self-registration, no online applications.

## Season Structure

Each season runs approximately 3–4 weeks, end-to-end. The exact timeline flexes around meetup scheduling feasibility.

### Phase sequence:

1. **Sign-up window** (~5 days) — Existing pool members opt in for the upcoming season. Collect/update: preferred Nova Poshta branch. Re-confirmation required every season; no auto-enrollment. (Initial registration — name, phone — is handled by the organizer at invitation time, not during the season window.)
2. **Creation period** (~10–14 days) — Participants create their self-expression. An optional, non-binding theme may be suggested. Medium: anything sendable as a Nova Poshta document-tier parcel.
3. **Ready-confirm gate** (deadline) — Participants confirm in-app that their mail is created and ready to send. This confirmation is irreversible — once confirmed, you're in. Non-confirmers receive an SMS nudge approximately one hour before the deadline. The confirm button is removed once the deadline passes. Only confirmed participants enter the assignment algorithm. Non-confirmers simply don't participate this season — no penalty, no drama.
4. **Assignment** (organizer-triggered) — After the ready-confirm deadline, the organizer reviews the confirmed count and decides whether to proceed or cancel the season. If proceeding, the organizer runs the assignment algorithm. The app generates the Hamiltonian cycle, applying social-awareness constraints. The organizer can adjust individual pairings before releasing assignments to participants. Each sender receives: recipient's real name, phone number, and Nova Poshta branch. Recipients learn nothing about their sender.
5. **Sending window** (~2 days) — Senders ship via Nova Poshta. Same-city next-day delivery is standard. Sender pays standard document-tier shipping (~100 UAH / ~$2–3).
6. **Receive-confirm** (5 days after assignment) — App asks each participant whether they received mail. Binary response (received / not received) plus an optional free-text note ("anything the organizer should know?"). With a 2-day sending window and next-day delivery, this allows ~2 days of buffer. A "no" triggers the forwarding protocol. The free-text note surfaces edge cases (damaged mail, wrong package, concerns) without complicating the primary flow.
7. **Meetup** (within ~1 week of receive-confirm) — In-person gathering at a rented venue, cost covered by organizer.

## Assignment Algorithm

### Social-awareness constraints

The algorithm minimizes pairings between people who already know each other. Social graph data is manually entered by the organizer (or community managers, once the team grows) — no automatic inference from past-season data.

Input categories (weighted by connection strength, strongest first):
- Organizing team members
- Known social groups (entered by organizer)

The constraint is "minimize," not "forbid." In later seasons with a mature pool, some known-person pairings are inevitable. The algorithm degrades gracefully.

### Organizer override

After algorithmic generation, the organizer can manually swap individual assignments before release.

## Failure Protocols

**Foundational constraint:** The organizer has no reach into mail logistics. Participants made unique physical artifacts — no copies exist. The only intervention available is asking people to do things, with no guarantee of execution. Digital-world assumptions (duplication, remote access, forwarding on demand) do not apply.

### Pre-assignment dropout (participant doesn't confirm ready)

Harmless. They simply aren't in the graph. No action needed.

### Post-assignment inability to send

The mail physically exists (confirmed at the gate). If the sender can't get to Nova Poshta, the organizer personally picks up and sends the mail.

### Non-compliance (participant doesn't send, or lied at ready-confirm)

Discovered via receive-confirm. The organizer asks the non-compliant participant to forward the mail they *received* to the person they were supposed to send to. This re-routes the chain around the failure: the downstream recipient gets someone's self-expression (just not the one originally intended).

Forwarding is acceptable because all mail was created without a specific recipient in mind.

If the non-compliant participant refuses to forward: that's the end of it. The downstream recipient receives nothing. The organizer acknowledges this at the meetup. There is no fallback — the physical artifact is a unique object in someone's possession, and the organizer cannot compel its movement.

### Contiguous failures (multiple adjacent non-compliant participants)

The loop is A→B→C→D→E. If B and C both fail: B received A's mail. The organizer asks B to forward A's mail to D (C's intended recipient), skipping the entire failed segment. One re-routing per contiguous block of failures, regardless of block length.

This scales to the degenerate case: if every participant except one fails, that one person receives their own mail back. The chain always heals with a single re-routing per contiguous failure block — as long as the first person in the block cooperates with forwarding.

### Meetup no-show

Not structurally catastrophic — all mail has landed. The no-show's self-expression is still represented through their recipient's circle-share. The no-show's sender misses live feedback. Post-meetup: team shares brief written notes with absent participants.

### Repeat failures

Account deactivation. A participant who fails to send or repeatedly no-shows has their account deactivated by the organizer. To rejoin, they go through the full re-invitation cycle: fresh in-person meeting with the organizer, new account creation. The system enforces the gap — a deactivated account cannot enroll. Same trust mechanism as initial entry, but the bar is implicitly higher because both parties know why the previous account was deactivated.

## Meetup Format

### Venue
Rented space. Cost covered by organizer (Добробіт budget). No cost to participants.

### Structure (three phases, in order):

**1. Circle share.** Each participant shares one specific thing about the mail they received — a detail, a feeling, a surprise. The organizer goes first to model the depth and tone expected: specific and genuine, not evaluative or performative. Fully open attribution — the sender is named. Bringing the physical mail is the recipient's choice. By the end, every creation has been spoken about once.

**Facilitation guidance:** The organizer prompts with: "Share one specific thing — a detail that struck you, a feeling it gave you, something that surprised you." If a participant gives a one-word answer, the organizer can gently follow up: "What specifically about it?" The goal is not to force eloquence but to ensure each sender hears something concrete about how their work landed.

**2. Pair time.** Unstructured but prompted: "Find your sender and your recipient, talk." Happens after the circle because the ice is already broken — you've heard your recipient speak about your work publicly.

**3. Open mingling.** No structure. Organic connections form outside the graph. This is the community-building phase.

### The organizer attends every cohort's meetup.

This means parallel cohort count is bounded by organizer capacity (~2–3 per season at monthly cadence).

## Privacy Model

- **Sender knows:** recipient's real name, phone number, and Nova Poshta branch. The phone number is required for Nova Poshta document-tier shipping (recipient receives a pickup SMS). Sharing one's phone number is part of the trust contract of participation.
- **Recipient knows before meetup:** only what's physically on/in the mail itself
- **Participants know before meetup:** only their own recipient — not who else is in the cohort, not who sent to them
- **Full graph revealed:** at the meetup, through the circle-share process

## Creative Constraints

- **Medium:** Anything sendable as a Nova Poshta document-tier parcel. Letters, drawings, collages, pressed flowers, photographs, handwritten journals — sender's choice.
- **Theme:** Optional, suggested per season, not enforced. Scaffold for newcomers; experienced participants can ignore it.
- **NOT allowed:** Purely digital mail (email, PDF, etc.). The medium is physical. A mixed-media artifact that arrives as a physical object but includes a digital component (e.g., a letter with a QR code to a video) is acceptable — the physical form is primary, digital is supplementary.

## Content Guidelines

Displayed to participants during enrollment each season. Not a legal document — a social contract.

The mail should be an act of self-expression: something you made that comes from you. Not a message targeted at a specific person (you don't know them), not something illegal, threatening, or deliberately hurtful. The structure removes the audience — what's left should be honest, not hostile.

Violations are handled by the organizer case-by-case: conversation with the participant, potential account deactivation. There is no pre-screening — the organizer never sees the mail before it's sent. This is a post-incident mechanism, not a gatekeeping one.

## Ownership & Consent

- The recipient keeps the physical mail permanently.
- Public sharing of anyone's mail is prohibited.
- Photographing or digitizing received mail without the sender's explicit consent is prohibited.
- Group chats between participants are fine — organic connections are welcome.

## Platform

Simple web app. No Telegram integration, no Telegram bots, no Telegram mini-apps — not now, not later. SMS remains the out-of-app communication channel for authentication and notifications.

## App Scope (Season 1)

The app handles:
- Registration (organizer creates account for new participant)
- Authentication (SMS-based, long-living sessions to minimize SMS costs)
- Data collection (name, phone, Nova Poshta branch)
- Content guidelines display (shown during enrollment)
- Season creation and launch (organizer sets timeline, optional theme)
- Season theme display (shown during enrollment and creation period)
- Season enrollment (re-confirm opt-in)
- Ready-confirm gate (in-app button)
- Assignment algorithm (Hamiltonian cycle, social-aware, with override)
- Assignment delivery (in-app: recipient name, phone, Nova Poshta branch)
- SMS nudges (actual carrier SMS)
- Receive-confirm (in-app, with optional free-text note)
- Account deactivation (organizer action for soft exclusion)

The app does NOT handle (season 1):
- Social graph management (organizer edits database directly)
- Meetup scheduling or RSVP
- Content creation or submission
- Anything mail-related (all physical, all manual)

## Organizer Role

- Solo operator in season 1 (single point of failure — see Known Risks)
- Participates in the mail loop when logistics permit, skips when overwhelmed
- Attends every cohort's meetup
- Facilitates the circle share (goes first, models tone, prompts depth)
- Manually manages social graph data
- Can override algorithmic assignments
- Decides whether to proceed or cancel after seeing confirmed count
- Personally intervenes for shipping failures
- Deactivates accounts for repeat failures
- Covers venue and SMS costs from Добробіт budget (uncapped at current scale; revisit if pool exceeds ~50)

## Success Criteria (MVP)

One cohort completes a single season:
- Sign-up → creation → ready-confirm → assignment → send → receive → meetup
- All participants receive mail (with or without forwarding protocol)
- Meetup happens with structured circle-share and facilitated sharing

## Excluded by Design

- **Fees and monetization.** This is not a revenue-generating product. No participation fees, no subscriptions, no sponsorship-driven features.
- **Purely digital mail formats.** The medium is physical. This is a structural commitment, not a temporary limitation.

## Data Policy

- Participant data (name, phone, Nova Poshta branch, social graph connections) is retained until the participant requests deletion
- Deletion requests are handled by the organizer manually (removes participant from the system)
- No automated retention or purge policy in season 1
- Assignment history is retained for social-awareness algorithm (past pairings inform future avoidance)

## Known Risks (Season 1)

- **Single point of failure.** The organizer is the sole operator for all community, logistical, and technical functions. If the organizer is unavailable during a season, the season pauses or fails. Accepted risk for season 1; revisit after.
- **Forwarding protocol reliability.** The forwarding protocol for non-compliance relies on the least reliable person in the chain (someone who already failed to comply). Refusal to forward is the likely outcome, not the edge case. The spec acknowledges this — "that's the end of it" — but the downstream participant receives nothing. This is a known cost of the physical medium.
- **Community homogeneity.** Invitation-only entry through existing participants naturally converges on a demographic/psychographic type. Not a season 1 problem at ~15 people (you start with one network), but a dynamic to watch as the pool grows. Revisit at 50+ pool.

## Deferred

- Multi-city expansion
- Trusted meetup hosts (needed when parallel cohorts exceed organizer capacity)
- Automated social graph inference
- Season history / participant profiles in-app
- Formal data retention automation (automated purge after inactivity)
- Community diversity mechanisms (organizer-initiated invitations outside existing network)
- Whether season 1 runs fully manual (spreadsheet + personal SMS) or with the app — decision pending
