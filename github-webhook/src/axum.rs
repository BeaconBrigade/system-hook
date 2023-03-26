use std::{collections::HashMap, str::FromStr};

use axum::{
    async_trait,
    body::Body,
    extract::FromRequest,
    http::{self, StatusCode},
    Form, Json,
};
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;

use crate::{Event, GithubPayload};

/// Extract a github event from a request
#[async_trait]
impl<S> FromRequest<S, Body> for GithubPayload
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    #[instrument(skip(req, state))]
    async fn from_request(req: http::Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        // get information from headers
        let event: String = req
            .headers()
            .get("X-Github-Event")
            .map(|v| v.to_str().map_err(|_| StatusCode::BAD_REQUEST))
            .ok_or(StatusCode::BAD_REQUEST)??
            .to_string();
        tracing::debug!(?event);
        let guid = req
            .headers()
            .get("X-Github-Delivery")
            .map(|v| {
                v.to_str()
                    .map(|s| Uuid::from_str(s).map_err(|_| StatusCode::BAD_REQUEST))
                    .map_err(|_| StatusCode::BAD_REQUEST)
            })
            .ok_or(StatusCode::BAD_REQUEST)???;
        tracing::debug!(?guid);
        let signature_sha1: Option<Vec<_>> = req
            .headers()
            .get("X-Hub-Signature")
            .map(|v| v.as_bytes().into_iter().copied().collect());
        tracing::debug!(?signature_sha1);
        let signature_sha256: Option<Vec<_>> = req
            .headers()
            .get("X-Hub-Signature-256")
            .map(|v| v.as_bytes().into_iter().copied().collect());
        tracing::debug!(?signature_sha256);

        // get body
        let content_type = req
            .headers()
            .get("content-type")
            .map(|v| {
                v.to_str()
                    .map(|s| s.to_lowercase())
                    .map_err(|_| StatusCode::BAD_REQUEST)
            })
            .ok_or(StatusCode::BAD_REQUEST)??;
        tracing::debug!(?content_type);
        let json = if content_type == "application/json" {
            let Json(json): Json<serde_json::Value> = Json::from_request(req, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            serde_json::to_string(&json!({ event: json })).map_err(|_| StatusCode::BAD_REQUEST)?
        } else {
            let Form(form): Form<HashMap<String, String>> = Form::from_request(req, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            serde_json::to_string(&form).map_err(|_| StatusCode::BAD_REQUEST)?
        };
        let deserializer = &mut serde_json::Deserializer::from_str(&json);
        let event: Event = serde_path_to_error::deserialize(deserializer).map_err(|e| {
            tracing::error!("failed to deserialize event: {}", e);
            StatusCode::BAD_REQUEST
        })?;

        tracing::debug!(?event);

        Ok(Self {
            guid,
            signature_sha1,
            signature_sha256,
            event,
        })
    }
}
