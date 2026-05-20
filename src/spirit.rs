use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verse {
    pub book: String,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

const BIBLE_DB_FILENAME: &str = "bible.db";

fn bible_db_resource_path() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("src/spirit")
        .join(BIBLE_DB_FILENAME)
}

fn bible_db_data_path(cfg: &Config) -> PathBuf {
    cfg.db_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/tmp"))
        .join("data")
        .join(BIBLE_DB_FILENAME)
}

pub fn ensure_bible_db(cfg: &Config) -> Result<PathBuf> {
    let data_path = bible_db_data_path(cfg);

    if data_path.exists() {
        return Ok(data_path);
    }

    if let Some(parent) = data_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create Bible data directory {}", parent.display())
        })?;
    }

    let resource = bible_db_resource_path();
    if !resource.exists() {
        anyhow::bail!(
            "Bundled Bible database not found at {}. Run the generator first.",
            resource.display()
        );
    }

    std::fs::copy(&resource, &data_path).with_context(|| {
        format!(
            "Failed to copy Bible database from {} to {}",
            resource.display(),
            data_path.display()
        )
    })?;

    Ok(data_path)
}

fn open_bible_db(path: &std::path::Path) -> Result<Connection> {
    Connection::open(path)
        .with_context(|| format!("Failed to open Bible database at {}", path.display()))
}

fn row_to_verse(row: &rusqlite::Row<'_>) -> rusqlite::Result<Verse> {
    Ok(Verse {
        book: row.get(0)?,
        chapter: row.get::<_, i64>(1)? as u32,
        verse: row.get::<_, i64>(2)? as u32,
        text: row.get(3)?,
    })
}

pub fn daily_verse(cfg: &Config) -> Result<Verse> {
    let db_path = ensure_bible_db(cfg)?;
    let conn = open_bible_db(&db_path)?;

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM verses", [], |row| row.get(0))
        .context("Failed to count verses")?;

    let seed = chrono::Utc::now().format("%Y%m%d").to_string();
    let seed_num: i64 = seed.parse::<i64>().unwrap_or(1);
    let target_id = ((seed_num.abs()) % total) + 1;

    conn.query_row(
        "SELECT b.name, v.chapter, v.verse, v.text
         FROM verses v JOIN books b ON v.book_id = b.id
         WHERE v.id = ?1",
        [target_id],
        row_to_verse,
    )
    .context("Failed to fetch daily verse")
}

pub fn search_verses(cfg: &Config, query: &str, limit: usize) -> Result<Vec<Verse>> {
    let db_path = ensure_bible_db(cfg)?;
    let conn = open_bible_db(&db_path)?;

    let pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT b.name, v.chapter, v.verse, v.text
         FROM verses v JOIN books b ON v.book_id = b.id
         WHERE v.text LIKE ?1
         ORDER BY v.book_id, v.chapter, v.verse
         LIMIT ?2",
        )
        .context("Failed to prepare search query")?;

    let results: Vec<Verse> = stmt
        .query_map(
            [&pattern as &dyn rusqlite::types::ToSql, &(limit as i64)],
            row_to_verse,
        )
        .context("Failed to execute search query")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse search results")?;

    Ok(results)
}

