mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static CARGO_TOML: &str = r#"[workspace]

[package]
name = "non_lib_build_dep"
version = "0.0.0"

[build-dependencies]
diffr = "=0.1.5"
"#;

static LIB_RS: &str = "";

#[test]
fn without_all_targets() -> CargoResult<()> {
    let (code, stdout_masked) =
        Runner::new("cargo_udeps_test_non_lib_build_dep_without_all_targets")?
            .cargo_toml(CARGO_TOML)?
            .dir("./src")?
            .file("./src/lib.rs", LIB_RS)?
            .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`non_lib_build_dep v0.0.0 (██████████)`
└─── build-dependencies
     └─── "diffr"
Note: These dependencies might be used by other targets.
      To find dependencies that are not used by any target, enable `--all-targets`.
Note: Some dependencies are non-library packages.
      `cargo-udeps` regards them as unused.
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
    let (code, stdout_masked) = Runner::new("cargo_udeps_test_non_lib_build_dep_with_all_targets")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`non_lib_build_dep v0.0.0 (██████████)`
└─── build-dependencies
     └─── "diffr"
Note: Some dependencies are non-library packages.
      `cargo-udeps` regards them as unused.
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
"#,
        stdout_masked,
    );
    Ok(())
}
