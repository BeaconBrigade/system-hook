use crate::config::Init;

pub fn init(args: Init) -> color_eyre::Result<()> {
    tracing::info!("creating project");
    tracing::debug!("{:#?}", args);

    Ok(())
}
