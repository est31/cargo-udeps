mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static CARGO_TOML: &str = r#"[workspace]
[package]
name = "unused_transitive"
version = "0.0.1"
[dependencies]
chrono = "=0.4.0"
time = "0.1"
"#;

static LIB_RS: &str = r#"
pub fn main() {
    println!("{:?}", chrono::Local::now());
}
"#;

/*
#[test]
fn show_unused_transitive() -> CargoResult<()> {
    let (code, stdout_masked) =
        Runner::new("cargo_udeps_test_show_unused_transitive")?
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
`unused_transitive v0.0.1 (██████████)`
└─── dependencies
     └─── "time"
Note: They might be false-positive.
      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.
      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.
"#,
        stdout_masked,
    );
    Ok(())
}
*/

#[test]
fn unused_transitive() -> CargoResult<()> {
    let (code, stdout_masked) = Runner::new("cargo_udeps_test_unused_transitive")?
        .cargo_toml(CARGO_TOML)?
        .dir("./src")?
        .file("./src/lib.rs", LIB_RS)?
        .arg("--all-targets")
        .run()?;
    assert_eq!(0, code);
    assert_eq!("All deps seem to have been used.\n", stdout_masked);
    Ok(())
}
