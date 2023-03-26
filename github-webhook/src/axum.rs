use std::{env, str::FromStr};

use axum::{
    async_trait,
    body::{Body, Bytes},
    extract::FromRequest,
    http::{self, StatusCode},
};
use digest::CtOutput;
use generic_array::GenericArray;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha1::Sha1;
use sha2::Sha256;
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
        tracing::info!("parsing event: {}", event);
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
        let signature_sha1: Option<String> = req
            .headers()
            .get("X-Hub-Signature")
            .map(|v| {
                v.to_str()
                    .map(|s| s.to_lowercase())
                    .map_err(|_| StatusCode::BAD_REQUEST)
            })
            .transpose()?;
        let signature_sha256: Option<String> = req
            .headers()
            .get("X-Hub-Signature-256")
            .map(|v| {
                v.to_str()
                    .map(|s| s.to_lowercase())
                    .map_err(|_| StatusCode::BAD_REQUEST)
            })
            .transpose()?;

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

        // verify signatures
        match (&signature_sha1, &signature_sha256) {
            (Some(sha1), None) => {
                tracing::debug!("using sha1");
                let token = env::var("GITHUB_TOKEN").map_err(|_| {
                    tracing::error!("secret github token is missing");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                let mut mac = Hmac::<Sha1>::new_from_slice(token.as_bytes()).map_err(|e| {
                    tracing::error!("error creating hmac: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                mac.update(&raw_bytes);
                let result = mac.finalize();
                let signature = hex::decode(&sha1.split_once('=').ok_or(StatusCode::BAD_REQUEST)?.1)
                    .map_err(|e| {
                        tracing::debug!(?e);
                        StatusCode::BAD_REQUEST
                    })?;

                if result != CtOutput::new(*GenericArray::from_slice(&signature)) {
                    return Err(StatusCode::FORBIDDEN);
                }
            }
            (_, Some(sha256)) => {
                tracing::debug!("using sha256");
                let token = env::var("GITHUB_TOKEN").map_err(|_| {
                    tracing::error!("secret github token is missing");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                let mut mac = Hmac::<Sha256>::new_from_slice(token.as_bytes()).map_err(|e| {
                    tracing::error!("error creating hmac: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                mac.update(&raw_bytes);
                let result = mac.finalize();
                let signature = hex::decode(&sha256.split_once('=').ok_or(StatusCode::BAD_REQUEST)?.1)
                    .map_err(|e| {
                        tracing::debug!(?e);
                        StatusCode::BAD_REQUEST
                    })?;

                if result != CtOutput::new(*GenericArray::from_slice(&signature)) {
                    return Err(StatusCode::FORBIDDEN);
                }
            }
            (None, None) => tracing::debug!("no signature verification"),
        }

        let deserializer = &mut serde_json::Deserializer::from_str(&json);
        let event: Event = serde_path_to_error::deserialize(deserializer).map_err(|e| {
            tracing::error!("failed to deserialize event: {}", e);
            StatusCode::BAD_REQUEST
        })?;

        tracing::debug!("finished extracing github payload");
        Ok(Self {
            guid,
            signature_sha1,
            signature_sha256,
            event,
        })
    }
}
