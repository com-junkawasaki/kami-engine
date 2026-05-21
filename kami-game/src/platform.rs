//! Platform detection for KAMI Engine SDK.
//!
//! Detects iOS, Android, and Web (desktop browser) at runtime.
//! Used by input systems to decide touch vs keyboard controls.

/// Runtime platform the game is running on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Ios,
    Android,
    Web,
}

impl Platform {
    /// Returns true if running on a mobile device (iOS or Android).
    pub fn is_mobile(&self) -> bool {
        matches!(self, Platform::Ios | Platform::Android)
    }

    /// Returns true if touch input should be the primary input method.
    pub fn is_touch(&self) -> bool {
        self.is_mobile()
    }
}

/// Detect the current platform from a user-agent string.
pub fn detect_from_user_agent(ua: &str) -> Platform {
    let ua_lower = ua.to_ascii_lowercase();
    if ua_lower.contains("iphone")
        || ua_lower.contains("ipad")
        || ua_lower.contains("ipod")
        || (ua_lower.contains("macintosh") && ua_lower.contains("mobile"))
    {
        Platform::Ios
    } else if ua_lower.contains("android") {
        Platform::Android
    } else {
        Platform::Web
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_iphone() {
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15";
        assert_eq!(detect_from_user_agent(ua), Platform::Ios);
        assert!(detect_from_user_agent(ua).is_mobile());
        assert!(detect_from_user_agent(ua).is_touch());
    }

    #[test]
    fn detect_ipad() {
        let ua = "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15";
        assert_eq!(detect_from_user_agent(ua), Platform::Ios);
    }

    #[test]
    fn detect_ipad_desktop_mode() {
        // iPadOS 13+ reports as Macintosh but has touch
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
        assert_eq!(detect_from_user_agent(ua), Platform::Ios);
    }

    #[test]
    fn detect_android() {
        let ua = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36";
        assert_eq!(detect_from_user_agent(ua), Platform::Android);
        assert!(detect_from_user_agent(ua).is_mobile());
    }

    #[test]
    fn detect_desktop_chrome() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0";
        assert_eq!(detect_from_user_agent(ua), Platform::Web);
        assert!(!detect_from_user_agent(ua).is_mobile());
        assert!(!detect_from_user_agent(ua).is_touch());
    }

    #[test]
    fn detect_desktop_safari() {
        let ua =
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 Safari/605.1.15";
        assert_eq!(detect_from_user_agent(ua), Platform::Web);
    }
}
