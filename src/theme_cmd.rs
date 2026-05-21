use anyhow::{Context, Result};
use std::io::Write;

use crate::cli::ThemeAction;
use crate::config::Config;

pub fn run(action: &ThemeAction) -> Result<()> {
    match action {
        ThemeAction::List => {
            let themes = crate::theme::available_themes();
            let current_cfg = Config::load().ok();
            let current_name = current_cfg
                .as_ref()
                .map(|c| c.theme.as_str())
                .unwrap_or("synthwave84");

            println!(
                "{}",
                crate::theme::style_bold_header(
                    "Available Themes",
                    crate::theme::get_default_theme()
                )
            );
            println!(
                "{}",
                crate::theme::style_border(
                    "────────────────────────────────",
                    crate::theme::get_default_theme()
                )
            );

            for name in &themes {
                let theme = crate::theme::get_theme(name);
                if *name == current_name {
                    println!(
                        "  {} {} {}",
                        crate::theme::style_accent(&format!("• {name}"), theme),
                        crate::theme::style_muted("(12 colors)", theme),
                        crate::theme::style_success(
                            "← active",
                            crate::theme::get_theme(current_name)
                        )
                    );
                } else {
                    println!(
                        "  {} {}",
                        crate::theme::style_accent(&format!("• {name}"), theme),
                        crate::theme::style_muted("(12 colors)", theme)
                    );
                }
            }

            println!();
            println!(
                "Run {} to see a theme in action.",
                crate::theme::style_value(
                    "forge theme preview <name>",
                    crate::theme::get_default_theme()
                )
            );
        }
        ThemeAction::Preview { name } => {
            let theme_name = name.as_deref().unwrap_or("synthwave84");
            let theme = crate::theme::get_theme(theme_name);
            println!("{}", crate::theme::theme_preview(theme));
        }
        ThemeAction::Set { name } => {
            let available = crate::theme::available_themes();
            let lower = name.to_lowercase();
            if !available.contains(&lower.as_str()) {
                anyhow::bail!(
                    "Unknown theme '{}'. Run 'forge theme list' to see available themes.",
                    name
                );
            }

            let mut cfg = Config::load().context("Run 'forge init' first.")?;
            cfg.theme = lower;
            cfg.save().context("Failed to save config")?;

            let active_theme = crate::theme::get_theme(&cfg.theme);
            println!(
                "{} Theme set to {}",
                crate::theme::style_success("✔", active_theme),
                crate::theme::style_bold_header(&cfg.theme, active_theme),
            );
        }
        ThemeAction::Create => {
            run_create()?;
        }
        ThemeAction::Export { name, format } => {
            run_export(name.as_deref(), format.as_deref())?;
        }
    }

    Ok(())
}

fn run_create() -> Result<()> {
    let default = crate::theme::get_default_theme();

    println!();
    println!(
        "{}",
        crate::theme::style_bold_header("Forge — Interactive Theme Builder", default)
    );
    println!(
        "{}",
        crate::theme::style_border("═══════════════════════════════════════", default)
    );
    println!(
        "{}",
        crate::theme::style_muted(
            "Enter hex colors (e.g. #8f00ff) for each of the 12 color slots.",
            default
        )
    );
    println!(
        "{}",
        crate::theme::style_muted("Press Enter to accept the default value shown in brackets.", default)
    );
    println!();

    let name = prompt("Theme name", "my-custom-theme")?;

    let slots = [
        ("header", "Primary heading color", "#8f00ff"),
        ("accent", "Highlight accent color", "#a855f7"),
        ("success", "Success / positive color", "#03edf9"),
        ("error", "Error / danger color", "#ff0040"),
        ("warning", "Warning color", "#f3e70f"),
        ("info", "Info / notice color", "#ff7edb"),
        ("muted", "Subdued / secondary text", "#614d85"),
        ("border", "Border and divider lines", "#8f00ff"),
        ("value", "Data values / numbers", "#ffffff"),
        ("label", "Label text", "#8f00ff"),
        ("progress_bar", "Progress bar fill", "#a855f7"),
    ];

    let mut values = Vec::new();
    for (slot, desc, default_hex) in &slots {
        let hex = prompt_hex(slot, desc, default_hex)?;
        values.push(hex);
    }

    let theme = crate::theme::CustomThemeDef {
        name: name.clone(),
        header: values[0].clone(),
        accent: values[1].clone(),
        success: values[2].clone(),
        error: values[3].clone(),
        warning: values[4].clone(),
        info: values[5].clone(),
        muted: values[6].clone(),
        border: values[7].clone(),
        value: values[8].clone(),
        label: values[9].clone(),
        progress_bar: values[10].clone(),
    };

    // Write the TOML file
    let dir = crate::theme::custom_theme_dir();
    std::fs::create_dir_all(&dir)?;
    let file_path = dir.join(format!("{}.toml", name.to_lowercase()));
    let toml_str = toml::to_string_pretty(&theme)?;
    std::fs::write(&file_path, &toml_str)?;

    println!();
    println!(
        "{}",
        crate::theme::style_success("✔ Theme created successfully!", default)
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Path:", default),
        crate::theme::style_value(file_path.to_string_lossy().as_ref(), default)
    );
    println!();
    println!(
        "{}",
        crate::theme::style_label("Activate it with:", default)
    );
    println!(
        "  {} {}",
        crate::theme::style_accent("forge theme set", default),
        crate::theme::style_value(&name.to_lowercase(), default)
    );
    println!();

    // Try to show preview if the theme loaded
    let theme_to_preview = crate::theme::get_theme(&name.to_lowercase());
    if theme_to_preview.name != "synthwave84" {
        println!(
            "{}",
            crate::theme::style_bold_header("Preview:", default)
        );
        println!("{}", crate::theme::theme_preview(theme_to_preview));
    }

    Ok(())
}

