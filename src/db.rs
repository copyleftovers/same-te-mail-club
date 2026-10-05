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
/// existing user with that phone to an active admin (a deactivated account is
/// reactivated, otherwise the organizer could not sign in). Idempotent; runs at
/// every boot.
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
        ON CONFLICT (phone) DO UPDATE SET role = 'admin', status = 'active'
        "#,
        admin.phone,
        admin.name,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_admin;
    use crate::config::AdminBootstrap;
    use sqlx::PgPool;

    const ADMIN_PHONE: &str = "+380671234567";

    fn bootstrap() -> AdminBootstrap {
        AdminBootstrap {
            phone: ADMIN_PHONE.into(),
            name: "Організатор".into(),
        }
    }

    async fn active_admin_count(pool: &PgPool) -> i64 {
        sqlx::query_scalar(
            "SELECT count(*) FROM users WHERE phone = $1 AND role = 'admin' AND status = 'active'",
        )
        .bind(ADMIN_PHONE)
        .fetch_one(pool)
        .await
        .expect("count query")
    }

    #[sqlx::test]
    #[ignore = "needs live Postgres (DATABASE_URL)"]
    async fn inserts_missing_admin_and_is_idempotent(pool: PgPool) {
        ensure_admin(&pool, &bootstrap()).await.expect("first boot");
        ensure_admin(&pool, &bootstrap())
            .await
            .expect("second boot");

        assert_eq!(active_admin_count(&pool).await, 1);
    }

    #[sqlx::test]
    #[ignore = "needs live Postgres (DATABASE_URL)"]
    async fn promotes_and_reactivates_existing_deactivated_participant(pool: PgPool) {
        sqlx::query(
            "INSERT INTO users (phone, name, role, status) VALUES ($1, 'Учасник', 'participant', 'deactivated')",
        )
        .bind(ADMIN_PHONE)
        .execute(&pool)
        .await
        .expect("seed participant");

        ensure_admin(&pool, &bootstrap()).await.expect("bootstrap");

        assert_eq!(active_admin_count(&pool).await, 1);
    }
}
