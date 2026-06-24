use leptos::prelude::*;

#[component]
pub fn SkeletonFallback() -> impl IntoView {
    view! {
        <div aria-hidden="true" class="flex flex-col gap-3">
            <div class="skeleton-line h-4 w-3/4"></div>
            <div class="skeleton-line h-4 w-1/2"></div>
            <div class="skeleton-line h-4 w-5/8"></div>
        </div>
    }
}
