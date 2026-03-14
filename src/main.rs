#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::get_configuration;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use samete::app::{App, shell};
    use tower_http::compression::CompressionLayer;
    use tower_http::trace::TraceLayer;
    use tracing_subscriber::prelude::*;

    // 1. Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "samete=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Config
    let config = samete::config::Config::from_env().expect("configuration error");

    // 3. Database
    let pool = samete::db::create_pool(&config.database_url)
        .await
        .expect("database connection failed");
    samete::db::run_migrations(&pool)
        .await
        .expect("migrations failed");

    // 4. Leptos
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // 5. Router with context injection
    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                let config = config.clone();
                move || {
                    leptos::context::provide_context(pool.clone());
                    leptos::context::provide_context(config.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(leptos_options);

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
