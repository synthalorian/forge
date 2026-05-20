use anyhow::Result;

use crate::cli::{ReflectAction, ReflectArgs, WordAction, WordArgs};
use crate::config::Config;

pub fn run_word(cfg: &Config, args: &WordArgs) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    match &args.action {
        Some(WordAction::Daily) => word_daily(cfg, theme),
        Some(WordAction::Search { query }) => word_search(cfg, theme, query),
        Some(WordAction::Reference {
            book,
            chapter,
            verse,
        }) => word_reference(cfg, theme, book, *chapter, *verse),
        None => word_daily(cfg, theme),
    }
}

pub fn run_reflect(cfg: &Config, args: &ReflectArgs) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    match &args.action {
        Some(ReflectAction::Entry { text }) => reflect_entry(cfg, theme, text),
        Some(ReflectAction::History) => reflect_history(cfg, theme),
        Some(ReflectAction::Read { id }) => reflect_read(cfg, theme, *id),
        Some(ReflectAction::Search { query }) => reflect_search(cfg, theme, query),
        None => reflect_history(cfg, theme),
    }
}

pub fn run_rest(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    sabbath_mode(theme)
}

fn word_daily(cfg: &Config, theme: &crate::theme::Theme) -> Result<()> {
    let verse = crate::spirit::daily_verse(cfg)?;

    println!();
    println!(
        "  {}",
        crate::theme::style_bold_header("\u{1f4d6} Today's Verse", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", theme)
    );
    println!(
        "  {}\"{}\"{}",
        crate::theme::style_accent("", theme),
        crate::theme::style_accent(&verse.text, theme),
        crate::theme::style_accent("", theme),
    );
    println!(
        "  {}{} {}:{}{}",
        crate::theme::style_muted("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500} ", theme),
        crate::theme::style_muted("\u{2014}", theme),
        crate::theme::style_muted(&verse.book, theme),
        crate::theme::style_muted(&verse.chapter.to_string(), theme),
        crate::theme::style_muted(&format!(":{}", verse.verse), theme),
    );
    println!();

    Ok(())
}

fn word_search(cfg: &Config, theme: &crate::theme::Theme, query: &str) -> Result<()> {
    let results = crate::spirit::search_verses(cfg, query, 10)?;

    if results.is_empty() {
        println!(
            "  {} No verses found for \"{}\"",
            crate::theme::style_muted("\u{2139}", theme),
            crate::theme::style_value(query, theme),
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} {} \"{}\"",
        crate::theme::style_bold_header("\u{1f50d} Search Results", theme),
        crate::theme::style_muted("for", theme),
        crate::theme::style_value(query, theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", theme)
    );

    for v in &results {
        let preview = crate::utils::truncate_str(&v.text, 72);
        println!(
            "  {} {}:{}{} {}",
            crate::theme::style_label(&v.book, theme),
            crate::theme::style_value(&v.chapter.to_string(), theme),
            crate::theme::style_value(&v.verse.to_string(), theme),
            crate::theme::style_muted(" \u{2014}", theme),
            crate::theme::style_accent(&preview, theme),
        );
    }

    println!(
        "\n  {} {} verse(s) found",
        crate::theme::style_muted("\u{2139}", theme),
        crate::theme::style_value(&results.len().to_string(), theme),
    );
    println!();

    Ok(())
}

fn word_reference(
    cfg: &Config,
    theme: &crate::theme::Theme,
    book: &str,
    chapter: Option<u32>,
    verse: Option<u32>,
) -> Result<()> {
    let results = crate::spirit::lookup_reference(cfg, book, chapter, verse)?;

    if results.is_empty() {
        let location = match (chapter, verse) {
            (Some(ch), Some(v)) => format!("{} {}:{}", book, ch, v),
            (Some(ch), None) => format!("{} {}", book, ch),
            _ => book.to_string(),
        };
        println!(
            "  {} No verses found for {}",
            crate::theme::style_muted("\u{2139}", theme),
            crate::theme::style_value(&location, theme),
        );
        return Ok(());
    }

    if results.len() == 1 {
        let v = &results[0];
        println!();
        println!(
            "  {} {} {}:{}",
            crate::theme::style_bold_header("\u{1f4d6}", theme),
            crate::theme::style_label(&v.book, theme),
            crate::theme::style_value(&v.chapter.to_string(), theme),
            crate::theme::style_value(&v.verse.to_string(), theme),
        );
        println!(
            "  {}",
            crate::theme::style_border("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", theme)
        );
        println!(
            "  {}\"{}\"{}",
            crate::theme::style_accent("", theme),
            crate::theme::style_accent(&v.text, theme),
            crate::theme::style_accent("", theme),
        );
        println!();
        return Ok(());
    }

    println!();
    println!(
        "  {} {} ({} verse(s))",
        crate::theme::style_bold_header("\u{1f4d6}", theme),
        crate::theme::style_label(book, theme),
        crate::theme::style_value(&results.len().to_string(), theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", theme)
    );

    for v in &results {
        println!(
            "  {} {}:{} {}",
            crate::theme::style_label(&v.book, theme),
            crate::theme::style_value(&format!("{}:{}", v.chapter, v.verse), theme),
            crate::theme::style_muted("\u{2014}", theme),
            crate::theme::style_accent(&v.text, theme),
        );
    }

    println!(
        "\n  {} Use {} to narrow results",
        crate::theme::style_muted("Tip:", theme),
        crate::theme::style_value(
            &format!("forge word reference {} --chapter N --verse M", book),
            theme
        ),
    );
    println!();

    Ok(())
}

fn reflect_entry(cfg: &Config, theme: &crate::theme::Theme, text: &str) -> Result<()> {
    let id = crate::reflect::add_entry(cfg, text)?;
    let word_count = text.split_whitespace().count();

    println!(
        "  {} Journal entry #{} saved ({} word(s))",
        crate::theme::style_success("\u{2713}", theme),
        crate::theme::style_value(&id.to_string(), theme),
        crate::theme::style_value(&word_count.to_string(), theme),
    );

    Ok(())
}

fn reflect_history(cfg: &Config, theme: &crate::theme::Theme) -> Result<()> {
    let entries = crate::reflect::list_entries(cfg, 20, 0)?;

    if entries.is_empty() {
        println!(
            "  {} No journal entries yet. Use {} to write one.",
            crate::theme::style_muted("\u{2139}", theme),
            crate::theme::style_value("forge reflect entry <text>", theme),
        );
        return Ok(());
    }

    println!();
    println!(
        "  {}",
        crate::theme::style_bold_header("\u{1f4dd} Prayer Journal", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", theme)
    );

    println!(
        "  {}   {}         {}",
        crate::theme::style_header("ID", theme),
        crate::theme::style_header("Date", theme),
        crate::theme::style_header("Words", theme),
    );

    for entry in &entries {
        let date_str = entry.created_at.format("%Y-%m-%d %H:%M").to_string();
        let tag_display = entry.tag.as_deref().unwrap_or("");
        let tag_colored = if tag_display.is_empty() {
            String::new()
        } else {
            format!(
                " {}",
                crate::theme::style_info(&format!("[{}]", tag_display), theme)
            )
        };

        println!(
            "  {}   {}  {} {}{}",
            crate::theme::style_value(&format!("{:>4}", entry.id), theme),
            crate::theme::style_muted(&date_str, theme),
            crate::theme::style_value(&format!("{:>3}", entry.word_count), theme),
            crate::theme::style_muted("w", theme),
            tag_colored,
        );
    }

    println!();
    println!(
        "  {} {}  (use {} to read one)",
        crate::theme::style_muted("\u{2139}", theme),
        crate::theme::style_value(&entries.len().to_string(), theme),
        crate::theme::style_value("forge reflect read <id>", theme),
    );
    println!();

    Ok(())
}

fn reflect_read(cfg: &Config, theme: &crate::theme::Theme, id: i64) -> Result<()> {
    let entry = crate::reflect::get_entry(cfg, id)?
        .ok_or_else(|| anyhow::anyhow!("Journal entry #{} not found", id))?;

    println!();
    println!(
        "  {} #{}",
        crate::theme::style_bold_header("\u{1f4dd} Entry", theme),
        crate::theme::style_value(&id.to_string(), theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", theme)
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Date:", theme),
        crate::theme::style_muted(
            &entry.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
            theme
        ),
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Words:", theme),
        crate::theme::style_value(&entry.word_count.to_string(), theme),
    );

    if let Some(tag) = &entry.tag {
        println!(
            "  {} {}",
            crate::theme::style_label("Tag:", theme),
            crate::theme::style_info(tag, theme),
        );
    }

    println!();
    for line in entry.content.lines() {
        println!("    {}", crate::theme::style_accent(line, theme),);
    }
    println!();

    Ok(())
}

fn reflect_search(cfg: &Config, theme: &crate::theme::Theme, query: &str) -> Result<()> {
    let results = crate::reflect::search_entries(cfg, query)?;

    if results.is_empty() {
        println!(
            "  {} No entries matching \"{}\"",
            crate::theme::style_muted("\u{2139}", theme),
            crate::theme::style_value(query, theme),
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} {} \"{}\"",
        crate::theme::style_bold_header("\u{1f50d} Journal Search", theme),
        crate::theme::style_muted("for", theme),
        crate::theme::style_value(query, theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", theme)
    );

    for entry in &results {
        let date_str = entry.created_at.format("%Y-%m-%d").to_string();
        let preview = crate::utils::truncate_str(&entry.content, 68);

        println!(
            "  {} {} {} {}",
            crate::theme::style_value(&format!("#{:<4}", entry.id), theme),
            crate::theme::style_muted(&date_str, theme),
            crate::theme::style_muted("\u{2014}", theme),
            crate::theme::style_accent(&preview, theme),
        );
    }

    println!(
        "\n  {} {} result(s) found. Use {} to read full entries.",
        crate::theme::style_muted("\u{2139}", theme),
        crate::theme::style_value(&results.len().to_string(), theme),
        crate::theme::style_value("forge reflect read <id>", theme),
    );
    println!();

    Ok(())
}

fn sabbath_mode(theme: &crate::theme::Theme) -> Result<()> {
    let rest_verses = [
        (
            "Matthew 11:28",
            "Come to me, all who labor and are heavy laden, and I will give you rest.",
        ),
        ("Psalm 46:10", "Be still, and know that I am God."),
        ("Exodus 20:8", "Remember the Sabbath day, to keep it holy."),
        (
            "Isaiah 40:31",
            "But they who wait for the Lord shall renew their strength.",
        ),
    ];

    let idx = (chrono::Utc::now().timestamp() as usize) % rest_verses.len();
    let (reference, text) = &rest_verses[idx];

    println!();
    println!(
        "  {}",
        crate::theme::style_bold_header("\u{1f4d6} Sabbath Rest", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", theme)
    );
    println!(
        "  {}\"{}\"{}",
        crate::theme::style_accent("", theme),
        crate::theme::style_accent(text, theme),
        crate::theme::style_accent("", theme),
    );
    println!(
        "  {} {}",
        crate::theme::style_muted("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500} \u{2014}", theme),
        crate::theme::style_muted(reference, theme),
    );
    println!();
    println!(
        "  {} Sabbath mode activated. All forge processes stopped. Rest well.",
        crate::theme::style_success("\u{1f525}", theme),
    );
    println!();

    Ok(())
}
