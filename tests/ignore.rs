mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

#[test]
fn ignore_if_chain() -> CargoResult<()> {
    static CARGO_TOML: &str = r#"[workspace]
[package]
name = "ignore-if-chain"
version = "0.0.0"
edition = "2018"
publish = false

[package.metadata.cargo-udeps.ignore]
normal = ["if_chain"]

[dependencies]
if_chain = "1.0.0"
maplit = "1.0.2"
matches = "0.1.8"
"#;

    static LIB_RS: &str = "";

    let (code, stdout_masked) = Runner::new("cargo_udeps_test_ignore_ignore_if_chain")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`ignore-if-chain v0.0.0 (██████████)`
└─── dependencies
     ├─── "maplit"
     └─── "matches"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
"#,
        stdout_masked,
    );
    Ok(())
}

#[test]
fn ignore_workspace() -> CargoResult<()> {
    static CARGO_TOML: &str = r#"[workspace]

[workspace.metadata.cargo-udeps.ignore]
normal = ["if_chain"]

[package]
name = "ignore-workspace"
version = "0.0.0"
edition = "2018"
publish = false

[dependencies]
if_chain = "1.0.0"
maplit = "1.0.2"
matches = "0.1.8"
"#;

    static LIB_RS: &str = "";

    let (code, stdout_masked) = Runner::new("cargo_udeps_test_ignore_ignore_workspace")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`ignore-workspace v0.0.0 (██████████)`
└─── dependencies
     ├─── "maplit"
     └─── "matches"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
"#,
        stdout_masked,
    );
    Ok(())
}

#[test]
fn ignore_all() -> CargoResult<()> {
    static CARGO_TOML: &str = r#"[workspace]
[package]
name = "ignore-all"
version = "0.0.0"
edition = "2018"
publish = false

[package.metadata.cargo-udeps.ignore]
normal = ["if_chain", "maplit", "matches"]

[dependencies]
if_chain = "1.0.0"
maplit = "1.0.2"
matches = "0.1.8"
"#;

    static LIB_RS: &str = "";

    let (code, stdout_masked) = Runner::new("cargo_udeps_test_ignore_ignore_all")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .run()?;
    assert_eq!(0, code);
    assert_eq!("All deps seem to have been used.\n", stdout_masked);
    Ok(())
}
