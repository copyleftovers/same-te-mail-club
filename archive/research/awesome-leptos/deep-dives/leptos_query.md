# leptos_query — Deep Technical Verification

## Crate Metadata

| Metric | Value |
|--------|-------|
| **Current Version** | 0.5.3 (released 2024-03-09) |
| **Total Downloads** | 47,487 |
| **Recent Downloads (weekly)** | ~11,400 |
| **Leptos Dependency** | `^0.6` (requires Leptos 0.6.x or later) |
| **License** | MIT |
| **Repository** | https://github.com/nicoburniske/leptos_query |
| **Crate Size** | ~28KB |
| **Code Lines** | ~2,718 (18 files) |
| **Last Publish** | 2024-03-09 (12 months old relative to 2026) |

---

## Leptos 0.8 Compatibility

**INCOMPATIBLE**

The crate requires `leptos = "^0.6"` (0.6.x and up), but the target project runs **Leptos 0.8**, which is a major version upgrade with breaking API changes. While semver suggests ^0.6 should allow 0.8, Leptos has made significant architectural changes between 0.6 and 0.8:

- Leptos 0.8 moved from a `cx` (context parameter) model to a context-free, implicit API using thread-local contexts.
- Reactive primitives (`signal()`, `effect()`) changed in Leptos 0.8.
- The Resource API itself evolved slightly.
- leptos_query 0.5.3 was built against 0.6 APIs and is not tested against 0.8.

**Result:** Using leptos_query 0.5.3 with Leptos 0.8 would likely compile due to the semantic versioning of Leptos itself (0.8 is technically a breaking change, but the crate may not reject it at compile time), but runtime behavior is unverified and risky.

**Note:** Version compatibility table in crate README explicitly maps Leptos 0.6 → leptos_query 0.5.x. There is no mention of Leptos 0.7 or 0.8 support in the published README or release notes.

---

## What It Provides Over Native Leptos Resource

leptos_query is a **TanStack Query (React Query) port** that wraps and extends Leptos's native Resource primitive with:

### 1. **Query Client & Global Caching**
- Centralized query cache at the app root (via `provide_query_client()`)
- Shared cache key deduplication across components
- Multiple components fetching the same key will reuse the same cached result

### 2. **Request Deduplication**
- When multiple components request the same query key simultaneously, only one fetch executes
- Subsequent requests get the same promise/result

### 3. **Stale-While-Revalidate (SWR)**
- Background refetching strategy: serve stale data immediately, refetch in background
- Configurable stale intervals
- Automatic cache invalidation

### 4. **Advanced Cache Lifecycle Management**
- Cache time (how long data is considered fresh)
- Stale time (when background refetch should trigger)
- GC max age (when to evict from memory)
- Fine-grained invalidation by key prefix or partial match

### 5. **Query Invalidation API**
- Manual invalidation: `query_client.invalidate_queries(key_pattern)`
- Cascading refetches after mutations

### 6. **Optimistic Updates**
- Rollback on error
- Predictable UI feedback

### 7. **Persistence (Optional)**
- Built-in localStorage, IndexDB, or custom persistence layer support
- Feature-gated: `local_storage`, `indexed_db`

### 8. **Devtools Integration**
- Browser devtools panel to inspect cache state, refetch history, and timings
- Feature: `leptos_query_devtools`

### 9. **Async Error Handling**
- Built-in error state management per query
- Retry logic (exponential backoff, configurable)

**Native Leptos Resource does NOT provide:**
- Global query cache or deduplication
- Stale-while-revalidate
- Query invalidation API (must refetch via `action.version()`)
- Persistence
- Devtools
- Automatic retry

---

## SSR + Hydration

**UNVERIFIED (likely INCOMPATIBLE)**

leptos_query claims SSR support via `hydrate` feature flag. However:

1. **Feature structure exists:** The crate has `ssr` and `hydrate` feature flags that enable necessary dependencies (`tokio` for server, `js-sys`/`web-sys` for client).

2. **Current project does NOT use leptos_query:** So no hydration behavior has been tested in this codebase.

