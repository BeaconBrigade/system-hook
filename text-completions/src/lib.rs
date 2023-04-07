//! # text-completion
//!
//! This crate provides tab completions for the [dialoguer](https://docs.rs/dialoguer)

use dialoguer::Completion;
use envmnt::{ExpandOptions, ExpansionType};
use std::{fs, path::PathBuf};

/// [`MultiCompletion`] combines multiple completions
#[derive(Default)]
pub struct MultiCompletion {
    completions: Vec<Box<dyn Completion>>,
}

impl Completion for MultiCompletion {
    fn get(&self, input: &str) -> Option<String> {
        let res = self
            .completions
            .iter()
            .fold(input.to_string(), |i, c| c.get(&i).unwrap_or(i));

        Some(res)
    }
}

impl MultiCompletion {
    pub fn new(completions: Vec<Box<dyn Completion>>) -> Self {
        Self { completions }
    }

    pub fn with<C: Completion + 'static>(mut self, new: C) -> Self {
        self.completions.push(Box::new(new));
        self
    }
}

/// [`EnvCompletion`] will automatically expand environment variables using [`envmnt::expand`]
#[derive(Debug, Clone)]
pub struct EnvCompletion {
    opts: ExpandOptions,
}

impl Default for EnvCompletion {
    fn default() -> Self {
        let mut opts = ExpandOptions::new();
        opts.expansion_type = Some(ExpansionType::Unix);

        Self { opts }
    }
}

impl Completion for EnvCompletion {
    fn get(&self, input: &str) -> Option<String> {
        let expanded = envmnt::expand(input, Some(self.opts));

        Some(expanded)
    }
}

/// [`PathCompletion`] will automatically complete paths when possible
#[derive(Debug, Default, Clone)]
pub struct PathCompletion;

impl Completion for PathCompletion {
    fn get(&self, input: &str) -> Option<String> {
        let mut path = PathBuf::from(input);

        if path.is_dir() {
            return if input.ends_with('/') {
                None
            } else {
                let mut str = input.to_string();
                str.push('/');
                Some(str)
            };
        }

        let parent = path.parent().unwrap_or(&path);
        if !parent.try_exists().ok()? {
            return None;
        }

        let base_name = path.file_name()?.to_str()?;
        let dir = fs::read_dir(parent).ok()?;

        for ent in dir.into_iter() {
            let ent = ent.ok()?;
            let Some(mut name) = ent.file_name().to_str().map(|s| s.to_owned()) else {
                continue;
            };
            if ent.path().is_dir() {
                name.push('/');
            }

            if name.starts_with(base_name) {
                path.set_file_name(name);
                break;
            }
        }

        path.to_str().map(|s| s.to_owned())
    }
}
