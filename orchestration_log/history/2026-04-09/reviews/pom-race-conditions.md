# POM Navigation Race Conditions — Investigation Report

Date: 2026-04-07 to 2026-04-08

## The Question
Why does test 13 ("4.1 — second active season is rejected") consistently timeout on `page.goto("/admin/season")`?

## Root Cause
`login()` returned before the 302 redirect completed. `page.goto()` fired mid-redirect, cancelling the in-flight navigation. The new SSR response arrived partially (header rendered, Suspense never resolved) because the server was still streaming the cancelled response.

## The Race Pattern
```
OTP verify POST → 302 redirect → browser navigates to /admin
                                    ↑ login() returns here (URL changed)
                                    → page.goto("/admin/season") cancels in-flight redirect
                                    → new SSR request competes with cancelled response on same connection
```

## Fix
`waitForLoadState("domcontentloaded")` after every POM method that triggers a 302 redirect. This ensures the HTML response is fully committed before the next navigation.

## All affected methods
| Method | Trigger | Fix commit |
|--------|---------|------------|
| `login()` | OTP verify → 302 | d89757e |
| `goHome()` | Redundant `goto("/")` | 78cc221 |
| `logout()` | Logout → 302 to / → /login | 49b56d6 |
| `completeOnboarding()` | Onboarding submit → 302 | 49b56d6 |
| `goToDashboard()` | Bare goto, no wait | 49b56d6 |
| `advanceSeason()` | No DOM wait after POST | 49b56d6 |
| `deactivateParticipant()` | No hydration wait after goto | 6f979fc |

## Why `domcontentloaded` not `load`
`load` waits for ALL subresources including the 14MB WASM bundle. `domcontentloaded` waits only for HTML parsing. We only need the HTML committed to prevent the race — not the WASM downloaded.