3. **Leptos 0.8 mismatch:** Because leptos_query targets Leptos 0.6, the SSR/hydration integration was built for 0.6's context model. Leptos 0.8's implicit context API may not play well with leptos_query's `provide_query_client()` pattern.

4. **Unverified timing:** The crate's last update was 2024-03. Since then, neither Leptos (0.8 release was late 2024/early 2025) nor leptos_query have been co-tested.

**Verdict:** SSR + hydration with Leptos 0.8 is technically unproven. It may work if leptos_query's internal context calls are compatible with 0.8's implicit model, but there is zero official validation.

---

## Current Project Data Fetching

### Frequency & Pattern

The project uses **7 `Resource::new()` calls** across the codebase, all following the same idiomatic Leptos 0.8 pattern:

| Component | Type | Refetch Trigger | Pain Point |
|-----------|------|---|---|
| `app.rs` — `get_current_user()` | App-level auth | Once at load | None — global context, no refetch needed |
| `dashboard.rs` — `get_dashboard()` | Admin view | Manual (page nav) | None — admin page, infrequent access |
| `participants.rs` — `list_participants()` | Admin list | Via `register_participant` action | **Moderate** — must manually wire `action.version()` to refetch |
| `participants.rs` — `deactivate_participant()` | Mutation | Part of same list | Same as above |
| `assignments.rs` — `get_confirmed_count()` | Admin stat | Via `generateAssignments` action | **Moderate** — tight coupling between action and resource |
| `assignments.rs` — `preview` (algorithm preview) | Computation | Via mutation input | **Moderate** — preview invalidation manual |
| `home.rs` — `get_home_state()` | Participant view | Via 3 actions (enroll, confirm, receipt) | **High** — tuple of `action.version()` calls to refetch |

### Current Pain Points

1. **Manual Refetch Orchestration**
   - When an ActionForm mutation succeeds, components must manually wire `action.version().get()` as the Resource source
   - Three separate actions trigger the same Resource in `home.rs` — requires a 3-tuple source closure
   - No automatic invalidation: if a mutation succeeds but the Resource refetch is not explicitly wired, stale data persists

2. **No Request Deduplication**
   - If two admin pages both call `list_participants()` on the same session, two separate requests hit the server (no cache)

3. **No Cache Lifecycle**
   - Resources are torn down and recreated on component unmount/remount
   - No persistent cache across navigations

4. **Verbose Refetch Wiring**
   ```rust
   let resource = Resource::new(
       move || (action1.version().get(), action2.version().get(), action3.version().get()),
       |_| fetch_data()
   );
   ```
   This is explicit but noisy.

### Data Fetching Snapshot

All 7 Resources are in **participant-facing or admin-facing components**, not core business logic. The project has:
- **0 complex query dependencies** (no cascading fetches)
- **0 cache reuse patterns** (each component owns its Resource)
- **No polling or background refetch** (all refetches are action-triggered)
- **No error retry logic** (errors bubble up to Suspense error boundary)

---

## Would It Help This Project?

**SHORT ANSWER: No — it would add complexity without solving real problems.**

### Why Not

