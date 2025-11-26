//! Lua API Bindings
//!
//! Provides Lua API for modding:
//! - Utility APIs: log, random
//! - Entity APIs: get_component, set_component, etc.
//! - Event APIs: subscribe_event, publish_event
//! - Query APIs: query_entities
//! - Commands APIs: spawn_entity, etc.
//! - Resource APIs: get_resource, set_resource

use mlua::{Lua, Result as LuaResult};

/// Register all Lua APIs
pub fn register_all_apis(lua: &Lua) -> LuaResult<()> {
    register_utility_apis(lua)?;
    // TODO: register_entity_apis(lua, world)?;
    // TODO: register_event_apis(lua)?;
    // TODO: register_query_apis(lua, world)?;
    // TODO: register_commands_apis(lua)?;
    // TODO: register_resource_apis(lua, world)?;
    Ok(())
}

/// Register utility APIs: log, random
fn register_utility_apis(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    // log(message) - info level logging
    let log_fn = lua.create_function(|_, message: String| {
        tracing::info!("[Lua] {}", message);
        Ok(())
    })?;
    globals.set("log", log_fn)?;

    // log_warn(message) - warning level logging
    let log_warn_fn = lua.create_function(|_, message: String| {
        tracing::warn!("[Lua] {}", message);
        Ok(())
    })?;
    globals.set("log_warn", log_warn_fn)?;

    // log_error(message) - error level logging
    let log_error_fn = lua.create_function(|_, message: String| {
        tracing::error!("[Lua] {}", message);
        Ok(())
    })?;
    globals.set("log_error", log_error_fn)?;

    // random() - returns float in [0, 1)
    let random_fn = lua.create_function(|_, ()| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok(rng.gen::<f64>())
    })?;
    globals.set("random", random_fn)?;

    // random_range(min, max) - returns float in [min, max)
    let random_range_fn = lua.create_function(|_, (min, max): (f64, f64)| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(min..max))
    })?;
    globals.set("random_range", random_range_fn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_api() {
        let lua = Lua::new();
        register_utility_apis(&lua).unwrap();

        // Test log()
        lua.load(r#"log("test message")"#).exec().unwrap();

        // Test log_warn()
        lua.load(r#"log_warn("warning message")"#)
            .exec()
            .unwrap();

        // Test log_error()
        lua.load(r#"log_error("error message")"#).exec().unwrap();
    }

    #[test]
    fn test_random_api() {
        let lua = Lua::new();
        register_utility_apis(&lua).unwrap();

        // Test random() returns value in [0, 1)
        let result: f64 = lua.load(r#"return random()"#).eval().unwrap();
        assert!(result >= 0.0 && result < 1.0);

        // Test random_range(min, max)
        let result: f64 = lua
            .load(r#"return random_range(10.0, 20.0)"#)
            .eval()
            .unwrap();
        assert!(result >= 10.0 && result < 20.0);
    }

    #[test]
    fn test_random_range_negative() {
        let lua = Lua::new();
        register_utility_apis(&lua).unwrap();

        // Test with negative range
        let result: f64 = lua
            .load(r#"return random_range(-10.0, -5.0)"#)
            .eval()
            .unwrap();
        assert!(result >= -10.0 && result < -5.0);
    }
}
