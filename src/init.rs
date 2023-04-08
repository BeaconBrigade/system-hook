use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use color_eyre::eyre::{eyre, Context};
use dialoguer::{theme::ColorfulTheme, Completion, Confirm, Input};
use github_webhook::EventDiscriminants;
use text_completions::{EnvCompletion, MultiCompletion, PathCompletion};

use crate::config::{parse_multiple_events, Init, InitConfig, TcpOrUnix};

const SERVICE_TEMPLATE: &str = include_str!("shook.service");
const SERVICE_PATH: &str = "/etc/systemd/system/shook.service";

pub fn init_project(args: Init) -> color_eyre::Result<()> {
    tracing::info!("creating project");

    let completion = MultiCompletion::default()
        .with(EnvCompletion::default())
        .with(PathCompletion::default());
    let username = get_input("the linux user to run git as", args.username)?;
    let repo_path = get_input_pathbuf("path to the repository", args.repo_path, &completion)?;
    let remote = get_input_default(
        "the remote to track for changes",
        args.remote,
        "origin".to_string(),
    )?;
    let branch = get_input_default(
        "the branch to track for changes",
        args.branch,
        "main".to_string(),
    )?;
    let system_name = get_input(
        "name of systemd service to update on github events",
        args.system_name,
    )?;
    let update_events = get_input_events("github events to update", args.update_events)?;
    let addr = get_input_default(
        "address to serve on (unix socket path or tcp address)",
        args.addr,
        TcpOrUnix::Unix("/var/run/shook.sock".into()),
    )?;
    let (socket_group, socket_user) = if let TcpOrUnix::Unix(_) = addr {
        let group = get_input_default(
            "group to put socket under",
            args.socket_group,
            "www-data".to_string(),
        )?;
        let user = get_input_default("user to put socket under", args.socket_user, "www-data".to_string())?;

        (group, user)
    } else {
        ("".to_string(), "".to_string())
    };

    let config = InitConfig {
        username,
        repo_path,
        remote,
        branch,
        system_name,
        update_events,
        addr,
        socket_group,
        socket_user,
    };

    tracing::debug!(?config);

    if !Path::try_exists(&config.repo_path)? {
        tracing::warn!("repository could not be found");

        let should_clone = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("clone the repository?")
            .interact()?;

        if should_clone {
            let url: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("repository url")
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

            let mut handle = Command::new("su")
                .arg(&config.username)
                .arg("-c")
                .arg(format!(
                    "git clone '{}' '{}'",
                    url,
                    config.repo_path.to_string_lossy()
                ))
                .current_dir(parent)
                .spawn()?;
            let exit_code = handle.wait()?;
            tracing::debug!("git exited with exit code {:?}", exit_code.code());
            if exit_code.code().unwrap_or(1) != 0 {
                tracing::error!("could not clone repository");
                return Err(eyre!("could not clone repository"));
            }
        }
    }

    let toml = toml::to_string_pretty(&config).context("serializing config to toml")?;

    let config_path = config.repo_path.join("shook.toml");
    if Path::exists(&config_path) {
        tracing::warn!("{:?} already exists", config_path);

        let should_replace = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("replace existing shook.toml?")
            .interact()?;
        if !should_replace {
            tracing::info!("aborting init process");
            return Err(eyre!("aborting due to existing config"));
        }
    }

    let mut file = File::create(&config_path).context("creating shook.toml")?;
    file.write_all(toml.as_bytes())
        .context("writing shook.toml")?;
    tracing::info!("finished writing shook.toml");

    let systemd = SERVICE_TEMPLATE.replace(
        "{REPO_PATH}",
        config
            .repo_path
            .to_str()
            .ok_or_else(|| eyre!("repo path is not vaid utf8"))?,
    );

    tracing::info!("installing systemd config");
    tracing::debug!("systemd file:\n{}", systemd);
    let service_path = PathBuf::from(SERVICE_PATH);
    if Path::exists(&service_path) {
        tracing::warn!("shook.service already exists");

        let should_replace = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("replace existing shook.service")
            .interact()?;
        if !should_replace {
            return Err(eyre!("aborting due to existing service"));
        }
    }

    let mut file = File::create(&service_path).context("creating shook.service")?;
    file.write_all(systemd.as_bytes())
        .context("writing shook.service")?;

    tracing::info!("finished creating project");

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

fn get_input_default<T>(prompt: &str, initial: Option<T>, default: T) -> color_eyre::Result<T>
where
    T: Clone + ToString + FromStr,
    <T as FromStr>::Err: std::fmt::Debug + ToString,
{
    if let Some(v) = initial {
        return Ok(v);
    }
    let res = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default)
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
