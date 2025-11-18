use issun::context::ResourceContext;
use issun::event::EventBus;

#[derive(Debug, PartialEq)]
struct PlayerDamaged {
    amount: u32,
}

#[tokio::test]
async fn system_publish_scene_read_flow() {
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());

    // System publishes damage events during update.
    {
        let mut bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("event bus exists");
        bus.publish(PlayerDamaged { amount: 5 });
        bus.publish(PlayerDamaged { amount: 7 });
    }

    // Runner pumps the bus between frames.
    {
        let mut bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("event bus exists");
        bus.dispatch();
    }

    // Scene reads damage events during its next on_update.
    {
        let mut bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("event bus exists");
        let reader = bus.reader::<PlayerDamaged>();
        let amounts: Vec<_> = reader.iter().map(|evt| evt.amount).collect();
        assert_eq!(amounts, vec![5, 7]);
    }

    // No new publishing -> dispatch clears queue.
    {
        let mut bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("event bus exists");
        bus.dispatch();
        let reader = bus.reader::<PlayerDamaged>();
        assert!(reader.iter().next().is_none());
    }
}
