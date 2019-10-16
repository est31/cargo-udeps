use std::env;

use cargo::core::shell::Shell;

fn main() {
	let mut config = cargo::Config::default()
		.unwrap_or_else(|e| cargo::exit_with_error(e.into(), &mut Shell::new()));
	if let Err(err) = cargo_udeps::run(env::args_os(), &mut config) {
		cargo::exit_with_error(err, &mut config.shell());
	}
}
