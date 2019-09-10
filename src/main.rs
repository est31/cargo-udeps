extern crate cargo;
extern crate cargo_metadata;
extern crate serde;
extern crate serde_json;
extern crate which;

mod defs;

use std::fmt::Display;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Mutex;
use defs::CrateSaveAnalysis;
use cargo::core::shell::Shell;
use cargo::core::compiler::{Executor, DefaultExecutor};
use cargo::util::process_builder::ProcessBuilder;
use cargo::core::package_id::PackageId;
use cargo::core::manifest::Target;
use cargo::util::errors::CargoResult;
use cargo::core::shell::Verbosity;
use cargo::util::command_prelude::{App, Arg, opt, ArgMatchesExt,
	AppExt, CompileMode, Config};
use cargo::ops::OutputMetadataOptions;
use cargo_metadata::Metadata;

fn cli() -> App {
	App::new("cargo-udeps")
		.version(env!("CARGO_PKG_VERSION"))
		.arg(Arg::with_name("dummy")
			.hidden(true)
			.possible_value("udeps"))
		.about("Find unused dependencies in your Cargo.toml")
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
the `--release` flag will use the `release` profile instead. ")

/*
The `--profile test` flag can be used to check unit tests with the
`#[cfg(test)]` attribute.
",
		)*/
}

#[derive(Debug)]
pub struct StrErr(String);

impl<T :Display> From<T> for StrErr {
	fn from(v :T) -> Self {
		StrErr(format!("{}", v))
	}
}

struct ExecData {
	cargo_exe :Option<String>,
	relevant_cmd_infos :Vec<CmdInfo>,
}

impl ExecData {
	fn new() -> Self {
		let cargo_exe = which::which("cargo")
			.map_err(|e| println!("warning: couldn't load cargo executable file: {:?}", e))
			.ok()
			.and_then(|p| p.to_str().map(str::to_owned));
		Self {
			cargo_exe,
			relevant_cmd_infos : Vec::new(),
		}
	}
}

struct Exec {
	data :Arc<Mutex<ExecData>>,
}

