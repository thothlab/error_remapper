use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cmd() -> Command {
    let mut c = Command::cargo_bin("error-remapper").unwrap();
    c.current_dir(project_root());
    c
}

#[test]
fn test_exact_code_match_with_custom_desc() {
    let input = r#"{"error":{"code":2001,"title":"Got unexpected symbol: @ in input"}}"#;
    cmd()
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("81002"))
        .stdout(predicate::str::contains("Недопустимый символ в назначении перевода"));
}

#[test]
fn test_exact_code_match_no_custom_desc() {
    let input = r#"{"error":{"code":"2002","title":"Уточните у получателя реквизиты счёта"}}"#;
    cmd()
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("81001"))
        .stdout(predicate::str::contains("Уточните у получателя реквизиты счёта"));
}

#[test]
fn test_multiple_code_matches_fuzzy_resolves() {
    // key 3011 has two entries, fuzzy should pick the right one
    let input = r#"{"error":{"code":"3011","title":"Не пройден фрод-мониторинг операции"}}"#;
    cmd()
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("81005"))
        .stdout(predicate::str::contains("Перевод отклонён банком получателя"));
}

#[test]
fn test_no_code_match_fuzzy_on_all() {
    let input = r#"{"errorCode":"9999","message":"Перевод отклонен провайдером получателя"}"#;
    cmd()
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("81006"));
}

#[test]
fn test_no_match_returns_original() {
    let input = r#"{"error":{"code":"9999","title":"Completely unrelated error text"}}"#;
    cmd()
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("9999"))
        .stdout(predicate::str::contains("Completely unrelated error text"));
}

#[test]
fn test_stdin_input() {
    let input = r#"{"error":{"code":"3010","title":"Мы заблокировали переводы на этот счёт"}}"#;
    cmd()
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("81003"));
}

#[test]
fn test_invalid_json() {
    cmd()
        .arg("not a json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse input JSON"));
}

#[test]
fn test_verbose_flag() {
    let input = r#"{"error":{"code":"2001","title":"unexpected symbol: test"}}"#;
    cmd()
        .arg("--verbose")
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("81002"));
}

#[test]
fn test_custom_errors_path() {
    let input = r#"{"error":{"code":"2001","title":"unexpected symbol: test"}}"#;
    cmd()
        .arg("--errors")
        .arg("config/errors.yaml")
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("81002"));
}
