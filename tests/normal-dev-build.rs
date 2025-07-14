mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static CARGO_TOML: &str = r#"[workspace]

[package]
name = "normal_dev_build"
version = "0.0.1"
edition = "2018"
publish = false

[dependencies]
if_chain = "1.0.0"

[dev-dependencies]
maplit = "1.0.2"

[build-dependencies]
matches = "0.1.8"
"#;

static LIB_RS: &str = "";
static BUILD_RS: &str = "fn main() {}\n";

#[test]
fn without_all_targets() -> CargoResult<()> {
    let (code, stdout_masked) =
        Runner::new("cargo_udeps_test_normal_dev_build_without_all_targets")?
            .cargo_toml(CARGO_TOML)?
            .dir("./src")?
            .file("./src/lib.rs", LIB_RS)?
            .file("./build.rs", BUILD_RS)?
            .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`normal_dev_build v0.0.1 (██████████)`
├─── dependencies
│    └─── "if_chain"
└─── build-dependencies
     └─── "matches"
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
    let (code, stdout_masked) = Runner::new("cargo_udeps_test_normal_dev_build_with_all_targets")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .file("./build.rs", BUILD_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`normal_dev_build v0.0.1 (██████████)`
├─── dependencies
│    └─── "if_chain"
├─── dev-dependencies
│    └─── "maplit"
└─── build-dependencies
     └─── "matches"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
"#,
        stdout_masked,
    );
    Ok(())
}
