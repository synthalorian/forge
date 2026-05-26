use std::collections::HashMap;

use anyhow::{Context, Result};
use rusqlite::Connection;

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
        Some(WordAction::Plan {
            name,
            today,
            activate,
            list,
            progress,
        }) => word_plan(cfg, theme, name.as_deref(), *today, activate.as_deref(), *list, *progress),
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
        crate::theme::style_success("🔥", theme),
    );
    println!();

    Ok(())
}

// ── Reading Plans ─────────────────────────────────────────────────────

struct ReadingPlan {
    name: &'static str,
    description: &'static str,
    total_days: u32,
    readings: &'static [(u32, &'static str, u32, u32)],
}

static PLANS: &[ReadingPlan] = &[
    ReadingPlan {
        name: "psalms-30",
        description: "30 days through the Psalms",
        total_days: 30,
        readings: &[
            (1,"Psalms",1,5),(2,"Psalms",6,10),(3,"Psalms",11,15),(4,"Psalms",16,20),
            (5,"Psalms",21,25),(6,"Psalms",26,30),(7,"Psalms",31,35),(8,"Psalms",36,40),
            (9,"Psalms",41,45),(10,"Psalms",46,50),(11,"Psalms",51,55),(12,"Psalms",56,60),
            (13,"Psalms",61,65),(14,"Psalms",66,70),(15,"Psalms",71,75),(16,"Psalms",76,80),
            (17,"Psalms",81,85),(18,"Psalms",86,90),(19,"Psalms",91,95),(20,"Psalms",96,100),
            (21,"Psalms",101,105),(22,"Psalms",106,110),(23,"Psalms",111,115),(24,"Psalms",116,120),
            (25,"Psalms",121,125),(26,"Psalms",126,130),(27,"Psalms",131,135),(28,"Psalms",136,140),
            (29,"Psalms",141,145),(30,"Psalms",146,150),
        ],
    },
    ReadingPlan {
        name: "proverbs-month",
        description: "31 days of Proverbs",
        total_days: 31,
        readings: &[
            (1,"Proverbs",1,1),(2,"Proverbs",2,2),(3,"Proverbs",3,3),(4,"Proverbs",4,4),
            (5,"Proverbs",5,5),(6,"Proverbs",6,6),(7,"Proverbs",7,7),(8,"Proverbs",8,8),
            (9,"Proverbs",9,9),(10,"Proverbs",10,10),(11,"Proverbs",11,11),(12,"Proverbs",12,12),
            (13,"Proverbs",13,13),(14,"Proverbs",14,14),(15,"Proverbs",15,15),(16,"Proverbs",16,16),
            (17,"Proverbs",17,17),(18,"Proverbs",18,18),(19,"Proverbs",19,19),(20,"Proverbs",20,20),
            (21,"Proverbs",21,21),(22,"Proverbs",22,22),(23,"Proverbs",23,23),(24,"Proverbs",24,24),
            (25,"Proverbs",25,25),(26,"Proverbs",26,26),(27,"Proverbs",27,27),(28,"Proverbs",28,28),
            (29,"Proverbs",29,29),(30,"Proverbs",30,30),(31,"Proverbs",31,31),
        ],
    },
    ReadingPlan {
        name: "gospels-40",
        description: "40 days through Matthew, Mark, Luke & John",
        total_days: 40,
        readings: &[
            (1,"Matthew",1,4),(2,"Matthew",5,7),(3,"Matthew",8,11),(4,"Matthew",12,15),
            (5,"Matthew",16,19),(6,"Matthew",20,23),(7,"Matthew",24,26),(8,"Matthew",27,28),
            (9,"Mark",1,4),(10,"Mark",5,8),(11,"Mark",9,12),(12,"Mark",13,16),
            (13,"Luke",1,3),(14,"Luke",4,7),(15,"Luke",8,10),(16,"Luke",11,14),
            (17,"Luke",15,18),(18,"Luke",19,21),(19,"Luke",22,24),(20,"John",1,3),
            (21,"John",4,6),(22,"John",7,9),(23,"John",10,12),(24,"John",13,15),
            (25,"John",16,18),(26,"John",19,21),
        ],
    },
    ReadingPlan {
        name: "new-testament-90",
        description: "90 days through the entire New Testament",
        total_days: 90,
        readings: &[
            (1,"Matthew",1,3),(2,"Matthew",4,6),(3,"Matthew",7,9),(4,"Matthew",10,12),
            (5,"Matthew",13,15),(6,"Matthew",16,18),(7,"Matthew",19,21),(8,"Matthew",22,24),
            (9,"Matthew",25,26),(10,"Matthew",27,28),(11,"Mark",1,3),(12,"Mark",4,6),
            (13,"Mark",7,9),(14,"Mark",10,12),(15,"Mark",13,16),(16,"Luke",1,3),
            (17,"Luke",4,6),(18,"Luke",7,9),(19,"Luke",10,12),(20,"Luke",13,15),
            (21,"Luke",16,18),(22,"Luke",19,21),(23,"Luke",22,24),(24,"John",1,3),
            (25,"John",4,6),(26,"John",7,9),(27,"John",10,12),(28,"John",13,15),
            (29,"John",16,18),(30,"John",19,21),(31,"Acts",1,4),(32,"Acts",5,8),
            (33,"Acts",9,12),(34,"Acts",13,16),(35,"Acts",17,20),(36,"Acts",21,24),
            (37,"Acts",25,28),(38,"Romans",1,4),(39,"Romans",5,8),(40,"Romans",9,12),
            (41,"Romans",13,16),(42,"1 Corinthians",1,4),(43,"1 Corinthians",5,8),(44,"1 Corinthians",9,12),
            (45,"1 Corinthians",13,16),(46,"2 Corinthians",1,4),(47,"2 Corinthians",5,9),(48,"2 Corinthians",10,13),
            (49,"Galatians",1,3),(50,"Galatians",4,6),(51,"Ephesians",1,3),(52,"Ephesians",4,6),
            (53,"Philippians",1,4),(54,"Colossians",1,4),(55,"1 Thessalonians",1,5),(56,"2 Thessalonians",1,3),
            (57,"1 Timothy",1,3),(58,"1 Timothy",4,6),(59,"2 Timothy",1,4),(60,"Titus,Philemon",1,1),
            (61,"Hebrews",1,4),(62,"Hebrews",5,8),(63,"Hebrews",9,12),(64,"Hebrews",13,13),
            (65,"James",1,3),(66,"James",4,5),(67,"1 Peter",1,3),(68,"1 Peter",4,5),
            (69,"2 Peter",1,3),(70,"1 John",1,3),(71,"1 John",4,5),(72,"2,3 John,Jude",1,1),
            (73,"Revelation",1,3),(74,"Revelation",4,7),(75,"Revelation",8,11),(76,"Revelation",12,15),
            (77,"Revelation",16,19),(78,"Revelation",20,22),
        ],
    },
];

