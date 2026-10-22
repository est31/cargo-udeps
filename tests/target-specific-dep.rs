mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

// The same package listed under the same name for several targets.
static CARGO_TOML :&str = r#"[workspace]

[package]
name = "target_specific_dep"
version = "0.0.1"
edition = "2018"
publish = false

[target.'cfg(unix)'.dependencies]
if_chain = "1.0.0"

[target.'cfg(not(unix))'.dependencies]
if_chain = "1.0.0"
"#;

static LIB_RS :&str = "pub use if_chain::if_chain;\n";

#[test]
fn no_duplicate_package_warning() -> CargoResult<()> {
	let (code, stdout_masked, stderr_masked) =
		Runner::new("cargo_udeps_test_target_specific_dep")?
			.cargo_toml(CARGO_TOML)?
			.dir("./src")?
			.file("./src/lib.rs", LIB_RS)?
			.run_with_stderr()?;
	assert_eq!(0, code);
	assert_eq!("All deps seem to have been used.\n", stdout_masked);
	assert!(!stderr_masked.contains("duplicate package"), "{}", stderr_masked);
	Ok(())
}
