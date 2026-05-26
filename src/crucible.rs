//! Crucible module — Phase 3: Creative tools (music theory, color palettes, diagrams).
//!
//! The Crucible is where raw material becomes something beautiful.

use anyhow::{Context, Result};
use image::GenericImageView;
use image::Rgb;
use std::collections::HashMap;

use crate::cli::{MeltAction, MeltArgs};
use crate::config::Config;

pub fn run_melt(cfg: &Config, args: &MeltArgs) -> Result<()> {
    match &args.action {
        MeltAction::Chords { key, scale, mood } => run_chords(cfg, key, scale, mood),
        MeltAction::Palette {
            color,
            harmony,
            format,
            file,
        } => run_palette(cfg, color, harmony, format, file),
        MeltAction::Diagram {
            diag_type,
            description,
        } => run_diagram(cfg, diag_type, description),
        MeltAction::Markdown { path } => run_markdown(cfg, path),
        MeltAction::Image {
            prompt,
            width,
            height,
            output,
        } => run_image(cfg, prompt, width, height, output),
        MeltAction::Fractal {
            preset,
            axiom,
            rule,
            iterations,
            angle,
            output,
        } => run_fractal(cfg, preset, axiom, rule, iterations, angle, output),
    }
}

// ── Chord Progressions ──────────────────────────────────────────────

fn run_chords(
    cfg: &Config,
    key: &Option<String>,
    scale: &Option<String>,
    mood: &Option<String>,
) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Melt — Chord Progressions", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    // Default key and scale
    let key = key.as_deref().unwrap_or("C");
    let scale = scale.as_deref().unwrap_or("major");

    // Parse key to root note
    let (root_note, is_minor) = parse_key(key);
    let effective_scale = if is_minor {
        "minor".to_string()
    } else {
        scale.to_lowercase()
    };

    println!(
        "  {} {} {}",
        crate::theme::style_label("Key:", theme),
        crate::theme::style_value(&format!("{} {}", root_note, effective_scale), theme),
        crate::theme::style_muted(
            &format!(
                "({})",
                if mood.is_some() {
                    mood.as_deref().unwrap_or("")
                } else {
                    ""
                }
            ),
            theme
        ),
    );
    println!();

    // Scale intervals (semitones from root)
    let intervals: &[i32] = match effective_scale.as_str() {
        "minor" | "natural minor" => &[0, 2, 3, 5, 7, 8, 10],
        "dorian" => &[0, 2, 3, 5, 7, 9, 10],
        "mixolydian" => &[0, 2, 4, 5, 7, 9, 10],
        "phrygian" => &[0, 1, 3, 5, 7, 8, 10],
        "harmonic minor" => &[0, 2, 3, 5, 7, 8, 11],
        "melodic minor" => &[0, 2, 3, 5, 7, 9, 11],
        "blues" => &[0, 3, 5, 6, 7, 10],
        _ => &[0, 2, 4, 5, 7, 9, 11], // major / ionian
    };

    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let root_idx = note_names.iter().position(|&n| n == root_note).unwrap_or(0);

    // Build scale notes
    let scale_notes: Vec<String> = intervals
        .iter()
        .map(|&i| note_names[(root_idx as i32 + i) as usize % 12].to_string())
        .collect();

    println!(
        "  {} {}",
        crate::theme::style_label("Scale:", theme),
        crate::theme::style_value(&scale_notes.join(" — "), theme),
    );
    println!();

    // Build chord types for each scale degree
    let chord_qualities = match effective_scale.as_str() {
        "minor" | "natural minor" | "harmonic minor" => &["m", "°", "+", "m", "m", "", "°"],
        "dorian" => &["m", "m", "+", "", "", "°", "°"],
        "mixolydian" => &["", "", "m", "°", "", "m", "°"],
        _ => &["", "m", "m", "", "", "m", "°"], // major
    };

    let roman_numerals = ["I", "II", "III", "IV", "V", "VI", "VII"];
    let degree_count = scale_notes
        .len()
        .min(roman_numerals.len())
        .min(chord_qualities.len());

    println!("  {}", crate::theme::style_header("Diatonic Chords", theme));
    for i in 0..degree_count {
        let quality = if chord_qualities[i].is_empty() {
            ""
        } else {
            chord_qualities[i]
        };
        println!(
            "  {} {} — {}{}",
            crate::theme::style_value(roman_numerals[i], theme),
            crate::theme::style_label(&format!("{}:", roman_numerals[i]), theme),
            crate::theme::style_accent(&scale_notes[i], theme),
            crate::theme::style_muted(quality, theme),
        );
    }
    println!();

    // Common progressions
    let progressions = if mood.is_some() {
        match mood.as_deref().unwrap_or("") {
            "happy" => vec![
                ("Happy Pop", vec!["I", "V", "vi", "IV"]),
                ("Upbeat", vec!["I", "IV", "I", "V"]),
                ("Bright", vec!["I", "II", "V", "I"]),
            ],
            "sad" => vec![
                ("Sad", vec!["vi", "IV", "I", "V"]),
                ("Melancholy", vec!["i", "iv", "V", "i"]),
                ("Minor Ballad", vec!["i", "VI", "III", "VII"]),
            ],
            "epic" => vec![
                ("Epic", vec!["i", "III", "VII", "VI"]),
                ("Cinematic", vec!["i", "iv", "VI", "V"]),
                ("Power", vec!["I", "V", "IV", "I"]),
            ],
            "chill" => vec![
                ("Chill", vec!["ii", "V", "I", "vi"]),
                ("Lofi", vec!["iii", "vii", "I", "IV"]),
                ("Jazzy", vec!["ii7", "V7", "Imaj7", "vi7"]),
            ],
            "worship" => vec![
                ("Praise", vec!["I", "IV", "I", "V"]),
                ("Hymn", vec!["I", "IV", "V", "I"]),
                ("Modern Worship", vec!["I", "V", "vi", "IV"]),
            ],
            _ => vec![
                ("Pop Classic", vec!["I", "V", "vi", "IV"]),
                ("50s Progression", vec!["I", "vi", "IV", "V"]),
                ("Jazz ii-V-I", vec!["ii", "V", "I"]),
            ],
        }
    } else {
        vec![
            ("Pop Classic", vec!["I", "V", "vi", "IV"]),
            ("50s Progression", vec!["I", "vi", "IV", "V"]),
            ("Canon", vec!["I", "V", "vi", "iii", "IV", "I", "IV", "V"]),
            (
                "Blues 12-bar",
                vec![
                    "I", "I", "I", "I", "IV", "IV", "I", "I", "V", "IV", "I", "V",
                ],
            ),
            ("Sad", vec!["vi", "IV", "I", "V"]),
            ("Jazz ii-V-I", vec!["ii", "V", "I"]),
        ]
    };

    println!(
        "  {}",
        crate::theme::style_header("Common Progressions", theme)
    );
    for (name, prog) in &progressions {
        println!(
            "  {} {}",
            crate::theme::style_accent(&format!("{:<20}", name), theme),
            crate::theme::style_value(&prog.join(" → "), theme),
        );
    }

    Ok(())
}

