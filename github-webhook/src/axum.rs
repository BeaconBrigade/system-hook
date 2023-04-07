use std::str::FromStr;

use axum::{
    async_trait,
    body::{Body, Bytes},
    extract::FromRequest,
    http::{self, StatusCode},
};
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;

use crate::GithubPayload;

/// Extract a github event from a request
#[async_trait]
impl<S> FromRequest<S, Body> for GithubPayload
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    #[instrument(skip_all)]
    async fn from_request(req: http::Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        // get information from headers
        let event: String = req
            .headers()
            .get("X-Github-Event")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();
        tracing::info!("parsing event: {}", event);
        let guid = req
            .headers()
            .get("X-Github-Delivery")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::from_str(s).ok())
            .ok_or(StatusCode::BAD_REQUEST)?;
        tracing::debug!(?guid);
        let signature_sha1: Option<String> = req
            .headers()
            .get("X-Hub-Signature")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase());
        let signature_sha256: Option<String> = req
            .headers()
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase());

        // get body
        let content_type = req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase())
            .ok_or(StatusCode::BAD_REQUEST)?;
        tracing::debug!(?content_type);
        let raw_bytes = Bytes::from_request(req, state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let json = if content_type == "application/json" {
            let json: serde_json::Value = serde_json::from_slice(&raw_bytes).map_err(|e| {
                tracing::error!(?e);
                StatusCode::BAD_REQUEST
            })?;
            serde_json::to_string(&json!({ event: json })).map_err(|_| StatusCode::BAD_REQUEST)?
        } else {
            let form: serde_json::Value =
                serde_urlencoded::from_bytes(&raw_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
            serde_json::to_string(&json!({ event: form })).map_err(|_| StatusCode::BAD_REQUEST)?
        };

        let res = crate::verify(guid, signature_sha1, signature_sha256, raw_bytes, &json);
        use crate::VerifyError::*;
        match res {
            Ok(payload) => Ok(payload),
            Err(TokenMissing | HmacCreation) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            Err(Sha1ParseError | Sha256ParseError | HexParseError | EventParseError) => {
                Err(StatusCode::BAD_REQUEST)
            }
            Err(NotVerified) => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
