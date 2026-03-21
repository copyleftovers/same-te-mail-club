# Lower-Priority Categories — Combined Report

**Assessed:** 2026-03-19
**Context:** Leptos 0.8 / Axum / Postgres project at completion (all 6 phases done). This survey focuses on learning resources, alternative patterns, and real-world precedents for an already-built application.

---

## Resources

### Leptos Book
- **Status:** Moved to https://book.leptos.dev and https://github.com/leptos-rs/book (separate repository from main Leptos repo)
- **Relevance to project:** The book is authoritative for Leptos 0.8 fundamentals, but the Mail Club's codebase already embeds learned patterns (ActionForm idioms, Resource patterns, server function design). Best used as a reference for new team members or unfamiliar edge cases, not foundational learning.
- **Assessment:** Primary value is as a stable reference. The project's `guidance/leptos-idioms.md` has already captured the essential patterns for this scope.

### Discord Community
- **Status:** https://discord.gg/YdRAhS7eQB
- **Relevance:** Active community for debugging, design questions, and ecosystem awareness. Worth joining for real-time unblocking on Leptos-specific issues if they arise post-deployment.
- **Assessment:** Low immediate relevance for a completed project, but valuable for maintenance and edge-case questions.

---

## Starter Templates (Official + Unofficial)

### Official: start-axum-workspace
- **Structure:** Cargo workspace organizing frontend (app), backend (server), assets (public, style), and testing (end2end) into separate crates
- **Build tool:** cargo-leptos for full-stack coordination
- **Relevance to Mail Club:** The workspace structure mirrors Mail Club's own layout. The integrated Playwright E2E setup and islands-architecture support are patterns the project already uses. Nothing novel to adopt, but confirms the workspace approach is idiomatic.
- **Assessment:** Validates Mail Club's architecture choices. If onboarding new Rust developers, this is a useful reference for workspace conventions.

### Official: start-trunk
- **Use case:** Client-side rendering (CSR) without a backend server
- **Relevance:** Mail Club is full-stack SSR + Axum. Not applicable unless pivoting to a pure frontend project.
- **Assessment:** Skip.

### Official: start-actix
- **Use case:** Leptos + Actix (alternative to Axum)
- **Relevance:** Mail Club committed to Axum. Useful only for comparison or if Axum needed replacing (unlikely).
- **Assessment:** Skip.

### Official: start-aws, start-spin
- **Use cases:** AWS Lambda, Spin (serverless WASI) deployments
- **Relevance:** Mail Club is deployed as a standalone binary on traditional infrastructure. These are deployment shape alternatives, not architectural improvements.
- **Assessment:** Skip for current project; reference if deploying to serverless later.

### Unofficial: leptos-fullstack (Nix + crane)
- **Tooling:** Flakes for reproducible dev environments, crane for Nix-integrated Rust builds, direnv integration
- **Notable approach:** Separates dev and build environments using Nix, making the entire stack reproducible across machines
- **Relevance to Mail Club:** The project doesn't currently use Nix. Adopting Nix would be a significant infrastructure change and is justified only if team growth requires reproducible dev environments or if CI complexity increases. Current `just` commands and local `cargo leptos` are simpler for a solo developer.
- **Assessment:** Worth revisiting if the project grows or moves to distributed development; not critical now.

### Unofficial: leptos-workers, tauri-leptos-ssr
- **leptos-workers:** Cloudflare Workers edge deployment
- **tauri-leptos-ssr:** Desktop + web hybrid using Tauri
- **Relevance:** Neither fits Mail Club's current shape (traditional web app with backend). Desktop client or edge deployment would require separate project briefs.
- **Assessment:** Skip unless product requirements change.

---

## Alternate Macros

### leptos-mview
- **Purpose:** Concise alternative to the `view!` macro, inspired by `maud` (Rust HTML templating)
- **Key improvements over standard `view!`:**
  - CSS selector syntax: `h1.title` instead of `h1 class="title"`
  - Reduced punctuation: semicolons end childless elements, no quotes on simple attributes
  - Bracket shortcuts for reactivity: `when=[condition]` → `{move || condition}`
  - Format macro shorthand: `f[format_string, args]` instead of explicit `format!()`
  - Better IDE autocomplete performance due to lazier parsing
- **Relevance to Mail Club:**
  - **Against adoption:** The project's `view!` macros are already written and tested. Migrating existing code would require mechanical refactoring with zero functional gain. The current codebase is readable and maintainable.
  - **For new code:** If a significant new feature were added, leptos-mview might reduce boilerplate. But the savings are marginal for a 15-screen app.
  - **Ecosystem concern:** leptos-mview is less widely used than the standard `view!`. Tooling (formatting, IDE support) is less mature.
- **Assessment:** **Skip for now.** If a major new feature area is built, consider it as a localized experiment. Not worth a codebase rewrite.

---

## Blogs / Websites

Real-world Leptos applications provide patterns worth studying. The ecosystem shows diverse deployment and design approaches:

### Architectural Patterns Observed

