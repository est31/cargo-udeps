use std::env;

use cargo::CliResult;

fn main() -> CliResult {
	cargo_udeps::run(env::args_os())
}
