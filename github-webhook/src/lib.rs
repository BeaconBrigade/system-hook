//! # Github Webhook
//!
//! Contains types for github webhooks
//!
//! ## Features
//!
//! * axum: enable the axum feature to get extractor implementations

#[cfg(feature = "axum")]
mod axum;
mod types;

use std::env;
use bytes::Bytes;
pub use types::*;

use digest::CtOutput;
use generic_array::GenericArray;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::Sha256;
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

/// Verify and parse a github payload. Pass your parsed
/// data from your web library for parsing and return errors.
///
/// ## fields:
/// * `guid`: value of `X-GitHub-Delivery` header
/// * `signature_sha1`: value of `X-Hub-Signature` header
/// * `signature_sha256`: value of `X-Hub-Signature-256` header
/// * `bytes`: raw body of the request
/// * `json`: body of the request in json form
#[instrument(skip_all)]
pub fn verify(
    guid: Uuid,
    signature_sha1: Option<String>,
    signature_sha256: Option<String>,
    bytes: Bytes,
    json: &str,
) -> Result<GithubPayload, VerifyError> {
    // verify signatures
    match (&signature_sha1, &signature_sha256) {
        (Some(sha1), None) => {
            tracing::debug!("using sha1");
            let token = env::var("GITHUB_TOKEN").map_err(|_| {
                tracing::error!("secret github token is missing");
                VerifyError::TokenMissing
            })?;

            let mut mac = Hmac::<Sha1>::new_from_slice(token.as_bytes()).map_err(|e| {
                tracing::error!("error creating hmac: {:?}", e);
                VerifyError::HmacCreation
            })?;
            mac.update(&bytes);
            let result = mac.finalize();
            let signature = hex::decode(
                sha1.split_once('=').ok_or(VerifyError::Sha1ParseError)?.1,
            )
                .map_err(|e| {
                    tracing::debug!(?e);
                    VerifyError::HexParseError
                })?;

            if result != CtOutput::new(*GenericArray::from_slice(&signature)) {
                return Err(VerifyError::NotVerified);
            }
        }
        (_, Some(sha256)) => {
            tracing::debug!("using sha256");
            let token = env::var("GITHUB_TOKEN").map_err(|_| {
                tracing::error!("secret github token is missing");
                VerifyError::TokenMissing
            })?;

            let mut mac = Hmac::<Sha256>::new_from_slice(token.as_bytes()).map_err(|e| {
                tracing::error!("error creating hmac: {:?}", e);
                VerifyError::HmacCreation
            })?;
            mac.update(&bytes);
            let result = mac.finalize();
            let signature =
                hex::decode(sha256.split_once('=').ok_or(VerifyError::Sha256ParseError)?.1)
                    .map_err(|e| {
                        tracing::debug!(?e);
                        VerifyError::HexParseError
                    })?;

            if result != CtOutput::new(*GenericArray::from_slice(&signature)) {
                return Err(VerifyError::NotVerified);
            }
        }
        (None, None) => tracing::debug!("no signature verification"),
    }

    let deserializer = &mut serde_json::Deserializer::from_str(json);
    let event: Event = serde_path_to_error::deserialize(deserializer).map_err(|e| {
        tracing::warn!("failed to deserialize event: {}", e);
        VerifyError::EventParseError
    })?;

    tracing::debug!("finished extracting github payload");
    Ok(GithubPayload {
        guid,
        signature_sha1,
        signature_sha256,
        event,
    })
}

/// Error verifying a github payload
#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("github token not found in environment")]
    TokenMissing,
    #[error("could not create hmac")]
    HmacCreation,
    #[error("could not parse sha1 header")]
    Sha1ParseError,
    #[error("could not parse sha256 header")]
    Sha256ParseError,
    #[error("could not parse hex signature")]
    HexParseError,
    #[error("payload not verified correctly")]
    NotVerified,
    #[error("could not parse event")]
    EventParseError,
}
