use porter_core::config::PorterConfig;
use std::path::Path;

pub async fn run(config_path: &str) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,porter_server=debug,porter_core=debug".into()),
        )
        .init();

    let path = Path::new(config_path);
    if !path.exists() {
        anyhow::bail!("Config file not found: {}", config_path);
    }

    let config = PorterConfig::load(path)?;
    tracing::info!(
        "Starting Porter '{}' on port {}",
        config.instance.name,
        config.instance.port
    );

    porter_server::run_server(config).await
}
