extern crate ansi_term;
extern crate cargo;
extern crate serde;
extern crate serde_json;
extern crate which;

mod defs;

use std::fmt::{Display, Write as _};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Mutex;
use std::ops::Deref as _;
use defs::CrateSaveAnalysis;
use ansi_term::Colour;
use cargo::core::shell::Shell;
use cargo::core::compiler::{Executor, DefaultExecutor, Unit};
use cargo::util::process_builder::ProcessBuilder;
use cargo::core::package_id::PackageId;
use cargo::core::manifest::Target;
use cargo::util::errors::CargoResult;
use cargo::core::shell::Verbosity;
use cargo::util::command_prelude::{App, Arg, opt, ArgMatchesExt,
	AppExt, CompileMode, Config};
use cargo::core::{InternedString, Package, Resolve};
use cargo::ops::Packages;

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

pub struct StrErr(String);

impl<T :Display> From<T> for StrErr {
	fn from(v :T) -> Self {
		StrErr(format!("{}", v))
	}
}

impl std::fmt::Debug for StrErr {
	fn fmt(&self, f :&mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		// Difference of this debug impl to the one provided by the derive macro
		// is that special chars like newlines and " aren't escaped.
		// We have some human-readable errors where newlines help with the output.
		write!(f, "StrErr(\"{}\")", self.0)
	}
}

struct ExecData {
	cargo_exe :Option<String>,
	supports_color :bool,
	relevant_cmd_infos :Vec<CmdInfo>,
}

