mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static CARGO_TOML :&str = r#"[workspace]
[package]
name = "unused_byteorder"
version = "0.0.1"
[dependencies]
byteorder = "1.0.0"
"#;

static LIB_RS :&str = "";

#[test]
fn without_all_targets() -> CargoResult<()> {
	let (code, stdout_masked) =
		Runner::new("cargo_udeps_test_unused_byteorder_without_all_targets")?
			.cargo_toml(CARGO_TOML)?
			.dir("./src")?
			.file("./src/lib.rs", LIB_RS)?
			.run()?;
	assert_eq!(1, code);
	assert_eq!(
		r#"unused dependencies:
`unused_byteorder v0.0.1 (██████████)`
└─── (dev-)dependencies
     └─── "byteorder"
Note: These dependencies might be used by other targets.
      To find dependencies that are not used by any target, enable `--all-targets`.
"#,
		stdout_masked,
	);
	Ok(())
}


#[test]
fn with_all_targets() -> CargoResult<()> {
	let (code, stdout_masked) =
		Runner::new("cargo_udeps_test_unused_byteorder_with_all_targets")?
			.cargo_toml(CARGO_TOML)?
			.dir("./src")?
			.file("./src/lib.rs", LIB_RS)?
			.arg("--all-targets")
			.run()?;
	assert_eq!(1, code);
	assert_eq!(
		r#"unused dependencies:
`unused_byteorder v0.0.1 (██████████)`
└─── (dev-)dependencies
     └─── "byteorder"
"#,
		stdout_masked,
	);
	Ok(())
}
