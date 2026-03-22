use crate::i18n::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use leptos_use::use_preferred_dark;

use crate::{
    admin::{
        assignments::AssignmentsPage, dashboard::DashboardPage, nav::AdminNav,
        participants::ParticipantsPage, season::SeasonManagePage, sms::SmsPage,
    },
    components::toast::{Toast, provide_toast_context},
    pages::{
        home::HomePage,
        login::{LoginPage, Logout},
        onboarding::OnboardingPage,
    },
};

// ── Server function ───────────────────────────────────────────────────────────

/// Get the current user from the session cookie.
///
/// Returns `None` if no valid session exists or the user is inactive.
/// Used by components for route-level auth guards.
#[server]
pub async fn get_current_user() -> Result<Option<crate::types::CurrentUser>, ServerFnError> {
    use crate::auth;

    match auth::require_auth().await {
        Ok((_pool, user)) => Ok(Some(user)),
        Err(_) => Ok(None),
    }
}

// ── Shell ─────────────────────────────────────────────────────────────────────

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="uk">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover, user-scalable=yes" />
                <meta name="theme-color" content="#D93A12" media="(prefers-color-scheme: light)" />
                <meta name="theme-color" content="#161616" media="(prefers-color-scheme: dark)" />
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
                <link rel="manifest" href="/manifest.json" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
                <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
            </head>
            <body>
                <App />
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
    // Create a logout action at the app level so its version can trigger refetches
    let logout_action = ServerAction::<Logout>::new();
    let current_user = Resource::new(
        move || logout_action.version().get(),
        |_| get_current_user(),
    );

    provide_context(logout_action);
    provide_context(current_user);

    // Provide toast context — used for success feedback across the app
    provide_toast_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/samete.css" />
        <Title text=t_string!(i18n, app_title) />

        <Router>
            <div class="flex min-h-dvh flex-col">
                <Header />
                <main class="flex-1">
                    <Routes fallback=move || t!(i18n, app_not_found)>
                    <Route path=StaticSegment("login") view=LoginPage />
                    <Route
                        path=StaticSegment("onboarding")
                        view=move || {
                            view! {
                                <AuthGuard require_onboarded=false>
                                    <OnboardingPage />
                                </AuthGuard>
                            }
                        }
                    />
                    <Route
                        path=StaticSegment("")
                        view=move || {
                            view! {
                                <AuthGuard require_onboarded=true>
                                    <HomePage />
                                </AuthGuard>
                            }
                        }
                    />
                    // Admin routes — flat list, exact matching.
                    // Each tuple path matches exactly (no prefix overlap with
                    // `StaticSegment("admin")`).
                    <Route
                        path=StaticSegment("admin")
                        view=move || {
                            view! {
                                <AdminGuard>
                                    <DashboardPage />
                                </AdminGuard>
                            }
                        }
                    />
                    <Route
                        path=(StaticSegment("admin"), StaticSegment("season"))
                        view=move || {
                            view! {
                                <AdminGuard>
                                    <SeasonManagePage />
                                </AdminGuard>
                            }
                        }
                    />
                    <Route
                        path=(StaticSegment("admin"), StaticSegment("participants"))
                        view=move || {
                            view! {
                                <AdminGuard>
                                    <ParticipantsPage />
                                </AdminGuard>
                            }
                        }
                    />
                    <Route
                        path=(StaticSegment("admin"), StaticSegment("assignments"))
                        view=move || {
                            view! {
                                <AdminGuard>
                                    <AssignmentsPage />
                                </AdminGuard>
                            }
                        }
                    />
                    <Route
                        path=(StaticSegment("admin"), StaticSegment("sms"))
                        view=move || {
                            view! {
                                <AdminGuard>
                                    <SmsPage />
                                </AdminGuard>
                            }
                        }
                    />
                    </Routes>
                </main>
            </div>
        </Router>

        <Toast />
    }
}

// ── Header ────────────────────────────────────────────────────────────────────

/// App header with dark-mode–aware logo and admin nav.
#[component]
fn Header() -> impl IntoView {
    let is_dark = use_preferred_dark();
    let (menu_open, set_menu_open) = signal(false);

    view! {
        <header class="app-header">
            <a href="/">
                <img
                    src=move || {
                        if is_dark.get() {
                            "/same_te_mark_white.svg"
                        } else {
                            "/same_te_mark_orange.svg"
                        }
                    }
                    alt="Саме Те"
                    class="h-10 w-auto"
                />
            </a>
            <HeaderNav />
            <button
                class="menu-toggle"
                aria-label="Menu"
                aria-expanded=move || menu_open.get()
                data-testid="menu-toggle"
                on:click=move |_| set_menu_open.update(|v| *v = !*v)
            >
                <span class="block h-0.5 w-5 bg-current"></span>
                <span class="block h-0.5 w-5 bg-current mt-1"></span>
                <span class="block h-0.5 w-5 bg-current mt-1"></span>
            </button>
            <Show when=move || menu_open.get()>
                <MobileMenu on_close=Callback::new(move |()| set_menu_open.set(false)) />
            </Show>
        </header>
    }
}

