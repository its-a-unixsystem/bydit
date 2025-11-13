use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_help_message() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: bydit [OPTIONS]"));
}

#[test]
fn test_run_without_args_shows_help() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Failed to locate config file 'config.toml'")
                .and(predicate::str::contains(".config/bydit/config.toml")),
        ); // Expect specific config search failure message
    temp_dir.close()?;
    Ok(())
}
