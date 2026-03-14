use sqlx::PgPool;

/// Create a Postgres connection pool.
///
/// # Errors
///
/// Returns `Err` if the connection cannot be established.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(database_url).await
}

/// Run all pending migrations.
///
/// # Errors
///
/// Returns `Err` if a migration fails.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}
