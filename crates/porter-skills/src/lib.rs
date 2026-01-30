pub mod tasks;

// Skill module stubs - to be implemented in later phases
pub mod calendar;
pub mod deliveries;
pub mod dining;
pub mod documents;
pub mod email;
pub mod flights;
pub mod slack;

use porter_core::skills::SkillRegistry;
use std::sync::Arc;

/// Register all built-in skills into the registry.
pub fn register_builtin_skills(registry: &mut SkillRegistry, enabled: &[String]) {
    if enabled.contains(&"tasks".to_string()) {
        registry.register(Arc::new(tasks::TaskSkill::new()));
    }
    // Future skills will be registered here as they're implemented
}
