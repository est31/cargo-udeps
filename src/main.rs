use std::env;

use cargo::CliResult;

fn main() -> CliResult {
	let mut config = cargo::Config::default()?;
	cargo_udeps::run(env::args_os(), &mut config)
}