1. **Leptos 0.8 Incompatibility is Disqualifying**
   - leptos_query 0.5.3 targets Leptos 0.6
   - No guarantee it works with Leptos 0.8's context API
   - Porting leptos_query to 0.8 would require understanding its internals (it's 2,718 lines of Rust)
   - Risk/reward is upside-down: adopt a library that's already one major version behind

2. **Project Data Fetching is Already Idiomatic**
   - All 7 Resources are simple, non-overlapping queries
   - No deduplication pain: each Resource is called from a single component
   - Refetch orchestration is explicit, not hidden — developers can see the trigger
   - This is **exactly** what native Leptos Resource is designed for

3. **No Request Deduplication Benefit**
   - The project has no scenario where two components simultaneously fetch the same data
   - Even if they did, the overhead is negligible: 7 total requests across the app lifetime

4. **No Cache Reuse Benefit**
   - No component fetches data, unmounts, then remounts and refetches
   - No deep navigation paths where caching would save round-trips
   - The app is small (6 pages, mostly sequential flows)

5. **Refetch Orchestration is Not a Problem**
   - `action.version()` as a Resource source is the official Leptos pattern
   - The 3-action tuple in `home.rs` is verbose but readable and debuggable
   - leptos_query's `invalidate_queries()` would trade explicitness for brevity (not worth it at this scale)

6. **No Persistence Need**
   - The project doesn't need offline-first behavior
   - No localStorage/IndexDB use case

7. **No Devtools Value**
   - 7 Resources across 6 pages is not enough complexity to benefit from query devtools
   - Console logging `resource.get()` is sufficient for debugging

### Hypothetical Scenarios Where It *Would* Help

- **50+ Resources** across the app with significant overlap → deduplication saves bandwidth
- **Deep cascading fetches** (fetch A, then B depends on A's result) → invalidation API reduces boilerplate
- **Offline-first mobile app** → persistence + stale-while-revalidate is critical
- **High-concurrency polling** (real-time dashboards) → background refetch scheduling
- **Leptos 0.8 officially supported version** → zero compatibility risk

**This project has none of these.**

---

## Integration Effort

If the project *did* want to use leptos_query (hypothetically, ignoring compatibility):

1. **Add Cargo dependency** (1 file)
   ```toml
   leptos_query = { version = "0.5", features = ["hydrate", "ssr"] }
   ```

2. **Wrap App in QueryClientProvider** (1 file: `app.rs`)
   ```rust
   let query_client = create_query_client();
   provide_query_client(query_client);
   ```

3. **Migrate each Resource to create_query** (7 files)
   - Replace `Resource::new()` with `create_query()`
   - Update refetch wiring: `action.version()` → `query_client.invalidate_queries()`

4. **Add types** (1 file)
   - Define query key types and response types per query

5. **Update tests** (maybe +2 files)
   - Hydration tests would need adjustment (if they exist)

**Total:** ~11 files touched, ~200-300 lines of code changes.

**Time estimate:** 2-4 hours for a developer familiar with leptos_query.

**BUT:** This assumes Leptos 0.8 compatibility, which is unverified. First, someone would need to:
1. Test leptos_query 0.5.3 against Leptos 0.8
2. Port/fork leptos_query to 0.8 if incompatible (~6-10 hours for a Leptos expert)

**Total realistic effort:** 8-14 hours.

---

## Verdict

**SKIP**

leptos_query is a well-maintained library (47.5K downloads, single maintainer, active for 2+ years) for a specific use case: **large, cache-heavy Leptos apps with query overlap and persistence needs.**

This project is:
- Small (6 pages)
- Cache-simple (7 non-overlapping Resources)
- Query-light (no deduplication benefit, no persistence)
- On an incompatible Leptos version (0.8 vs 0.6 target)

**Action:**
- Keep using native Leptos Resource + ActionForm (`action.version()` pattern)
- If the codebase grows to 30+ Resources with significant refetch orchestration, revisit
- If Leptos 0.8 support is officially added to leptos_query (or a fork emerges), re-evaluate
- Current status: no lock-in, no migration needed

---

## Bonus: Native Leptos Resource Pattern This Project Already Follows

This project is already using the **exact pattern Leptos authors recommend** for small/medium apps:

```rust
// ✓ Good: explicit, composable, native
let resource = Resource::new(
    move || action.version().get(),  // source: refetch on action
    |_| fetch_data()  // fetcher: untracked
);

view! {
    <Suspense fallback=|| "Loading...">
        {move || resource.get().map(|result| /* render */ )}
    </Suspense>
}
```

This pattern:
1. Is guaranteed to work with Leptos 0.8
2. Requires zero external dependencies
3. Is easy to trace (no hidden cache)
4. Compiles to minimal JavaScript

**Recommendation:** Stay the course. The project's data fetching architecture is sound.

---

## Reference

- **Crate:** https://crates.io/crates/leptos_query
- **Repository:** https://github.com/nicoburniske/leptos_query
- **Release history:** First published 2023-07-19 (v0.1.0), last update 2024-03-09 (v0.5.3)
- **Leptos docs:** https://book.leptos.dev/server/25_server_functions.html (Resource section)
- **TanStack Query (inspiration):** https://tanstack.com/query/latest