fn prompt(label: &str, default: &str) -> Result<String> {
    let theme = crate::theme::get_default_theme();
    print!(
        "  {} [{}]: ",
        crate::theme::style_label(label, theme),
        crate::theme::style_muted(default, theme),
    );
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();

    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed)
    }
}

fn prompt_hex(slot: &str, desc: &str, default: &str) -> Result<String> {
    let theme = crate::theme::get_default_theme();
    loop {
        print!(
            "  {} {} {} [{}]: ",
            crate::theme::style_label(slot, theme),
            crate::theme::style_muted(&format!("({desc})"), theme),
            crate::theme::style_value("●", theme),
            crate::theme::style_muted(default, theme),
        );
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();

        if trimmed.is_empty() {
            return Ok(default.to_string());
        }

        // Validate hex color
        let hex = trimmed.trim_start_matches('#');
        if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(if trimmed.starts_with('#') { trimmed } else { format!("#{hex}") });
        }

        println!(
            "  {} Invalid hex color. Use format #RRGGBB (e.g. {})",
            crate::theme::style_error("✗", theme),
            crate::theme::style_value(default, theme),
        );
    }
}

fn run_export(name: Option<&str>, format: Option<&str>) -> Result<()> {
    let cfg = Config::load().ok();
    let theme_name = name
        .or_else(|| cfg.as_ref().map(|c| c.theme.as_str()))
        .unwrap_or("synthwave84");
    let theme = crate::theme::get_theme(theme_name);
    let fmt = format.unwrap_or("alacritty").to_lowercase();

    let export = match fmt.as_str() {
        "kitty" => export_kitty(theme),
        "ghostty" => export_ghostty(theme),
        _ => export_alacritty(theme),
    };

    let t = crate::theme::get_default_theme();
    println!();
    println!("{} {} → {}", crate::theme::style_success("✓", t), crate::theme::style_accent(theme_name, t), crate::theme::style_value(&fmt, t));
    println!("{}", crate::theme::style_border(&"─".repeat(48), t));
    println!("{export}");
    Ok(())
}

