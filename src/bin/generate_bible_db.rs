//! Standalone binary to generate the KJV Bible SQLite database.
//!
//! Run: `cargo run --bin generate_bible_db`
//! Reads `/tmp/kjv.json` (3D array [book][chapter][verse])
//! and `/tmp/books.json` (book names array).
//! Outputs to `src/spirit/bible.db` relative to project root.

use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::path::Path;

const KJV_BOOKS: [&str; 66] = [
    "Genesis", "Exodus", "Leviticus", "Numbers", "Deuteronomy",
    "Joshua", "Judges", "Ruth",
    "1 Samuel", "2 Samuel", "1 Kings", "2 Kings",
    "1 Chronicles", "2 Chronicles", "Ezra", "Nehemiah", "Esther",
    "Job", "Psalms", "Proverbs", "Ecclesiastes", "Song of Solomon",
    "Isaiah", "Jeremiah", "Lamentations", "Ezekiel", "Daniel",
    "Hosea", "Joel", "Amos", "Obadiah", "Jonah", "Micah", "Nahum",
    "Habakkuk", "Zephaniah", "Haggai", "Zechariah", "Malachi",
    "Matthew", "Mark", "Luke", "John", "Acts",
    "Romans", "1 Corinthians", "2 Corinthians", "Galatians", "Ephesians",
    "Philippians", "Colossians", "1 Thessalonians", "2 Thessalonians",
    "1 Timothy", "2 Timothy", "Titus", "Philemon", "Hebrews",
    "James", "1 Peter", "2 Peter", "1 John", "2 John", "3 John",
    "Jude", "Revelation",
];

fn main() -> anyhow::Result<()> {
    // Locate project root (Cargo.toml dir)
    let project_root = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let db_path = Path::new(&project_root).join("src/spirit/bible.db");

    // Also try reading from /tmp if kjv.json exists there
    let kjv_path = Path::new("/tmp/kjv.json");
    if !kjv_path.exists() {
        anyhow::bail!("KJV JSON not found at /tmp/kjv.json. Download it first.");
    }

    let kjv_data = fs::read_to_string(kjv_path)?;
    let bible: Value = serde_json::from_str(&kjv_data)?;

    let books_arr = bible.as_array()
        .ok_or_else(|| anyhow::anyhow!("Expected JSON array"))?;

    if books_arr.len() != 66 {
        anyhow::bail!("Expected 66 books, got {}", books_arr.len());
    }

    // Create output directory
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Remove existing DB
    if db_path.exists() {
        fs::remove_file(&db_path)?;
    }

    let conn = Connection::open(&db_path)?;

    // Create schema
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS books (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            testament   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS verses (
            id          INTEGER PRIMARY KEY,
            book_id     INTEGER NOT NULL,
            chapter     INTEGER NOT NULL,
            verse       INTEGER NOT NULL,
            text        TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_verses_book_chapter ON verses(book_id, chapter);
        CREATE INDEX IF NOT EXISTS idx_verses_text ON verses(text);
    ")?;

    // Insert books
    {
        let tx = conn.unchecked_transaction()?;
        for (i, name) in KJV_BOOKS.iter().enumerate() {
            let book_id = (i + 1) as i64;
            let testament = if i < 39 { "OT" } else { "NT" };
            tx.execute(
                "INSERT INTO books (id, name, testament) VALUES (?1, ?2, ?3)",
                (book_id, *name, testament),
            )?;
        }
        tx.commit()?;
    }

    // Insert verses
    let mut total_verses = 0u32;
    {
        let tx = conn.unchecked_transaction()?;
        for (book_idx, book) in books_arr.iter().enumerate() {
            let book_id = (book_idx + 1) as i64;
            let chapters = book.as_array()
                .ok_or_else(|| anyhow::anyhow!("Book {} is not an array", book_idx))?;

            for (chap_idx, chapter) in chapters.iter().enumerate() {
                let chapter_num = (chap_idx + 1) as i64;
                let verses = chapter.as_array()
                    .ok_or_else(|| anyhow::anyhow!(
                        "Book {} chapter {} is not an array", book_idx, chap_idx
                    ))?;

                for (verse_idx, verse_text) in verses.iter().enumerate() {
                    let verse_num = (verse_idx + 1) as i64;
                    let text = verse_text.as_str()
                        .ok_or_else(|| anyhow::anyhow!(
                            "Verse text is not a string at book {} ch {} v {}",
                            book_idx, chap_idx, verse_idx
                        ))?;

                    total_verses += 1;
                    tx.execute(
                        "INSERT INTO verses (id, book_id, chapter, verse, text) VALUES (?1, ?2, ?3, ?4, ?5)",
                        (total_verses as i64, book_id, chapter_num, verse_num, text),
                    )?;
                }
            }
        }
        tx.commit()?;
    }

    // Verify
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM verses", [], |r| r.get(0))?;
    println!("Generated {} with {} verses across {} books", db_path.display(), count, books_arr.len());

    // Quick sanity check: John 3:16
    let jn316: String = conn.query_row(
        "SELECT text FROM verses WHERE book_id = 44 AND chapter = 3 AND verse = 16",
        [],
        |r| r.get(0),
    )?;
    println!("John 3:16: {}", &jn316[..80.min(jn316.len())]);

    Ok(())
}