fn plan_connect_db(cfg: &Config) -> Result<Connection> {
    let db_path = crate::reflect::spirit_db_path(cfg);
    let conn = Connection::open(&db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS reading_plans (
            name TEXT PRIMARY KEY,
            active INTEGER NOT NULL DEFAULT 0,
            current_day INTEGER NOT NULL DEFAULT 1,
            started_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;
    Ok(conn)
}

fn word_plan(
    cfg: &Config,
    theme: &crate::theme::Theme,
    name: Option<&str>,
    today: bool,
    activate: Option<&str>,
    list: bool,
    progress: bool,
) -> Result<()> {
    // --list: show all available plans (default if no flags)
    if list || (name.is_none() && !today && activate.is_none() && !progress) {
        let conn = plan_connect_db(cfg)?;
        let active_plan: Option<String> = conn
            .query_row("SELECT name FROM reading_plans WHERE active = 1", [], |row| row.get(0))
            .ok();
        println!();
        println!("{}", crate::theme::style_bold_header("Reading Plans", theme));
        println!("{}", crate::theme::style_border(&"=".repeat(50), theme));
        for plan in PLANS {
            let is_active = active_plan.as_deref() == Some(plan.name);
            let marker = if is_active { "●" } else { "○" };
            println!("  {} {} — {} days",
                if is_active { crate::theme::style_success(marker, theme) } else { crate::theme::style_muted(marker, theme) },
                crate::theme::style_value(plan.name, theme),
                crate::theme::style_muted(&plan.total_days.to_string(), theme),
            );
            println!("    {}", crate::theme::style_muted(plan.description, theme));
            if is_active {
                let day: u32 = conn.query_row(
                    "SELECT current_day FROM reading_plans WHERE active = 1", [], |row| row.get(0)
                ).unwrap_or(1);
                println!("    {} Day {} of {}", crate::theme::style_success("→", theme),
                    crate::theme::style_value(&day.to_string(), theme),
                    crate::theme::style_value(&plan.total_days.to_string(), theme));
            }
        }
        println!();
        println!("  {} {}", crate::theme::style_muted("Tip:", theme),
            crate::theme::style_value("forge word plan --activate <name>", theme));
        return Ok(());
    }

    // --activate
    if let Some(plan_name) = activate {
        let plan = PLANS.iter().find(|p| p.name == plan_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown plan '{}'", plan_name))?;
        let conn = plan_connect_db(cfg)?;
        conn.execute("UPDATE reading_plans SET active = 0 WHERE active = 1", [])?;
        conn.execute(
            "INSERT OR REPLACE INTO reading_plans (name, active, current_day, started_at) VALUES (?1, 1, 1, datetime('now'))",
            rusqlite::params![plan_name],
        )?;
        println!();
        println!("  {} {} activated! Day 1 of {}",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(plan_name, theme),
            crate::theme::style_value(&plan.total_days.to_string(), theme),
        );
        println!("  {} {} {}",
            crate::theme::style_muted("→", theme),
            crate::theme::style_value(plan.readings[0].1, theme),
            crate::theme::style_value(&plan.readings[0].2.to_string(), theme),
        );
        println!();
        return Ok(());
    }

    // --progress
    if progress {
        let conn = plan_connect_db(cfg)?;
        let (pname, cday): (String, u32) = conn.query_row(
            "SELECT name, current_day FROM reading_plans WHERE active = 1", [],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).map_err(|_| anyhow::anyhow!("No active plan"))?;
        let plan = PLANS.iter().find(|p| p.name == pname).unwrap();
        let pct = (cday as f64 / plan.total_days as f64 * 100.0) as u32;
        let bw = 30usize;
        let f = (pct as f64 / 100.0 * bw as f64) as usize;
        let bar = format!("{}{}", "━".repeat(f), "─".repeat(bw.saturating_sub(f)));
        println!();
        println!("  {} {} — Day {} of {}",
            crate::theme::style_label("Plan:", theme), crate::theme::style_value(&pname, theme),
            crate::theme::style_value(&cday.to_string(), theme),
            crate::theme::style_value(&plan.total_days.to_string(), theme));
        println!("  {} [{}] {}%", crate::theme::style_label("Progress:", theme),
            crate::theme::style_value(&bar, theme), crate::theme::style_value(&pct.to_string(), theme));
        println!();
        return Ok(());
    }

    // Show today's reading
    let plan_name = match name {
        Some(n) => n.to_string(),
        None => {
            let conn = plan_connect_db(cfg)?;
            conn.query_row("SELECT name FROM reading_plans WHERE active = 1", [], |row| row.get(0))
                .map_err(|_| anyhow::anyhow!("No active plan"))?
        }
    };
    let plan = PLANS.iter().find(|p| p.name == plan_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown plan '{}'", plan_name))?;
    let conn = plan_connect_db(cfg)?;
    let current_day: u32 = conn.query_row(
        "SELECT current_day FROM reading_plans WHERE name = ?1",
        rusqlite::params![plan_name], |row| row.get(0)
    ).unwrap_or(1);

    if let Some(reading) = plan.readings.iter().find(|r| r.0 == current_day) {
        let (_, book, cs, ce) = reading;
        println!();
        println!("  {} {} — Day {} of {}",
            crate::theme::style_bold_header(&format!("{}", plan.name), theme),
            crate::theme::style_muted(&format!("Day {} of {}", current_day, plan.total_days), theme),
            crate::theme::style_value(&current_day.to_string(), theme),
            crate::theme::style_value(&plan.total_days.to_string(), theme),
        );
        println!("{}", crate::theme::style_border(&"=".repeat(50), theme));
        for chap in *cs..=*ce {
            match crate::spirit::lookup_reference(cfg, book, Some(chap), None) {
                Ok(verses) => {
                    if let Some(first) = verses.first() {
                        println!("  {} {}: {}",
                            crate::theme::style_accent(book, theme),
                            crate::theme::style_value(&chap.to_string(), theme),
                            crate::theme::style_muted(&first.text, theme));
                    }
                }
                Err(_) => {
                    println!("  {} {}: ({})",
                        crate::theme::style_accent(book, theme),
                        crate::theme::style_value(&chap.to_string(), theme),
                        crate::theme::style_muted("verses not found", theme));
                }
            }
        }
        let next_day = (current_day + 1).min(plan.total_days);
        conn.execute("UPDATE reading_plans SET current_day = ?1 WHERE name = ?2",
            rusqlite::params![next_day, plan_name])?;
        println!();
        if current_day < plan.total_days {
            println!("  {} Day {} tomorrow", crate::theme::style_muted("→ Next:", theme),
                crate::theme::style_value(&next_day.to_string(), theme));
        } else {
            println!("  {} {} complete!", crate::theme::style_success("✓", theme),
                crate::theme::style_value(plan.name, theme));
        }
    }
    Ok(())
}
