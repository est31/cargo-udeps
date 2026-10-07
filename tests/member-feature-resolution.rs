mod runner;

use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

static WORKSPACE_CARGO_TOML: &str = r#"[workspace]
members = ["consumer", "checkpoints"]
resolver = "2"
"#;

static CONSUMER_CARGO_TOML: &str = r#"[package]
name = "consumer"
version = "0.0.1"

[features]
tracing = []
"#;

static CHECKPOINTS_CARGO_TOML: &str = r#"[package]
name = "checkpoints"
version = "0.0.1"

[features]
enable = ["dep:tracing"]

[dependencies]
tracing = { version = "0.1", optional = true }
"#;

fn workspace(prefix: &str) -> CargoResult<Runner> {
	Ok(Runner::new(prefix)?
		.cargo_toml(WORKSPACE_CARGO_TOML)?
		.dir("./consumer/src")?
		.file("./consumer/Cargo.toml", CONSUMER_CARGO_TOML)?
		.file("./consumer/src/lib.rs", "")?
		.dir("./checkpoints/src")?
		.file("./checkpoints/Cargo.toml", CHECKPOINTS_CARGO_TOML)?
		.file("./checkpoints/src/lib.rs", "")?)
}

#[test]
fn manifest_path_features_do_not_apply_to_unselected_workspace_members() -> CargoResult<()> {
	let (code, stdout_masked) =
		workspace("cargo_udeps_test_manifest_path_feature_resolution")?
			.arg("--manifest-path")
			.arg("consumer/Cargo.toml")
			.arg("--features")
			.arg("tracing")
			.run()?;
	assert_eq!(0, code);
	assert_eq!("All deps seem to have been used.\n", stdout_masked);
	Ok(())
}

#[test]
fn package_features_do_not_apply_to_unselected_workspace_members() -> CargoResult<()> {
	let (code, stdout_masked) =
		workspace("cargo_udeps_test_package_feature_resolution")?
			.arg("--package")
			.arg("consumer")
			.arg("--features")
			.arg("tracing")
			.run()?;
	assert_eq!(0, code);
	assert_eq!("All deps seem to have been used.\n", stdout_masked);
	Ok(())
}
