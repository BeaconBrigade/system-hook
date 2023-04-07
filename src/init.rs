use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr, io::Write,
};

use color_eyre::eyre::{eyre, Context};
use dialoguer::{theme::ColorfulTheme, Completion, Confirm, Input};
use github_webhook::EventDiscriminants;
use text_completions::{EnvCompletion, MultiCompletion, PathCompletion};

use crate::config::{parse_multiple_events, Init, InitConfig};

pub fn init_project(args: Init) -> color_eyre::Result<()> {
    tracing::info!("creating project");

    let completion = MultiCompletion::default()
        .with(EnvCompletion::default())
        .with(PathCompletion::default());
    let repo_path = get_input_pathbuf("path to the repository", args.repo_path, &completion)?;
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
        system_name,
        update_events,
        addr,
    };

    tracing::debug!(?config);

    if !Path::try_exists(&config.repo_path)? {
        tracing::warn!("repository could not be found");

        let should_clone = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Clone the repository?")
            .interact()?;

        if should_clone {
            let url: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Repository url")
                .interact_text()?;

            tracing::info!("cloning repository into {:?}", config.repo_path);
            let parent = config
                .repo_path
                .parent()
                .ok_or_else(|| eyre!("repo-path has no parent"))?;
            if !Path::try_exists(parent)? {
                tracing::info!("creating {:?}", parent);
                fs::create_dir_all(parent)?;
            }

            let mut handle = Command::new("git")
                .arg("clone")
                .arg(url)
                .arg(config.repo_path.file_name().unwrap())
                .current_dir(parent)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let exit_code = handle.wait()?;
            tracing::debug!("git exited with exit code {exit_code}");
        }
    }

    let toml = toml::to_string_pretty(&config).context("serializing config to toml")?;

    let config_path = config.repo_path.join("shook.toml");
    if Path::exists(&config_path) {
        tracing::warn!("{:?} already exists", config_path);

        let should_replace = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Replace existing shook.toml?")
            .interact()?;
        if !should_replace {
            tracing::info!("aborting init process");
            return Err(eyre!("aborting due to existing config"));
        }
    }

    let mut file = File::create(&config_path).context("creating shook.toml")?;
    file.write_all(toml.as_bytes())?;
    tracing::info!("finished writing shook.toml");

    tracing::info!("finished creating project");

    // TODO: install systemd config for shook

    Ok(())
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

fn get_input_pathbuf(
    prompt: &str,
    initial: Option<PathBuf>,
    completion: &MultiCompletion,
) -> color_eyre::Result<PathBuf> {
    if let Some(v) = initial {
        return Ok(v);
    }

    let path = loop {
        let res: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .completion_with(completion)
            .interact_text()?;

        let res = completion.get(&res).unwrap_or(res);
        if let Ok(p) = PathBuf::from(res).canonicalize() {
            break p;
        }
        tracing::warn!("type in a valid path");
    };

    Ok(path)
}

fn get_input_events(
    prompt: &str,
    initial: Option<Vec<EventDiscriminants>>,
) -> color_eyre::Result<Vec<EventDiscriminants>> {
    if let Some(v) = initial {
        return Ok(v);
    }
    let res = loop {
        let str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default("push".into())
            .interact_text()?;

        if let Ok(e) = parse_multiple_events(&str) {
            break e;
        }
        tracing::warn!("type in a comma delimited list of valid events, refer to https://docs.github.com/en/webhooks-and-events/webhooks/webhook-events-and-payloads");
    };

    Ok(res)
}