fn darken_color(c: &crate::theme::ThemeColor, amount: u8) -> String {
    let r = c.r.saturating_sub(amount);
    let g = c.g.saturating_sub(amount);
    let b = c.b.saturating_sub(amount);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn brighten_color(c: &crate::theme::ThemeColor, amount: u8) -> String {
    let r = c.r.saturating_add(amount);
    let g = c.g.saturating_add(amount);
    let b = c.b.saturating_add(amount);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn export_alacritty(theme: &crate::theme::Theme) -> String {
    let bg = darken_color(&theme.header, 200);
    let fg = brighten_color(&theme.value, 0);
    let cursor = brighten_color(&theme.accent, 30);
    format!(
        r##"[colors.primary]
background = "{bg}"
foreground = "{fg}"

[colors.cursor]
cursor = "{cursor}"
text = "{bg}"

[colors.normal]
black = "{bg}"
red = "#{er:02x}{eg:02x}{eb:02x}"
green = "#{sr:02x}{sg:02x}{sb:02x}"
yellow = "#{wr:02x}{wg:02x}{wb:02x}"
blue = "#{hr:02x}{hg:02x}{hb:02x}"
magenta = "#{ir:02x}{ig:02x}{ib:02x}"
cyan = "#{ar:02x}{ag:02x}{ab:02x}"
white = "{fg}"

[colors.bright]
black = "#{mr:02x}{mg:02x}{mb:02x}"
red = "{ebr}"
green = "{sbr}"
yellow = "{wbr}"
blue = "{hbr}"
magenta = "{ibr}"
cyan = "#{ar:02x}{ag:02x}{ab:02x}"
white = "{fg}"
"##,
        bg = bg, fg = fg, cursor = cursor,
        er = theme.error.r, eg = theme.error.g, eb = theme.error.b,
        sr = theme.success.r, sg = theme.success.g, sb = theme.success.b,
        wr = theme.warning.r, wg = theme.warning.g, wb = theme.warning.b,
        hr = theme.header.r, hg = theme.header.g, hb = theme.header.b,
        ir = theme.info.r, ig = theme.info.g, ib = theme.info.b,
        ar = theme.accent.r, ag = theme.accent.g, ab = theme.accent.b,
        mr = theme.muted.r, mg = theme.muted.g, mb = theme.muted.b,
        ebr = brighten_color(&theme.error, 40),
        sbr = brighten_color(&theme.success, 40),
        wbr = brighten_color(&theme.warning, 40),
        hbr = brighten_color(&theme.header, 40),
        ibr = brighten_color(&theme.info, 40),
    )
}

fn export_kitty(theme: &crate::theme::Theme) -> String {
    let bg = darken_color(&theme.header, 200);
    let fg = brighten_color(&theme.value, 0);
    let cursor = brighten_color(&theme.accent, 30);
    format!(
        r##"# Kitty theme derived from forge "{name}"
foreground {fg}
background {bg}
cursor {cursor}
cursor_text_color {bg}
selection_foreground {fg}
selection_background #{hr:02x}{hg:02x}{hb:02x}

# black
color0 {bg}
color8 #{mr:02x}{mg:02x}{mb:02x}

# red
color1 #{er:02x}{eg:02x}{eb:02x}
color9 {ebr}

# green
color2 #{sr:02x}{sg:02x}{sb:02x}
color10 {sbr}

# yellow
color3 #{wr:02x}{wg:02x}{wb:02x}
color11 {wbr}

# blue
color4 #{hr:02x}{hg:02x}{hb:02x}
color12 {hbr}

# magenta
color5 #{ir:02x}{ig:02x}{ib:02x}
color13 {ibr}

# cyan
color6 #{ar:02x}{ag:02x}{ab:02x}
color14 #{ar:02x}{ag:02x}{ab:02x}

# white
color7 {fg}
color15 {fg}
"##,
        name = theme.name,
        bg = bg, fg = fg, cursor = cursor,
        hr = theme.header.r, hg = theme.header.g, hb = theme.header.b,
        mr = theme.muted.r, mg = theme.muted.g, mb = theme.muted.b,
        er = theme.error.r, eg = theme.error.g, eb = theme.error.b,
        sr = theme.success.r, sg = theme.success.g, sb = theme.success.b,
        wr = theme.warning.r, wg = theme.warning.g, wb = theme.warning.b,
        ir = theme.info.r, ig = theme.info.g, ib = theme.info.b,
        ar = theme.accent.r, ag = theme.accent.g, ab = theme.accent.b,
        ebr = brighten_color(&theme.error, 40),
        sbr = brighten_color(&theme.success, 40),
        wbr = brighten_color(&theme.warning, 40),
        hbr = brighten_color(&theme.header, 40),
        ibr = brighten_color(&theme.info, 40),
    )
}

fn export_ghostty(theme: &crate::theme::Theme) -> String {
    let bg = darken_color(&theme.header, 200);
    let fg = brighten_color(&theme.value, 0);
    let cursor = brighten_color(&theme.accent, 30);
    format!(
        r##"# Ghostty theme derived from forge "{name}"
[color]
background = {bg}
foreground = {fg}
cursor-color = {cursor}
cursor-text = {bg}
selection-foreground = {fg}
selection-background = #{hr:02x}{hg:02x}{hb:02x}

# Black / Bright Black
color0 = {bg}
color8 = #{mr:02x}{mg:02x}{mb:02x}

# Red / Bright Red
color1 = #{er:02x}{eg:02x}{eb:02x}
color9 = {ebr}

# Green / Bright Green
color2 = #{sr:02x}{sg:02x}{sb:02x}
color10 = {sbr}

# Yellow / Bright Yellow
color3 = #{wr:02x}{wg:02x}{wb:02x}
color11 = {wbr}

# Blue / Bright Blue
color4 = #{hr:02x}{hg:02x}{hb:02x}
color12 = {hbr}

# Magenta / Bright Magenta
color5 = #{ir:02x}{ig:02x}{ib:02x}
color13 = {ibr}

# Cyan / Bright Cyan
color6 = #{ar:02x}{ag:02x}{ab:02x}
color14 = #{ar:02x}{ag:02x}{ab:02x}

# White / Bright White
color7 = {fg}
color15 = {fg}
"##,
        name = theme.name,
        bg = bg, fg = fg, cursor = cursor,
        hr = theme.header.r, hg = theme.header.g, hb = theme.header.b,
        mr = theme.muted.r, mg = theme.muted.g, mb = theme.muted.b,
        er = theme.error.r, eg = theme.error.g, eb = theme.error.b,
        sr = theme.success.r, sg = theme.success.g, sb = theme.success.b,
        wr = theme.warning.r, wg = theme.warning.g, wb = theme.warning.b,
        ir = theme.info.r, ig = theme.info.g, ib = theme.info.b,
        ar = theme.accent.r, ag = theme.accent.g, ab = theme.accent.b,
        ebr = brighten_color(&theme.error, 40),
        sbr = brighten_color(&theme.success, 40),
        wbr = brighten_color(&theme.warning, 40),
        hbr = brighten_color(&theme.header, 40),
        ibr = brighten_color(&theme.info, 40),
    )
}