fn parse_key(key: &str) -> (String, bool) {
    let key = key.trim();
    let is_minor = key.ends_with('m') && !key.starts_with('m');
    let root = if is_minor { &key[..key.len() - 1] } else { key };
    let note_names = [
        "C", "C#", "Db", "D", "D#", "Eb", "E", "F", "F#", "Gb", "G", "G#", "Ab", "A", "A#", "Bb",
        "B",
    ];

    let normalized = note_names.iter().find(|&&n| n.eq_ignore_ascii_case(root));
    (normalized.unwrap_or(&"C").to_string(), is_minor)
}

// ── Color Palettes ──────────────────────────────────────────────────

fn run_palette(
    cfg: &Config,
    color: &Option<String>,
    harmony: &Option<String>,
    format: &Option<String>,
    file: &Option<String>,
) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Melt — Color Palette", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    // If --file is provided, extract palette from image
    if let Some(image_path) = file {
        return extract_palette_from_image(cfg, image_path, format);
    }

    // Parse base color
    let base = color.as_deref().unwrap_or("#FF6B9D");
    let harmony = harmony.as_deref().unwrap_or("complementary");
    let output_format = format.as_deref().unwrap_or("terminal");

    let base_rgb = parse_hex_color(base);
    let base_hsl = rgb_to_hsl(base_rgb);

    println!(
        "  {} {} (HSL: {:.0}°, {:.0}%, {:.0}%)",
        crate::theme::style_label("Base:", theme),
        crate::theme::style_value(base, theme),
        base_hsl.0,
        base_hsl.1,
        base_hsl.2,
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Harmony:", theme),
        crate::theme::style_value(harmony, theme),
    );
    println!();

    // Generate palette based on harmony type
    let palette: Vec<(String, String, String)> = match harmony {
        "complementary" => {
            let comp = hsl_to_rgb((base_hsl.0 + 180.0, base_hsl.1, base_hsl.2));
            let comp_hex = rgb_to_hex(comp);
            vec![
                (base.to_string(), "Base".to_string(), "primary".to_string()),
                (comp_hex, "Complement".to_string(), "complement".to_string()),
            ]
        }
        "analogous" => {
            let mut colors = Vec::new();
            for offset in &[-30.0, 0.0, 30.0] {
                let h = (base_hsl.0 + offset + 360.0) % 360.0;
                let rgb = hsl_to_rgb((h, base_hsl.1, base_hsl.2));
                let name = if *offset < 0.0 {
                    "Cool"
                } else if *offset == 0.0 {
                    "Base"
                } else {
                    "Warm"
                };
                colors.push((rgb_to_hex(rgb), name.to_string(), name.to_lowercase()));
            }
            colors
        }
        "triadic" => {
            let mut colors = Vec::new();
            for offset in &[0.0, 120.0, 240.0] {
                let h = (base_hsl.0 + offset) % 360.0;
                let rgb = hsl_to_rgb((h, base_hsl.1, base_hsl.2));
                let name = if *offset == 0.0 {
                    "Base"
                } else if *offset == 120.0 {
                    "Triad A"
                } else {
                    "Triad B"
                };
                colors.push((rgb_to_hex(rgb), name.to_string(), name.to_lowercase()));
            }
            colors
        }
        "split" => {
            let mut colors = Vec::new();
            for offset in &[0.0, 150.0, 210.0] {
                let h = (base_hsl.0 + offset) % 360.0;
                let rgb = hsl_to_rgb((h, base_hsl.1, base_hsl.2));
                let name = if *offset == 0.0 {
                    "Base"
                } else if *offset == 150.0 {
                    "Split A"
                } else {
                    "Split B"
                };
                colors.push((rgb_to_hex(rgb), name.to_string(), name.to_lowercase()));
            }
            colors
        }
        _ => {
            vec![(base.to_string(), "Base".to_string(), "primary".to_string())]
        }
    };

    match output_format {
        "css" => {
            println!("  :root {{");
            for (hex, _, name) in &palette {
                println!("    --color-{}: {};", name, hex);
            }
            println!("  }}");
        }
        "tailwind" => {
            println!("  // tailwind.config.js");
            println!("  colors: {{");
            for (hex, _, name) in &palette {
                println!("    '{}': '{}',", name, hex);
            }
            println!("  }}");
        }
        _ => {
            // Terminal output with colored blocks
            for (hex, name, _) in &palette {
                let (_r, _g, _b) = parse_hex_color(hex);
                println!(
                    "  {} {}",
                    crate::theme::style_value(&format!("{:<15}", name), theme),
                    crate::theme::style_accent(hex, theme),
                );
            }
        }
    }

    println!();

    // Also show tints and shades
    println!("  {}", crate::theme::style_header("Tints & Shades", theme));
    for lightness in &[20.0, 35.0, 50.0, 65.0, 80.0] {
        let rgb = hsl_to_rgb((base_hsl.0, base_hsl.1, *lightness));
        let hex = rgb_to_hex(rgb);
        println!(
            "  {} {}",
            crate::theme::style_value(&format!("{:.0}%", lightness), theme),
            crate::theme::style_accent(&hex, theme),
        );
    }

    Ok(())
}

// ── Diagrams ────────────────────────────────────────────────────────

fn run_diagram(
    cfg: &Config,
    diag_type: &Option<String>,
    description: &Option<String>,
) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let diag_type = diag_type.as_deref().unwrap_or("flow");

    println!(
        "{}",
        crate::theme::style_bold_header(
            &format!("Forge Melt — {} Diagram", capitalize(diag_type)),
            theme
        )
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    match diag_type {
        "flow" => {
            println!();
            println!("  ┌─────────┐");
            println!("  │  START   │");
            println!("  └────┬────┘");
            println!("       │");
            println!("       ▼");
            println!("  ┌─────────┐");
            println!("  │ PROCESS  │");
            println!("  └────┬────┘");
            println!("       │");
            println!("       ▼");
            println!("  ┌─────────┐     ┌─────────┐");
            println!("  │ DECIDE? ├─NO─→│  ACTION  │");
            println!("  └────┬────┘     └────┬────┘");
            println!("       │ YES            │");
            println!("       ▼                ▼");
            println!("  ┌─────────┐     ┌─────────┐");
            println!("  │  RESULT  │     │  RETRY   │");
            println!("  └────┬────┘     └─────────┘");
            println!("       │");
            println!("       ▼");
            println!("  ┌─────────┐");
            println!("  │   END    │");
            println!("  └─────────┘");
        }
        "sequence" => {
            println!();
            println!("  Client        Server        Database");
            println!("    │              │              │");
            println!("    │── REQUEST ──→│              │");
            println!("    │              │── QUERY ────→│");
            println!("    │              │←── RESULT ──│");
            println!("    │              │              │");
            println!("    │              │── PROCESS    │");
            println!("    │              │              │");
            println!("    │← RESPONSE ──│              │");
            println!("    │              │              │");
        }
        "architecture" => {
            println!();
            println!("  ╔══════════════════════════════════════════╗");
            println!("  ║           FORGE ARCHITECTURE             ║");
            println!("  ╠══════════════════════════════════════════╣");
            println!("  ║                                          ║");
            println!("  ║   ┌──────────┐  ┌──────────┐  ┌──────┐  ║");
            println!("  ║   │   Anvil   │  │  Bellows │  │Flame │  ║");
            println!("  ║   │ (backup)  │  │  (AI)    │  │(spirit)│ ║");
            println!("  ║   └─────┬─────┘  └────┬─────┘  └──┬───┘  ║");
            println!("  ║         │              │           │       ║");
            println!("  ║   ┌─────┴──────────────┴───────────┴───┐  ║");
            println!("  ║   │          Forge Core Engine           │  ║");
            println!("  ║   │    (SQLite, themes, config, CLI)     │  ║");
            println!("  ║   └────────────────┬───────────────────┘  ║");
            println!("  ║                    │                       ║");
            println!("  ║   ┌────────────────┴───────────────────┐  ║");
            println!("  ║   │     ChunkStore + Archive Engine      │  ║");
            println!("  ║   │   (dedup, zstd, content-addressable)  │  ║");
            println!("  ║   └─────────────────────────────────────┘  ║");
            println!("  ║                                          ║");
            println!("  ╚══════════════════════════════════════════╝");
        }
        _ => {
            println!(
                "  {}",
                crate::theme::style_muted(
                    "Unknown diagram type. Use: flow, sequence, architecture",
                    theme
                )
            );
        }
    }

    if let Some(desc) = description {
        println!();
        println!(
            "  {} {}",
            crate::theme::style_label("Description:", theme),
            crate::theme::style_muted(desc, theme),
        );
    }

    Ok(())
}