impl ExecData {
	fn new(supports_color :bool) -> Self {
		let cargo_exe = which::which("cargo")
			.map_err(|e| println!("warning: couldn't load cargo executable file: {:?}", e))
			.ok()
			.and_then(|p| p.to_str().map(str::to_owned));
		Self {
			cargo_exe,
			supports_color,
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

		let cmd_info = cmd_info(id, target.is_custom_build(), &cmd).unwrap_or_else(|e| {
			panic!("Couldn't obtain crate info {:?}: {:?}", id, e);
		});
		let is_path = id.source_id().is_path();
		{
			// TODO unwrap used
			let mut bt = self.data.lock().unwrap();

			// If the crate is not a library crate,
			// we are not interested in its information.
			if is_path {
				bt.relevant_cmd_infos.push(cmd_info.clone());
			}
			if (!cmd_info.cap_lints_allow) != is_path {
				on_stderr_line(&format!(
					"{} (!cap_lints_allow)={} differs from is_path={} for id={}",
					if bt.supports_color {
						Colour::Yellow.bold().paint("warning:").to_string()
					} else {
						"warning:".to_owned()
					},
					!cmd_info.cap_lints_allow,
					is_path,
					id,
				))?;
			}
			if let Some(cargo_exe) = &bt.cargo_exe {
				cmd.env(cargo::CARGO_ENV, cargo_exe);
			}
		}
		if is_path {
			std::env::set_var("RUST_SAVE_ANALYSIS_CONFIG",
				r#"{ "reachable_only": true, "full_docs": false, "pub_only": false, "distro_crate": false, "signatures": false, "borrow_data": false }"#);
			cmd.arg("-Z").arg("save-analysis");
		}
		DefaultExecutor.exec(cmd, id, target, mode, on_stderr_line, on_stdout_line)?;
		Ok(())
	}
	fn force_rebuild(&self, unit :&Unit) -> bool {
		let source_id = (*unit).pkg.summary().source_id();
		source_id.is_path()
	}
}

#[derive(Clone, Debug)]
struct CmdInfo {
	pkg :PackageId,
	custom_build :bool,
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
	fn get_save_analysis(&self, shell :&mut Shell) -> Result<CrateSaveAnalysis, StrErr> {
		let p = self.get_save_analysis_path();
		shell.print_ansi(
			format!(
				"{} Loading save analysis from {:?}\n",
				if shell.supports_color() {
					Colour::Cyan.bold().paint("info:").to_string()
				} else {
					"info:".to_owned()
				},
				p,
			)
			.as_ref(),
		)?;
		let f = std::fs::read_to_string(p)?;
		let res = serde_json::from_str(&f)?;
		Ok(res)
	}
}

fn cmd_info(id :PackageId, custom_build :bool, cmd :&ProcessBuilder) -> Result<CmdInfo, StrErr> {
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
	let pkg = id;
	let crate_name = crate_name.ok_or("crate name needed")?;
	let crate_type = crate_type.unwrap_or("bin".to_owned());
	let extra_filename = extra_filename.ok_or("extra-filename needed")?;
	let out_dir = out_dir.ok_or("outdir needed")?;

	Ok(CmdInfo {
		pkg,
		custom_build,
		crate_name,
		crate_type,
		extra_filename,
		cap_lints_allow,
		out_dir,
		externs,
	})
}

#[derive(Debug, Default)]
struct DependencyNames {
	normal_dev_by_extern_crate_name :HashMap<String, InternedString>,
	normal_dev_by_lib_true_snakecased_name :HashMap<String, HashSet<InternedString>>,
	build_by_extern_crate_name :HashMap<String, InternedString>,
	build_by_lib_true_snakecased_name :HashMap<String, HashSet<InternedString>>,
}

impl DependencyNames {
	fn new(
		from :&Package,
		packages :&HashMap<PackageId, &Package>,
		resolve :&Resolve,
		shell :&mut Shell,
	) -> CargoResult<Self> {
		fn ambiguous_names(
			names :&HashMap<String, HashSet<InternedString>>,
		) -> BTreeMap<InternedString, &str> {
			names
				.iter()
				.filter(|(_, v)| v.len() > 1)
				.flat_map(|(k, v)| v.iter().map(move |&v| (v, k.deref())))
				.collect()
		}

		let mut this = Self::default();

		if let Some(lib) = from.targets().iter().find(|t| t.is_lib()) {
			let name = resolve.extern_crate_name(from.package_id(), from.package_id(), lib)?;
			this.normal_dev_by_extern_crate_name.insert(name.clone(), from.name());
			this.normal_dev_by_lib_true_snakecased_name
				.entry(name.clone())
				.or_insert_with(HashSet::new)
				.insert(from.name());
		}

		let from = from.package_id();

		for (to_pkg, deps) in resolve.deps(from) {
			let to_lib = packages
				.get(&to_pkg)
				.unwrap_or_else(|| panic!("could not find `{}`", &to_pkg))
				.targets()
				.iter()
				.find(|t| t.is_lib())
				.unwrap_or_else(|| panic!("`{}` does not have any `lib` target", to_pkg));

			let extern_crate_name = resolve.extern_crate_name(from, to_pkg, to_lib)?;
			let lib_true_snakecased_name = to_lib.name().replace('-', "_");

			for dep in deps {
				let (by_extern_crate_name, by_lib_true_snakecased_name) = if dep.is_build() {
					(
						&mut this.build_by_extern_crate_name,
						&mut this.build_by_lib_true_snakecased_name,
					)
				} else {
					(
						&mut this.normal_dev_by_extern_crate_name,
						&mut this.normal_dev_by_lib_true_snakecased_name,
					)
				};

				by_extern_crate_name.insert(extern_crate_name.clone(), dep.name_in_toml());

				// Two `Dependenc`ies with the same name point at the same `Package`.
				by_lib_true_snakecased_name
					.entry(lib_true_snakecased_name.clone())
					.or_insert_with(HashSet::new)
					.insert(dep.name_in_toml());
			}
		}

		let ambiguous_normal_dev = ambiguous_names(&this.normal_dev_by_lib_true_snakecased_name);
		let ambiguous_build = ambiguous_names(&this.build_by_lib_true_snakecased_name);

		if !(ambiguous_normal_dev.is_empty() && ambiguous_build.is_empty()) {
			let mut msg = format!(
				"Currently `cargo-udeps` cannot distinguish multiple crates with the same `lib` name. This may cause false negative\n\
				 `{}`\n",
				from,
			);
			let (edge, joint) = if ambiguous_build.is_empty() {
				(' ', '└')
			} else {
				('│', '├')
			};
			for (ambiguous, edge, joint, prefix) in &[
				(ambiguous_normal_dev, edge, joint, "(dev-)"),
				(ambiguous_build, ' ', '└', "build-"),
			] {
				if !ambiguous.is_empty() {
					writeln!(msg, "{}─── {}dependencies", joint, prefix).unwrap();
					let mut ambiguous = ambiguous.iter().peekable();
					while let Some((dep, lib)) = ambiguous.next() {
						let joint = if ambiguous.peek().is_some() {
							'├'
						} else {
							'└'
						};
						writeln!(msg, "{}    {}─── {:?} → {:?}", edge, joint, dep, lib).unwrap();
					}
				}
			}
			shell.warn(msg.trim_end())?;
		}

		Ok(this)
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

	let (packages, resolve) = cargo::ops::resolve_ws_precisely(
		&ws,
		&args
			.values_of("features")
			.map(|vs| vs.map(ToOwned::to_owned).collect::<Vec<_>>())
			.unwrap_or_default(),
		args.is_present("all-features"),
		args.is_present("no-default-features"),
		&Packages::All.to_package_id_specs(&ws)?,
	)?;
	let packages = packages
		.get_many(packages.package_ids())?
		.into_iter()
		.map(|p| (p.package_id(), p))
		.collect::<HashMap<_, _>>();

	let dependency_names = ws
		.members()
		.map(|from| {
             let val = DependencyNames::new(from, &packages, &resolve, &mut ws.config().shell())?;
			 let key = from.package_id();
			 Ok((key, val))
		})
		.collect::<CargoResult<HashMap<_, _>>>()?;

	let data = Arc::new(Mutex::new(ExecData::new(ws.config().shell().supports_color())));
	let exec :Arc<dyn Executor + 'static> = Arc::new(Exec { data : data.clone() });
	cargo::ops::compile_with_exec(&ws, &compile_opts, &exec)?;
	let data = data.lock()?;

	let mut used_normal_dev_dependencies = HashSet::new();
	let mut used_build_dependencies = HashSet::new();
	let mut normal_dev_dependencies = HashSet::new();
	let mut build_dependencies = HashSet::new();

	for cmd_info in data.relevant_cmd_infos.iter() {
		let analysis = cmd_info.get_save_analysis(&mut ws.config().shell())?;
		// may not be workspace member
		if let Some(dependency_names) = dependency_names.get(&cmd_info.pkg) {
			let (
				by_extern_crate_name,
				by_lib_true_snakecased_name,
				used_dependencies,
				dependencies
			) = if cmd_info.custom_build {
				(
					&dependency_names.build_by_extern_crate_name,
					&dependency_names.build_by_lib_true_snakecased_name,
					&mut used_build_dependencies,
					&mut build_dependencies,
				)
			} else {
				(
					&dependency_names.normal_dev_by_extern_crate_name,
					&dependency_names.normal_dev_by_lib_true_snakecased_name,
					&mut used_normal_dev_dependencies,
					&mut normal_dev_dependencies,
				)
			};
			for ext in &analysis.prelude.external_crates {
				if let Some(dependency_names) = by_lib_true_snakecased_name.get(&ext.id.name) {
					for dependency_name in dependency_names {
						used_dependencies.insert((cmd_info.pkg, *dependency_name));
					}
				}
			}
			for (name, _) in &cmd_info.externs {
				let dependency_name = by_extern_crate_name
					.get(name)
					.unwrap_or_else(|| panic!("could not find {:?}", name));
				dependencies.insert((cmd_info.pkg, *dependency_name));
			}
		}
	}

	let mut unused_dependencies = HashMap::new();
	for (dependencies, used_dependencies, custom_build) in &[
		(&normal_dev_dependencies, &used_normal_dev_dependencies, false),
		(&build_dependencies, &used_build_dependencies, true),
	] {
		for (id, dependency) in *dependencies {
			if !used_dependencies.contains(&(*id, *dependency)) {
				let (normal_dev, build) = unused_dependencies
					.entry(id)
					.or_insert_with(|| (BTreeSet::new(), BTreeSet::new()));
				if *custom_build {
					build.insert(dependency);
				} else {
					normal_dev.insert(dependency);
				}
			}
		}
	}
	if !unused_dependencies.values().all(|(ps1, ps2)| ps1.is_empty() && ps2.is_empty()) {
		println!("unused dependencies:");
		for (member, (normal_dev_dependencies, build_dependencies)) in unused_dependencies {
			println!("`{}`", member);
			let (edge, joint) = if build_dependencies.is_empty() {
				(' ', '└')
			} else {
				('│', '├')
			};
			for (dependencies, edge, joint, prefix) in &[
				(normal_dev_dependencies, edge, joint, "(dev-)"),
				(build_dependencies, ' ', '└', "build-"),
			] {
				if !dependencies.is_empty() {
					println!("{}─── {}dependencies", joint, prefix);
					let mut dependencies = dependencies.iter().peekable();
					while let Some(dependency) = dependencies.next() {
						let joint = if dependencies.peek().is_some() {
							'├'
						} else {
							'└'
						};
						println!("{}    {}─── {:?}", edge, joint, dependency);
					}
				}
			}
		}
		std::process::exit(1);
	} else {
		println!("All deps seem to have been used.");
	}
	Ok(())
}
