use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Create a Postgres connection pool.
///
/// `acquire_timeout` is set to 5 s (well below Playwright's 30 s
/// `navigationTimeout`) so that transient pool contention surfaces as a fast
/// error rather than a silent 30 s SSR Suspense stall.
///
/// # Errors
///
/// Returns `Err` if the connection cannot be established.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}

/// Run all pending migrations.
///
/// # Errors
///
/// Returns `Err` if a migration fails.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

/// Ensure the configured first admin exists: insert it, or promote the
/// existing user with that phone to admin. Idempotent; runs at every boot.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn ensure_admin(
    pool: &PgPool,
    admin: &crate::config::AdminBootstrap,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (phone, name, role, onboarded)
        VALUES ($1, $2, 'admin', true)
        ON CONFLICT (phone) DO UPDATE SET role = 'admin'
        "#,
        admin.phone,
        admin.name,
    )
    .execute(pool)
    .await?;
    Ok(())
}
