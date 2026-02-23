use std::process::Command;
use assert_cmd::prelude::*;
use anyhow::Result;

#[test]
fn test_help_output() -> Result<()> {
    let mut cmd = Command::cargo_bin("jfrog2nexus")?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("sync"))
        .stdout(predicates::str::contains("config"));
    Ok(())
}

#[test]
fn test_config_help_output() -> Result<()> {
    let mut cmd = Command::cargo_bin("jfrog2nexus")?;
    cmd.arg("config").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("validate"));
    Ok(())
}
