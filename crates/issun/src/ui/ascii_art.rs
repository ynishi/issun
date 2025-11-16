//! Preset ASCII art for title screens

/// Robot-themed ASCII art
pub const ROBOT_ART: &str = r#"
    ╔═══════════════════════════════════╗
    ║   _____ _____ _____ _____ _____   ║
    ║  |  __ \  _  | ___ \  _  |_   _|  ║
    ║  | |__) | | | | |_/ / | | | | |   ║
    ║  |  _  /| | | | ___ \ | | | | |   ║
    ║  | | \ \\ \_/ / |_/ / \_/ /_| |_  ║
    ║  \_| |_|\___/\____/ \___/ \___/   ║
    ╚═══════════════════════════════════╝
"#;

/// Sword-themed ASCII art
pub const SWORD_ART: &str = r#"
    ╔═══════════════════════════════════╗
    ║         />                        ║
    ║      //                           ║
    ║   //                              ║
    ║  ================                 ║
    ║   \\                              ║
    ║      \\                           ║
    ║         \>                        ║
    ╚═══════════════════════════════════╝
"#;

/// Minimal border
pub const MINIMAL_ART: &str = r#"
    ╔═══════════════════════════════════╗
    ║                                   ║
    ║                                   ║
    ║                                   ║
    ║                                   ║
    ║                                   ║
    ║                                   ║
    ╚═══════════════════════════════════╝
"#;

/// Get preset art by name
pub fn get_art_by_name(name: &str) -> Option<&'static str> {
    match name {
        "robot" => Some(ROBOT_ART),
        "sword" => Some(SWORD_ART),
        "minimal" => Some(MINIMAL_ART),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_art_by_name() {
        assert!(get_art_by_name("robot").is_some());
        assert!(get_art_by_name("sword").is_some());
        assert!(get_art_by_name("minimal").is_some());
        assert!(get_art_by_name("nonexistent").is_none());
    }
}
