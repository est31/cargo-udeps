mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static CARGO_TOML: &str = r#"[workspace]
[package]
name = "has-test-cfg"
version = "0.0.1"
[dependencies]
"#;

static LIB_RS: &str = r#"
#![deny(unexpected_cfgs)]
#[cfg(test)]
pub fn something_conditional() {}
"#;

#[test]
fn cfg_test_no_unexpected_cfg() -> CargoResult<()> {
    let (code, stdout_masked) = Runner::new("cargo_udeps_cfg_test_no_unexpected_cfg")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(0, code);
    assert_eq!("All deps seem to have been used.\n", stdout_masked);
    Ok(())
}
