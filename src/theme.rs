use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ThemeColor {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Returns the raw ANSI 24-bit foreground escape sequence.
    /// Bypasses the `colored` crate's TrueColor → ANSI 8-bit fallback.
    fn fg_escape(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }
}

/// A styled string that wraps text with raw 24-bit ANSI color codes.
pub struct StyledString {
    text: String,
    color: ThemeColor,
    bold: bool,
}

impl StyledString {
    fn new(text: &str, color: ThemeColor, bold: bool) -> Self {
        Self {
            text: text.to_string(),
            color,
            bold,
        }
    }
}

impl fmt::Display for StyledString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bold {
            write!(f, "\x1b[1m{}{}\x1b[0m", self.color.fg_escape(), self.text)
        } else {
            write!(f, "{}{}\x1b[0m", self.color.fg_escape(), self.text)
        }
    }
}

pub struct Theme {
    pub name: &'static str,
    pub header: ThemeColor,
    pub accent: ThemeColor,
    pub success: ThemeColor,
    pub error: ThemeColor,
    pub warning: ThemeColor,
    pub info: ThemeColor,
    pub muted: ThemeColor,
    pub border: ThemeColor,
    pub value: ThemeColor,
    pub label: ThemeColor,
    pub progress_bar: ThemeColor,
}

/// Omarchy Synthwave84 — purple-first palette matching the desktop theme.
/// Backgrounds: #0d0221 / #240037  |  Primary: #8f00ff  |  Magenta: #ff00ff
/// Pink: #ff7edb  |  Cyan accent: #03edf9  |  Yellow: #f3e70f  |  Muted: #614d85
pub const SYNTHWAVE84: Theme = Theme {
    name: "synthwave84",
    header: ThemeColor::new(143, 0, 255),       // electric purple — primary
    accent: ThemeColor::new(255, 0, 255),      // magenta
    success: ThemeColor::new(3, 237, 249),     // neon cyan
    error: ThemeColor::new(255, 0, 64),        // red
    warning: ThemeColor::new(243, 231, 15),    // yellow
    info: ThemeColor::new(255, 126, 219),      // pink
    muted: ThemeColor::new(97, 77, 133),       // muted purple
    border: ThemeColor::new(143, 0, 255),      // purple border
    value: ThemeColor::new(255, 255, 255),     // white values
    label: ThemeColor::new(143, 0, 255),       // purple labels
    progress_bar: ThemeColor::new(255, 0, 255), // magenta progress
};

pub const SYNTHWAVE_NIGHT: Theme = Theme {
    name: "synthwave-night",
    header: ThemeColor::new(255, 0, 255),
    accent: ThemeColor::new(3, 237, 249),
    success: ThemeColor::new(131, 0, 255),
    error: ThemeColor::new(255, 0, 64),
    warning: ThemeColor::new(255, 255, 102),
    info: ThemeColor::new(0, 128, 255),
    muted: ThemeColor::new(60, 40, 90),
    border: ThemeColor::new(80, 50, 120),
    value: ThemeColor::new(230, 220, 255),
    label: ThemeColor::new(255, 126, 219),
    progress_bar: ThemeColor::new(131, 0, 255),
};

pub const SYNTHWAVE_SUNSET: Theme = Theme {
    name: "synthwave-sunset",
    header: ThemeColor::new(255, 126, 219),
    accent: ThemeColor::new(255, 0, 64),
    success: ThemeColor::new(255, 84, 66),
    error: ThemeColor::new(255, 0, 64),
    warning: ThemeColor::new(255, 255, 102),
    info: ThemeColor::new(243, 231, 15),
    muted: ThemeColor::new(120, 60, 80),
    border: ThemeColor::new(140, 70, 90),
    value: ThemeColor::new(255, 230, 220),
    label: ThemeColor::new(255, 126, 219),
    progress_bar: ThemeColor::new(255, 0, 255),
};

pub const NEON_CITY: Theme = Theme {
    name: "neon-city",
    header: ThemeColor::new(0, 255, 65),
    accent: ThemeColor::new(0, 255, 255),
    success: ThemeColor::new(0, 255, 65),
    error: ThemeColor::new(255, 0, 65),
    warning: ThemeColor::new(255, 255, 0),
    info: ThemeColor::new(0, 200, 255),
    muted: ThemeColor::new(80, 80, 80),
    border: ThemeColor::new(0, 180, 180),
    value: ThemeColor::new(200, 255, 200),
    label: ThemeColor::new(0, 255, 255),
    progress_bar: ThemeColor::new(0, 255, 65),
};

