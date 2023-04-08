use std::{
    path::PathBuf,
    pin::Pin,
    process::Command,
    task::{Context, Poll},
};

use axum::{debug_handler, extract::State, routing::post, Router};
use color_eyre::eyre::{eyre, Context as _};
use futures::ready;
use github_webhook::GithubPayload;
use hyper::{server::accept::Accept, StatusCode};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    net::{UnixListener, UnixStream},
};
use tower_http::{trace::TraceLayer, BoxError};
use tracing::instrument;

use crate::config::{Serve, ServerConfig, TcpOrUnix};

pub async fn serve(args: Serve) -> color_eyre::Result<()> {
    tracing::info!("serving project");

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

    let app = Router::new()
        .route("/", post(handler))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            config: config.clone(),
        });

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

/// state of the app
#[derive(Clone)]
pub struct AppState {
    pub config: ServerConfig,
}

#[debug_handler]
#[instrument(skip_all)]
async fn handler(State(state): State<AppState>, payload: GithubPayload) -> Result<(), StatusCode> {
    let event = payload.event.into();
    if !state.config.update_events.contains(&event) {
        tracing::debug!("event not matched: {:?}", &event);
        return Ok(());
    }
    tracing::debug!("event matched");

    // pull repository
    pull_updates(&state).map_err(|e| {
        tracing::error!("failed to pull changes: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    restart_service(&state).map_err(|e| {
        tracing::error!("failed to restart service: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("restarted system");

    Ok(())
}

/// pull code updates
#[instrument(skip_all)]
fn pull_updates(state: &AppState) -> color_eyre::Result<()> {
    tracing::info!(
        "pulling changes from {} {}",
        state.config.remote,
        state.config.branch
    );
    let mut handle = Command::new("su")
        .arg(&state.config.username)
        .arg("-c")
        .arg(format!(
            "git pull '{}' '{}'",
            state.config.remote, state.config.branch
        ))
        .current_dir(&state.config.repo_path)
        .env("GIT_TERMINAL_PROMPT", "0")
        .spawn()?;

    let status = handle.wait()?;
    tracing::info!(
        "tracing finished with exit code {:?}",
        status
            .code()
            .map(|s| s.to_string())
            .unwrap_or("<terminated by signal>".to_string())
    );
    if status.code().unwrap_or(1) != 0 {
        tracing::error!("git finished with error");
        return Err(eyre!("git finished with non zero exit code"));
    }

    Ok(())
}

/// restart the systemd service which code we are watching
#[instrument(skip_all)]
fn restart_service(state: &AppState) -> color_eyre::Result<()> {
    // restart systemd service
    let mut handle = Command::new("systemctl")
        .arg("restart")
        .arg(&state.config.system_name)
        .spawn()
        .map_err(|e| eyre!("could not spawn systemctl: {e}"))?;

    let status = handle
        .wait()
        .map_err(|e| eyre!("error waiting for systemctl: {e}"))?;
    tracing::info!(
        "tracing finished with exit code {:?}",
        status
            .code()
            .map(|s| s.to_string())
            .unwrap_or("<terminated by signal>".to_string())
    );
    if status.code().unwrap_or(1) != 0 {
        return Err(eyre!("systemctl finished with error"));
    }

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
