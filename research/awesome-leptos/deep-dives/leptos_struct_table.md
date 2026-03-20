# leptos-struct-table — Deep Technical Verification

## Crate Metadata

- **Latest version**: 0.18.0
- **Published**: 2026-02-03
- **License**: MIT OR Apache-2.0
- **Downloads**: 70,387 total; 15,722 recent (last 30 days)
- **Repository**: https://github.com/Synphonyte/leptos-struct-table
- **Dependencies**:
  - `leptos = "0.8"` ✓
  - `leptos-struct-table-macro = "0.15.0"` (proc-macro layer)
  - `leptos-use = "0.18"` (with feature-gated SSR support)

## Leptos 0.8 Compatibility

**VERIFIED** ✓

- Crate versions 0.15–0.18 target Leptos 0.8 explicitly
- README documents compatibility table: 0.15–0.18 → Leptos 0.8
- Server-side rendering officially supported via `leptos-use` SSR features
- Cargo.toml explicitly pins `leptos = "0.8"` with no `~` (not semver compatible, strict match)

## Tailwind v4 Compatibility

**LIKELY COMPATIBLE** (with caveats) — **STATUS: UNVERIFIED**

### How TailwindClassesPreset Works

The `TailwindClassesPreset` is a **class string generator**, not a config-aware tool:

```rust
pub struct TailwindClassesPreset;

impl TableClassesProvider for TailwindClassesPreset {
    fn thead_row(&self, template_classes: &str) -> String {
        format!(
            "{} {}",
            "text-xs text-gray-700 uppercase bg-gray-200 dark:bg-gray-700 dark:text-gray-300",
            template_classes
        )
    }

    fn row(&self, row_index: usize, selected: bool, template_classes: &str) -> String {
        let bg_color = if row_index.is_multiple_of(2) {
            if selected {
                "bg-sky-300 text-gray-700 dark:bg-sky-700 dark:text-gray-400"
            } else {
                "bg-white dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800"
            }
        } else if selected {
            "bg-sky-300 text-gray-700 dark:bg-sky-700 dark:text-gray-400"
        } else {
            "bg-gray-50 dark:bg-gray-800 hover:bg-gray-100 dark:hover:bg-gray-700"
        };
        format!("{} {} {}", "border-b border-gray-300 dark:border-gray-700", bg_color, template_classes)
    }

    fn cell(&self, template_classes: &str) -> String {
        format!("{} {}", "px-5 py-2", template_classes)
    }
}
```

**Key observation**: All class strings are **hardcoded at compile time**. The preset outputs **literal Tailwind utility class names** as strings (e.g., `"text-xs text-gray-700 bg-gray-200"`). These are then injected into the HTML via Leptos's `class` attribute.

### Tailwind v4 CSS-Based Config Compatibility

The preset **generates standard Tailwind v3/v4 utility class names**. It makes **NO assumptions** about:
- `tailwind.config.js` (not needed for v4)
- Theme customization files
- CSS layer configuration
- Arbitrary value syntax

All class names used are **vanilla Tailwind utilities** that work identically in v3 and v4:
- `text-xs`, `text-gray-700`, `bg-gray-200`, `dark:bg-gray-700` — all standard
- The preset does NOT generate arbitrary classes like `bg-[#123456]` or custom theme values
- No CSS variables are referenced (no `var(--color-brand-blue)`)

**Verdict on v4**: The class strings generated will work with Tailwind v4's standalone binary (`cargo-leptos` integration). However, this has **NOT been explicitly tested** by the crate maintainers. Tailwind v4 maintains backward compatibility with v3 utilities, so the risk is low.

### Critical Detail: Custom Classes Override via `cell_class`

The project uses this pattern extensively:

```rust
#[derive(TableRow, Clone)]
pub struct Book {
    id: u32,
    title: String,
    #[table(
        cell_class = "text-red-600 dark:text-red-400",
        head_class = "text-red-700 dark:text-red-300"
    )]
    pub publish_date: NaiveDate,
}
```

This allows extending the preset's base classes with custom Tailwind utilities. Since **this project uses custom CSS tokens** (e.g., `--color-brand-orange`, `--color-accent`), **you would need to use arbitrary value syntax or map custom utilities**, which could complicate adoption:

