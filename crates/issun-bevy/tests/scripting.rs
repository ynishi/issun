//! Scripting Plugin Tests

use issun_bevy::plugins::scripting::{MluaBackend, ScriptingBackend};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_mlua_backend_creation() {
    // Test that MLua backend can be created

    let backend = MluaBackend::new();
    assert!(backend.is_ok(), "Failed to create MLua backend");

    let backend = backend.unwrap();
    assert_eq!(backend.backend_name(), "mlua (Lua 5.4)");
}

#[test]
fn test_execute_simple_lua_chunk() {
    // Test basic Lua code execution

    let mut backend = MluaBackend::new().unwrap();

    // Execute simple Lua code
    let result = backend.execute_chunk("x = 42");
    assert!(
        result.is_ok(),
        "Failed to execute Lua chunk: {:?}",
        result.err()
    );
}

#[test]
fn test_load_and_call_function() {
    // Test loading a script file and calling a function

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test.lua");

    // Create test script
    let script_content = r#"
        function greet()
            message = "Hello from Lua!"
        end
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut backend = MluaBackend::new().unwrap();

    // Load script
    let handle = backend.load_script(script_path.to_str().unwrap());
    assert!(handle.is_ok(), "Failed to load script: {:?}", handle.err());

    let handle = handle.unwrap();

    // Check if function exists
    assert!(backend.has_function(handle, "greet"));
    assert!(!backend.has_function(handle, "nonexistent"));

    // Call function
    let result = backend.call_function(handle, "greet");
    assert!(
        result.is_ok(),
        "Failed to call function: {:?}",
        result.err()
    );
}

#[test]
fn test_function_not_found_error() {
    // Test that calling non-existent function returns error

    let backend = MluaBackend::new().unwrap();

    // Try to call non-existent function
    let result = backend.call_function(
        issun_bevy::plugins::scripting::ScriptHandle::new(0),
        "nonexistent",
    );

    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        issun_bevy::plugins::scripting::ScriptError::FunctionNotFound(_)
    ));
}

#[test]
fn test_syntax_error_handling() {
    // Test that syntax errors are properly handled

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("bad.lua");

    // Create script with syntax error
    fs::write(&script_path, "function bad(").unwrap();

    let mut backend = MluaBackend::new().unwrap();

    // Try to load script
    let result = backend.load_script(script_path.to_str().unwrap());

    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        issun_bevy::plugins::scripting::ScriptError::SyntaxError(_)
    ));
}

#[test]
fn test_sandbox_io_disabled() {
    // Test that io library is disabled in sandbox

    let mut backend = MluaBackend::new().unwrap();

    // Try to use io.open (should fail)
    let result = backend.execute_chunk(r#"io.open("test.txt", "r")"#);

    assert!(result.is_err(), "io library should be disabled in sandbox");
}

#[test]
fn test_sandbox_os_disabled() {
    // Test that os library is disabled in sandbox

    let mut backend = MluaBackend::new().unwrap();

    // Try to use os.execute (should fail)
    let result = backend.execute_chunk(r#"os.execute("echo hello")"#);

    assert!(result.is_err(), "os library should be disabled in sandbox");
}

#[test]
fn test_sandbox_require_disabled() {
    // Test that require is disabled in sandbox

    let mut backend = MluaBackend::new().unwrap();

    // Try to use require (should fail)
    let result = backend.execute_chunk(r#"require("os")"#);

    assert!(result.is_err(), "require should be disabled in sandbox");
}

#[test]
fn test_sandbox_dofile_disabled() {
    // Test that dofile is disabled in sandbox

    let mut backend = MluaBackend::new().unwrap();

    // Try to use dofile (should fail)
    let result = backend.execute_chunk(r#"dofile("test.lua")"#);

    assert!(result.is_err(), "dofile should be disabled in sandbox");
}

#[test]
fn test_sandbox_allows_safe_operations() {
    // Test that sandbox still allows safe operations

    let mut backend = MluaBackend::new().unwrap();

    // Math operations should work
    let result = backend.execute_chunk("x = math.sin(math.pi / 2)");
    assert!(result.is_ok(), "Math operations should be allowed");

    // String operations should work
    let result = backend.execute_chunk(r#"s = string.upper("hello")"#);
    assert!(result.is_ok(), "String operations should be allowed");

    // Table operations should work
    let result = backend.execute_chunk("t = {1, 2, 3}; table.insert(t, 4)");
    assert!(result.is_ok(), "Table operations should be allowed");
}
