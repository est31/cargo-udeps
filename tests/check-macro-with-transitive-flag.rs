mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static CARGO_TOML: &str = r#"[workspace]
[package]
name = "strum_macro"
version = "0.0.1"
[dependencies]
strum = { version = "0.24", features = ["derive"] }
"#;

static LIB_RS: &str = r#"
#[derive(strum::AsRefStr)]
pub enum Color {
    Red,
}
"#;

#[test]
fn macro_call() -> CargoResult<()> {
    let (code, stdout_masked) = Runner::new("cargo_udeps_test_macro_call")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(0, code);
    assert_eq!("All deps seem to have been used.\n", stdout_masked);
    Ok(())
}

/*
/// For now we can't check if macro is really used in crate, because macro usages do not populate
/// "refs" section of save-analysis file.
#[test]
fn macro_call_with_unused_transitive_flag() -> CargoResult<()> {
    let (code, stdout_masked) =
        Runner::new("cargo_udeps_test_macro_call_with_unused_transitive_flag")?
            .cargo_toml(CARGO_TOML)?
            .dir("./src")?
            .file("./src/lib.rs", LIB_RS)?
            .arg("--all-targets")
            .arg("--backend")
            .arg("save-analysis")
            .arg("--show-unused-transitive")
            .run()?;
    assert_eq!(1, code);
    assert_eq!(
        r#"unused dependencies:
`strum_macro v0.0.1 (██████████)`
└─── dependencies
     └─── "strum"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
"#,
        stdout_masked,
    );
    Ok(())
}
*/
