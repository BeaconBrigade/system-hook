# `github-webhook-extract`

This library provides a way to parse and verify Github webhook payloads. The core of the
verification is provided in the `verify` function. A web framework integration (currently
only supports [axum](https://github.com/tokio-rs/axum) will provide the required information
and the `verify` function will parse and check the payload.