**1. Server-Side Rendering + WASM Hydration (SSR pattern)**
- **leptos.dev** (official): Demonstrates progressive enhancement using `requestIdleCallback` to defer interactive island loading. Server renders complete HTML first; WASM hydrates asynchronously.
- **khuedoan.com:** Full Leptos SSR with Axum backend and Cloudflare deployment. Intentionally sophisticated stack for a markdown blog (author's own words: "overengineered").
- **nicoburniske.com:** Blog + photo gallery using `leptos_image` (optimization) and `leptos_query` (caching). Blur-up progressive image loading pattern worth studying for any image-heavy features.
- **www.everdev.it:** Leptos SSR + Tailwind + Astro hybrid (integration). Shows Leptos can coexist with other meta-frameworks.

**2. Type-Safe Backends (Database + ORM)**
- **quanticbox.app:** Financial dashboard with Leptos + Axum + Diesel ORM. Demonstrates data-heavy dashboard pattern.
- **Mail Club parallel:** Uses sqlx for type-safe queries. Diesel is an alternative; sqlx's approach is more aligned with the project's offline-first philosophy.

**3. Design System & Styling**
- **leptos.dev:** Tailwind CSS for utility-first styling. Responsive design with fine-grained reactivity for state.
- **www.everdev.it:** OKLCH color system with CSS custom properties for theming. Variable fonts (EB Garamond, Inter). Semantic spacing units. **This design approach (OKLCH + custom properties) is similar to Mail Club's chosen palette.** Worth referencing for future refinements.
- **nicoburniske.com:** Lazy-loaded images with blur placeholders. Pattern for optimizing media-heavy content.
- **s1n7ax.com (not Leptos, but similar stack):** Flickering grid background, lime-green accent (rgb(132, 204, 22)), dark theme. JetBrains Mono typography. Shows how minimalist design can still have personality.

**4. Specialized Domains**
- **moturbo.com:** E-commerce shop in Leptos. Demonstrates product catalog, cart, and checkout flows.
- **Owdle:** Daily guessing game. Example of game/interactive-heavy Leptos app.
- **Ibis (ibis.wiki):** Federated encyclopedia on ActivityPub. Shows Leptos's capability for protocol-heavy applications (similar to Mail Club's offline-sync design thinking).
- **rustytube.rs:** YouTube client for desktop & web (Tauri + Leptos). Desktop + web code reuse pattern.

### Styling Approaches Worth Noting

| Site | Tech | Notable Pattern |
|------|------|-----------------|
| leptos.dev | Tailwind + Leptos | Progressive enhancement, lazy island loading |
| www.everdev.it | Tailwind v4 + OKLCH + variable fonts | Semantic spacing, CSS custom properties, dark mode toggle |
| nicoburniske.com | Tailwind + leptos_image | Blur-up image optimization, lazy loading |
| khuedoan.com | Tailwind + Axum SSR | Sophisticated stack for content-focused site |
| Ibis | (unclear, federated) | ActivityPub integration pattern |

### Deployment Patterns
- **www.everdev.it, khuedoan.com:** Axum SSR deployed to traditional cloud (Cloudflare, VPS)
- **benw.is:** Spin (serverless WASI) with SQLite storage — shows lightweight deployment
- **nicoburniske.com:** Likely traditional VPS with image optimization at build time
- **s1n7ax.com:** Vercel (Next.js) — not Leptos, but shows modern deployment conventions

---

## Key Takeaways

### 1. Architecture Validation
Mail Club's chosen stack (Leptos 0.8, Axum SSR, Postgres, cargo-leptos workspace, Playwright E2E) is idiomatic and well-established in the ecosystem. No structural changes needed.

### 2. Design System Reference
The OKLCH + CSS custom property approach (seen in www.everdev.it) aligns with Mail Club's design tokens. This is a **validated pattern** across modern Leptos applications.

### 3. Image Optimization Opportunity
If Mail Club adds image-heavy features (participant avatars, event photos), the `nicoburniske.com` pattern of blur-up progressive loading via `leptos_image` is worth adopting.

### 4. Styling Evolution
Progressive Enhancement (leptos.dev pattern) and lazy island loading via `requestIdleCallback` are advanced optimizations. Mail Club's current hydration gate pattern is simpler and sufficient for the current scope.

### 5. Alternative Macros: Not Worth Adopting
leptos-mview's brevity is marginal for this project's size. Standard `view!` is mature, well-documented, and suitable for Mail Club's idiom guide.

### 6. Nix/Reproducible Builds: Optional
leptos-fullstack's Nix setup is valuable for distributed teams. Mail Club doesn't need it now, but it's a low-friction migration if the team grows.

### 7. Real-World Validation
The ecosystem shows Leptos is production-ready for:
- Content sites (blogs, portfolios)
- Data dashboards (financial, analytics)
- E-commerce (product catalogs)
- Games and interactive tools
- Federated/protocol-heavy apps

Mail Club's seasonal mail exchange domain is simpler than most of these examples—no special patterns needed.

---

## No Action Items
All categories in this report are **lower-priority** for Mail Club:
- **Resources:** Already captured in project guidance
- **Templates:** Architecture is already established; workspace is validated
- **Alternate macros:** Not worth migration or adoption
- **Real-world sites:** Provide confidence in stack choice and isolated patterns to consider (blur-up images, OKLCH theming, progressive enhancement) if expanding scope

Continue maintenance and iteration on the current patterns.