pub const DARK: Theme = Theme {
    name: "dark",
    header: ThemeColor::new(100, 200, 255),
    accent: ThemeColor::new(120, 180, 255),
    success: ThemeColor::new(80, 200, 120),
    error: ThemeColor::new(255, 85, 85),
    warning: ThemeColor::new(255, 200, 80),
    info: ThemeColor::new(160, 180, 200),
    muted: ThemeColor::new(100, 100, 120),
    border: ThemeColor::new(80, 80, 100),
    value: ThemeColor::new(220, 220, 230),
    label: ThemeColor::new(140, 170, 210),
    progress_bar: ThemeColor::new(100, 200, 255),
};

pub const LIGHT: Theme = Theme {
    name: "light",
    header: ThemeColor::new(0, 80, 160),
    accent: ThemeColor::new(0, 100, 200),
    success: ThemeColor::new(0, 120, 60),
    error: ThemeColor::new(180, 30, 30),
    warning: ThemeColor::new(160, 120, 0),
    info: ThemeColor::new(80, 80, 100),
    muted: ThemeColor::new(120, 120, 140),
    border: ThemeColor::new(180, 180, 200),
    value: ThemeColor::new(30, 30, 40),
    label: ThemeColor::new(0, 80, 140),
    progress_bar: ThemeColor::new(0, 80, 160),
};

pub const OCEAN: Theme = Theme {
    name: "ocean",
    header: ThemeColor::new(0, 200, 220),
    accent: ThemeColor::new(0, 150, 200),
    success: ThemeColor::new(0, 180, 140),
    error: ThemeColor::new(220, 60, 80),
    warning: ThemeColor::new(255, 200, 100),
    info: ThemeColor::new(100, 180, 220),
    muted: ThemeColor::new(60, 100, 130),
    border: ThemeColor::new(50, 90, 120),
    value: ThemeColor::new(200, 230, 255),
    label: ThemeColor::new(0, 180, 200),
    progress_bar: ThemeColor::new(0, 200, 220),
};

pub const FOREST: Theme = Theme {
    name: "forest",
    header: ThemeColor::new(120, 200, 80),
    accent: ThemeColor::new(80, 160, 60),
    success: ThemeColor::new(60, 180, 60),
    error: ThemeColor::new(200, 60, 40),
    warning: ThemeColor::new(200, 160, 40),
    info: ThemeColor::new(140, 160, 100),
    muted: ThemeColor::new(80, 100, 60),
    border: ThemeColor::new(60, 80, 40),
    value: ThemeColor::new(220, 230, 200),
    label: ThemeColor::new(100, 160, 80),
    progress_bar: ThemeColor::new(80, 160, 60),
};

pub const SUNSET: Theme = Theme {
    name: "sunset",
    header: ThemeColor::new(255, 140, 50),
    accent: ThemeColor::new(255, 100, 80),
    success: ThemeColor::new(200, 160, 60),
    error: ThemeColor::new(220, 50, 50),
    warning: ThemeColor::new(255, 200, 80),
    info: ThemeColor::new(255, 160, 120),
    muted: ThemeColor::new(140, 90, 70),
    border: ThemeColor::new(120, 80, 60),
    value: ThemeColor::new(255, 240, 220),
    label: ThemeColor::new(255, 120, 80),
    progress_bar: ThemeColor::new(255, 140, 50),
};

pub const MIDNIGHT: Theme = Theme {
    name: "midnight",
    header: ThemeColor::new(150, 180, 255),
    accent: ThemeColor::new(100, 140, 255),
    success: ThemeColor::new(120, 200, 180),
    error: ThemeColor::new(255, 100, 120),
    warning: ThemeColor::new(200, 200, 140),
    info: ThemeColor::new(140, 160, 200),
    muted: ThemeColor::new(70, 80, 120),
    border: ThemeColor::new(60, 70, 110),
    value: ThemeColor::new(200, 210, 240),
    label: ThemeColor::new(130, 160, 220),
    progress_bar: ThemeColor::new(100, 140, 255),
};

