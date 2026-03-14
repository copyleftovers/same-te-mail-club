use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

use crate::{
    admin::participants::ParticipantsPage,
    pages::{login::LoginPage, onboarding::OnboardingPage},
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

    // Fetch current user once at app load — used for all auth guards
    let current_user = Resource::new(|| (), |()| get_current_user());

    provide_context(current_user);

    view! {
        <Stylesheet id="leptos" href="/pkg/samete.css"/>
        <Title text="The Mail Club"/>

        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("login") view=LoginPage/>
                    <Route path=StaticSegment("onboarding") view=move || {
                        view! { <AuthGuard require_onboarded=false><OnboardingPage/></AuthGuard> }
                    }/>
                    <Route path=StaticSegment("") view=move || {
                        view! { <AuthGuard require_onboarded=true><HomePage/></AuthGuard> }
                    }/>
                    <Route path=StaticSegment("admin/participants") view=move || {
                        view! { <AdminGuard><ParticipantsPage/></AdminGuard> }
                    }/>
                </Routes>
            </main>
        </Router>
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

    let navigate = leptos_router::hooks::use_navigate();

    view! {
        <Suspense fallback=|| ()>
            {move || {
                let children = children.clone();
                current_user.get().map(|result| {
                    match result {
                        Ok(None) | Err(_) => {
                            navigate("/login", leptos_router::NavigateOptions::default());
                            ().into_any()
                        }
                        Ok(Some(user)) if require_onboarded && !user.onboarded => {
                            navigate("/onboarding", leptos_router::NavigateOptions::default());
                            ().into_any()
                        }
                        Ok(Some(user)) if !require_onboarded && user.onboarded => {
                            // Already onboarded — skip onboarding page
                            navigate("/", leptos_router::NavigateOptions::default());
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

    let navigate = leptos_router::hooks::use_navigate();

    view! {
        <Suspense fallback=|| ()>
            {move || {
                let children = children.clone();
                current_user.get().map(|result| {
                    match result {
                        Ok(None) | Err(_) => {
                            navigate("/login", leptos_router::NavigateOptions::default());
                            ().into_any()
                        }
                        Ok(Some(user)) if user.role == UserRole::Admin => {
                            children().into_any()
                        }
                        Ok(Some(_)) => {
                            navigate("/", leptos_router::NavigateOptions::default());
                            ().into_any()
                        }
                    }
                })
            }}
        </Suspense>
    }
}

// ── Home page ─────────────────────────────────────────────────────────────────

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"The Mail Club"</h1>
        <p>"Seasonal self-expression ritual."</p>
    }
}