// ── Color Helpers ───────────────────────────────────────────────────

fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (255, 107, 157) // Default pink
    }
}

fn rgb_to_hex((r, g, b): (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn rgb_to_hsl((r, g, b): (u8, u8, u8)) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l * 100.0);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
    } else if max == g {
        ((b - r) / d + 2.0) * 60.0
    } else {
        ((r - g) / d + 4.0) * 60.0
    };

    (h, s * 100.0, l * 100.0)
}

fn hsl_to_rgb((h, s, l): (f64, f64, f64)) -> (u8, u8, u8) {
    let s = s / 100.0;
    let l = l / 100.0;

    if s.abs() < f64::EPSILON {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let hue_to_rgb = |t: f64| -> f64 {
        let t = if t < 0.0 {
            t + 1.0
        } else if t > 1.0 {
            t - 1.0
        } else {
            t
        };
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };

    let h_norm = h / 360.0;
    let r = (hue_to_rgb(h_norm + 1.0 / 3.0) * 255.0) as u8;
    let g = (hue_to_rgb(h_norm) * 255.0) as u8;
    let b = (hue_to_rgb(h_norm - 1.0 / 3.0) * 255.0) as u8;

    (r, g, b)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

// ── Markdown Renderer ──────────────────────────────────────────────

fn run_markdown(cfg: &Config, path: &Option<String>) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    let content = match path {
        Some(p) if p == "-" => {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
            buf
        }
        Some(p) => std::fs::read_to_string(p)
            .with_context(|| format!("Failed to read markdown file: {p}"))?,
        None => {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
            if buf.trim().is_empty() {
                anyhow::bail!("No content. Provide a file path or pipe markdown to stdin.");
            }
            buf
        }
    };

    // Detect a document title from the first H1
    let title = content.lines().find(|l| l.starts_with("# "));
    if let Some(t) = title {
        let title_text = t.trim_start_matches("# ").trim();
        println!(
            "{}",
            crate::theme::style_bold_header(&format!("📄 {}", title_text), theme)
        );
        println!("{}", crate::theme::style_border(&"═".repeat(50), theme));
    } else {
        println!(
            "{}",
            crate::theme::style_bold_header("📄 Markdown Preview", theme)
        );
        println!("{}", crate::theme::style_border(&"═".repeat(50), theme));
    }

    let mut in_code_block = false;
    let mut in_blockquote = false;

    for raw_line in content.lines() {
        let line = raw_line.trim_end();

        // Fenced code blocks
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                let lang = line.trim_start_matches("```").trim();
                if !lang.is_empty() {
                    println!(
                        "  {} {}",
                        crate::theme::style_label("┌─", theme),
                        crate::theme::style_muted(lang, theme),
                    );
                } else {
                    println!("  {}", crate::theme::style_label("┌─", theme));
                }
            } else {
                println!("  {}", crate::theme::style_label("└─", theme));
            }
            continue;
        }

        if in_code_block {
            println!(
                "  {} {}",
                crate::theme::style_label("│", theme),
                crate::theme::style_value(line, theme),
            );
            continue;
        }

        // Blank line
        if line.trim().is_empty() {
            if in_blockquote {
                in_blockquote = false;
            }
            println!();
            continue;
        }

        // Thematic break
        if line == "---" || line == "***" || line == "___" {
            println!("  {}", crate::theme::style_border(&"─".repeat(48), theme));
            continue;
        }

        // Blockquote
        if line.starts_with("> ") || line.starts_with('>') {
            let quote_text = line.trim_start_matches('>').trim();
            if !in_blockquote {
                in_blockquote = true;
            }
            println!(
                "  {} {}",
                crate::theme::style_label("▍", theme),
                crate::theme::style_muted(quote_text, theme),
            );
            continue;
        }

        in_blockquote = false;

        // Headers
        if let Some(rest) = line.strip_prefix("###### ") {
            println!("  {}", crate::theme::style_muted(rest.trim(), theme));
            continue;
        }
        if let Some(rest) = line.strip_prefix("##### ") {
            println!("  {}", crate::theme::style_muted(rest.trim(), theme));
            continue;
        }
        if let Some(rest) = line.strip_prefix("#### ") {
            println!("  {}", crate::theme::style_muted(rest.trim(), theme));
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            println!(
                "  {}",
                crate::theme::style_accent(&inline_format(rest.trim(), theme), theme)
            );
            continue;
        }
        if let Some(rest) = line.strip_prefix("## ") {
            println!(
                "  {}",
                crate::theme::style_header(&inline_format(rest.trim(), theme), theme)
            );
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            // H1 already used for title, render muted
            println!(
                "  {}",
                crate::theme::style_muted(&inline_format(rest.trim(), theme), theme)
            );
            continue;
        }

        // Unordered list
        let list_match = line.trim_start().starts_with("- ")
            || line.trim_start().starts_with("* ")
            || line.trim_start().starts_with("+ ");
        if list_match {
            let content = line
                .trim_start()
                .trim_start_matches(&['-', '*', '+'][..])
                .trim();
            println!(
                "  {} {}",
                crate::theme::style_accent("•", theme),
                crate::theme::style_value(&inline_format(content, theme), theme),
            );
            continue;
        }

        // Ordered list
        if let Some(idx_str) = line.trim_start().split('.').next() {
            if let Ok(idx) = idx_str.parse::<u32>() {
                let after_dot = line
                    .trim_start()
                    .trim_start_matches(&format!("{}.", idx))
                    .trim();
                println!(
                    "  {} {}",
                    crate::theme::style_accent(&format!("{}.", idx), theme),
                    crate::theme::style_value(&inline_format(after_dot, theme), theme),
                );
                continue;
            }
        }

        // Paragraph — render with basic inline formatting
        println!(
            "  {}",
            crate::theme::style_value(&inline_format(line, theme), theme)
        );
    }

    println!();
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));
    if let Some(p) = path {
        if p != "-" {
            println!(
                "  {} {}",
                crate::theme::style_muted("Source:", theme),
                crate::theme::style_value(p, theme),
            );
        }
    }
    println!();

    Ok(())
}

