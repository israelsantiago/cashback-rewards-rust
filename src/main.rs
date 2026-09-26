use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cashback_rewards_rust::{
    adapters::persistence::{PgCashbackRepository, PgCategoryRepository, PgMerchantRepository},
    application::{service::{ListCustomerCashbackService, ManageProductCategoriesService, RecordPurchaseService, RegisterMerchantService, TotalProductCashbackService}, ApplicationState},
    web,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "cashback_rewards_rust=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL")?;
    let bind_addr: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let merchants = Arc::new(PgMerchantRepository::new(pool.clone()));
    let categories = Arc::new(PgCategoryRepository::new(pool.clone()));
    let cashbacks = Arc::new(PgCashbackRepository::new(pool));

    let state = ApplicationState::new(
        ListCustomerCashbackService::new(cashbacks.clone()),
        ManageProductCategoriesService::new(categories.clone()),
        RecordPurchaseService::new(merchants.clone(), categories, cashbacks.clone()),
        RegisterMerchantService::new(merchants),
        TotalProductCashbackService::new(cashbacks),
    );

    let app: Router = web::router(state);
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    info!(%bind_addr, "cashback rewards API listening");
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
