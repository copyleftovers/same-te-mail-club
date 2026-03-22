use crate::i18n::i18n::{t, use_i18n};
use crate::pages::login::Logout;
use leptos::prelude::*;

/// Admin navigation bar with active-page indicator.
///
/// Uses `aria-current="page"` on the current route's link.
/// CSS in `style/tailwind.css` (`@layer components`) styles the active link.
#[component]
pub fn AdminNav() -> impl IntoView {
    use crate::hooks::use_hydrated;
    let i18n = use_i18n();
    let location = leptos_router::hooks::use_location();
    let hydrated = use_hydrated();
    let logout_action =
        use_context::<ServerAction<Logout>>().expect("logout action must be provided");

    let is_active = move |path: &'static str| move || location.pathname.get() == path;

    view! {
        <nav class="admin-nav">
            <a href="/admin" aria-current=move || if is_active("/admin")() { "page" } else { "" }>
                {t!(i18n, admin_nav_dashboard)}
            </a>
            <a
                href="/admin/season"
                aria-current=move || if is_active("/admin/season")() { "page" } else { "" }
            >
                {t!(i18n, admin_nav_season)}
            </a>
            <a
                href="/admin/participants"
                aria-current=move || if is_active("/admin/participants")() { "page" } else { "" }
            >
                {t!(i18n, admin_nav_participants)}
            </a>
            <a
                href="/admin/assignments"
                aria-current=move || if is_active("/admin/assignments")() { "page" } else { "" }
            >
                {t!(i18n, admin_nav_assignments)}
            </a>
            <a
                href="/admin/sms"
                aria-current=move || if is_active("/admin/sms")() { "page" } else { "" }
            >
                {t!(i18n, admin_nav_sms)}
            </a>
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
        </nav>
    }
}