fn resolve_book_id(conn: &Connection, name: &str) -> Result<Option<i64>> {
    let lower = name.to_lowercase();

    let result: Option<i64> = conn
        .query_row(
            "SELECT id FROM books WHERE LOWER(name) = ?1",
            [lower.clone()],
            |row| row.get(0),
        )
        .ok();

    if result.is_some() {
        return Ok(result);
    }

    let mut stmt = conn.prepare("SELECT id, name FROM books ORDER BY id")?;

    let books: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    for (id, book_name) in &books {
        let bn_lower = book_name.to_lowercase();
        if bn_lower.starts_with(&lower) {
            return Ok(Some(*id));
        }
    }

    let abbreviations: &[(&str, &str)] = &[
        ("gen", "Genesis"),
        ("ex", "Exodus"),
        ("lev", "Leviticus"),
        ("num", "Numbers"),
        ("deut", "Deuteronomy"),
        ("josh", "Joshua"),
        ("judg", "Judges"),
        ("rut", "Ruth"),
        ("1sam", "1 Samuel"),
        ("2sam", "2 Samuel"),
        ("1ki", "1 Kings"),
        ("2ki", "2 Kings"),
        ("1ch", "1 Chronicles"),
        ("2ch", "2 Chronicles"),
        ("ezr", "Ezra"),
        ("neh", "Nehemiah"),
        ("est", "Esther"),
        ("ps", "Psalms"),
        ("prov", "Proverbs"),
        ("eccl", "Ecclesiastes"),
        ("song", "Song of Solomon"),
        ("isa", "Isaiah"),
        ("jer", "Jeremiah"),
        ("lam", "Lamentations"),
        ("ezek", "Ezekiel"),
        ("dan", "Daniel"),
        ("hos", "Hosea"),
        ("joel", "Joel"),
        ("amos", "Amos"),
        ("obad", "Obadiah"),
        ("jon", "Jonah"),
        ("mic", "Micah"),
        ("nah", "Nahum"),
        ("hab", "Habakkuk"),
        ("zep", "Zephaniah"),
        ("hag", "Haggai"),
        ("zech", "Zechariah"),
        ("mal", "Malachi"),
        ("matt", "Matthew"),
        ("mk", "Mark"),
        ("lk", "Luke"),
        ("jn", "John"),
        ("acts", "Acts"),
        ("rom", "Romans"),
        ("1cor", "1 Corinthians"),
        ("2cor", "2 Corinthians"),
        ("gal", "Galatians"),
        ("eph", "Ephesians"),
        ("phil", "Philippians"),
        ("col", "Colossians"),
        ("1th", "1 Thessalonians"),
        ("2th", "2 Thessalonians"),
        ("1tim", "1 Timothy"),
        ("2tim", "2 Timothy"),
        ("titus", "Titus"),
        ("phm", "Philemon"),
        ("heb", "Hebrews"),
        ("jas", "James"),
        ("1pet", "1 Peter"),
        ("2pet", "2 Peter"),
        ("1jn", "1 John"),
        ("2jn", "2 John"),
        ("3jn", "3 John"),
        ("jude", "Jude"),
        ("rev", "Revelation"),
    ];

    for (abbr, full_name) in abbreviations {
        if lower == *abbr {
            let id: i64 = conn
                .query_row(
                    "SELECT id FROM books WHERE name = ?1",
                    [full_name.to_string()],
                    |row| row.get(0),
                )
                .context(format!("Book '{}' not found", full_name))?;
            return Ok(Some(id));
        }
    }

    Ok(None)
}

pub fn lookup_reference(
    cfg: &Config,
    book: &str,
    chapter: Option<u32>,
    verse: Option<u32>,
) -> Result<Vec<Verse>> {
    let db_path = ensure_bible_db(cfg)?;
    let conn = open_bible_db(&db_path)?;

    let book_id = resolve_book_id(&conn, book)?
        .ok_or_else(|| anyhow::anyhow!("Book '{}' not found", book))?;

    let results = match (chapter, verse) {
        (Some(ch), Some(v)) => {
            let mut stmt = conn.prepare(
                "SELECT b.name, v.chapter, v.verse, v.text
                 FROM verses v JOIN books b ON v.book_id = b.id
                 WHERE v.book_id = ?1 AND v.chapter = ?2 AND v.verse = ?3",
            )?;
            let rows: Vec<Verse> = stmt
                .query_map([book_id, ch as i64, v as i64], row_to_verse)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .context("Failed to parse verse lookup results")?;
            rows
        }
        (Some(ch), None) => {
            let mut stmt = conn.prepare(
                "SELECT b.name, v.chapter, v.verse, v.text
                 FROM verses v JOIN books b ON v.book_id = b.id
                 WHERE v.book_id = ?1 AND v.chapter = ?2
                 ORDER BY v.verse",
            )?;
            let rows: Vec<Verse> = stmt
                .query_map([book_id, ch as i64], row_to_verse)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .context("Failed to parse chapter lookup results")?;
            rows
        }
        (None, _) => {
            let mut stmt = conn.prepare(
                "SELECT b.name, v.chapter, v.verse, v.text
                 FROM verses v JOIN books b ON v.book_id = b.id
                 WHERE v.book_id = ?1
                 ORDER BY v.chapter, v.verse",
            )?;
            let rows: Vec<Verse> = stmt
                .query_map([book_id], row_to_verse)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .context("Failed to parse book lookup results")?;
            rows
        }
    };

    Ok(results)
}

