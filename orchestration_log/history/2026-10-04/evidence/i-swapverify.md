# i-swapverify — admin swap_assignment verification

Binding: stop-yapping (I must not pad; I will emit tables only), first-principles (I must derive from source; I will model the math), kiss (I must propose the simplest fix), dry (I must not duplicate validation; I will reuse validate_cycles).

## Verdict
**BLOCKER: swap (Story 3.3) can never succeed. Every call either errors at the DB (UNIQUE) or, if that were bypassed, always fails topology validation (swap of two recipients in a single cycle always splits it).** E2E does not detect it.

## Q table
| # | Answer | Evidence |
|---|---|---|
| 1a mutates | `assignments.recipient_id` on exactly 2 rows (sa, sb) of season sid | src/admin/assignments.rs:366-384 |
| 1b transaction | NO. Three independent `&pool` statements; no `begin()` | assignments.rs:366,376,387 |
| 1c validation order | AFTER both writes | write 366-384, validate 406 |
| 1d rollback on validation fail | NO rollback (nothing to roll back with). Failed validation leaves split cycles persisted | assignments.rs:406 `?` returns Err, no compensating write |
| 1e actual first failure | First UPDATE sets sa.recipient_id = rec_b, but sb still holds rec_b; `UNIQUE (season_id, recipient_id)` is non-deferrable (no DEFERRABLE in migrations) -> unique_violation -> `db_err` -> Err. Nothing written (single statement atomic). | migrations/20260314000002_create_tables.sql:56-67 ; assignments.rs:366-374 ; `grep -ri DEFERRABLE migrations` = empty |
| 2 math | See below: always fails (never passes) for sa != sb | |
| 3 re-validate before release | NO. `validate_cycles`/`validate_swap_topology` called only at assignments.rs:301 (generate) and :406 (swap). advance_season has no assignment/validate/cycle reference | `grep -rn validate_cycles src` ; grep of advance_season body empty |
| 4 E2E 3.3 asserts effect | NO. Only `cycle-visualization` visible, which renders regardless of swap outcome; POM waits for POST response (any status) and the viz. Error banner never asserted. | mail_club.spec.ts:649-654 ; fixtures/mail_club_page.ts:483-498 |

## Domain validity rule (src/assignment.rs:316-358)
Per cohort: >=3 participants; list is ordered, edge i -> i+1 mod n; no duplicate sender in a cohort; each sender and each receiver globally once. Multiple cohorts allowed, but each cohort is ONE loop. Story 3.3 AC: "Swaps must preserve the single-loop topology" (User Stories.md:238-239). Note validate_swap_topology wraps ALL assignments as one cohort (assignments.rs:212-217), so even a legit multi-cohort season (>11 participants, split_cohorts) can never pass: walk from first stays inside its own cohort, repeats nodes -> duplicate error. (Same bug class; swap on >=12-node seasons fails even for a no-op a==b.)

## Math
Cycle a1->a2->...->an, swap recipients of x=ai and y=aj (i<j): x now -> a(j+1), y now -> a(i+1). Result = two cycles: (a(i+1)..aj -> a(i+1)) is closed by y, and (a1..ai, a(j+1)..an) by x. Always 2 cycles, for any x != y. So never a single loop. Degenerate: adjacent (j=i+1): y -> a(i+1)=y self-assignment.
- 4-node a->b->c->d->a, swap a,c: a->d, c->b; b->c, d->a. Cycles: a->d->a, b->c->b. Two 2-cycles. Fails (walk from a: [a,d,a,d] duplicate).
- 5-node a->b->c->d->e->a, swap a,c: a->d, c->b. Cycles: a->d->e->a (3), b->c->b (2). Fails.
- 3-node a->b->c->a, swap a,b: a->c, b->b self-loop. Fails.
- a==b (UI allows: both selects list all senders; server has no a!=b check): no-op, passes. Only "successful" call is a useless one.

## Hidden-bug note (secondary)
If UNIQUE were removed/deferred, failing swaps would leave the DB in the split-cycle state (no tx), and get_assignment_preview (assignments.rs:462-481) would render only the chain from the first sender, silently hiding the other cycle(s); advance_season would then release it.

## Unit test (report only; not in repo)
Against real `validate_cycles`, reproducing the swap's resulting topology as validate_swap_topology builds it (walk n steps from first):
```rust
#[test]
fn swapping_recipients_in_single_cycle_is_never_a_valid_single_loop() {
    use std::collections::HashMap;
    let ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect(); // a b c d e
    let mut next: HashMap<Uuid, Uuid> =
        (0..5).map(|i| (ids[i], ids[(i + 1) % 5])).collect();
    let (x, y) = (ids[0], ids[2]);
    let (rx, ry) = (next[&x], next[&y]);
    next.insert(x, ry);
    next.insert(y, rx); // what swap_assignment does
    // same walk as validate_swap_topology (assignments.rs:190-202)
    let mut cur = ids[0];
    let mut ordered = vec![];
    for _ in 0..5 { ordered.push(cur); cur = next[&cur]; }
    let r = AssignmentResult { cohorts: vec![Cycle { participants: ordered, score: 0 }] };
    // EXPECTED by Story 3.3 (a valid swap exists): Ok(())
    // ACTUAL: Err("cohort 0: duplicate participant ...") -- walk is a,d,e,a,d
    assert!(validate_cycles(&r).is_ok());
}
```
Expected Ok, actual Err (test fails => demonstrates defect). Parametrize over all x!=y, n in 3..=11: all Err. (Not executed; derived from source.)

## Minimal correct-by-construction fix (1 paragraph)
Define a swap that preserves a single loop by construction: operate on the ordered cycle, not on recipients. Swap the POSITIONS of the two senders in the cycle order (exchange x and y in the participants Vec), which keeps one loop of the same members, then rewrite the cohort's rows from the new order inside one transaction (`pool.begin()`; delete the cohort's rows or update with recipient_id temporarily NULL-free via a single `UPDATE ... FROM (VALUES ...)` so UNIQUE is checked per statement after all rows change — Postgres checks non-deferrable UNIQUE per row, so prefer DELETE + INSERT of the cohort in the tx, or make the constraint DEFERRABLE INITIALLY DEFERRED), call `validate_cycles` on the new order BEFORE commit and rollback on Err, reject sender_a == sender_b, and keep per-cohort grouping (derive cohorts from the existing rows by walking, rather than wrapping everything as one cohort). Add the unit test above (inverted: position-swap result is Ok) and make E2E 3.3 assert the effect (swapped recipient name changes in `cycle-visualization`, no `action-error`).
