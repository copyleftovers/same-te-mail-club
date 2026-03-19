use crate::i18n::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

use crate::{
    admin::{
        assignments::AssignmentsPage, dashboard::DashboardPage, nav::AdminNav,
        participants::ParticipantsPage, season::SeasonManagePage, sms::SmsPage,
    },
    pages::{home::HomePage, login::LoginPage, onboarding::OnboardingPage},
};

// ── Server function ───────────────────────────────────────────────────────────

/// Get the current user from the session cookie.
///
/// Returns `None` if no valid session exists or the user is inactive.
/// Used by components for route-level auth guards.
#[server]
pub async fn get_current_user() -> Result<Option<crate::types::CurrentUser>, ServerFnError> {
    use crate::auth;
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    match auth::current_user(&pool, &parts).await {
        Ok(user) => Ok(Some(user)),
        Err(_) => Ok(None),
    }
}

// ── Shell ─────────────────────────────────────────────────────────────────────

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="uk">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
                <link rel="icon" href="/favicon.svg" type="image/svg+xml"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    // provide_i18n_context is deprecated in favour of <I18nContextProvider>
    // component, but the component pattern requires wrapping App children in the
    // view macro which restructures the entire routing setup — out of scope for
    // this i18n pass.
    #[allow(deprecated)]
    crate::i18n::i18n::provide_i18n_context();
    let i18n = use_i18n();

    // Fetch current user once at app load — used for all auth guards
    let current_user = Resource::new(|| (), |()| get_current_user());

    provide_context(current_user);

    view! {
        <Stylesheet id="leptos" href="/pkg/samete.css"/>
        <Title text=t_string!(i18n, app_title)/>

        <Router>
            <header class="app-header">
                <a href="/">
                    <img src="/same_te_mark_orange.svg" alt="Саме Те" class="h-10 w-auto" />
                </a>
                <HeaderNav/>
            </header>
            <main>
                <Routes fallback=move || t!(i18n, app_not_found)>
                    <Route path=StaticSegment("login") view=LoginPage/>
                    <Route path=StaticSegment("onboarding") view=move || {
                        view! { <AuthGuard require_onboarded=false><OnboardingPage/></AuthGuard> }
                    }/>
                    <Route path=StaticSegment("") view=move || {
                        view! { <AuthGuard require_onboarded=true><HomePage/></AuthGuard> }
                    }/>
                    // Admin routes — flat list, exact matching.
                    // Each tuple path matches exactly (no prefix overlap with
                    // `StaticSegment("admin")`).
                    <Route path=StaticSegment("admin") view=move || {
                        view! { <AdminGuard><DashboardPage/></AdminGuard> }
                    }/>
                    <Route path=(StaticSegment("admin"), StaticSegment("season")) view=move || {
                        view! { <AdminGuard><SeasonManagePage/></AdminGuard> }
                    }/>
                    <Route path=(StaticSegment("admin"), StaticSegment("participants")) view=move || {
                        view! { <AdminGuard><ParticipantsPage/></AdminGuard> }
                    }/>
                    <Route path=(StaticSegment("admin"), StaticSegment("assignments")) view=move || {
                        view! { <AdminGuard><AssignmentsPage/></AdminGuard> }
                    }/>
                    <Route path=(StaticSegment("admin"), StaticSegment("sms")) view=move || {
                        view! { <AdminGuard><SmsPage/></AdminGuard> }
                    }/>
                </Routes>
            </main>
        </Router>
    }
}

// ── Header nav ────────────────────────────────────────────────────────────────

/// Renders `AdminNav` when the current path is under `/admin`, nothing otherwise.
///
/// Must be rendered inside `<Router>` so `use_location` has context.
#[component]
fn HeaderNav() -> impl IntoView {
    let pathname = leptos_router::hooks::use_location().pathname;
    move || {
        pathname
            .get()
            .starts_with("/admin")
            .then(|| view! { <AdminNav/> })
    }
}

// ── Navigation helper ─────────────────────────────────────────────────────────

/// Redirect that works during both SSR and client-side hydration.
///
/// On the server: sets a 302 Location header via `leptos_axum::redirect`.
/// On the client: uses the router's `use_navigate` for SPA navigation.
fn isomorphic_redirect(path: &str) {
    #[cfg(feature = "ssr")]
    leptos_axum::redirect(path);

    #[cfg(not(feature = "ssr"))]
    {
        let navigate = leptos_router::hooks::use_navigate();
        navigate(path, leptos_router::NavigateOptions::default());
    }
}

// ── Auth guards ───────────────────────────────────────────────────────────────

/// Guard that requires authentication. Redirects to `/login` if unauthenticated.
/// If `require_onboarded=true`, also redirects to `/onboarding` if not yet onboarded.
/// If `require_onboarded=false` (onboarding page itself), redirects to `/` when already onboarded.
#[component]
fn AuthGuard(require_onboarded: bool, children: ChildrenFn) -> impl IntoView {
    let current_user =
        use_context::<Resource<Result<Option<crate::types::CurrentUser>, ServerFnError>>>()
            .expect("CurrentUser resource must be provided");

    view! {
        <Suspense fallback=|| ()>
            {move || {
                let children = children.clone();
                current_user.get().map(|result| {
                    match result {
                        Ok(None) | Err(_) => {
                            isomorphic_redirect("/login");
                            ().into_any()
                        }
                        Ok(Some(ref user)) if require_onboarded && !user.onboarded => {
                            isomorphic_redirect("/onboarding");
                            ().into_any()
                        }
                        Ok(Some(ref user)) if !require_onboarded && user.onboarded => {
                            isomorphic_redirect("/");
                            ().into_any()
                        }
                        Ok(Some(_)) => children().into_any(),
                    }
                })
            }}
        </Suspense>
    }
}

/// Guard that requires admin role. Redirects to `/` if not admin (or not logged in → `/login`).
#[component]
fn AdminGuard(children: ChildrenFn) -> impl IntoView {
    use crate::types::UserRole;

    let current_user =
        use_context::<Resource<Result<Option<crate::types::CurrentUser>, ServerFnError>>>()
            .expect("CurrentUser resource must be provided");

    view! {
        <Suspense fallback=|| ()>
            {move || {
                let children = children.clone();
                current_user.get().map(|result| {
                    match result {
                        Ok(None) | Err(_) => {
                            isomorphic_redirect("/login");
                            ().into_any()
                        }
                        Ok(Some(ref user)) if user.role == UserRole::Admin => {
                            view! { <div data-layout="admin">{children()}</div> }.into_any()
                        }
                        Ok(Some(_)) => {
                            isomorphic_redirect("/");
                            ().into_any()
                        }
                    }
                })
            }}
        </Suspense>
    }
}