pub fn parse_reference(input: &str) -> Option<(String, Option<u32>, Option<u32>)> {
    let trimmed = input.trim();

    let re_with_verse =
        regex::Regex::new(r"(?i)^\s*([\d]?\s*\w+(?:\s+\w+)?)\s+(\d+)\s*:\s*(\d+)\s*$").ok()?;

    if let Some(caps) = re_with_verse.captures(trimmed) {
        let book = caps.get(1)?.as_str().trim().to_string();
        let chapter = caps.get(2)?.as_str().parse().ok();
        let verse = caps.get(3)?.as_str().parse().ok();
        return Some((book, chapter, verse));
    }

    let re_with_chapter =
        regex::Regex::new(r"(?i)^\s*([\d]?\s*\w+(?:\s+\w+)?)\s+(\d+)\s*$").ok()?;

    if let Some(caps) = re_with_chapter.captures(trimmed) {
        let book = caps.get(1)?.as_str().trim().to_string();
        let chapter = caps.get(2)?.as_str().parse().ok();
        return Some((book, chapter, None));
    }

    let re_book_only = regex::Regex::new(r"(?i)^\s*([\d]?\s*\w+(?:\s+\w+)?)\s*$").ok()?;

    if let Some(caps) = re_book_only.captures(trimmed) {
        let book = caps.get(1)?.as_str().trim().to_string();
        return Some((book, None, None));
    }

    None
}

pub fn format_verse(verse: &Verse, cfg: &Config) -> String {
    let theme = crate::theme::load_from_config(cfg);
    format!(
        "{} {}:{}{} {}",
        crate::theme::style_accent(&verse.book, theme),
        crate::theme::style_value(&verse.chapter.to_string(), theme),
        crate::theme::style_value(&verse.verse.to_string(), theme),
        crate::theme::style_muted(" —", theme),
        crate::theme::style_value(&verse.text, theme),
    )
}

