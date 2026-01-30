pub mod tasks;

use porter_core::config::{resolve_env_values, IntegrationsConfig};
use porter_core::db::Database;
use porter_core::integrations::{Integration, IntegrationConfig, IntegrationRegistry};
use std::collections::HashMap;
use std::sync::Arc;

/// Register all built-in integrations into the registry.
///
/// For each enabled integration, builds an `IntegrationConfig` from the
/// matching TOML section (if any), resolves `env:` values, and calls `init()`.
pub async fn register_builtin_integrations(
    registry: &mut IntegrationRegistry,
    config: &IntegrationsConfig,
    db: Database,
) {
    for name in &config.enabled {
        let (values, tick_interval_secs) = match config.settings.get(name) {
            Some(toml::Value::Table(table)) => {
                let mut table = table.clone();
                // Extract tick_interval before passing the rest as values
                let tick = table.remove("tick_interval").and_then(|v| v.as_integer()).map(|v| v as u64);
                // Resolve env: prefixed values
                let mut val = toml::Value::Table(table);
                resolve_env_values(&mut val);
                let values = match val {
                    toml::Value::Table(t) => t.into_iter().map(|(k, v)| (k, v)).collect(),
                    _ => HashMap::new(),
                };
                (values, tick)
            }
            _ => (HashMap::new(), None),
        };

        let integration_config = IntegrationConfig {
            values,
            db: db.clone(),
            tick_interval_secs,
        };

        match name.as_str() {
            "tasks" => {
                let mut integration = tasks::TaskIntegration::new();
                if let Err(e) = integration.init(&integration_config).await {
                    tracing::error!("Failed to initialize tasks integration: {e}");
                    continue;
                }
                let arc = Arc::new(integration);
                if let Some(tick) = tick_interval_secs {
                    registry.register_with_tick(arc, tick);
                } else {
                    registry.register(arc);
                }
                tracing::info!(tick_interval = ?tick_interval_secs, "Registered integration: tasks");
            }
            other => {
                tracing::warn!("Unknown integration: {other}");
            }
        }
    }
}