// ── Header nav ────────────────────────────────────────────────────────────────

/// Renders `AdminNav` when the current path is under `/admin`, logout button otherwise.
///
/// Must be rendered inside `<Router>` so `use_location` has context.
#[component]
fn HeaderNav() -> impl IntoView {
    use crate::hooks::use_hydrated;
    let i18n = use_i18n();
    let pathname = leptos_router::hooks::use_location().pathname;
    let hydrated = use_hydrated();
    let logout_action =
        use_context::<ServerAction<Logout>>().expect("logout action must be provided");

    let is_admin = move || pathname.get().starts_with("/admin");
    let show_logout = move || {
        !pathname.get().starts_with("/admin")
            && pathname.get() != "/login"
            && pathname.get() != "/onboarding"
    };

    view! {
        <Show when=is_admin fallback=move || {
            view! {
                <Show when=show_logout>
                    <div class="header-nav">
                        <leptos::form::ActionForm action=logout_action>
                            <button
                                type="submit"
                                class="btn"
                                data-variant="secondary"
                                data-size="sm"
                                data-testid="logout-button"
                                disabled=move || !hydrated.get()
                            >
                                {t!(i18n, nav_logout)}
                            </button>
                        </leptos::form::ActionForm>
                    </div>
                </Show>
            }
        }>
            <AdminNav />
        </Show>
    }
}

// ── Mobile menu ────────────────────────────────────────────────────────────────

/// Mobile navigation menu with overlay and slide-in panel.
///
/// Closes on overlay click, link click, or Escape key.
#[component]
fn MobileMenu(on_close: Callback<()>) -> impl IntoView {
    use crate::hooks::use_hydrated;
    let i18n = use_i18n();
    let pathname = leptos_router::hooks::use_location().pathname;
    let is_active = move |path: &'static str| move || pathname.get() == path;
    let hydrated = use_hydrated();
    let logout_action =
        use_context::<ServerAction<Logout>>().expect("logout action must be provided");

    // Close menu on Escape key (client-side only)
    #[cfg(not(feature = "ssr"))]
    Effect::new(move |_| {
        use leptos::prelude::document;
        use leptos::wasm_bindgen::JsCast;
        use leptos::wasm_bindgen::closure::Closure;
        use leptos::web_sys;

        let document = document();
        let closure: Closure<dyn Fn(web_sys::KeyboardEvent)> =
            Closure::new(move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Escape" {
                    on_close.run(());
                }
            });

        let _ =
            document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());

        // Cleanup: remove listener when Effect is dropped
        move || {
            let _ = document
                .remove_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
        }
    });

    view! {
        <div
            class="mobile-menu-overlay"
            on:click=move |_| on_close.run(())
            data-testid="mobile-menu-overlay"
        ></div>
        <nav class="mobile-menu" data-testid="mobile-menu">
            <a
                href="/"
                on:click=move |_| on_close.run(())
                aria-current=move || if is_active("/")() { "page" } else { "" }
            >
                {t!(i18n, nav_home)}
            </a>
            <Show when=move || pathname.get().starts_with("/admin")>
                <a
                    href="/admin"
                    on:click=move |_| on_close.run(())
                    aria-current=move || if is_active("/admin")() { "page" } else { "" }
                >
                    {t!(i18n, admin_nav_dashboard)}
                </a>
                <a
                    href="/admin/season"
                    on:click=move |_| on_close.run(())
                    aria-current=move || if is_active("/admin/season")() { "page" } else { "" }
                >
                    {t!(i18n, admin_nav_season)}
                </a>
                <a
                    href="/admin/participants"
                    on:click=move |_| on_close.run(())
                    aria-current=move || if is_active("/admin/participants")() { "page" } else { "" }
                >
                    {t!(i18n, admin_nav_participants)}
                </a>
                <a
                    href="/admin/assignments"
                    on:click=move |_| on_close.run(())
                    aria-current=move || if is_active("/admin/assignments")() { "page" } else { "" }
                >
                    {t!(i18n, admin_nav_assignments)}
                </a>
                <a
                    href="/admin/sms"
                    on:click=move |_| on_close.run(())
                    aria-current=move || if is_active("/admin/sms")() { "page" } else { "" }
                >
                    {t!(i18n, admin_nav_sms)}
                </a>
            </Show>
            <Show when=move || pathname.get() != "/login" && pathname.get() != "/onboarding">
                <leptos::form::ActionForm action=logout_action>
                    <button
                        type="submit"
                        class="btn w-full"
                        data-variant="secondary"
                        data-size="sm"
                        data-testid="logout-button-mobile"
                        disabled=move || !hydrated.get()
                    >
                        {t!(i18n, nav_logout)}
                    </button>
                </leptos::form::ActionForm>
            </Show>
        </nav>
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
                current_user
                    .get()
                    .map(|result| {
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
                current_user
                    .get()
                    .map(|result| {
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