/// Apply basic inline markdown formatting (**bold**, *italic*, `code`, [links](url))
fn inline_format(text: &str, theme: &crate::theme::Theme) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // Escape — next char is literal
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            '*' => {
                // Could be **bold** or *italic*
                if chars.peek() == Some(&'*') {
                    // Bold: **text**
                    chars.next(); // consume second *
                    let mut inner = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '*' && chars.clone().nth(1) == Some('*') {
                            chars.next(); // consume first *
                            chars.next(); // consume second *
                            break;
                        }
                        match chars.next() {
                            Some(c) => inner.push(c),
                            None => break,
                        }
                    }
                    let _ = write!(
                        out,
                        "\x1b[1m{}\x1b[0m{}",
                        crate::theme::style_accent(&inner, theme),
                        crate::theme::style_value("", theme)
                    );
                } else {
                    // Italic: *text*
                    let mut inner = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '*' {
                            chars.next();
                            break;
                        }
                        match chars.next() {
                            Some(c) => inner.push(c),
                            None => break,
                        }
                    }
                    let _ = write!(
                        out,
                        "\x1b[3m{}\x1b[23m",
                        crate::theme::style_muted(&inner, theme)
                    );
                }
            }
            '`' => {
                // Inline code
                let mut inner = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '`' {
                        chars.next();
                        break;
                    }
                    match chars.next() {
                        Some(c) => inner.push(c),
                        None => break,
                    }
                }
                let _ = write!(out, "{}", crate::theme::style_value(&inner, theme));
            }
            '[' => {
                // Link: [text](url)
                let mut link_text = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        chars.next();
                        break;
                    }
                    match chars.next() {
                        Some(c) => link_text.push(c),
                        None => break,
                    }
                }
                let mut url = String::new();
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume (
                    while let Some(&c) = chars.peek() {
                        if c == ')' {
                            chars.next();
                            break;
                        }
                        match chars.next() {
                            Some(c) => url.push(c),
                            None => break,
                        }
                    }
                }
                if url.is_empty() {
                    let _ = write!(out, "[{}]", link_text);
                } else {
                    let _ = write!(
                        out,
                        "{} {}",
                        crate::theme::style_accent(&link_text, theme),
                        crate::theme::style_muted(&format!("({})", url), theme)
                    );
                }
            }
            _ => out.push(ch),
        }
    }

    out
}

// ── Palette from Image ─────────────────────────────────────────────

fn extract_palette_from_image(
    cfg: &Config,
    image_path: &str,
    format: &Option<String>,
) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let output_format = format.as_deref().unwrap_or("terminal");

    let img =
        image::open(image_path).with_context(|| format!("Failed to open image: {image_path}"))?;
    let (w, h) = img.dimensions();
    let total_pixels = (w as u64) * (h as u64);

    println!(
        "  {} {} ({}×{} — {} px)",
        crate::theme::style_label("Image:", theme),
        crate::theme::style_value(image_path, theme),
        crate::theme::style_muted(&w.to_string(), theme),
        crate::theme::style_muted(&h.to_string(), theme),
        crate::theme::style_value(&total_pixels.to_string(), theme),
    );

    // Resize to a max of 200px on longest edge for speed
    let max_dim = 200u32;
    let resized = if w > max_dim || h > max_dim {
        let scale = max_dim as f64 / w.max(h) as f64;
        let nw = (w as f64 * scale) as u32;
        let nh = (h as f64 * scale) as u32;
        img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // Collect all pixels into quantized color buckets
    let mut color_counts: HashMap<u32, u64> = HashMap::new();
    let quantization = 32u32; // 8 levels per channel → 512 distinct buckets

    for pixel in resized.pixels() {
        let r = pixel.2[0] as u32 / quantization * quantization;
        let g = pixel.2[1] as u32 / quantization * quantization;
        let b = pixel.2[2] as u32 / quantization * quantization;
        let key = (r << 16) | (g << 8) | b;
        *color_counts.entry(key).or_insert(0) += 1;
    }

    // Sort by frequency, take top 8
    let mut sorted: Vec<(u32, u64)> = color_counts.into_iter().collect();
    use std::cmp::Reverse;
    sorted.sort_by_key(|(_, count)| Reverse(*count));
    let dominant: Vec<(u8, u8, u8)> = sorted
        .iter()
        .take(8)
        .map(|(key, _)| {
            let r = ((key >> 16) & 0xFF) as u8;
            let g = ((key >> 8) & 0xFF) as u8;
            let b = (key & 0xFF) as u8;
            (r, g, b)
        })
        .collect();

    println!();
    println!("  {}", crate::theme::style_header("Dominant Colors", theme));

    match output_format {
        "css" => {
            println!("  :root {{");
            for (i, (r, g, b)) in dominant.iter().enumerate() {
                let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                println!("    --color-{i}: {hex};");
            }
            println!("  }}");
        }
        "tailwind" => {
            println!("  // tailwind.config.js");
            println!("  colors: {{");
            for (i, (r, g, b)) in dominant.iter().enumerate() {
                let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                println!("    'extract-{i}': '{hex}',");
            }
            println!("  }}");
        }
        _ => {
            for (i, (r, g, b)) in dominant.iter().enumerate() {
                let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                let hsl = rgb_to_hsl((*r, *g, *b));
                println!(
                    "  {} {} {} (HSL: {:.0}°, {:.0}%, {:.0}%)",
                    crate::theme::style_value(&format!("{}.", i + 1), theme),
                    crate::theme::style_accent(&hex, theme),
                    crate::theme::style_muted(&hex, theme),
                    hsl.0,
                    hsl.1,
                    hsl.2,
                );
            }
        }
    }

    println!();
    println!(
        "  {}",
        crate::theme::style_muted(
            &format!("Extracted from {} sampled pixels", resized.pixels().count()),
            theme,
        )
    );
    println!();

    Ok(())
}

// ── Procedural Image Generation ────────────────────────────────────

