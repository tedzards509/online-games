use arena_server::{App, api};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arena_server=info,tower_http=info".into()),
        )
        .init();
    if std::env::args().any(|arg| arg == "--openapi") {
        println!("{}", api::openapi().to_pretty_json()?);
        return Ok(());
    }
    let database = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://arena.db".into());
    let origin = std::env::var("PUBLIC_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".into());
    let address = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let app = App::open(&database, &origin).await?;
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(%address, "listening");
    axum::serve(listener, api::router(app))
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install signal handler");
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
