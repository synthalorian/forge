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
