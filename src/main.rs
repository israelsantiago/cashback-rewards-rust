use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cashback_rewards_rust::{
    adapter::{
        r#in::web::{self, WebState},
        out::persistence::{PgCashbackRepository, PgCategoryRepository, PgMerchantRepository},
    },
    application::{
        port::r#in::{
            ListCustomerCashbackUseCase, ManageProductCategoriesUseCase, RecordPurchaseUseCase,
            RegisterMerchantUseCase, TotalProductCashbackUseCase,
        },
        service::{
            ListCustomerCashbackService, ManageProductCategoriesService, RecordPurchaseService,
            RegisterMerchantService, TotalProductCashbackService,
        },
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cashback_rewards_rust=info,tower_http=info".into()),
        )
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

    // Outbound adapters: infrastructure implementations of the application's ports.
    let merchants = Arc::new(PgMerchantRepository::new(pool.clone()));
    let categories = Arc::new(PgCategoryRepository::new(pool.clone()));
    let cashbacks = Arc::new(PgCashbackRepository::new(pool));

    // Application services: implementations of inbound use-case ports.
    let list_cashback: Arc<dyn ListCustomerCashbackUseCase> =
        Arc::new(ListCustomerCashbackService::new(cashbacks.clone()));
    let manage_categories: Arc<dyn ManageProductCategoriesUseCase> =
        Arc::new(ManageProductCategoriesService::new(categories.clone()));
    let record_purchase: Arc<dyn RecordPurchaseUseCase> = Arc::new(RecordPurchaseService::new(
        merchants.clone(),
        categories,
        cashbacks.clone(),
    ));
    let register_merchant: Arc<dyn RegisterMerchantUseCase> =
        Arc::new(RegisterMerchantService::new(merchants));
    let total_product_cashback: Arc<dyn TotalProductCashbackUseCase> =
        Arc::new(TotalProductCashbackService::new(cashbacks));

    // Inbound adapter state contains only inbound ports, not concrete services.
    let state = WebState::new(
        list_cashback,
        manage_categories,
        record_purchase,
        register_merchant,
        total_product_cashback,
    );

    let app: Router = web::router(state);
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    info!(%bind_addr, "cashback rewards API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

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
