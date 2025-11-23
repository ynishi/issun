use issun::builder::RuntimeResourceEntry;
use issun::plugin::Plugin as PluginTrait; // Rename trait to avoid confusion, though not strictly necessary
use issun::prelude::*;
use issun::resources::{Resource, Resources};
use issun::Plugin; // Import derive macro
use std::any::TypeId;

// Mock resources/types for testing
#[derive(Debug, Clone, Default, PartialEq)]
struct TestConfig {
    value: i32,
}
impl Resource for TestConfig {}

#[derive(Debug, Clone, Default)]
struct TestState {
    #[allow(dead_code)]
    count: i32,
}

#[derive(Default, Clone)]
struct TestService;
#[async_trait::async_trait]
impl issun::service::Service for TestService {
    fn name(&self) -> &'static str {
        "test_service"
    }
    fn clone_box(&self) -> Box<dyn issun::service::Service> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Default, Clone)]
struct TestSystem;
#[async_trait::async_trait]
impl issun::system::System for TestSystem {
    fn name(&self) -> &'static str {
        "test_system"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Plugin with field registration
#[derive(Plugin)]
#[plugin(name = "configurable_plugin")]
struct ConfigurablePlugin {
    #[resource]
    config: TestConfig,

    #[state]
    state: TestState,

    #[service]
    service: TestService,

    #[system]
    system: TestSystem,
}

// Mock Builder
struct MockBuilder {
    resources: Resources,
    services: Vec<String>,
    systems: Vec<String>,
    states: Vec<TypeId>,
}

impl MockBuilder {
    fn new() -> Self {
        Self {
            resources: Resources::default(),
            services: Vec::new(),
            systems: Vec::new(),
            states: Vec::new(),
        }
    }
}

impl PluginBuilder for MockBuilder {
    fn register_entity(&mut self, _name: &str, _entity: Box<dyn issun::entity::Entity>) {}

    fn register_service(&mut self, service: Box<dyn issun::service::Service>) {
        self.services.push(service.name().to_string());
    }

    fn register_system(&mut self, system: Box<dyn issun::system::System>) {
        self.systems.push(system.name().to_string());
    }

    fn register_runtime_resource_boxed(
        &mut self,
        type_id: TypeId,
        _resource: Box<dyn RuntimeResourceEntry>,
    ) {
        self.states.push(type_id);
    }

    fn register_asset(&mut self, _name: &str, _asset: Box<dyn std::any::Any + Send + Sync>) {}

    fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

#[test]
fn test_plugin_field_registration() {
    let plugin = ConfigurablePlugin {
        config: TestConfig { value: 42 },
        state: TestState { count: 10 },
        service: TestService,
        system: TestSystem,
    };

    let mut builder = MockBuilder::new();
    // Use the trait method
    PluginTrait::build(&plugin, &mut builder);

    // Verify resource registration
    let config = builder
        .resources
        .get::<TestConfig>()
        .expect("TestConfig should be registered");
    assert_eq!(config.value, 42);

    // Verify state registration
    assert!(builder.states.contains(&TypeId::of::<TestState>()));

    // Verify service registration
    assert!(builder.services.contains(&"test_service".to_string()));

    // Verify system registration
    assert!(builder.systems.contains(&"test_system".to_string()));
}

// Test struct-level type-based registration
#[derive(Default, Clone, Debug)]
struct GlobalState {
    #[allow(dead_code)]
    value: i32,
}
impl Resource for GlobalState {}

#[derive(Default, Clone, Debug)]
struct GlobalConfig {
    #[allow(dead_code)]
    setting: String,
}
impl Resource for GlobalConfig {}

// Plugin with struct-level attributes (type-based registration)
#[derive(Default, Plugin)]
#[plugin(name = "type_based_plugin")]
#[plugin(state = GlobalState)]
#[plugin(resource = GlobalConfig)]
#[plugin(service = TestService)]
#[plugin(system = TestSystem)]
pub struct TypeBasedPlugin;

#[test]
fn test_plugin_type_based_registration() {
    let plugin = TypeBasedPlugin;
    let mut builder = MockBuilder::new();
    PluginTrait::build(&plugin, &mut builder);

    // Verify state registration (using ::default())
    assert!(
        builder.states.contains(&TypeId::of::<GlobalState>()),
        "GlobalState should be registered"
    );

    // Verify resource registration (using ::default())
    let _config = builder
        .resources
        .get::<GlobalConfig>()
        .expect("GlobalConfig should be registered");

    // Verify service registration
    assert!(
        builder.services.contains(&"test_service".to_string()),
        "TestService should be registered"
    );

    // Verify system registration
    assert!(
        builder.systems.contains(&"test_system".to_string()),
        "TestSystem should be registered"
    );
}

// Plugin with field-level #[plugin(...)] attributes (new format)
#[derive(Plugin)]
#[plugin(name = "new_format_plugin")]
struct NewFormatPlugin {
    #[plugin(resource)]
    config: GlobalConfig,

    #[plugin(state)]
    state: GlobalState,

    #[plugin(service)]
    service: TestService,

    #[plugin(system)]
    system: TestSystem,
}

#[test]
fn test_plugin_field_new_format() {
    let plugin = NewFormatPlugin {
        config: GlobalConfig {
            setting: "test".to_string(),
        },
        state: GlobalState { value: 99 },
        service: TestService,
        system: TestSystem,
    };

    let mut builder = MockBuilder::new();
    PluginTrait::build(&plugin, &mut builder);

    // Verify resource registration
    let config = builder
        .resources
        .get::<GlobalConfig>()
        .expect("GlobalConfig should be registered");
    assert_eq!(config.setting, "test");

    // Verify state registration
    assert!(
        builder.states.contains(&TypeId::of::<GlobalState>()),
        "GlobalState should be registered"
    );

    // Verify service registration
    assert!(
        builder.services.contains(&"test_service".to_string()),
        "TestService should be registered"
    );

    // Verify system registration
    assert!(
        builder.systems.contains(&"test_system".to_string()),
        "TestSystem should be registered"
    );
}