pub const RETRO: Theme = Theme {
    name: "retro",
    header: ThemeColor::new(255, 176, 0),
    accent: ThemeColor::new(255, 200, 60),
    success: ThemeColor::new(200, 255, 0),
    error: ThemeColor::new(255, 100, 0),
    warning: ThemeColor::new(255, 220, 100),
    info: ThemeColor::new(200, 160, 0),
    muted: ThemeColor::new(120, 90, 0),
    border: ThemeColor::new(100, 75, 0),
    value: ThemeColor::new(255, 220, 120),
    label: ThemeColor::new(255, 190, 40),
    progress_bar: ThemeColor::new(255, 176, 0),
};

pub const DRACULA: Theme = Theme {
    name: "dracula",
    header: ThemeColor::new(189, 147, 249),
    accent: ThemeColor::new(255, 121, 198),
    success: ThemeColor::new(80, 250, 123),
    error: ThemeColor::new(255, 85, 85),
    warning: ThemeColor::new(241, 250, 140),
    info: ThemeColor::new(139, 233, 253),
    muted: ThemeColor::new(98, 114, 164),
    border: ThemeColor::new(68, 71, 90),
    value: ThemeColor::new(248, 248, 242),
    label: ThemeColor::new(189, 147, 249),
    progress_bar: ThemeColor::new(255, 121, 198),
};

const ALL_THEMES: [&Theme; 12] = [
    &SYNTHWAVE84,
    &SYNTHWAVE_NIGHT,
    &SYNTHWAVE_SUNSET,
    &NEON_CITY,
    &DARK,
    &LIGHT,
    &OCEAN,
    &FOREST,
    &SUNSET,
    &MIDNIGHT,
    &RETRO,
    &DRACULA,
];

pub fn get_default_theme() -> &'static Theme {
    &SYNTHWAVE84
}

pub fn get_theme(name: &str) -> &'static Theme {
    let lower = name.to_lowercase();
    ALL_THEMES
        .iter()
        .find(|t| t.name == lower)
        .copied()
        .unwrap_or(&SYNTHWAVE84)
}

pub fn load_from_config(cfg: &crate::config::Config) -> &'static Theme {
    get_theme(&cfg.theme)
}

pub fn available_themes() -> Vec<&'static str> {
    ALL_THEMES.iter().map(|t| t.name).collect()
}

pub fn style_header(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.header, false)
}

pub fn style_success(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.success, false)
}

pub fn style_error(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.error, false)
}

pub fn style_warning(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.warning, false)
}

pub fn style_info(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.info, false)
}

pub fn style_accent(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.accent, false)
}

pub fn style_muted(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.muted, false)
}

pub fn style_label(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.label, false)
}

pub fn style_value(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.value, false)
}

pub fn style_border(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.border, false)
}

pub fn style_bold_header(text: &str, theme: &Theme) -> StyledString {
    StyledString::new(text, theme.header, true)
}

