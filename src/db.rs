use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Create a Postgres connection pool.
///
/// # Errors
///
/// Returns `Err` if the connection cannot be established.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(5))
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