fn run_image(
    cfg: &Config,
    prompt: &str,
    width: &Option<u32>,
    height: &Option<u32>,
    output: &Option<String>,
) -> Result<()> {
    use image::{ImageBuffer, Rgb, RgbImage};
    use std::path::PathBuf;

    let theme = crate::theme::load_from_config(cfg);
    let w = width.unwrap_or(800);
    let h = height.unwrap_or(600);

    let img_dir = match output {
        Some(p) => PathBuf::from(p),
        None => {
            let xdg = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
                format!("{}/.local/share", std::env::var("HOME").unwrap_or_default())
            });
            PathBuf::from(xdg).join("forge").join("images")
        }
    };
    std::fs::create_dir_all(&img_dir)?;
    let colors = parse_prompt_colors(prompt);

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Melt — Image Generation", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));
    println!(
        "  {} {}",
        crate::theme::style_label("Prompt:", theme),
        crate::theme::style_value(prompt, theme)
    );
    println!(
        "  {} {}×{}",
        crate::theme::style_label("Size:", theme),
        crate::theme::style_value(&w.to_string(), theme),
        crate::theme::style_value(&h.to_string(), theme)
    );
    println!(
        "  {} {} colors",
        crate::theme::style_label("Palette:", theme),
        crate::theme::style_value(&colors.len().to_string(), theme)
    );

    let mut img: RgbImage = ImageBuffer::new(w, h);
    let seed = prompt
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let mut rng = SimpleRng::new(seed);

    let bg_top = colors[0 % colors.len()];
    let bg_bot = colors[1 % colors.len()];
    for y in 0..h {
        let t = y as f64 / h.max(1) as f64;
        let r = lerp(bg_top.0, bg_bot.0, t);
        let g = lerp(bg_top.1, bg_bot.1, t);
        let b = lerp(bg_top.2, bg_bot.2, t);
        for x in 0..w {
            img.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }

    for i in 0..(12 + (seed % 8) as usize) {
        let col = colors[i % colors.len()];
        let alpha = rng.gen_range(30u32, 180u32) as u8;
        match rng.gen_range(0u32, 3u32) {
            0 => draw_filled_circle(
                &mut img,
                rng.gen_range(0u32, w),
                rng.gen_range(0u32, h),
                rng.gen_range(20u32, w.min(h) / 3),
                col,
                alpha,
            ),
            1 => draw_filled_rect(
                &mut img,
                rng.gen_range(0u32, w),
                rng.gen_range(0u32, h),
                rng.gen_range(0u32, w.min(w / 3)),
                rng.gen_range(0u32, h.min(h / 3)),
                col,
                alpha,
            ),
            _ => draw_line(
                &mut img,
                rng.gen_range(0u32, w),
                rng.gen_range(0u32, h),
                rng.gen_range(0u32, w),
                rng.gen_range(0u32, h),
                col,
                2 + (i as u32 % 6),
            ),
        }
    }

    let noise = 10 + (seed % 15) as u8;
    for _ in 0..(w * h / 20) {
        let x = rng.gen_range(0u32, w);
        let y = rng.gen_range(0u32, h);
        let px = img.get_pixel(x, y);
        let nr = (px[0] as i16 + rng.gen_range(0u32, noise as u32 * 2) as i16 - noise as i16)
            .clamp(0, 255) as u8;
        let ng = (px[1] as i16 + rng.gen_range(0u32, noise as u32 * 2) as i16 - noise as i16)
            .clamp(0, 255) as u8;
        let nb = (px[2] as i16 + rng.gen_range(0u32, noise as u32 * 2) as i16 - noise as i16)
            .clamp(0, 255) as u8;
        img.put_pixel(x, y, Rgb([nr, ng, nb]));
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let slug: String = prompt
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .take(30)
        .collect::<String>()
        .replace(' ', "_");
    let outpath = img_dir.join(format!("forge_{}_{}.png", timestamp, slug));
    img.save(&outpath)?;
    let file_size = std::fs::metadata(&outpath).map(|m| m.len()).unwrap_or(0);

    println!();
    println!(
        "  {} {}",
        crate::theme::style_success("✓", theme),
        crate::theme::style_value("Image generated!", theme)
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Saved:", theme),
        crate::theme::style_value(&outpath.display().to_string(), theme)
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Size:", theme),
        crate::theme::style_value(&crate::utils::format_size(file_size), theme)
    );
    println!();
    println!("  {}", crate::theme::style_header("Color Palette", theme));
    for (i, (r, g, b)) in colors.iter().enumerate() {
        println!(
            "  {} {}",
            crate::theme::style_value(&format!("{}.", i + 1), theme),
            crate::theme::style_accent(&format!("#{:02X}{:02X}{:02X}", r, g, b), theme)
        );
    }
    println!();
    Ok(())
}

fn parse_prompt_colors(prompt: &str) -> Vec<(u8, u8, u8)> {
    let lower = prompt.to_lowercase();
    let named: Vec<(&str, (u8, u8, u8))> = vec![
        ("red", (220, 40, 60)),
        ("crimson", (180, 20, 40)),
        ("fire", (255, 80, 30)),
        ("orange", (255, 140, 30)),
        ("yellow", (240, 220, 50)),
        ("gold", (255, 200, 60)),
        ("green", (40, 180, 60)),
        ("forest", (30, 120, 50)),
        ("emerald", (20, 200, 120)),
        ("teal", (20, 180, 180)),
        ("cyan", (30, 210, 230)),
        ("blue", (40, 100, 220)),
        ("ocean", (20, 100, 200)),
        ("sky", (100, 180, 240)),
        ("indigo", (80, 60, 200)),
        ("purple", (140, 40, 200)),
        ("violet", (180, 60, 200)),
        ("magenta", (220, 40, 180)),
        ("pink", (240, 100, 180)),
        ("rose", (220, 80, 120)),
        ("brown", (140, 80, 50)),
        ("grey", (140, 140, 150)),
        ("gray", (140, 140, 150)),
        ("white", (240, 240, 245)),
        ("black", (20, 20, 30)),
        ("neon", (0, 255, 200)),
        ("synthwave", (143, 0, 255)),
    ];

    let mut p = Vec::new();
    if lower.contains("sunset") {
        p.extend_from_slice(&[
            (255, 100, 60),
            (200, 50, 100),
            (255, 180, 80),
            (100, 20, 80),
        ]);
    } else if lower.contains("synthwave") || lower.contains("neon") || lower.contains("retro") {
        p.extend_from_slice(&[(143, 0, 255), (255, 0, 128), (3, 237, 249), (255, 126, 219)]);
    } else if lower.contains("ocean") || lower.contains("sea") || lower.contains("water") {
        p.extend_from_slice(&[
            (20, 80, 180),
            (40, 160, 210),
            (10, 200, 200),
            (200, 230, 255),
        ]);
    } else if lower.contains("forest") || lower.contains("nature") {
        p.extend_from_slice(&[(30, 100, 40), (60, 160, 60), (100, 180, 60), (80, 50, 30)]);
    } else if lower.contains("space") || lower.contains("galaxy") {
        p.extend_from_slice(&[(10, 10, 40), (60, 20, 100), (140, 60, 180), (200, 160, 255)]);
    } else if lower.contains("fire") || lower.contains("lava") {
        p.extend_from_slice(&[(200, 30, 10), (255, 120, 20), (255, 200, 50), (100, 10, 5)]);
    } else if lower.contains("ice") || lower.contains("winter") {
        p.extend_from_slice(&[
            (200, 230, 255),
            (140, 200, 240),
            (80, 160, 220),
            (255, 255, 255),
        ]);
    } else {
        p.extend_from_slice(&[(100, 60, 180), (60, 140, 220), (220, 80, 140)]);
    }

    for (name, color) in &named {
        if lower.contains(name) {
            p.push(*color);
        }
    }
    p.dedup();
    p.truncate(6);
    if p.len() < 2 {
        p.push((60, 60, 180));
        p.push((180, 60, 60));
    }
    p
}

fn lerp(a: u8, b: u8, t: f64) -> f64 {
    a as f64 + (b as f64 - a as f64) * t
}

fn draw_filled_circle(
    img: &mut image::RgbImage,
    cx: u32,
    cy: u32,
    radius: u32,
    color: (u8, u8, u8),
    alpha: u8,
) {
    let r = radius as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let px = cx as i32 + dx;
                let py = cy as i32 + dy;
                if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                    let pix = img.get_pixel(px as u32, py as u32);
                    img.put_pixel(
                        px as u32,
                        py as u32,
                        Rgb([
                            blend(pix[0], color.0, alpha),
                            blend(pix[1], color.1, alpha),
                            blend(pix[2], color.2, alpha),
                        ]),
                    );
                }
            }
        }
    }
}

