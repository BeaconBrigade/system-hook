use axum::{debug_handler, routing::post, Router};
use github_webhook::GithubPayload;
use tower_http::trace::TraceLayer;

use crate::config::Serve;

pub async fn serve(args: Serve) -> color_eyre::Result<()> {
    tracing::info!("serving project");
    tracing::debug!("{:#?}", args);

    let app = Router::new()
        .route("/", post(handler))
        .layer(TraceLayer::new_for_http());

    tracing::info!("serving on {}", args.addr);
    axum::Server::bind(&args.addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[debug_handler]
async fn handler(_payload: GithubPayload) -> &'static str {
    tracing::info!("serving index");

    "Hello, World!"
}
