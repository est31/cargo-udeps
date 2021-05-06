use std::{env, io};

use cargo::core::shell::Shell;

fn main() {
	let mut config = cargo::Config::default()
		.unwrap_or_else(|e| cargo::exit_with_error(e.into(), &mut Shell::new()));
	config.nightly_features_allowed = true;
	if let Err(err) = cargo_udeps::run(env::args_os(), &mut config, io::stdout()) {
		cargo::exit_with_error(err, &mut config.shell());
	}
}
