use std::{
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{debug_handler, routing::post, Router};
use color_eyre::eyre::Context as _;
use futures::ready;
use github_webhook::GithubPayload;
use hyper::server::accept::Accept;
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    net::{UnixListener, UnixStream},
};
use tower_http::{trace::TraceLayer, BoxError};

use crate::config::{Serve, ServerConfig, TcpOrUnix};

pub async fn serve(args: Serve) -> color_eyre::Result<()> {
    tracing::info!("serving project");
    tracing::debug!("{:#?}", args);

    let app = Router::new()
        .route("/", post(handler))
        .layer(TraceLayer::new_for_http());

    let mut config: ServerConfig = {
        let config_path = args
            .repo_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("shook.toml");
        let mut file = File::open(&config_path)
            .await
            .context("opening shook config")?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .await
            .context("reading shook config")?;
        toml::from_str(&buf).context("parsing shook config")?
    };
    config.merge(args);

    tracing::info!("serving on {}", config.addr.to_string());
    match config.addr {
        TcpOrUnix::Unix(path) => {
            let _ = fs::remove_file(&path).await;
            fs::create_dir_all(path.parent().unwrap()).await.unwrap();

            let uds = UnixListener::bind(path.clone()).unwrap();

            axum::Server::builder(ServerAccept { uds })
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        TcpOrUnix::Tcp(socket) => {
            axum::Server::bind(&socket)
                .serve(app.into_make_service())
                .await?;
        }
    };

    Ok(())
}

#[derive(Debug)]
struct ServerAccept {
    uds: UnixListener,
}

impl Accept for ServerAccept {
    type Conn = UnixStream;
    type Error = BoxError;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let (stream, _addr) = ready!(self.uds.poll_accept(cx))?;
        Poll::Ready(Some(Ok(stream)))
    }
}

#[debug_handler]
async fn handler(_payload: GithubPayload) -> &'static str {
    tracing::info!("serving index");

    "Hello, World!"
}
