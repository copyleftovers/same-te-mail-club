# Leptos 0.8 Idioms — Agent Reference

Authoritative patterns for this project. Derived from Leptos MCP documentation and battle-tested in Phase 2 E2E.

## Forms: ActionForm, Not Signal-Driven

**MANDATORY for all server function forms.**

`ActionForm` reads `FormData` directly from the DOM at submit time via `name` attributes. No reactive signals needed for form input values.

```rust
#[server(CreateThing)]
pub async fn create_thing(title: String, count: i32) -> Result<(), ServerFnError> {
    // title and count deserialized from FormData automatically
    Ok(())
}

#[component]
fn CreateForm() -> impl IntoView {
    let action = ServerAction::<CreateThing>::new();
    let pending = action.pending();

    view! {
        <ActionForm action=action>
            <input type="text" name="title" />       // name matches server fn param
            <input type="number" name="count" />      // name matches server fn param
            <button type="submit" disabled=pending>
                {move || if pending.get() { "Saving..." } else { "Save" }}
            </button>
        </ActionForm>
    }
}
```

### Why Not `on:input` + Signals?

Playwright's `.fill()` does not reliably fire Leptos `on:input` event handlers on hydrated elements. Signals end up empty at submit time → server gets empty strings → silent failure.

`ActionForm` reads the DOM directly. Works with Playwright. Works without WASM (progressive enhancement). Is the idiomatic Leptos pattern.

### BANNED Pattern

```rust
// BANNED: signal-driven form input for server function submission
let (value, set_value) = signal(String::new());
view! {
    <form on:submit=move |ev| {
        ev.prevent_default();
        action.dispatch(MyServerFn { field: value.get() }); // value may be empty
    }>
        <input on:input=move |ev| set_value.set(event_target_value(&ev)) />
    </form>
}
```

## Data Loading: Resource

Use `Resource` for reads. Separate source (tracked) and fetcher (untracked) for proper SSR hydration.

```rust
let data = Resource::new(
    move || version.get(),           // source — tracked, triggers refetch
    |_version| fetch_data(),         // fetcher — untracked
);

view! {
    <Suspense fallback=|| "Loading...">
        {move || data.get().map(|result| match result {
            Ok(items) => view! { /* render */ }.into_any(),
            Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
        })}
    </Suspense>
}
```

## Refetching After Mutation

Use `action.version()` as the Resource source to auto-refetch after an ActionForm submission.

```rust
let action = ServerAction::<CreateThing>::new();
let things = Resource::new(
    move || action.version().get(),  // refetches when action completes
    |_| list_things(),
);
```

## Server Functions

```rust
#[server(MyFunction)]
pub async fn my_function(param: String) -> Result<ReturnType, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();  // preferred over extractors
    // ...
}
```

- Always return `Result<T, ServerFnError>`
- Use `expect_context::<T>()` for server-side context (pool, config)
- Use `http::request::Parts` via `leptos::context::use_context` for request data (cookies)
- Return types must be `Serialize + Deserialize + Clone`

## Routing

Nested paths use tuple syntax, NOT slash-separated strings:

```rust
// CORRECT
<Route path=(StaticSegment("admin"), StaticSegment("season")) view=SeasonPage/>

// WRONG — does not match in Leptos 0.8
<Route path=StaticSegment("admin/season") view=SeasonPage/>
```

## Controlled Inputs (non-form use cases)

Only use signals for inputs when you need live reactivity (e.g., search-as-you-type, derived display). NOT for form submission.

```rust
let (query, set_query) = signal(String::new());
view! {
    <input
        type="text"
        prop:value=move || query.get()       // prop:value, not value
        on:input=move |ev| set_query.set(event_target_value(&ev))
    />
    <p>"Searching: " {move || query.get()}</p>  // live preview
}
```

## Error Display from Actions

```rust
let action = ServerAction::<MyFn>::new();
view! {
    {move || action.value().get().and_then(|r| r.err()).map(|e| {
        view! { <p class="error">{e.to_string()}</p> }
    })}
}
```

## Leptos MCP — Authoritative Documentation & Autofixer

**This is your primary Leptos reference.** This idioms file covers project-specific patterns. For anything beyond — API signatures, reactivity model, component lifecycle, error handling, routing internals — use the MCP tools. They contain distilled, version-correct Leptos 0.8 documentation. **Do not guess at Leptos APIs. Query MCP first.**

### Tools

| Tool | Purpose | When to use |
|------|---------|-------------|
| `mcp__plugin_leptos-mcp_leptos__list-sections` | Index of all 15 doc sections | Once at session start — know what's available |
| `mcp__plugin_leptos-mcp_leptos__get-documentation` | Full docs for section(s), comma-separated | Before writing any Leptos code you're unsure about |
| `mcp__plugin_leptos-mcp_leptos__leptos-autofixer` | Analyze Leptos code for issues | After writing a component — paste it in, get fixes |

### Section Index (query by name)

| Section | Covers | Use when... |
|---------|--------|-------------|
| `mental-model` | Components are setup fns, signals update DOM directly, no VDOM | **Read FIRST before writing any component** |
| `components` | Props, children, lifecycle, re-rendering model | Defining new components |
| `forms-and-actions` | ActionForm, controlled inputs, progressive enhancement | Any form or mutation work |
| `server-functions` | `#[server]`, dual compilation, DTOs, context access | Writing or debugging server fns |
| `resources` | Resource, Suspense, async data loading, refetch | Data fetching patterns |
| `signals` | signal(), reactive primitives, fine-grained updates | State management |
| `derived-state` | Closures vs Memo vs Effect for computed values | Computing values from signals |
| `control-flow` | `<Show/>`, `<For/>`, Either, `.into_any()` | Conditional/list rendering |
| `views` | view! macro, reactive attributes, event handlers | Building UI elements |
| `routing` | path!() macro, nested routes, params, navigation | Route setup or nav issues |
| `error-handling` | ServerFnError, ErrorBoundary, custom errors | Error propagation patterns |
| `effects` | Side effects, cleanup, no dependency arrays | DOM/network/external sync |
| `api-migrations` | 0.7→0.8 migration (no cx, new imports) | Code uses old patterns |
| `reactivity-internals` | Arena model, subscription graph, update ordering | Debugging reactive behavior |
| `ecosystem-and-fit` | Crate choices, templates, ecosystem overview | Library selection |

### Mandatory MCP Protocol

1. **Session start**: Call `list-sections` once to confirm tool availability.
2. **Before any new component**: Query `get-documentation` for `mental-model` (first time) + relevant sections.
3. **When stuck on a Leptos API**: Query the specific section rather than guessing. Guessing wastes cycles.
4. **After writing a component**: Run `leptos-autofixer` on the code before moving on. It catches issues the compiler won't.
