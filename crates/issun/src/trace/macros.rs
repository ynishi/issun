//! Macros for tracing hook calls

/// Hookをトレース付きで呼び出すマクロ
///
/// # Example
///
/// ```ignore
/// use issun::call_hook_traced;
///
/// let result = call_hook_traced!(
///     tracer,
///     "WorldMapPlugin",
///     hook,
///     can_start_travel,
///     "player",
///     "city_a",
///     "city_b"
/// ).await;
/// ```
#[macro_export]
macro_rules! call_hook_traced {
    (
        $tracer:expr,
        $plugin:expr,
        $hook:expr,
        $method:ident
        $(, $arg:expr)*
        $(,)?
    ) => {{
        let start = std::time::Instant::now();
        let plugin_name = $plugin.to_string();
        let method_name = stringify!($method).to_string();

        // Hook呼び出し記録
        if let Some(ref tracer) = $tracer {
            if let Ok(mut t) = tracer.lock() {
                t.record_simple(
                    $crate::trace::TraceEntryType::HookCalled {
                        hook_name: method_name.clone(),
                        plugin: plugin_name.clone(),
                        args: format!("{:?}", ($($arg,)*)),
                    },
                    plugin_name.clone(),
                );
            }
        }

        // Hook実行
        let result = $hook.$method($($arg),*).await;

        // Hook完了記録
        if let Some(ref tracer) = $tracer {
            if let Ok(mut t) = tracer.lock() {
                let hook_result = if result.is_ok() {
                    $crate::trace::HookResult::Success
                } else {
                    $crate::trace::HookResult::Error(format!("{:?}", result))
                };

                t.record_simple(
                    $crate::trace::TraceEntryType::HookCompleted {
                        hook_name: method_name,
                        plugin: plugin_name,
                        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
                        result: hook_result,
                    },
                    "HookTracer",
                );
            }
        }

        result
    }};
}

#[cfg(test)]
mod tests {
    use crate::trace::EventChainTracer;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // テスト用Hook
    #[async_trait]
    trait TestHook: Send + Sync {
        async fn test_method(&self, arg: &str) -> Result<(), String>;
    }

    struct DummyHook;

    #[async_trait]
    impl TestHook for DummyHook {
        async fn test_method(&self, _arg: &str) -> Result<(), String> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_call_hook_traced_macro() {
        let mut tracer = EventChainTracer::new();
        tracer.enable();
        let tracer = Arc::new(Mutex::new(tracer));

        let hook = DummyHook;

        let result = call_hook_traced!(
            Some(tracer.clone()),
            "TestPlugin",
            hook,
            test_method,
            "test_arg"
        );

        assert!(result.is_ok());

        // トレースが記録されているか確認
        let t = tracer.lock().unwrap();
        let traces = t.traces();

        assert_eq!(traces.len(), 2); // HookCalled + HookCompleted

        match &traces[0].entry_type {
            crate::trace::TraceEntryType::HookCalled { hook_name, .. } => {
                assert_eq!(hook_name, "test_method");
            }
            _ => panic!("Expected HookCalled"),
        }

        match &traces[1].entry_type {
            crate::trace::TraceEntryType::HookCompleted { hook_name, .. } => {
                assert_eq!(hook_name, "test_method");
            }
            _ => panic!("Expected HookCompleted"),
        }
    }
}
