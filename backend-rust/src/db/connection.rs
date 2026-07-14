use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::str::FromStr;
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> PgPool {
    let min_connections = std::env::var("DATABASE_MIN_CONNECTIONS")
        .ok()
        .and_then(|val| val.parse::<u32>().ok())
        .unwrap_or(5);

    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|val| val.parse::<u32>().ok())
        .unwrap_or(50);

    let acquire_timeout_secs = std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECS")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(10);

    tracing::info!(
        "Connecting to database pool with min_connections={}, max_connections={}, acquire_timeout_secs={}s...",
        min_connections,
        max_connections,
        acquire_timeout_secs
    );

    let connect_options = PgConnectOptions::from_str(database_url)
        .unwrap_or_else(|err| {
            tracing::error!("Invalid database URL: {:?}", err);
            panic!("Invalid database URL: {:?}", err);
        })
        .application_name("zafaf_rust");

    PgPoolOptions::new()
        .min_connections(min_connections)
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
        .idle_timeout(Some(Duration::from_secs(600)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .test_before_acquire(false) // Optimized for PgBouncer
        .connect_with(connect_options)
        .await
        .unwrap_or_else(|err| {
            tracing::error!(
                "CRITICAL: Failed to connect to database at {}: {:?}",
                database_url,
                err
            );
            panic!("Critical Database Connection Failure: {:?}", err);
        })
}
