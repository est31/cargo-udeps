mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static CARGO_TOML: &str = r#"
cargo-features = ["test-dummy-unstable"]
[workspace]
[package]
name = "unused_byteorder"
version = "0.0.1"
[dependencies]
byteorder = "1.0.0"
"#;

static LIB_RS: &str = "";

#[test]
fn without_all_targets() -> CargoResult<()> {
    let (code, stdout_masked) =
        Runner::new("cargo_udeps_test_nightly_features_without_all_targets")?
            .cargo_toml(CARGO_TOML)?
            .dir("./src")?
            .file("./src/lib.rs", LIB_RS)?
            .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`unused_byteorder v0.0.1 (██████████)`
└─── dependencies
     └─── "byteorder"
Note: These dependencies might be used by other targets.
      To find dependencies that are not used by any target, enable `--all-targets`.
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
"#,
        stdout_masked,
    );
    Ok(())
}

#[test]
fn with_all_targets() -> CargoResult<()> {
    let (code, stdout_masked) = Runner::new("cargo_udeps_test_nightly_features_with_all_targets")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`unused_byteorder v0.0.1 (██████████)`
└─── dependencies
     └─── "byteorder"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
"#,
        stdout_masked,
    );
    Ok(())
}

#[test]
fn ignore_if_chain() -> CargoResult<()> {
    static CARGO_TOML: &str = r#"
cargo-features = ["test-dummy-unstable"]
[workspace]
[package]
name = "ignore-if-chain"
version = "0.0.0"
edition = "2018"
publish = false
print-im-a-teapot = true

[package.metadata.cargo-udeps.ignore]
normal = ["if_chain"]

[dependencies]
if_chain = "1.0.0"
"#;

    static LIB_RS: &str = "";

    let (code, stdout_masked) = Runner::new("cargo_udeps_test_nightly_features_ignore_if_chain")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(0, code);
    assert_eq!(
        r#"All deps seem to have been used.
"#,
        stdout_masked,
    );
    Ok(())
}
