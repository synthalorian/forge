#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod archive;
mod backup;
mod chunkstore;
mod cli;
mod config;
mod db;
mod error;
mod models;
mod restore;
mod scheduler;
mod theme;
mod theme_cmd;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("forge=info".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
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
        Commands::Backup(args) => {
            let cfg = config::Config::load()?;
            backup::run(&cfg, &args)?;
        }
        Commands::Restore(args) => {
            let cfg = config::Config::load()?;
            restore::run(&cfg, &args)?;
        }
        Commands::List(args) => {
            let cfg = config::Config::load()?;
            db::list_backups(&cfg, &args)?;
        }
        Commands::Schedule(args) => {
            let cfg = config::Config::load()?;
            scheduler::run(&cfg, &args)?;
        }
        Commands::Theme(args) => {
            theme_cmd::run(&args.action)?;
        }
        Commands::Status => {
            let cfg = config::Config::load()?;
            db::show_status(&cfg)?;
        }
    }

    Ok(())
}
