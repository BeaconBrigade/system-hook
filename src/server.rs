use axum::{debug_handler, routing::get, Router};

use crate::config::Serve;

pub async fn serve(args: Serve) -> color_eyre::Result<()> {
    tracing::info!("serving project");
    tracing::debug!("{:#?}", args);

    let app = Router::new().route("/", get(handler));
    
    tracing::info!("serving on {}", args.addr);
    axum::Server::bind(&args.addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[debug_handler]
async fn handler() -> &'static str {
    tracing::info!("serving index");

    "Hello, World!"
}
