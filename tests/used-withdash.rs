mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static CARGO_TOML: &str = r#"[workspace]
[package]
name = "usesiso639dash1"
version = "0.0.1"
edition = "2021"
[dependencies]
iso639-1 = "0.3.0"
"#;

static LIB_RS: &str = r#"use iso639_1::{from_iso639_1, to_iso639_3, Iso639_1};

pub fn bla() {
    assert!(Iso639_1::Fr != Iso639_1::En);
    assert!(from_iso639_1("fr").unwrap() == Iso639_1::Fr);
    assert!(to_iso639_3("fr").unwrap() == "fra");
}
"#;

#[test]
fn without_all_targets() -> CargoResult<()> {
    let (code, stdout_masked) =
        Runner::new("cargo_udeps_test_usesiso639dash1_without_all_targets")?
            .cargo_toml(CARGO_TOML)?
            .dir("./src")?
            .file("./src/lib.rs", LIB_RS)?
            .run()?;
    assert_eq!(0, code, "STDOUT: {stdout_masked}");
    assert_eq!("All deps seem to have been used.\n", stdout_masked);
    Ok(())
}

#[test]
fn with_all_targets() -> CargoResult<()> {
    let (code, stdout_masked) = Runner::new("cargo_udeps_test_usesiso639dash1_with_all_targets")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(0, code, "STDOUT: {stdout_masked}");
    assert_eq!("All deps seem to have been used.\n", stdout_masked);
    Ok(())
}
