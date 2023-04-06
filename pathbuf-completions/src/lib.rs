//! # pathbuf-completion
//! 
//! This crate provides pathbuf tab completions for the [dialoguer](https://docs.rs/dialoguer)

use std::path::PathBuf;
use dialoguer::Completion;
use envmnt::{ExpandOptions, ExpansionType};

/// PathBuf completions. This will automatically expand environment variables using [`envmnt::expand`]
/// and then canocicalize paths.
#[derive(Debug, Clone)]
pub struct PathBufCompletion {
    opts: ExpandOptions,
}

impl Default for PathBufCompletion {
    fn default() -> Self {
        let mut opts = ExpandOptions::new();
        opts.expansion_type = Some(ExpansionType::Unix);

        Self { opts }
    }
}

impl Completion for PathBufCompletion {
    fn get(&self, input: &str) -> Option<String> {
        let expanded = envmnt::expand(input, Some(self.opts));

        let path = PathBuf::from(&expanded);

        let path = path.canonicalize().unwrap_or(path);

        Some(path.to_str()?.to_owned())
    }
}