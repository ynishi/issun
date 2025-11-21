//! Research plugin implementation

use super::config::ResearchConfig;
use super::hook::{DefaultResearchHook, ResearchHook};
use super::research_projects::ResearchProjects;
use super::state::ResearchState;
use super::system::ResearchSystem;
use crate::Plugin;
use std::sync::Arc;

/// Built-in research/development/learning management plugin
///
/// This plugin provides research/development/crafting progression for games.
/// It registers ResearchProjects, ResearchConfig, ResearchState resources and ResearchSystem that handles:
/// - Processing research queue requests
/// - Processing research start/cancel requests
/// - Processing progress updates (manual or auto)
/// - Custom hooks for game-specific behavior
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Validate prerequisites before queuing research
/// - Calculate dynamic costs based on game state
/// - Modify progress rates based on bonuses/penalties
/// - Unlock content when research completes
/// - Log research events to game log
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::research::{ResearchPlugin, ResearchHook};
/// use async_trait::async_trait;
///
/// // Custom hook for unlocking content
/// struct TechTreeHook;
///
/// #[async_trait]
/// impl ResearchHook for TechTreeHook {
///     async fn on_research_completed(
///         &self,
///         project: &ResearchProject,
///         result: &ResearchResult,
///         resources: &mut ResourceContext,
///     ) {
///         // Unlock units/buildings
///         println!("Research completed: {}", project.name);
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         ResearchPlugin::new()
///             .with_hook(TechTreeHook)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:research")]
pub struct ResearchPlugin {
    #[plugin(skip)]
    hook: Arc<dyn ResearchHook>,
    #[plugin(resource)]
    projects: ResearchProjects,
    #[plugin(resource)]
    config: ResearchConfig,
    #[plugin(runtime_state)]
    #[allow(dead_code)]
    state: ResearchState,
    #[plugin(system)]
    system: ResearchSystem,
}

impl ResearchPlugin {
    /// Create a new research plugin
    ///
    /// Uses the default hook (no-op) and default config by default.
    /// Use `with_hook()` to add custom behavior, `with_projects()` to define research projects,
    /// and `with_config()` to customize configuration.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultResearchHook);
        Self {
            hook: hook.clone(),
            projects: ResearchProjects::new(),
            config: ResearchConfig::default(),
            state: ResearchState::new(),
            system: ResearchSystem::new(hook),
        }
    }

    /// Add a custom hook for research behavior
    ///
    /// The hook will be called when:
    /// - Research is queued (`on_research_queued`)
    /// - Research starts (`on_research_started`)
    /// - Research completes (`on_research_completed`)
    /// - Research fails/is cancelled (`on_research_failed`)
    /// - Prerequisites are validated (`validate_prerequisites`)
    /// - Cost is calculated (`calculate_research_cost`)
    /// - Progress is calculated (`calculate_progress`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of ResearchHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::research::{ResearchPlugin, ResearchHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl ResearchHook for MyHook {
    ///     async fn on_research_completed(
    ///         &self,
    ///         project: &ResearchProject,
    ///         result: &ResearchResult,
    ///         resources: &mut ResourceContext,
    ///     ) {
    ///         // Custom behavior...
    ///     }
    /// }
    ///
    /// let plugin = ResearchPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: ResearchHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = ResearchSystem::new(hook);
        self
    }

    /// Add research project definitions
    ///
    /// # Arguments
    ///
    /// * `projects` - Collection of research project definitions
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::research::{ResearchPlugin, ResearchProjects, ResearchProject};
    ///
    /// let mut projects = ResearchProjects::new();
    /// projects.define(ResearchProject::new("tech_1", "Advanced Technology", "Research description"));
    ///
    /// let plugin = ResearchPlugin::new().with_projects(projects);
    /// ```
    pub fn with_projects(mut self, projects: ResearchProjects) -> Self {
        self.projects = projects;
        self
    }

    /// Set custom research configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Research configuration (queue mode, progress model, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::research::{ResearchPlugin, ResearchConfig, ProgressModel};
    ///
    /// let config = ResearchConfig {
    ///     allow_parallel_research: true,
    ///     max_parallel_slots: 3,
    ///     progress_model: ProgressModel::TurnBased,
    ///     auto_advance: true,
    ///     base_progress_per_turn: 0.1,
    /// };
    ///
    /// let plugin = ResearchPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: ResearchConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for ResearchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let _plugin = ResearchPlugin::new();
        // Plugin derive macro automatically implements name()
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl ResearchHook for CustomHook {}

        let _plugin = ResearchPlugin::new().with_hook(CustomHook);
        // Plugin derive macro automatically implements name()
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = ResearchConfig {
            allow_parallel_research: true,
            max_parallel_slots: 3,
            ..Default::default()
        };

        let _plugin = ResearchPlugin::new().with_config(config);
        // Plugin derive macro automatically implements name()
    }
}