```rust
// Would need something like:
#[table(cell_class = "bg-(--color-accent)")]  // Tailwind v4 arbitrary value syntax
```

**Not tested with this project's design tokens.**

## SSR + Hydration

**VERIFIED** ✓

- README documents explicit SSR support with `leptos-use` feature gates
- Dependencies include proper feature flags:
  ```toml
  [features]
  ssr = ["leptos/ssr", ..., "leptos-use/ssr"]
  hydrate = ["leptos/hydrate", ...]
  ```
- Example project (`examples/serverfn_sqlx/`) demonstrates SSR with server functions
- The crate uses `leptos-use` for measurements (`use_element_size`, `use_scroll`) — these are SSR-safe when properly feature-gated
- **No hydration issues documented** — table is a purely client-side component after hydration

One caveat: The table uses viewport measurement hooks from `leptos-use`. These must be called **after hydration**. The provided example wraps table in a component that hydrates correctly. Your project's hydration gate pattern (`disabled` until hydration) will work seamlessly with this crate.

## Current Project Need

### Admin Tables Today

Reading `src/admin/participants.rs`:

- **Participants table**: Hand-rolled `<table>` with manual `<thead>`/`<tbody>`/`<For>` iteration
- **Features used**:
  - Display (name, phone, status badge, action button)
  - Status filtering (active/inactive visual distinction)
  - Per-row action (deactivate button)
  - No sorting, no pagination, no virtual scrolling, no selection
- **Scale**: Admin lists participants — likely < 1000 rows, no performance concerns
- **Styling**: Uses `.data-table` CSS class (defined in `style/tailwind.css`)

### Does the project actually need `leptos-struct-table`?

**Current table is simple enough** to remain hand-rolled. The admin participant list:
- Filters by role (participant-only) on the server
- Uses a `Resource` for async loading
- Renders via `<For>` loop
- Has one action per row (deactivate)
- No sorting or pagination

**Scenarios where adoption makes sense**:
1. You add 5+ more tables to the admin dashboard (assignments visualization, season history, etc.)
2. Users request sorting or pagination on existing tables
3. You want to reduce boilerplate by deriving table structure from types

**Scenarios where it's overkill**:
1. One or two small tables with < 500 rows
2. Highly custom per-cell rendering (current table has conditional badge rendering)
3. Tables that are built incrementally (current style)

## Integration Effort (IF Adopted)

### If you decide to migrate the participants table:

1. **Derive macro** (~5 min):
   ```rust
   #[derive(TableRow, Clone)]
   #[table(
       sortable,
       classes_provider = "TailwindClassesPreset",
       impl_vec_data_provider
   )]
   pub struct ParticipantRow {
       #[table(title = "Name")]
       name: String,
       #[table(title = "Phone")]
       phone: String,
       #[table(title = "Status")]
       status: UserStatus,
       // Action button would need a custom renderer
   }
   ```

2. **Custom cell renderer for actions** (~30 min):
   - The crate supports field-level custom renderers
   - Would re-implement the `ActionForm` deactivate button as a renderer
   - Needs a `RwSignal<ParticipantRow>` to dispatch the action

3. **Update CSS** (~10 min):
   - Remove `.data-table` styles (replaced by `TailwindClassesPreset`)
   - Decide: keep `TailwindClassesPreset` as-is, or create custom `TableClassesProvider` to align with your design tokens

4. **Update Resource** (minimal):
   - Feed `Vec<ParticipantRow>` to the table instead of manual `<For>`
   - The `TableContent` component handles rendering

5. **E2E test updates** (~20 min):
   - The crate generates specific DOM structure (`<table><thead>...<tbody>`)
   - Update selectors in POM if they rely on current structure
   - Add `data-testid` to generated cells/rows (may require custom renderer)

**Total realistic effort**: ~1 hour for one table, assuming no custom renderers needed. More if you need tight control over cell layout.

## Risks

### 1. Tailwind v4 Customization Gap

**Risk**: The preset generates v3-style class names. Your project uses custom CSS tokens (`--color-brand-orange`, etc.). To use the table with your tokens, you'd need:

