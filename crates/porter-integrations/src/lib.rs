pub mod tasks;

use porter_core::integrations::IntegrationRegistry;
use std::sync::Arc;

/// Register all built-in integrations into the registry.
pub fn register_builtin_integrations(registry: &mut IntegrationRegistry, enabled: &[String]) {
    if enabled.contains(&"tasks".to_string()) {
        registry.register(Arc::new(tasks::TaskIntegration::new()));
    }
}
