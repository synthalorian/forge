use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::cli::RestoreArgs;
use crate::config::Config;

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Run the restore command.
///
/// Locates a backup archive by ID or name, verifies its integrity,
/// and extracts it to the target directory. Optionally checks out a
/// specific ref (branch/tag) after extraction.
pub fn run(cfg: &Config, args: &RestoreArgs) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let conn = crate::db::connect(cfg).context("Failed to connect to forge database")?;

    let entry = crate::db::get_backup_by_id(&conn, &args.backup_id)
        .context("Failed to look up backup")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "{}",
                crate::theme::style_error(&format!("Backup not found: {}", args.backup_id), theme,)
            )
        })?;

    let output_dir = match &args.output {
        Some(dir) => dir.clone(),
        None => format!("./restored/{}", entry.repo_name),
    };

    if args.dry_run {
        println!(
            "{}",
            crate::theme::style_bold_header("Dry run — would restore the following backup:", theme)
        );
        println!();
        println!(
            "  {}  {}",
            crate::theme::style_label("ID:", theme),
            crate::theme::style_value(&entry.id.to_string(), theme)
        );
        println!(
            "  {}  {}",
            crate::theme::style_label("Repo:", theme),
            crate::theme::style_value(&entry.repo_name, theme)
        );
        println!(
            "  {}  {}",
            crate::theme::style_label("Archive:", theme),
            crate::theme::style_value(&entry.archive_path, theme)
        );
        println!(
            "  {}  {}",
            crate::theme::style_label("Size:", theme),
            crate::theme::style_value(&format_size(entry.size_bytes), theme)
        );
        println!(
            "  {}  {}",
            crate::theme::style_label("Branches:", theme),
            crate::theme::style_value(&entry.branch_count.to_string(), theme)
        );
        println!(
            "  {}  {}",
            crate::theme::style_label("Tags:", theme),
            crate::theme::style_value(&entry.tag_count.to_string(), theme)
        );
        println!(
            "  {}  {}",
            crate::theme::style_label("Commits:", theme),
            crate::theme::style_value(&entry.commit_count.to_string(), theme)
        );
        println!(
            "  {}  {}",
            crate::theme::style_label("Type:", theme),
            crate::theme::style_value(&format!("{:?}", entry.backup_type), theme)
        );
        println!(
            "  {}  {}",
            crate::theme::style_label("Created:", theme),
            crate::theme::style_value(
                &entry.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                theme
            ),
        );
        println!();
        println!(
            "  {}  {}",
            crate::theme::style_label("Target dir:", theme),
            crate::theme::style_value(&output_dir, theme)
        );
        if let Some(ref_name) = &args.ref_name {
            println!(
                "  {} {}",
                crate::theme::style_label("Checkout ref:", theme),
                crate::theme::style_value(ref_name, theme)
            );
        }
        return Ok(());
    }

    let is_dedup = entry.archive_path.ends_with(".manifest.json");

    if is_dedup {
        println!(
            "{}",
            crate::theme::style_info("Extracting from dedup archive...", theme)
        );
        crate::archive::extract_dedup_archive(cfg, &entry.archive_path, &output_dir)
            .context("Failed to extract dedup archive")?;
        println!(
            "  {}",
            crate::theme::style_success("Extraction complete", theme)
        );
    } else {
        println!(
            "{}",
            crate::theme::style_info("Verifying archive integrity...", theme)
        );
        let valid = crate::archive::verify_archive(&entry.archive_path, &entry.sha256)
            .context("Failed to verify archive integrity")?;
        if !valid {
            bail!(
                "{}",
                crate::theme::style_error("Archive corrupted: SHA-256 mismatch", theme)
            );
        }
        println!(
            "  {}",
            crate::theme::style_success("SHA-256 verified OK", theme)
        );

        println!(
            "  {}",
            crate::theme::style_info(&format!("Extracting archive to {output_dir}..."), theme),
        );
        crate::archive::extract_archive(&entry.archive_path, &output_dir)
            .context("Failed to extract archive")?;
        println!(
            "  {}",
            crate::theme::style_success("Extraction complete", theme)
        );
    }

    if let Some(ref_name) = &args.ref_name {
        // Archives contain a bare repo directory named "{repo}.git"
        let bare_repo_dir = format!("{}.git", entry.repo_name);
        let bare_repo_path = Path::new(&output_dir).join(&bare_repo_dir);

        let bare_path_str = if bare_repo_path.exists() {
            bare_repo_path.to_string_lossy().into_owned()
        } else {
            let found = fs::read_dir(&output_dir)
                .with_context(|| format!("Failed to read output directory {output_dir}"))?
                .filter_map(|e| e.ok())
                .find(|e| {
                    e.path()
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().ends_with(".git"))
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not find a bare git repository (.git) in {output_dir}. \
                         Cannot check out ref '{ref_name}'."
                    )
                })?;
            println!(
                "  {}",
                crate::theme::style_muted(
                    &format!("Found bare repo at {}", found.path().display()),
                    theme
                ),
            );
            found.path().to_string_lossy().into_owned()
        };

        clone_and_checkout(
            &bare_path_str,
            &output_dir,
            &entry.repo_name,
            ref_name,
            theme,
        )?;
    }

    println!();
    println!(
        "{} {} {} {}",
        crate::theme::style_success("Restored", theme),
        crate::theme::style_accent(&entry.repo_name, theme),
        crate::theme::style_muted("to", theme),
        crate::theme::style_value(&output_dir, theme),
    );
    Ok(())
}

fn clone_and_checkout(
    bare_repo_path: &str,
    output_dir: &str,
    repo_name: &str,
    ref_name: &str,
    theme: &crate::theme::Theme,
) -> Result<()> {
    let working_dir = format!("{output_dir}/{repo_name}");

    println!(
        "  {}",
        crate::theme::style_info("Cloning bare repo to working directory...", theme)
    );
    let status = Command::new("git")
        .args(["clone", bare_repo_path, &working_dir])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .context("Failed to spawn git clone")?;

    if !status.success() {
        bail!(
            "{}",
            crate::theme::style_error(
                &format!("git clone from bare repo failed with exit code {}", status),
                theme,
            )
        );
    }

    println!(
        "  {}",
        crate::theme::style_info(&format!("Checking out ref '{ref_name}'..."), theme),
    );
    let status = Command::new("git")
        .args(["-C", &working_dir, "checkout", ref_name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .context("Failed to spawn git checkout")?;

    if !status.success() {
        bail!(
            "{}",
            crate::theme::style_error(
                &format!("git checkout '{ref_name}' failed with exit code {}", status),
                theme,
            )
        );
    }

    println!(
        "  {}",
        crate::theme::style_success(&format!("Checked out '{ref_name}' successfully"), theme),
    );
    Ok(())
}