- **Option A**: Create a custom `TableClassesProvider` that uses v4 arbitrary values:
  ```rust
  fn cell(&self, _template_classes: &str) -> String {
      "px-5 py-2 bg-(--color-surface) text-(--color-text)".to_string()
  }
  ```
  **Blocker**: v4 arbitrary value syntax is CSS-aware, not Rust-aware. The crate would generate *string literals*, and Tailwind's scanner would need to detect them. **Feasibility: unclear. Untested.**

- **Option B**: Stick with the preset (v3-style gray palette) and accept visual inconsistency with your design system.

- **Option C**: Map your tokens to preset values and add per-table overrides via `cell_class`:
  ```rust
  #[table(cell_class = "text-[--color-text]")]
  name: String,
  ```
  **Feasibility: probably works, needs testing.**

**Verdict**: This is the main blocker for seamless adoption. **Recommend testing with one table before rolling out.**

### 2. Leptos Version Pin

**Risk**: The crate pins `leptos = "0.8"` exactly. If you upgrade to Leptos 0.9 (months from now), this crate will not be available. However, given the crate's 15k recent downloads, maintainers will likely release 0.9 support quickly. Not a blocker for now.

### 3. Custom Rendering Complexity

**Risk**: If you need to customize cell rendering (e.g., conditional badges, multi-line content, or form inputs), you must implement the `TableCellRenderer` trait. This is more complex than hand-rolling `<td>` elements.

**Evidence**: The editable cells example exists, so it's possible. But it shifts complexity from component code to trait impls.

### 4. Bundle Size

**Risk**: The crate adds:
- `leptos-struct-table` (~50KB uncompressed)
- `leptos-struct-table-macro` (proc-macro, compiled separately)
- `leptos-use` features (if not already included)

Your project likely already uses `leptos-use` (for SSR utilities), so the incremental cost is one extra crate. Not significant for a 15-screen app.

### 5. API Stability

**Risk**: The crate is at 0.18.0 (not 1.0). The author may introduce breaking changes. However:
- Releases are frequent (37 versions since 0.3)
- Changelog exists and breaking changes are documented
- Community active (70k downloads)

Not a blocker for a solo project, but something to monitor.

## Verdict

**DEFER** — Confidence: **80%**

### Reasoning

Your current hand-rolled table is **fit-for-purpose**. The participants list is simple, SSR/hydration compatible, and aligns with your design system. Adopting `leptos-struct-table` would:

- Reduce boilerplate by ~50 lines (macro + derive vs hand-rolled `<For>`)
- Add complexity in understanding derived components and custom renderers
- Require you to solve the "custom Tailwind tokens" integration gap
- Create a dependency on a 0.x crate (not 1.0 stable)

### When to adopt (trigger conditions)

1. **You add a second complex table** (assignments, seasons, history) with sorting or pagination
2. **You want shared styling** across 5+ tables (macro+preset pays off)
3. **You need pagination or virtual scrolling** at scale (the crate's real strength)
4. **The custom Tailwind tokens problem** is solved and documented

### Short-term action

- **Keep current approach** for the participants table
- **Reference this crate** in awesome-leptos if another project needs data tables
- **Revisit in Q3 2026** if table needs grow or when Tailwind v4 support is explicitly documented

### If you do adopt

**Testing checklist**:
1. Add `leptos-struct-table` to a feature branch
2. Migrate participants table with `TailwindClassesPreset`
3. Test SSR rendering (verify classes appear in HTML)
4. Test hydration (verify table is sortable/interactive after WASM loads)
5. Verify E2E tests (POM selectors, testids, action buttons work)
6. Confirm custom Tailwind token strategy (arbitrary values, overrides, or compatibility)
7. Commit only if all tests pass and integration feels natural

---

## Sources

- [leptos-struct-table crate](https://crates.io/crates/leptos-struct-table)
- [Repository](https://github.com/Synphonyte/leptos-struct-table)
- [TailwindClassesPreset source](https://raw.githubusercontent.com/Synphonyte/leptos-struct-table/master/src/class_providers/tailwind.rs)
- [Tailwind example](https://github.com/Synphonyte/leptos-struct-table/blob/master/examples/tailwind/src/main.rs)
- [SSR example](https://github.com/Synphonyte/leptos-struct-table/blob/master/examples/serverfn_sqlx/Cargo.toml)
- Project: `src/admin/participants.rs` — current table usage
