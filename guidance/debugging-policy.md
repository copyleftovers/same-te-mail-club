# Debugging & Delegation Policy

## E2E Debugging

**The orchestrator NEVER debugs E2E failures directly.** This is a hard rule.

E2E failures are fiddly, time-consuming, and context-hungry. Every phase has hit them. The pattern is always the same: a small issue (missing wait, wrong selector, form data serialization) that requires reading test output, checking screenshots, reading component code, tweaking, re-running. This burns orchestrator context for zero strategic value.

**When E2E fails:**
1. Delegate to a sonnet agent with: the failure output, the screenshot path, the relevant source files, and the fix instructions.
2. The agent reads `end2end/README.md`, diagnoses, fixes, and re-runs.
3. Orchestrator reviews the result when the agent is done.

**Never:**
- Read E2E screenshots in orchestrator context
- Manually trace through server function logic to debug a test failure
- Run multiple E2E cycles yourself — each run is 2-3 minutes and burns context on waiting

## Long-Running Commands

- ALWAYS redirect to file (`> /tmp/output.log 2>&1`), NEVER pipe through `| tail` or `| head` (pipes buffer and make processes appear hung).
- Use `run_in_background` for anything over 30 seconds.
- Check results by reading the output file with `tail`, not by polling.