fn draw_filled_rect(
    img: &mut image::RgbImage,
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
    color: (u8, u8, u8),
    alpha: u8,
) {
    for y in y1..=y2.min(img.height() - 1) {
        for x in x1..=x2.min(img.width() - 1) {
            let pix = img.get_pixel(x, y);
            img.put_pixel(
                x,
                y,
                Rgb([
                    blend(pix[0], color.0, alpha),
                    blend(pix[1], color.1, alpha),
                    blend(pix[2], color.2, alpha),
                ]),
            );
        }
    }
}

fn draw_line(
    img: &mut image::RgbImage,
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
    color: (u8, u8, u8),
    thickness: u32,
) {
    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = -(y2 as i32 - y1 as i32).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x1 as i32;
    let mut y = y1 as i32;
    loop {
        for t in 0..thickness as i32 {
            for ty in 0..thickness as i32 {
                let px = (x + t - thickness as i32 / 2)
                    .max(0)
                    .min(img.width() as i32 - 1);
                let py = (y + ty - thickness as i32 / 2)
                    .max(0)
                    .min(img.height() as i32 - 1);
                img.put_pixel(px as u32, py as u32, Rgb([color.0, color.1, color.2]));
            }
        }
        if x == x2 as i32 && y == y2 as i32 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn blend(bg: u8, fg: u8, alpha: u8) -> u8 {
    ((fg as u16 * alpha as u16 + bg as u16 * (255 - alpha as u16)) / 255) as u8
}

struct SimpleRng {
    state: u64,
}
impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }
    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state >> 33
    }
    fn gen_range(&mut self, lo: u32, hi: u32) -> u32 {
        if lo >= hi {
            return lo;
        }
        lo + (self.next() as u32) % (hi - lo)
    }
}

// ── L-System / Fractal Generator ─────────────────────────────────────

fn run_fractal(
    cfg: &Config,
    preset: &Option<String>,
    axiom: &Option<String>,
    rule: &Option<String>,
    iterations: &Option<usize>,
    angle: &Option<f64>,
    output_format: &Option<String>,
) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    let n = iterations.unwrap_or(4);
    let n = n.clamp(1, 8);
    let angle_deg = angle.unwrap_or(90.0);
    let fmt = output_format.as_deref().unwrap_or("ascii");

    // Resolve preset or custom axiom/rules
    let (resolved_axiom, resolved_rules, resolved_angle, name) =
        resolve_lsystem_spec(preset, axiom, rule, angle_deg);

    println!(
        "{}",
        crate::theme::style_bold_header(&format!("Forge Melt — L-System: {}", name), theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));
    println!(
        "  {} {}",
        crate::theme::style_label("Axiom:", theme),
        crate::theme::style_value(&resolved_axiom, theme)
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Rules:", theme),
        crate::theme::style_value(&resolved_rules, theme)
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Iterations:", theme),
        crate::theme::style_value(&n.to_string(), theme)
    );
    println!(
        "  {} {}°",
        crate::theme::style_label("Angle:", theme),
        crate::theme::style_value(&resolved_angle.to_string(), theme)
    );
    println!();

    // Generate the L-system string
    let lstring = generate_lsystem(&resolved_axiom, &resolved_rules, n);

    println!(
        "  {} {} symbols",
        crate::theme::style_label("Generated:", theme),
        crate::theme::style_value(&lstring.len().to_string(), theme)
    );
    println!();

    if fmt == "svg" {
        render_lsystem_svg(&lstring, resolved_angle, &theme)?;
    } else {
        render_lsystem_ascii(&lstring, resolved_angle, &theme);
    }

    Ok(())
}

/// Named L-system presets.
struct LSystemSpec {
    axiom: String,
    rules: Vec<(char, String)>,
    angle: f64,
    name: String,
}

fn preset_lsystem(name: &str) -> Option<LSystemSpec> {
    match name.to_lowercase().as_str() {
        "koch" => Some(LSystemSpec {
            axiom: "F".to_string(),
            rules: vec![('F', "F+F-F-F+F".to_string())],
            angle: 90.0,
            name: "Koch Curve".to_string(),
        }),
        "dragon" => Some(LSystemSpec {
            axiom: "FX".to_string(),
            rules: vec![('X', "X+YF+".to_string()), ('Y', "-FX-Y".to_string())],
            angle: 90.0,
            name: "Dragon Curve".to_string(),
        }),
        "sierpinski" => Some(LSystemSpec {
            axiom: "F-G-G".to_string(),
            rules: vec![('F', "F-G+F+G-F".to_string()), ('G', "GG".to_string())],
            angle: 120.0,
            name: "Sierpinski Triangle".to_string(),
        }),
        "plant" | "tree" | "branching" => Some(LSystemSpec {
            axiom: "F".to_string(),
            rules: vec![('F', "FF+[+F-F-F]-[-F+F+F]".to_string())],
            angle: 25.0,
            name: "Branching Plant".to_string(),
        }),
        "hilbert" => Some(LSystemSpec {
            axiom: "A".to_string(),
            rules: vec![('A', "-BF+AFA+FB-".to_string()), ('B', "+AF-BFB-FA+".to_string())],
            angle: 90.0,
            name: "Hilbert Curve".to_string(),
        }),
        "gosper" | "flowsnake" => Some(LSystemSpec {
            axiom: "A".to_string(),
            rules: vec![('A', "A-B--B+A++AA+B-".to_string()), ('B', "+A-BB--B-A++A+B".to_string())],
            angle: 60.0,
            name: "Gosper Flowsnake".to_string(),
        }),
        "weed" => Some(LSystemSpec {
            axiom: "X".to_string(),
            rules: vec![('X', "F[-X][X]F[-X]+FX".to_string()), ('F', "FF".to_string())],
            angle: 30.0,
            name: "Weed".to_string(),
        }),
        _ => None,
    }
}

