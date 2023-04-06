use std::{path::PathBuf, str::FromStr};

use color_eyre::eyre::eyre;
use dialoguer::{theme::ColorfulTheme, Input};
use github_webhook::EventDiscriminants;

use crate::config::{parse_multiple_events, Init, InitConfig};

pub fn init_project(args: Init) -> color_eyre::Result<()> {
    tracing::info!("creating project");

    // TODO: better handling of bad inputs - ask the user to retry instead of ending the program
    let repo_path = get_input("path to the repository", args.repo_path.map(PathBufWrapper))?.0;
    let config_path = get_input(
        "path to create `shook.toml`",
        args.config_path.map(PathBufWrapper),
    )?
    .0;
    let system_name = get_input(
        "name of systemd service to update on github events",
        args.system_name,
    )?;
    let update_events = get_input_events("github events to update", args.update_events)?;
    let addr = get_input(
        "address to serve on (unix socket path or tcp address)",
        args.addr,
    )?;

    let config = InitConfig {
        repo_path,
        config_path,
        system_name,
        update_events,
        addr,
    };

    tracing::debug!(?config);

    Ok(())
}

#[derive(Debug, Clone)]
struct PathBufWrapper(PathBuf);

impl FromStr for PathBufWrapper {
    type Err = <PathBuf as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(PathBuf::from_str(s)?))
    }
}

impl ToString for PathBufWrapper {
    fn to_string(&self) -> String {
        format!("{}", self.0.to_string_lossy())
    }
}

fn get_input<T>(prompt: &str, initial: Option<T>) -> color_eyre::Result<T>
where
    T: Clone + ToString + FromStr,
    <T as FromStr>::Err: std::fmt::Debug + ToString,
{
    if let Some(v) = initial {
        return Ok(v);
    }
    let res = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()?;

    Ok(res)
}

fn get_input_events(
    prompt: &str,
    initial: Option<Vec<EventDiscriminants>>,
) -> color_eyre::Result<Vec<EventDiscriminants>> {
    if let Some(v) = initial {
        return Ok(v);
    }
    let str: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default("push".into())
        .interact_text()?;

    let res = parse_multiple_events(&str).map_err(|e| eyre!(e))?;

    Ok(res)
}