pub fn format_verse_brief(verse: &Verse, cfg: &Config) -> String {
    let theme = crate::theme::load_from_config(cfg);
    format!(
        "{} {}:{}{}",
        crate::theme::style_accent(&verse.book, theme),
        crate::theme::style_value(&verse.chapter.to_string(), theme),
        crate::theme::style_value(&format!(":{}", verse.verse), theme),
        crate::theme::style_muted(
            &format!(" {}", &verse.text.chars().take(60).collect::<String>()),
            theme
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, RetentionConfig};
    use tempfile::TempDir;

    fn test_config_with_bible_db(tmp: &TempDir) -> Config {
        let data_dir = tmp.path().join("forge");
        std::fs::create_dir_all(data_dir.join("data")).unwrap();

        let resource = bible_db_resource_path();
        if resource.exists() {
            std::fs::copy(&resource, data_dir.join("data/bible.db")).unwrap();
        }

        Config {
            archive_dir: tmp.path().join("archives"),
            db_path: data_dir.join("forge.db"),
            default_compression: 3,
            repo_paths: vec![],
            retention: RetentionConfig {
                keep_daily: 7,
                keep_weekly: 4,
                keep_monthly: 12,
            },
            theme: "synthwave84".to_string(),
        }
    }

    fn bible_db_exists() -> bool {
        bible_db_resource_path().exists()
    }

    #[test]
    fn test_daily_verse_returns_verse() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let verse = daily_verse(&cfg)?;
        assert!(!verse.book.is_empty());
        assert!(verse.chapter > 0);
        assert!(verse.verse > 0);
        assert!(!verse.text.is_empty());

        Ok(())
    }

    #[test]
    fn test_daily_verse_deterministic() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let v1 = daily_verse(&cfg)?;
        let v2 = daily_verse(&cfg)?;

        assert_eq!(v1.book, v2.book);
        assert_eq!(v1.chapter, v2.chapter);
        assert_eq!(v1.verse, v2.verse);
        assert_eq!(v1.text, v2.text);

        Ok(())
    }

    #[test]
    fn test_search_finds_verse() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let results = search_verses(&cfg, "For God so loved", 10)?;
        assert!(
            !results.is_empty(),
            "Expected to find at least one result for 'For God so loved'"
        );

        let found = results
            .iter()
            .any(|v| v.book == "John" && v.chapter == 3 && v.verse == 16);
        assert!(found, "Expected to find John 3:16 in search results");

        Ok(())
    }

    #[test]
    fn test_search_respects_limit() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let results = search_verses(&cfg, "the", 5)?;
        assert!(results.len() <= 5);

        Ok(())
    }

    #[test]
    fn test_lookup_by_book_chapter_verse() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let results = lookup_reference(&cfg, "John", Some(3), Some(16))?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].book, "John");
        assert_eq!(results[0].chapter, 3);
        assert_eq!(results[0].verse, 16);
        assert!(results[0].text.contains("God so loved"));

        Ok(())
    }

    #[test]
    fn test_lookup_by_book_chapter() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let results = lookup_reference(&cfg, "John", Some(3), None)?;
        assert!(!results.is_empty());
        assert!(results.len() >= 16, "John 3 has at least 16 verses");
        assert!(results.iter().all(|v| v.book == "John" && v.chapter == 3));

        Ok(())
    }

    #[test]
    fn test_lookup_by_book_only() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let results = lookup_reference(&cfg, "Genesis", None, None)?;
        assert!(!results.is_empty());
        assert!(results.len() > 1000, "Genesis has 1533 verses");
        assert!(results.iter().all(|v| v.book == "Genesis"));

        Ok(())
    }

    #[test]
    fn test_case_insensitive_book_name() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let r1 = lookup_reference(&cfg, "genesis", Some(1), Some(1))?;
        let r2 = lookup_reference(&cfg, "GENESIS", Some(1), Some(1))?;
        let r3 = lookup_reference(&cfg, "Genesis", Some(1), Some(1))?;

        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 1);
        assert_eq!(r3.len(), 1);
        assert_eq!(r1[0].text, r2[0].text);
        assert_eq!(r2[0].text, r3[0].text);

        Ok(())
    }

    #[test]
    fn test_abbreviation_lookup() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let results = lookup_reference(&cfg, "gen", Some(1), Some(1))?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].book, "Genesis");

        let results = lookup_reference(&cfg, "jn", Some(3), Some(16))?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].book, "John");

        let results = lookup_reference(&cfg, "rev", Some(1), Some(1))?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].book, "Revelation");

        Ok(())
    }

    #[test]
    fn test_partial_book_name() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let results = lookup_reference(&cfg, "Gen", Some(1), Some(1))?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].book, "Genesis");

        Ok(())
    }

    #[test]
    fn test_nonexistent_book_returns_error() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let result = lookup_reference(&cfg, "NonexistentBook", Some(1), Some(1));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_parse_reference_full() {
        let result = parse_reference("John 3:16");
        assert!(result.is_some());
        let (book, chapter, verse) = result.unwrap();
        assert_eq!(book, "John");
        assert_eq!(chapter, Some(3));
        assert_eq!(verse, Some(16));
    }

    #[test]
    fn test_parse_reference_book_chapter() {
        let result = parse_reference("Genesis 1");
        assert!(result.is_some());
        let (book, chapter, verse) = result.unwrap();
        assert_eq!(book, "Genesis");
        assert_eq!(chapter, Some(1));
        assert_eq!(verse, None);
    }

    #[test]
    fn test_parse_reference_book_only() {
        let result = parse_reference("Psalms");
        assert!(result.is_some());
        let (book, chapter, verse) = result.unwrap();
        assert_eq!(book, "Psalms");
        assert_eq!(chapter, None);
        assert_eq!(verse, None);
    }

    #[test]
    fn test_parse_reference_case_insensitive() {
        let result = parse_reference("proverbs 27:17");
        assert!(result.is_some());
        let (book, chapter, verse) = result.unwrap();
        assert_eq!(book, "proverbs");
        assert_eq!(chapter, Some(27));
        assert_eq!(verse, Some(17));
    }

    #[test]
    fn test_parse_reference_with_leading_number() {
        let result = parse_reference("1 Samuel 15:23");
        assert!(result.is_some());
        let (book, chapter, verse) = result.unwrap();
        assert_eq!(book, "1 Samuel");
        assert_eq!(chapter, Some(15));
        assert_eq!(verse, Some(23));
    }

    #[test]
    fn test_ensure_bible_db_copies_if_missing() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db resource not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let data_dir = tmp.path().join("forge");
        let cfg = Config {
            archive_dir: tmp.path().join("archives"),
            db_path: data_dir.join("forge.db"),
            default_compression: 3,
            repo_paths: vec![],
            retention: RetentionConfig {
                keep_daily: 7,
                keep_weekly: 4,
                keep_monthly: 12,
            },
            theme: "synthwave84".to_string(),
        };

        let path = ensure_bible_db(&cfg)?;
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("bible.db"));

        let path2 = ensure_bible_db(&cfg)?;
        assert_eq!(path, path2);

        Ok(())
    }

    #[test]
    fn test_format_verse_produces_output() {
        let cfg = Config::default();
        let verse = Verse {
            book: "John".to_string(),
            chapter: 3,
            verse: 16,
            text: "For God so loved the world...".to_string(),
        };

        let output = format_verse(&verse, &cfg);
        assert!(output.contains("John"));
        assert!(output.contains("For God so loved"));
    }

    #[test]
    fn test_search_empty_results() -> Result<()> {
        if !bible_db_exists() {
            eprintln!("Skipping: bible.db not available");
            return Ok(());
        }

        let tmp = TempDir::new()?;
        let cfg = test_config_with_bible_db(&tmp);

        let results = search_verses(&cfg, "xyznonexistent12345", 10)?;
        assert!(results.is_empty());

        Ok(())
    }
}