fn resolve_lsystem_spec(
    preset: &Option<String>,
    axiom: &Option<String>,
    rule: &Option<String>,
    default_angle: f64,
) -> (String, String, f64, String) {
    // Try preset first
    if let Some(p) = preset {
        if let Some(spec) = preset_lsystem(p) {
            let rules_str = spec
                .rules
                .iter()
                .map(|(k, v)| format!("{}→{}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            return (spec.axiom, rules_str, spec.angle, spec.name);
        }
    }

    // Use custom axiom and rules
    let ax = axiom
        .clone()
        .unwrap_or_else(|| "F".to_string());
    let rules_str = rule
        .clone()
        .unwrap_or_else(|| "F→F+F-F-F+F".to_string());
    let name = format!("\"{}\"", ax);

    (ax, rules_str, default_angle, name)
}

/// Parse rule specification string like "F→F+F-F-F+F, X→X+YF+"
fn parse_rules(spec: &str) -> Vec<(char, String)> {
    let mut rules = Vec::new();
    // Find the arrow character(s) to split on — support both ASCII and Unicode arrows
    for part in spec.split(',') {
        let part = part.trim();
        // Try Unicode arrow → first, then → with different forms, then ASCII ->
        let arrow_pos = part.find('→').or_else(|| part.find("->")).or_else(|| part.find("=>"));
        if let Some(pos) = arrow_pos {
            let key_char = part[..pos].trim().chars().next();
            // For Unicode → which is 3 bytes, we need to skip past the full arrow
            if part[pos..].starts_with('→') {
                if let Some(k) = key_char {
                    let val = part[pos + '→'.len_utf8()..].trim().to_string();
                    if !val.is_empty() {
                        rules.push((k, val));
                    }
                }
            } else if part[pos..].starts_with("->") {
                if let Some(k) = key_char {
                    let val = part[pos + 2..].trim().to_string();
                    if !val.is_empty() {
                        rules.push((k, val));
                    }
                }
            } else if part[pos..].starts_with("=>") {
                if let Some(k) = key_char {
                    let val = part[pos + 2..].trim().to_string();
                    if !val.is_empty() {
                        rules.push((k, val));
                    }
                }
            }
        }
    }
    rules
}

/// Apply production rules `n` times, starting from `axiom`.
fn generate_lsystem(axiom: &str, rule_spec: &str, iterations: usize) -> String {
    let rules = parse_rules(rule_spec);

    let mut current = axiom.to_string();
    for _i in 0..iterations {
        let mut next = String::with_capacity(current.len() * 2);
        for ch in current.chars() {
            let mut replaced = false;
            for (key, val) in &rules {
                if ch == *key {
                    next.push_str(val);
                    replaced = true;
                    break;
                }
            }
            if !replaced {
                next.push(ch);
            }
        }
        current = next;
    }
    current
}

/// Turtle state for interpretation.
#[derive(Clone, Copy)]
struct Turtle {
    x: f64,
    y: f64,
    angle: f64,
}

/// Render L-system string as ASCII art using Unicode box-drawing / line chars.
fn render_lsystem_ascii(lstring: &str, angle_deg: f64, theme: &crate::theme::Theme) {
    // Collect all line segments
    let mut segments: Vec<((i32, i32), (i32, i32))> = Vec::new();
    let mut turtle = Turtle {
        x: 0.0,
        y: 0.0,
        angle: -90.0, // start pointing up
    };
    let step = 1.0;
    let mut stack: Vec<Turtle> = Vec::new();

    // First pass: collect segments
    for ch in lstring.chars() {
        match ch {
            'F' | 'G' | 'A' | 'B' => {
                let start = (turtle.x, turtle.y);
                turtle.x += step * turtle.angle.to_radians().cos();
                turtle.y += step * turtle.angle.to_radians().sin();
                let end = (turtle.x, turtle.y);
                segments.push((
                    (start.0.round() as i32, start.1.round() as i32),
                    (end.0.round() as i32, end.1.round() as i32),
                ));
            }
            'f' => {
                // Move without drawing
                turtle.x += step * turtle.angle.to_radians().cos();
                turtle.y += step * turtle.angle.to_radians().sin();
            }
            '+' => {
                turtle.angle += angle_deg;
            }
            '-' => {
                turtle.angle -= angle_deg;
            }
            '[' => {
                stack.push(turtle);
            }
            ']' => {
                if let Some(saved) = stack.pop() {
                    turtle = saved;
                }
            }
            _ => {}
        }
    }

    if segments.is_empty() {
        println!(
            "  {}",
            crate::theme::style_muted("No drawable segments produced.", theme)
        );
        return;
    }

    // Determine bounds
    let min_x = segments
        .iter()
        .flat_map(|(a, b)| [a.0, b.0])
        .min()
        .unwrap_or(0);
    let max_x = segments
        .iter()
        .flat_map(|(a, b)| [a.0, b.0])
        .max()
        .unwrap_or(0);
    let min_y = segments
        .iter()
        .flat_map(|(a, b)| [a.1, b.1])
        .min()
        .unwrap_or(0);
    let max_y = segments
        .iter()
        .flat_map(|(a, b)| [a.1, b.1])
        .max()
        .unwrap_or(0);

    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;

    // Clamp to a reasonable canvas
    let canvas_w = width.min(100);
    let canvas_h = height.min(60);

    // Build a set of occupied cells and their connection directions
    use std::collections::HashSet;
    let mut cell: HashSet<(i32, i32)> = HashSet::new();
    let mut h_conn: HashSet<(i32, i32)> = HashSet::new(); // horizontal connections (right)
    let mut v_conn: HashSet<(i32, i32)> = HashSet::new(); // vertical connections (down)
    let mut d1_conn: HashSet<(i32, i32)> = HashSet::new(); // diagonal ↘ (down-right)
    let mut d2_conn: HashSet<(i32, i32)> = HashSet::new(); // diagonal ↙ (down-left)

    for (start, end) in &segments {
        let sx = start.0 - min_x;
        let sy = start.1 - min_y;
        let ex = end.0 - min_x;
        let ey = end.1 - min_y;

        cell.insert((sx, sy));
        cell.insert((ex, ey));

        let dx = ex - sx;
        let dy = ey - sy;

        if dx == 1 && dy == 0 {
            h_conn.insert((sx, sy));
        } else if dx == -1 && dy == 0 {
            h_conn.insert((ex, ey));
        } else if dx == 0 && dy == 1 {
            v_conn.insert((sx, sy));
        } else if dx == 0 && dy == -1 {
            v_conn.insert((ex, ey));
        } else if dx == 1 && dy == 1 {
            d1_conn.insert((sx, sy));
        } else if dx == -1 && dy == -1 {
            d1_conn.insert((ex, ey));
        } else if dx == -1 && dy == 1 {
            d2_conn.insert((sx, sy));
        } else if dx == 1 && dy == -1 {
            d2_conn.insert((ex, ey));
        }
    }

    // Render the grid
    println!("  {} {}×{}", 
        crate::theme::style_label("Canvas:", theme),
        crate::theme::style_value(&canvas_w.to_string(), theme),
        crate::theme::style_value(&canvas_h.to_string(), theme)
    );
    println!();

    for y in 0..canvas_h {
        let mut line = String::with_capacity(canvas_w);
        for x in 0..canvas_w {
            let cx = x as i32;
            let cy = y as i32;

            if !cell.contains(&(cx, cy)) {
                line.push(' ');
                continue;
            }

            let has_right = h_conn.contains(&(cx, cy));
            let has_down = v_conn.contains(&(cx, cy));
            let has_d1 = d1_conn.contains(&(cx, cy)); // ↘
            let has_d2 = d2_conn.contains(&(cx, cy)); // ↙
            let has_left = h_conn.contains(&(cx - 1, cy));
            let has_up = v_conn.contains(&(cx, cy - 1));
            let has_d1_inv = d1_conn.contains(&(cx - 1, cy - 1)); // ↖ (incoming ↘)
            let has_d2_inv = d2_conn.contains(&(cx + 1, cy - 1)); // ↗ (incoming ↙)

            let ch = match (has_right, has_down, has_left, has_up, has_d1, has_d2, has_d1_inv, has_d2_inv) {
                // Straight lines
                (true, false, true, false, false, false, false, false) => '─',
                (false, true, false, true, false, false, false, false) => '│',
                // Diagonals
                (false, false, false, false, true, false, false, false) => '╱',
                (false, false, false, false, false, true, false, false) => '╲',
                (false, false, false, false, false, false, true, false) => '╱',
                (false, false, false, false, false, false, false, true) => '╲',
                // Corners
                (true, true, false, false, false, false, false, false) => '┌',
                (false, true, true, false, false, false, false, false) => '┐',
                (true, false, false, true, false, false, false, false) => '└',
                (false, false, true, true, false, false, false, false) => '┘',
                // Crossings
                (true, true, true, false, false, false, false, false) => '├',
                (true, true, false, true, false, false, false, false) => '┴',
                (false, true, true, true, false, false, false, false) => '┤',
                (true, false, true, true, false, false, false, false) => '┬',
                (true, true, true, true, false, false, false, false) => '┼',
                // Isolated / default
                _ => '•',
            };
            line.push(ch);
        }
        if !line.trim().is_empty() {
            println!("  {}", crate::theme::style_accent(&line, theme));
        }
    }
    println!();
    println!(
        "  {} {} segments, {}×{} bounding box",
        crate::theme::style_success("✓", theme),
        crate::theme::style_value(&segments.len().to_string(), theme),
        crate::theme::style_muted(&width.to_string(), theme),
        crate::theme::style_muted(&height.to_string(), theme),
    );
}

/// Render L-system to SVG format printed to stdout.
fn render_lsystem_svg(
    lstring: &str,
    angle_deg: f64,
    theme: &crate::theme::Theme,
) -> Result<()> {
    let mut points: Vec<(f64, f64)> = Vec::new();
    let mut turtle = Turtle {
        x: 0.0,
        y: 0.0,
        angle: -90.0,
    };
    let step = 5.0;
    let mut stack: Vec<Turtle> = Vec::new();

    let mut min_x: f64 = 0.0;
    let mut max_x: f64 = 0.0;
    let mut min_y: f64 = 0.0;
    let mut max_y: f64 = 0.0;

    points.push((turtle.x, turtle.y));

    for ch in lstring.chars() {
        match ch {
            'F' | 'G' | 'A' | 'B' => {
                turtle.x += step * turtle.angle.to_radians().cos();
                turtle.y += step * turtle.angle.to_radians().sin();
                points.push((turtle.x, turtle.y));
                min_x = min_x.min(turtle.x);
                max_x = max_x.max(turtle.x);
                min_y = min_y.min(turtle.y);
                max_y = max_y.max(turtle.y);
            }
            'f' => {
                turtle.x += step * turtle.angle.to_radians().cos();
                turtle.y += step * turtle.angle.to_radians().sin();
            }
            '+' => {
                turtle.angle += angle_deg;
            }
            '-' => {
                turtle.angle -= angle_deg;
            }
            '[' => {
                stack.push(turtle);
            }
            ']' => {
                if let Some(saved) = stack.pop() {
                    turtle = saved;
                }
            }
            _ => {}
        }
    }

    if points.len() < 2 {
        println!(
            "  {}",
            crate::theme::style_muted("Not enough points for SVG.", theme)
        );
        return Ok(());
    }

    let padding = 20.0;
    let view_w = (max_x - min_x) + padding * 2.0;
    let view_h = (max_y - min_y) + padding * 2.0;

    // Build SVG path data from consecutive point pairs
    let mut path_data = String::new();
    for chunk in points.chunks(2) {
        if chunk.len() == 2 {
            let x1 = chunk[0].0 - min_x + padding;
            let y1 = chunk[0].1 - min_y + padding;
            let x2 = chunk[1].0 - min_x + padding;
            let y2 = chunk[1].1 - min_y + padding;
            path_data.push_str(&format!("M {:.1} {:.1} L {:.1} {:.1} ", x1, y1, x2, y2));
        }
    }

    // Detect theme foreground color for stroke
    let stroke = "currentColor";

    println!("{}", r#"  <svg xmlns="http://www.w3.org/2000/svg""#);
    println!(
        r#"    viewBox="0 0 {:.0} {:.0}" {}>"#,
        view_w.ceil(),
        view_h.ceil(),
        ""
    );
    println!("    <path d=\"{}\"", path_data.trim());
    println!(
        "          fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
        stroke
    );
    println!("  </svg>");

    println!();
    println!(
        "  {} {} points, path {} bytes",
        crate::theme::style_success("✓", theme),
        crate::theme::style_value(&points.len().to_string(), theme),
        crate::theme::style_muted(&path_data.len().to_string(), theme),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_major() {
        let (root, minor) = parse_key("C");
        assert_eq!(root, "C");
        assert!(!minor);
    }

    #[test]
    fn parse_key_minor() {
        let (root, minor) = parse_key("Am");
        assert_eq!(root, "A");
        assert!(minor);
    }

    #[test]
    fn parse_hex_color_valid() {
        let (r, g, b) = parse_hex_color("#FF6B9D");
        assert_eq!(r, 255);
        assert_eq!(g, 107);
        assert_eq!(b, 157);
    }

    #[test]
    fn rgb_hex_roundtrip() {
        let hex = "#FF6B9D";
        let rgb = parse_hex_color(hex);
        let back = rgb_to_hex(rgb);
        assert_eq!(hex, back);
    }

    #[test]
    fn hsl_roundtrip() {
        let rgb = (255, 107, 157);
        let hsl = rgb_to_hsl(rgb);
        let back = hsl_to_rgb(hsl);
        // Allow 1-unit tolerance for rounding
        assert!((back.0 as i32 - rgb.0 as i32).abs() <= 1);
        assert!((back.1 as i32 - rgb.1 as i32).abs() <= 1);
        assert!((back.2 as i32 - rgb.2 as i32).abs() <= 1);
    }

    #[test]
    fn capitalize_works() {
        assert_eq!(capitalize("flow"), "Flow");
        assert_eq!(capitalize("FLOW"), "FLOW");
    }
}
