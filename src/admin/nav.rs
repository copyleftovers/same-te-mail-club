use crate::i18n::i18n::{t, use_i18n};
use leptos::prelude::*;

/// Admin navigation bar with active-page indicator.
///
/// Uses `aria-current="page"` on the current route's link.
/// CSS in `style/tailwind.css` (`@layer components`) styles the active link.
#[component]
pub fn AdminNav() -> impl IntoView {
    let i18n = use_i18n();
    let location = leptos_router::hooks::use_location();

    let is_active = move |path: &'static str| move || location.pathname.get() == path;

    view! {
        <nav class="admin-nav">
            <a
                href="/admin"
                aria-current=move || if is_active("/admin")() { "page" } else { "" }
            >
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
        </nav>
    }
}
