use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper: initialize forge config in an isolated temp directory.
/// Returns the TempDir (must be kept alive for the test duration).
fn forge_init() -> TempDir {
    let tmp = TempDir::new().expect("temp dir");
    let config_home = tmp.path().join("config");
    let data_home = tmp.path().join("data");

    Command::cargo_bin("forge")
        .expect("forge binary")
        .env("XDG_CONFIG_HOME", config_home.to_str().unwrap())
        .env("XDG_DATA_HOME", data_home.to_str().unwrap())
        .arg("init")
        .assert()
        .success();

    tmp
}

/// Helper: build a forge command with isolated XDG dirs.
fn forge_cmd(tmp: &TempDir) -> assert_cmd::cmd::Command {
    let config_home = tmp.path().join("config");
    let data_home = tmp.path().join("data");
    let mut cmd = Command::cargo_bin("forge").expect("forge binary");
    cmd.env("XDG_CONFIG_HOME", config_home.to_str().unwrap())
        .env("XDG_DATA_HOME", data_home.to_str().unwrap());
    cmd
}

#[test]
fn spirit_word_daily_verse() {
    let tmp = forge_init();
    forge_cmd(&tmp)
        .arg("word")
        .assert()
        .success()
        .stdout(predicate::str::contains("Today's Verse"))
        .stdout(predicate::str::contains("──────"));
}

#[test]
fn spirit_word_search_love() {
    let tmp = forge_init();
    forge_cmd(&tmp)
        .args(["word", "search", "love"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search Results"))
        .stdout(predicate::str::contains("verse(s) found"));
}

#[test]
fn spirit_word_search_no_results() {
    let tmp = forge_init();
    forge_cmd(&tmp)
        .args(["word", "search", "xyznonexistent12345"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No verses found"));
}

#[test]
fn spirit_word_reference_john_3_16() {
    let tmp = forge_init();
    forge_cmd(&tmp)
        .args(["word", "reference", "John", "--chapter", "3", "--verse", "16"])
        .assert()
        .success()
        .stdout(predicate::str::contains("John"));
}

#[test]
fn spirit_reflect_entry() {
    let tmp = forge_init();
    forge_cmd(&tmp)
        .args(["reflect", "entry", "Test journal entry from integration test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Journal entry"))
        .stdout(predicate::str::contains("saved"))
        .stdout(predicate::str::contains("word(s)"));
}

#[test]
fn spirit_reflect_history_empty() {
    let tmp = forge_init();
    forge_cmd(&tmp)
        .arg("reflect")
        .assert()
        .success()
        .stdout(predicate::str::contains("No journal entries yet").or(predicate::str::contains("Journal")));
}

#[test]
fn spirit_reflect_history_after_entry() {
    let tmp = forge_init();

    // Create an entry first
    forge_cmd(&tmp)
        .args(["reflect", "entry", "History test entry"])
        .assert()
        .success();

    // Check history shows it
    forge_cmd(&tmp)
        .args(["reflect", "history"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Prayer Journal"));
}

#[test]
fn spirit_rest() {
    let tmp = forge_init();
    forge_cmd(&tmp)
        .arg("rest")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sabbath Rest"))
        .stdout(predicate::str::contains("Sabbath mode activated"));
}
