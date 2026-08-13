mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static WORKSPACE_CARGO_TOML: &str = r#"[workspace]
members = ["crate_a", "crate_b"]
resolver = "2"
"#;

static CRATE_A_CARGO_TOML: &str = r#"[package]
name = "crate_a"
version = "0.1.0"
edition = "2021"

[features]
foo = []

[dependencies]
"#;

static CRATE_B_CARGO_TOML: &str = r#"[package]
name = "crate_b"
version = "0.1.0"
edition = "2021"

[features]
bar = ["dep:foo"]

[dependencies]
foo = { package = "crate_a", path = "../crate_a", optional = true }
"#;

static LIB_RS: &str = "";

#[test]
fn package_with_features_in_workspace() -> CargoResult<()> {
    let (code, stdout_masked) = Runner::new("cargo_udeps_test_workspace_package_features")?
        .cargo_toml(WORKSPACE_CARGO_TOML)?
        .dir("crate_a/src")?
        .file("crate_a/Cargo.toml", CRATE_A_CARGO_TOML)?
        .file("crate_a/src/lib.rs", LIB_RS)?
        .dir("crate_b/src")?
        .file("crate_b/Cargo.toml", CRATE_B_CARGO_TOML)?
        .file("crate_b/src/lib.rs", LIB_RS)?
        .arg("--package")
        .arg("crate_a")
        .arg("--features")
        .arg("foo")
        .arg("--no-default-features")
        .run()?;
    assert_eq!(0, code);
    assert_eq!("All deps seem to have been used.\n", stdout_masked);
    Ok(())
}
