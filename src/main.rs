#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod anvil;
mod archive;
mod backup;
mod bridge;
mod chunkstore;
mod cli;
mod config;
mod crucible;
mod db;
mod error;
mod mind;
mod mind_cmd;
mod models;
mod reflect;
mod restore;
mod scheduler;
mod spirit;
mod spirit_cmd;
mod theme;
mod theme_cmd;
mod tongs;
mod utils;
mod workshop;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("forge=info".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        None => {
            // No subcommand — show dashboard / help
            let theme = theme::get_default_theme();
            println!(
                "{} {} {}",
                theme::style_accent("forge", theme),
                theme::style_value("— Craft Your Digital Future", theme),
                theme::style_muted("Run 'forge --help' for commands.", theme),
            );
        }
        Some(Commands::Init) => {
            let cfg = config::Config::default();
            let theme = theme::get_default_theme();
            cfg.save()?;
            println!(
                "{} {} {} {}",
                theme::style_success("✓", theme),
                theme::style_accent("forge", theme),
                theme::style_value("initialized.", theme),
                theme::style_muted("Config written to ~/.config/forge/config.toml", theme),
            );
        }
        Some(Commands::Backup(args)) => {
            let cfg = config::Config::load()?;
            backup::run(&cfg, &args)?;
        }
        Some(Commands::Restore(args)) => {
            let cfg = config::Config::load()?;
            restore::run(&cfg, &args)?;
        }
        Some(Commands::List(args)) => {
            let cfg = config::Config::load()?;
            db::list_backups(&cfg, &args)?;
        }
        Some(Commands::Schedule(args)) => {
            let cfg = config::Config::load()?;
            scheduler::run(&cfg, &args)?;
        }
        Some(Commands::Theme(args)) => {
            theme_cmd::run(&args.action)?;
        }
        Some(Commands::Status) => {
            let cfg = config::Config::load()?;
            db::show_status(&cfg)?;
        }
        Some(Commands::Word(args)) => {
            let cfg = config::Config::load()?;
            spirit_cmd::run_word(&cfg, &args)?;
        }
        Some(Commands::Reflect(args)) => {
            let cfg = config::Config::load()?;
            spirit_cmd::run_reflect(&cfg, &args)?;
        }
        Some(Commands::Rest) => {
            let cfg = config::Config::load()?;
            spirit_cmd::run_rest(&cfg)?;
        }
        Some(Commands::Breathe(args)) => {
            let cfg = config::Config::load()?;
            mind_cmd::run_breathe(&cfg, &args)?;
        }
        Some(Commands::Strike(args)) => {
            let cfg = config::Config::load()?;
            mind_cmd::run_strike(&cfg, &args)?;
        }
        Some(Commands::Temper) => {
            let cfg = config::Config::load()?;
            anvil::run_temper(&cfg)?;
        }
        Some(Commands::Anvil(args)) => {
            let cfg = config::Config::load()?;
            anvil::run_anvil(&cfg, &args)?;
        }
        Some(Commands::Grip(args)) => {
            let cfg = config::Config::load()?;
            tongs::run_grip(&cfg, &args)?;
        }
        Some(Commands::Melt(args)) => {
            let cfg = config::Config::load()?;
            crucible::run_melt(&cfg, &args)?;
        }
        Some(Commands::Bridge(args)) => {
            let cfg = config::Config::load()?;
            bridge::run_bridge(&cfg, &args)?;
        }
        Some(Commands::Heat) => {
            let cfg = config::Config::load()?;
            workshop::run_heat(&cfg)?;
        }
        Some(Commands::Anneal) => {
            let cfg = config::Config::load()?;
            workshop::run_anneal(&cfg)?;
        }
        Some(Commands::Alloy) => {
            let cfg = config::Config::load()?;
            workshop::run_alloy(&cfg)?;
        }
        Some(Commands::Cast) => {
            let cfg = config::Config::load()?;
            workshop::run_cast(&cfg)?;
        }
        Some(Commands::Grind) => {
            let cfg = config::Config::load()?;
            workshop::run_grind(&cfg)?;
        }
        Some(Commands::Polish) => {
            let cfg = config::Config::load()?;
            workshop::run_polish(&cfg)?;
        }
    }

    Ok(())
}
