use assert_cmd::prelude::*;
// Add methods on commands
use predicates::prelude::*;
// Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn start_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("json-gen")?;

    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout("json-generator 0.2");

    Ok(())
}

#[test]
fn help_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("json-gen")?;

    cmd.arg("--help");
    cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("json-generator 0.2"));

    Ok(())
}
#[test]

fn basic_body_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("json-gen")?;
    let json_body = r#"
    {
        "|type": "str_from_list(business,technical,analytical)",
        "|id": "uuid()",
        "|index": "seq()",
        "|created_tm": "dt(%Y-%m-%d)",
        "|related_records": "int(1,1000) -> array(5)"
    }
    "#;
    cmd.arg(format!("-b {}",json_body)).arg("--pretty");
    cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("created_tm")
            .and(predicate::str::contains("id"))
            .and(predicate::str::contains("index"))
            .and(predicate::str::contains("related_records"))
        );

    Ok(())
}