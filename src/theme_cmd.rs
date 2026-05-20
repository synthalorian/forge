use anyhow::{Context, Result};

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
                        crate::theme::style_accent(&format!("• {}", name), theme),
                        crate::theme::style_muted("(12 colors)", theme),
                        crate::theme::style_success(
                            "← active",
                            crate::theme::get_theme(current_name)
                        )
                    );
                } else {
                    println!(
                        "  {} {}",
                        crate::theme::style_accent(&format!("• {}", name), theme),
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
    }

    Ok(())
}
