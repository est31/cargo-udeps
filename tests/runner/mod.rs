use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{self, AtomicBool};
use std::{env, fs, io, str};

use anyhow::Context;
use cargo::core::shell::Shell;
use cargo::{CargoResult, CliError};
use tempfile::TempDir;

static DEFAULT_TOOLCHAIN :&str = "nightly";

static RUSTC_ENV_SET :AtomicBool = AtomicBool::new(false);

pub(crate) struct Runner {
	cwd :TempDir,
	homedir :PathBuf,
	args :Vec<OsString>,
}

impl Runner {
	pub(crate) fn new(prefix :&str) -> CargoResult<Self> {
		if !RUSTC_ENV_SET.swap(true, atomic::Ordering::SeqCst) {
			let toolchain =
				env::var("CARGO_UDEPS_TEST_TOOLCHAIN").unwrap_or_else(|_| DEFAULT_TOOLCHAIN.to_owned());
			let Output { status, stdout, .. } = Command::new("rustup")
				.args(&["which", "rustc"])
				.env("RUSTUP_TOOLCHAIN", &toolchain)
				.output()?;
			if !status.success() {
				return Err(anyhow::anyhow!("{}", status))
					.with_context(|| format!("could not get the {} rustc", toolchain));
			}
			env::set_var("RUSTC", str::from_utf8(&stdout)?.trim());
		}
		let cwd = tempfile::Builder::new().prefix(prefix).tempdir()?;
		let homedir = dirs::home_dir().expect("couldn't get the current directory of the process");
		let args = vec!["".into(), "udeps".into()];
		Ok(Self {
			cwd,
			homedir,
			args,
		})
	}

	pub(crate) fn cargo_toml(self, content :&str) -> io::Result<Self> {
		self.file("Cargo.toml", content)
	}

	pub(crate) fn file(self, file_name :&str, content :&str) -> io::Result<Self> {
		let path = self.cwd.path().join(file_name);
		fs::write(path, content)?;
		Ok(self)
	}

	pub(crate) fn dir(self, name :&str) -> io::Result<Self> {
		let path = self.cwd.path().join(name);
		fs::create_dir_all(path)?;
		Ok(self)
	}

	pub(crate) fn arg<S: Into<OsString>>(mut self, arg: S) -> Self {
		self.args.push(arg.into());
		self
	}

	pub(crate) fn run(self) -> CargoResult<(i32, String)> {
		let mut stdout = vec![];
		let stderr = Shell::from_write(Box::new(vec![]));
		let mut config = cargo::Config::new(stderr, self.cwd.path().to_owned(), self.homedir.clone());
		let code = match cargo_udeps::run(self.args.clone(), &mut config, &mut stdout) {
			Ok(()) => 0,
			Err(CliError {
				error, exit_code, ..
			}) => match error {
				None => exit_code,
				Some(error) => return Err(error),
			}
		};
		let cwd_lossy = self.cwd.path().to_string_lossy();
		let stdout = str::from_utf8(&stdout)?.replace(&*cwd_lossy, "██████████");
		Ok((code, stdout))
	}
}
