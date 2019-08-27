extern crate cargo;
extern crate serde;
extern crate serde_json;

mod defs;

use std::fmt::Display;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use defs::CrateSaveAnalysis;
use cargo::core::shell::Shell;
use cargo::core::compiler::{Executor, DefaultExecutor};
use cargo::util::process_builder::ProcessBuilder;
use cargo::core::package_id::PackageId;
use cargo::core::manifest::Target;
use cargo::util::errors::CargoResult;
use cargo::util::command_prelude::{App, subcommand, opt, ArgMatchesExt,
	AppExt, CompileMode, Config};

fn cli() -> App {
	subcommand("udeps")
		.about("Check a local package and all of its dependencies for errors")
		.arg(opt("quiet", "No output printed to stdout").short("q"))
		.arg_package_spec(
			"Package(s) to check",
			"Check all packages in the workspace",
			"Exclude packages from the check",
		)
		.arg_jobs()
		.arg_targets_all(
			"Check only this package's library",
			"Check only the specified binary",
			"Check all binaries",
			"Check only the specified example",
			"Check all examples",
			"Check only the specified test target",
			"Check all tests",
			"Check only the specified bench target",
			"Check all benches",
			"Check all targets",
		)
		.arg_release("Check artifacts in release mode, with optimizations")
		.arg(opt("profile", "Profile to build the selected target for").value_name("PROFILE"))
		.arg_features()
		.arg_target_triple("Check for the target triple")
		.arg_target_dir()
		.arg_manifest_path()
		.arg_message_format()
		.after_help(
			"\
If the `--package` argument is given, then SPEC is a package ID specification
which indicates which package should be built. If it is not given, then the
current package is built. For more information on SPEC and its format, see the
`cargo help pkgid` command.

All packages in the workspace are checked if the `--all` flag is supplied. The
`--all` flag is automatically assumed for a virtual manifest.
Note that `--exclude` has to be specified in conjunction with the `--all` flag.

Compilation can be configured via the use of profiles which are configured in
the manifest. The default profile for this command is `dev`, but passing
the `--release` flag will use the `release` profile instead.

The `--profile test` flag can be used to check unit tests with the
`#[cfg(test)]` attribute.
",
		)
}

#[derive(Debug)]
pub struct StrErr(String);

impl<T :Display> From<T> for StrErr {
	fn from(v :T) -> Self {
		StrErr(format!("{}", v))
	}
}

struct ExecData {
	times :HashMap<PackageId, SystemTime>,
	final_crate :Option<(PackageId, ProcessBuilder)>,
}

impl ExecData {
	fn new() -> Self {
		Self {
			times : HashMap::new(),
			final_crate : None,
		}
	}
}

struct Exec {
	data :Arc<Mutex<ExecData>>,
}

impl Executor for Exec {
	fn exec(&self, mut cmd :ProcessBuilder, id :PackageId, target :&Target,
			mode :CompileMode, on_stdout_line :&mut dyn FnMut(&str) -> CargoResult<()>,
			on_stderr_line: &mut dyn FnMut(&str) -> CargoResult<()>) -> CargoResult<()> {
		{
			// TODO unwrap used
			let mut bt = self.data.lock().unwrap();
			bt.times.insert(id, SystemTime::now());
			bt.final_crate = Some((id, cmd.clone()));
		}
		cmd.arg("-Z").arg("save-analysis");
		DefaultExecutor.exec(cmd, id, target, mode, on_stderr_line, on_stdout_line)?;
		Ok(())
	}
}

#[derive(Debug)]
struct CmdInfo {
	crate_name :String,
	extra_filename :String,
	out_dir :String,
	externs :Vec<(String, String)>,
}

impl CmdInfo {
	fn get_save_analysis_path(&self) -> PathBuf {
		Path::new(&self.out_dir)
			.join("save-analysis")
			.join(self.crate_name.clone() + &self.extra_filename + ".json")
	}
	fn get_save_analysis(&self) -> Result<CrateSaveAnalysis, StrErr> {
		let p = self.get_save_analysis_path();
		println!("Loading save analysis from {:?}", p);
		let f = std::fs::read_to_string(p)?;
		let res = serde_json::from_str(&f)?;
		Ok(res)
	}
}

fn cmd_info(cmd :&ProcessBuilder) -> CmdInfo {
	let mut args_iter = cmd.get_args().iter();
	let mut crate_name = None;
	let mut extra_filename = None;
	let mut out_dir = None;
	let mut externs = Vec::<(String, String)>::new();
	while let Some(v) = args_iter.next() {
		if v == "--extern" {
			let arg = args_iter.next()
				.map(|a| a.to_str().expect("non-utf8 paths not supported atm"))
				.map(|a| {
					let mut splitter = a.split("=");
					if let (Some(n), Some(p)) = (splitter.next(), splitter.next()) {
						(n.to_owned(), p.to_owned())
					} else {
						panic!("invalid format for extern arg: {}", a);
					}
				});
			if let Some(e) = arg {
				externs.push(e);
			}
		} else if v == "--crate-name" {
			if let Some(name) = args_iter.next() {
				crate_name = Some(name.to_str()
					.expect("non-utf8 crate names not supported")
					.to_owned());
			}
		} else if v == "--out-dir" {
			if let Some(d) = args_iter.next() {
				out_dir = Some(d.to_str()
					.expect("non-utf8 crate names not supported")
					.to_owned());
			}
		} else if v == "-C" {
			if let Some(arg) = args_iter.next() {
				let arg = arg.to_str().expect("non-utf8 args not supported atm");
				let mut splitter = arg.split("=");
				if let (Some(n), Some(p)) = (splitter.next(), splitter.next()) {
					if n == "extra-filename" {
						extra_filename = Some(p.to_owned());
					}
				}
			}
		}
	}
	let crate_name = crate_name.unwrap();
	let extra_filename = extra_filename.unwrap();
	let out_dir = out_dir.unwrap();

	CmdInfo {
		crate_name,
		extra_filename,
		out_dir,
		externs,
	}
}

fn main() -> Result<(), StrErr> {
	cargo::core::maybe_allow_nightly_features();
	let config = match Config::default() {
		Ok(cfg) => cfg,
		Err(e) => {
			let mut shell = Shell::new();
			cargo::exit_with_error(e.into(), &mut shell)
		}
	};
	let app = cli();
	let args = app.get_matches();
	let ws = args.workspace(&config)?;
	let mode = CompileMode::Check { test : false };
	let compile_opts = args.compile_options(&config, mode, Some(&ws))?;

	let data = Arc::new(Mutex::new(ExecData::new()));
	let exec :Arc<dyn Executor + 'static> = Arc::new(Exec { data : data.clone() });
	cargo::ops::compile_with_exec(&ws, &compile_opts, &exec)?;
	let data = data.lock()?;
	if let Some((f, cmd)) = &data.final_crate {
		let final_time = data.times.get(f).unwrap();
		let cmd_info = cmd_info(cmd);
		let analysis = cmd_info.get_save_analysis()?;
		let names = analysis.prelude.external_crates.iter()
			.map(|e| &e.id.name)
			.collect::<HashSet<_>>();
		let mut unused_externs = Vec::new();
		for (ext, _path) in cmd_info.externs.iter() {
			if !names.contains(&ext) {
				unused_externs.push(ext);
			}
		}
		if !unused_externs.is_empty() {
			println!("unused crates: {:?}", unused_externs);
		} else {
			println!("All deps seem to have been used.");
		}
	}
	Ok(())
}
