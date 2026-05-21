//! Crucible module — Phase 3: Creative tools (music theory, color palettes, diagrams).
//!
//! The Crucible is where raw material becomes something beautiful.

use anyhow::{Context, Result};
use image::GenericImageView;
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
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

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
