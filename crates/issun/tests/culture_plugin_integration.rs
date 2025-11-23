//! Integration test for CulturePlugin
//!
//! Tests that CulturePlugin can be properly used within
//! the full game context with EventBus and ResourceContext.

use issun::context::ResourceContext;
use issun::event::EventBus;
use issun::plugin::culture::{
    AlignmentCheckRequested, CultureConfig, CulturePlugin, CultureState, CultureSystem,
    CultureTag, DefaultCultureHook, Member, PersonalityTrait,
};
use issun::plugin::Plugin;

#[tokio::test]
async fn test_culture_plugin_name() {
    let plugin = CulturePlugin::new();
    assert_eq!(plugin.name(), "culture_plugin");
}

#[tokio::test]
async fn test_culture_plugin_default() {
    let plugin = CulturePlugin::default();
    assert_eq!(plugin.name(), "culture_plugin");
}

#[tokio::test]
async fn test_culture_plugin_with_config() {
    let config = CultureConfig::default().with_stress_rate(0.05);
    let plugin = CulturePlugin::new().with_config(config);
    assert_eq!(plugin.name(), "culture_plugin");
}

#[tokio::test]
async fn test_culture_plugin_full_workflow() {
    // Setup resources
    let mut resources = ResourceContext::new();

    // Register config
    let config = CultureConfig::default().with_stress_rate(0.05);
    resources.insert(config);

    // Register state with a faction
    let mut state = CultureState::new();
    state.register_faction("faction_a");
    resources.insert(state);

    // Register event bus
    let bus = EventBus::new();
    resources.insert(bus);

    // Add a member with personality traits
    {
        let mut state = resources.get_mut::<CultureState>().await.unwrap();
        let culture = state.get_culture_mut(&"faction_a".to_string()).unwrap();
        let member = Member::new("m1", "Alice")
            .with_trait(PersonalityTrait::Cautious)
            .with_stress(0.0);
        culture.add_member(member);

        // Add a culture tag that conflicts with member personality
        culture.add_culture_tag(CultureTag::RiskTaking);
    }

    // Create system
    let mut system = CultureSystem::new(DefaultCultureHook);

    // Publish alignment check request
    {
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        bus.publish(AlignmentCheckRequested { delta_turns: 1 });
        bus.dispatch();
    }

    // Process events
    system.process_events(&mut resources).await;

    // Verify stress increased (Cautious member in RiskTaking culture)
    let state = resources.get::<CultureState>().await.unwrap();
    let culture = state.get_culture(&"faction_a".to_string()).unwrap();
    let member = culture.get_member(&"m1".to_string()).unwrap();

    // Stress should have increased from 0.0
    assert!(
        member.stress > 0.0,
        "Stress should increase for misaligned member, got {}",
        member.stress
    );
}

#[tokio::test]
async fn test_culture_plugin_fervor_increase() {
    // Setup resources
    let mut resources = ResourceContext::new();

    // Register config
    let config = CultureConfig::default();
    resources.insert(config);

    // Register state with a faction
    let mut state = CultureState::new();
    state.register_faction("faction_a");
    resources.insert(state);

    // Register event bus
    let bus = EventBus::new();
    resources.insert(bus);

    // Add a member with aligned personality
    {
        let mut state = resources.get_mut::<CultureState>().await.unwrap();
        let culture = state.get_culture_mut(&"faction_a".to_string()).unwrap();
        let member = Member::new("m1", "Bob")
            .with_trait(PersonalityTrait::Cautious)
            .with_fervor(0.5);
        culture.add_member(member);

        // Add a culture tag that aligns with member personality
        culture.add_culture_tag(CultureTag::Bureaucratic);
    }

    // Create system
    let mut system = CultureSystem::new(DefaultCultureHook);

    // Publish alignment check request
    {
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        bus.publish(AlignmentCheckRequested { delta_turns: 1 });
        bus.dispatch();
    }

    // Process events
    system.process_events(&mut resources).await;

    // Verify fervor increased (Cautious member in Bureaucratic culture)
    let state = resources.get::<CultureState>().await.unwrap();
    let culture = state.get_culture(&"faction_a".to_string()).unwrap();
    let member = culture.get_member(&"m1".to_string()).unwrap();

    // Fervor should have increased from 0.5
    assert!(
        member.fervor > 0.5,
        "Fervor should increase for aligned member, got {}",
        member.fervor
    );
}

#[tokio::test]
async fn test_culture_plugin_with_custom_hook() {
    use async_trait::async_trait;
    use issun::context::ResourceContext;
    use issun::plugin::culture::{Alignment, CultureHook, FactionId, MemberId};

    #[derive(Clone, Copy)]
    struct TestHook;

    #[async_trait]
    impl CultureHook for TestHook {
        async fn on_alignment_checked(
            &self,
            _faction_id: &FactionId,
            _member_id: &MemberId,
            _alignment: &Alignment,
            _resources: &mut ResourceContext,
        ) {
            // Custom hook logic would go here
        }
    }

    let plugin = CulturePlugin::new().with_hook(TestHook);
    assert_eq!(plugin.name(), "culture_plugin");
}