impl Executor for Exec {
	fn exec(&self, mut cmd :ProcessBuilder, id :PackageId, target :&Target,
			mode :CompileMode, on_stdout_line :&mut dyn FnMut(&str) -> CargoResult<()>,
			on_stderr_line :&mut dyn FnMut(&str) -> CargoResult<()>) -> CargoResult<()> {

		let cmd_info = cmd_info(id, &cmd).unwrap_or_else(|e| {
			panic!("Couldn't obtain crate info {:?}: {:?}", id, e);
		});
		{
			// TODO unwrap used
			let mut bt = self.data.lock().unwrap();

			// If the crate is not a library crate,
			// we are not interested in its information.
			if !cmd_info.cap_lints_allow {
				bt.relevant_cmd_infos.push(cmd_info.clone());
			}
			if let Some(cargo_exe) = &bt.cargo_exe {
				cmd.env(cargo::CARGO_ENV, cargo_exe);
			}
		}
		if !cmd_info.cap_lints_allow {
			std::env::set_var("RUST_SAVE_ANALYSIS_CONFIG",
				r#"{ "reachable_only": true, "full_docs": false, "pub_only": false, "distro_crate": false, "signatures": false, "borrow_data": false }"#);
			cmd.arg("-Z").arg("save-analysis");
		}
		DefaultExecutor.exec(cmd, id, target, mode, on_stderr_line, on_stdout_line)?;
		Ok(())
	}
}

#[derive(Clone, Debug)]
struct CmdInfo {
	pkg :cargo_metadata::PackageId,
	crate_name :String,
	crate_type :String,
	extra_filename :String,
	cap_lints_allow :bool,
	out_dir :String,
	externs :Vec<(String, String)>,
}

impl CmdInfo {
	fn get_save_analysis_path(&self) -> PathBuf {
		let maybe_lib = if self.crate_type.ends_with("lib") ||
				self.crate_type == "proc-macro" {
			"lib"
		} else {
			""
		};
		let filename = maybe_lib.to_owned() +
			&self.crate_name + &self.extra_filename + ".json";
		Path::new(&self.out_dir)
			.join("save-analysis")
			.join(filename)
	}
	fn get_save_analysis(&self) -> Result<CrateSaveAnalysis, StrErr> {
		let p = self.get_save_analysis_path();
		println!("Loading save analysis from {:?}", p);
		let f = std::fs::read_to_string(p)?;
		let res = serde_json::from_str(&f)?;
		Ok(res)
	}
}

fn cmd_info(id :PackageId, cmd :&ProcessBuilder) -> Result<CmdInfo, StrErr> {
	let mut args_iter = cmd.get_args().iter();
	let mut crate_name = None;
	let mut crate_type = None;
	let mut extra_filename = None;
	let mut cap_lints_allow = false;
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
		} else if v == "--crate-type" {
			if let Some(ty) = args_iter.next() {
				crate_type = Some(ty.to_str()
					.expect("non-utf8 crate names not supported")
					.to_owned());
			}
		} else if v == "--cap-lints" {
			if let Some(c) = args_iter.next() {
				if c == "allow" {
					cap_lints_allow = true;
				}
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
	let pkg = id.to_cargo_metadata_package_id();
	let crate_name = crate_name.ok_or("crate name needed")?;
	let crate_type = crate_type.unwrap_or("bin".to_owned());
	let extra_filename = extra_filename.ok_or("extra-filename needed")?;
	let out_dir = out_dir.ok_or("outdir needed")?;

	Ok(CmdInfo {
		pkg,
		crate_name,
		crate_type,
		extra_filename,
		cap_lints_allow,
		out_dir,
		externs,
	})
}

trait PackageIdExt {
	fn to_cargo_metadata_package_id(self) -> cargo_metadata::PackageId;
}

impl PackageIdExt for PackageId {
	fn to_cargo_metadata_package_id(self) -> cargo_metadata::PackageId {
		cargo_metadata::PackageId {
			repr : format!(
				"{} {} ({})",
				self.name(),
				self.version(),
				self.source_id().into_url(),
			),
		}
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
	config.shell().set_verbosity(Verbosity::Normal);
	let app = cli();
	let args = app.get_matches();
	let ws = args.workspace(&config)?;
	let mode = CompileMode::Check { test : false };
	let compile_opts = args.compile_options(&config, mode, Some(&ws))?;

	let metadata = {
		let opts = OutputMetadataOptions {
			features : args
				.values_of("features")
				.map(|vs| vs.map(ToOwned::to_owned).collect())
				.unwrap_or_default(),
			no_default_features : args.is_present("no-default-features"),
			all_features : args.is_present("all-features"),
			no_deps : false,
			version : 1,
		};
		let metadata = cargo::ops::output_metadata(&ws, &opts)?;
		serde_json::from_str::<Metadata>(&serde_json::to_string(&metadata)?)?
	};

	let metadata_packages = metadata.packages
		.iter()
		.map(|p| (&p.id, p))
		.collect::<HashMap<_, _>>();

	let (dependency_names_by_lib_rename, dependency_names_by_lib_true_snakecased_name) = {
		let mut dependency_names_by_lib_rename = HashMap::new();
		let mut dependency_names_by_lib_true_snakecased_name = HashMap::new();

		for package in ws.members() {
			let id = package.package_id().to_cargo_metadata_package_id();
			let package = metadata_packages
				.get(&id)
				.unwrap_or_else(|| panic!("could not find {:?}", id.repr));

			let resolve = metadata.resolve
				.as_ref()
				.and_then(|r| r.nodes.iter().find(|n| n.id == id))
				.expect("could not find crate in resolve");

			let renamed = package.dependencies
				.iter()
				.flat_map(|d| d.rename.as_ref().map(|r| (r, d)))
				.collect::<HashMap<_, _>>();

			let unrenamed = package.dependencies
				.iter()
				.filter(|d| d.rename.is_none())
				.map(|d| (&d.name, d))
				.collect::<HashMap<_, _>>();

			for dep in resolve.deps.iter() {
				let pkg = &metadata_packages
					.get(&dep.pkg)
					.unwrap_or_else(|| panic!("could not find {:?}", dep.pkg.repr));
				let lib_name = &pkg.targets.iter()
					.find(|t| t.kind.iter().any(|k| k == "lib" || k == "proc-macro"))
					.unwrap_or_else(|| {
						panic!(
							"could not find any `lib` or `proc-macro` target in {:?}",
							dep.pkg.repr,
						)
					})
					.name;
				let dependency = &renamed.get(&dep.name)
					.or_else(|| unrenamed.get(lib_name))
					.or_else(|| unrenamed.get(&pkg.name))
					.unwrap_or_else(|| panic!("could not find {:?}", dep.pkg.repr));
				let dependency_name = dependency.rename.as_ref().unwrap_or(&dependency.name);

				dependency_names_by_lib_rename
					.entry(id.clone())
					.or_insert_with(HashMap::new)
					.insert(&dep.name, dependency_name);

				let lib_name_snakecased = lib_name.replace('-', "_");
				if let Some(dependency_name2) = dependency_names_by_lib_true_snakecased_name
					.entry(id.clone())
					.or_insert_with(HashMap::new)
					.insert(lib_name_snakecased.clone(), dependency_name)
				{
					return Err(StrErr(format!(
						"current implementation cannot handle multiple crates with the same `lib` name:\n\
						 {id:?}\n\
						 ├ {dependency_name1:?} → {lib_name_snakecased:?}\n\
						 └ {dependency_name2:?} → {lib_name_snakecased:?}",
						id = id,
						dependency_name1 = dependency_name,
						dependency_name2 = dependency_name2,
						lib_name_snakecased = lib_name_snakecased,
					)));
				}
			}
		}

		(dependency_names_by_lib_rename, dependency_names_by_lib_true_snakecased_name)
	};

	let data = Arc::new(Mutex::new(ExecData::new()));
	let exec :Arc<dyn Executor + 'static> = Arc::new(Exec { data : data.clone() });
	cargo::ops::compile_with_exec(&ws, &compile_opts, &exec)?;
	let data = data.lock()?;

	let mut used_dependencies = HashSet::new();
	let mut dependencies = HashSet::new();

	for cmd_info in data.relevant_cmd_infos.iter() {
		let analysis = cmd_info.get_save_analysis()?;
		for ext in &analysis.prelude.external_crates {
			if let Some(dependency_name) = dependency_names_by_lib_true_snakecased_name
				.get(&cmd_info.pkg)
				.and_then(|names| names.get(&ext.id.name))
			{
				used_dependencies.insert((&cmd_info.pkg.repr, *dependency_name));
			}
		}
		for (name, _) in &cmd_info.externs {
			let dependency_name = dependency_names_by_lib_rename
				.get(&cmd_info.pkg)
				.and_then(|names| names.get(name))
				.unwrap_or_else(|| panic!("could not find {:?}", name));
			dependencies.insert((&cmd_info.pkg.repr, *dependency_name));
		}
	}

	let mut unused_dependencies = BTreeMap::new();
	for (id, dependency) in dependencies {
		if !used_dependencies.contains(&(id, dependency)) {
			unused_dependencies
				.entry(id)
				.or_insert_with(BTreeSet::new)
				.insert(dependency);
		}
	}
	if !unused_dependencies.values().all(BTreeSet::is_empty) {
		println!("unused dependencies: {:#?}", unused_dependencies);
		std::process::exit(1);
	} else {
		println!("All deps seem to have been used.");
	}
	Ok(())
}
