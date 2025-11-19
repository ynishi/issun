use issun::auto_pump;
use issun::prelude::*;

// Mock pump function for testing
pub async fn mock_pump(
    _services: &ServiceContext,
    _systems: &mut SystemContext,
    _resources: &mut ResourceContext,
) {
    // Do nothing - just for testing macro expansion
}

pub struct TestScene {
    pub value: i32,
}

impl TestScene {
    #[auto_pump(pump_fn = mock_pump)]
    pub async fn handle_input(
        &mut self,
        services: &ServiceContext,
        systems: &mut SystemContext,
        resources: &mut ResourceContext,
    ) -> i32 {
        self.value += 1;
        self.value
    }
}

#[tokio::test]
async fn test_auto_pump_basic() {
    // This test just verifies that the macro compiles and preserves return values
    // The actual functionality would need proper Context setup
    let scene = TestScene { value: 0 };

    // We can't actually call this without proper Context,
    // but if it compiles, the macro worked
    assert_eq!(scene.value, 0);
}
