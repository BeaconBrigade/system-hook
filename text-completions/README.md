# `text-completions`

This library provides implementations of the [`Completion`](https://docs.rs/dialoguer/latest/dialoguer/trait.Completion.html)
trait. It contains the `EnvCompletion` which autocompletes environment variables typed in
the unix style, the `PathCompletion` which completes file system paths and `MultiCompletion`
which combines multiple `Completion`s together.