pub fn theme_preview(theme: &Theme) -> String {
    let lines = vec![
        format!(
            "{}",
            style_bold_header(&format!("═══ {} ═══", theme.name), theme)
        ),
        format!(
            "  {} {}",
            style_label("header:", theme),
            style_header("Electric purple text", theme)
        ),
        format!(
            "  {} {}",
            style_label("accent:", theme),
            style_accent("Highlighted text", theme)
        ),
        format!(
            "  {} {}",
            style_label("success:", theme),
            style_success("✔ Backup complete", theme)
        ),
        format!(
            "  {} {}",
            style_label("error:", theme),
            style_error("✖ Operation failed", theme)
        ),
        format!(
            "  {} {}",
            style_label("warning:", theme),
            style_warning("⚠ Low disk space", theme)
        ),
        format!(
            "  {} {}",
            style_label("info:", theme),
            style_info("i 3 backups found", theme)
        ),
        format!(
            "  {} {}",
            style_label("muted:", theme),
            style_muted("Secondary detail", theme)
        ),
        format!(
            "  {} {}",
            style_label("value:", theme),
            style_value("42 backups, 1.2 GB", theme)
        ),
        format!(
            "  {} {}",
            style_label("border:", theme),
            style_border("───────────────────", theme)
        ),
        format!(
            "  {} {}",
            style_label("progress:", theme),
            StyledString::new("███", theme.progress_bar, false)
        ),
    ];
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, RetentionConfig};

    #[test]
    fn all_twelve_themes_available() {
        let themes = available_themes();
        assert_eq!(themes.len(), 12);
    }

    #[test]
    fn expected_theme_names_present() {
        let themes = available_themes();
        let expected = [
            "synthwave84",
            "synthwave-night",
            "synthwave-sunset",
            "neon-city",
            "dark",
            "light",
            "ocean",
            "forest",
            "sunset",
            "midnight",
            "retro",
            "dracula",
        ];
        for name in &expected {
            assert!(themes.contains(name), "missing theme: {name}");
        }
    }

    #[test]
    fn get_synthwave84_by_name() {
        let theme = get_theme("synthwave84");
        assert_eq!(theme.name, "synthwave84");
    }

    #[test]
    fn get_theme_case_insensitive() {
        assert_eq!(get_theme("SYNTHWAVE84").name, "synthwave84");
        assert_eq!(get_theme("Dracula").name, "dracula");
        assert_eq!(get_theme("NEON-CITY").name, "neon-city");
    }

    #[test]
    fn unknown_theme_falls_back_to_default() {
        let theme = get_theme("nonexistent");
        assert_eq!(theme.name, "synthwave84");
    }

    #[test]
    fn default_theme_is_synthwave84() {
        assert_eq!(super::get_default_theme().name, "synthwave84");
    }

    #[test]
    fn load_from_config_with_custom_theme() {
        let cfg = Config {
            archive_dir: std::path::PathBuf::from("/tmp/test"),
            db_path: std::path::PathBuf::from("/tmp/test.db"),
            default_compression: 3,
            repo_paths: vec![],
            retention: RetentionConfig {
                keep_daily: 7,
                keep_weekly: 4,
                keep_monthly: 12,
            },
            theme: "dracula".to_string(),
        };
        assert_eq!(load_from_config(&cfg).name, "dracula");
    }

    #[test]
    fn style_functions_produce_output() {
        let theme = get_theme("synthwave84");

        // StyledString Display strips ANSI when checking plain text via to_string()
        // But we verify the text content is preserved
        let header = format!("{}", style_header("Header", theme));
        assert!(header.contains("Header"));
        assert!(header.contains("\x1b[")); // contains ANSI codes

        let success = format!("{}", style_success("OK", theme));
        assert!(success.contains("OK"));

        let error = format!("{}", style_error("FAIL", theme));
        assert!(error.contains("FAIL"));

        let warning = format!("{}", style_warning("WARN", theme));
        assert!(warning.contains("WARN"));

        let info = format!("{}", style_info("INFO", theme));
        assert!(info.contains("INFO"));

        let accent = format!("{}", style_accent("Accent", theme));
        assert!(accent.contains("Accent"));

        let muted = format!("{}", style_muted("muted", theme));
        assert!(muted.contains("muted"));

        let label = format!("{}", style_label("label:", theme));
        assert!(label.contains("label:"));

        let value = format!("{}", style_value("42", theme));
        assert!(value.contains("42"));

        let border = format!("{}", style_border("---", theme));
        assert!(border.contains("---"));

        let bold = format!("{}", style_bold_header("BOLD", theme));
        assert!(bold.contains("BOLD"));
        assert!(bold.contains("\x1b[1m")); // bold escape
    }

    #[test]
    fn style_functions_emit_24bit_truecolor() {
        let theme = get_theme("synthwave84");

        // Verify synthwave84 header emits 24-bit truecolor for #8f00ff (143, 0, 255)
        let header = format!("{}", style_header("test", theme));
        assert!(
            header.contains("38;2;143;0;255"),
            "Header should use truecolor #8f00ff, got: {:?}",
            header
        );

        // Verify accent emits truecolor for #ff00ff (255, 0, 255)
        let accent = format!("{}", style_accent("test", theme));
        assert!(
            accent.contains("38;2;255;0;255"),
            "Accent should use truecolor #ff00ff, got: {:?}",
            accent
        );

        // Verify success emits truecolor for #03edf9 (3, 237, 249)
        let success = format!("{}", style_success("test", theme));
        assert!(
            success.contains("38;2;3;237;249"),
            "Success should use truecolor #03edf9, got: {:?}",
            success
        );
    }

    #[test]
    fn theme_preview_not_empty() {
        let preview = theme_preview(get_theme("synthwave84"));
        assert!(!preview.is_empty());
        assert!(preview.contains("synthwave84"));
    }

    #[test]
    fn each_theme_has_unique_name() {
        let themes = available_themes();
        let mut seen = std::collections::HashSet::new();
        for name in &themes {
            assert!(seen.insert(*name), "duplicate theme: {name}");
        }
    }
}
