use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ffi::OsString;
use std::fmt::Write as _;
use std::io::{self, Write};
use std::ops::{Deref, Index, IndexMut};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::{env, fmt};

use cargo::core::compiler::{DefaultExecutor, Executor, RustcTargetData, Unit};
use cargo::core::manifest::Target;
use cargo::core::package_id::PackageId;
use cargo::core::resolver::features::{CliFeatures, ForceAllTargets};
use cargo::core::resolver::HasDevUnits;
use cargo::core::shell::Shell;
use cargo::core::{dependency, Package, Resolve, Verbosity, Workspace};
use cargo::ops::Packages;
use cargo::util::command_prelude::{ArgMatchesExt, CompileMode, ProfileChecking};
use cargo::util::context::GlobalContext;
use cargo::util::interning::InternedString;
use cargo::{CargoResult, CliError, CliResult};
use cargo_util::ProcessBuilder;
use clap::{ArgAction, ArgMatches, CommandFactory, Parser};
use nu_ansi_term::Color;
use serde::{Deserialize, Serialize};

pub fn run<I: IntoIterator<Item = OsString>, W: Write>(
    args: I,
    config: &mut GlobalContext,
    stdout: W,
) -> CliResult {
    let args = args.into_iter().collect::<Vec<_>>();
    let Opt::Udeps(opt) = Opt::try_parse_from(&args)?;
    let clap_matches = Opt::command().try_get_matches_from(args)?;
    match opt.run(
        config,
        stdout,
        clap_matches.subcommand_matches("udeps").unwrap(),
    )? {
        0 => Ok(()),
        code => Err(CliError::code(code)),
    }
}

#[derive(Parser, Debug)]
#[command(about, name = "cargo", bin_name = "cargo")]
enum Opt {
    #[command(
        about,
        version,
        name = "udeps",
        after_help(
            "\
If the `--package` argument is given, then SPEC is a package ID specification
which indicates which package should be built. If it is not given, then the
current package is built. For more information on SPEC and its format, see the
`cargo help pkgid` command.

All packages in the workspace are checked if the `--workspace` flag is supplied. The
`--workspace` flag is automatically assumed for a virtual manifest.
Note that `--exclude` has to be specified in conjunction with the `--workspace` flag.

Compilation can be configured via the use of profiles which are configured in
the manifest. The default profile for this command is `dev`, but passing
the `--release` flag will use the `release` profile instead.

The `--profile test` flag can be used to check unit tests with the
`#[cfg(test)]` attribute."
        )
    )]
    Udeps(OptUdeps),
}

#[derive(Parser, Debug)]
#[allow(dead_code)]
struct OptUdeps {
    #[arg(short, long, help("[cargo] No output printed to stdout"), value_parser = clap::value_parser!(bool))]
    quiet: bool,
    #[arg(
		short,
		long,
		value_name("SPEC"),
		num_args(1..),
		number_of_values(1),
		help("[cargo] Package(s) to check")
	)]
    package: Vec<String>,
    #[arg(long, help("[cargo] Alias for --workspace (deprecated)"), value_parser = clap::value_parser!(bool))]
    all: bool,
    #[arg(long, help("[cargo] Check all packages in the workspace"), value_parser = clap::value_parser!(bool))]
    workspace: bool,
    #[arg(
		long,
		value_name("SPEC"),
		num_args(1..),
		number_of_values(1),
		help("[cargo] Exclude packages from the check")
	)]
    exclude: Vec<String>,
    #[arg(
        short,
        long,
        value_name("N"),
        help("[cargo] Number of parallel jobs, defaults to # of CPUs")
    )]
    jobs: Option<String>,
    #[arg(long, help("[cargo] Check only this package's library"), value_parser = clap::value_parser!(bool))]
    lib: bool,
    #[arg(
		long,
		value_name("NAME"),
		num_args(0..),
		number_of_values(1),
		help("[cargo] Check only the specified binary")
	)]
    bin: Vec<String>,
    #[arg(long, help("[cargo] Check all binaries"), value_parser = clap::value_parser!(bool))]
    bins: bool,
    #[arg(
		long,
		value_name("NAME"),
		num_args(0..),
		number_of_values(1),
		help("[cargo] Check only the specified example")
	)]
    example: Vec<String>,
    #[arg(long, help("[cargo] Check all examples"), value_parser = clap::value_parser!(bool))]
    examples: bool,
    #[arg(
		long,
		value_name("NAME"),
		num_args(0..),
		number_of_values(1),
		help("[cargo] Check only the specified test target")
	)]
    test: Vec<String>,
    #[arg(long, help("[cargo] Check all tests"), value_parser = clap::value_parser!(bool))]
    tests: bool,
    #[arg(
		long,
		value_name("NAME"),
		num_args(0..),
		number_of_values(1),
		help("[cargo] Check only the specified bench target")
	)]
    bench: Vec<String>,
    #[arg(long, help("[cargo] Check all benches"), value_parser = clap::value_parser!(bool))]
    benches: bool,
    #[arg(long, help("[cargo] Check all targets"), id = "all-targets", value_parser = clap::value_parser!(bool))]
    all_targets: bool,
    #[arg(long, help("[cargo] Check artifacts in release mode, with optimizations"), value_parser = clap::value_parser!(bool))]
    release: bool,
    #[arg(
        long,
        value_name("PROFILE-NAME"),
        help("[cargo] Check artifacts with the specified profile")
    )]
    profile: Option<String>,
    #[arg(
		long,
		value_name("FEATURES"),
		num_args(0..),
		help("[cargo] Space-separated list of features to activate")
	)]
    features: Vec<String>,
    #[arg(long, help("[cargo] Activate all available features"), id = "all-features", value_parser = clap::value_parser!(bool))]
    all_features: bool,
    #[arg(long, help("[cargo] Do not activate the `default` feature"), id = "no-default-features", value_parser = clap::value_parser!(bool))]
    no_default_features: bool,
    #[arg(
        long,
        value_name("TRIPLE"),
        help("[cargo] Check for the target triple")
    )]
    target: Option<String>,
    #[arg(
        long,
        value_name("DIRECTORY"),
        help("[cargo] Directory for all generated artifacts")
    )]
    target_dir: Option<PathBuf>,
    #[arg(
        long,
        value_name("PATH"),
        id = "manifest-path",
        help("[cargo] Path to Cargo.toml")
    )]
    manifest_path: Option<String>,
    #[arg(
		long,
		value_name("FMT"),
		id = "message-format",
		ignore_case(true),
		value_parser(["human", "json", "short"]),
		default_value("human"),
		help("[cargo] Error format")
	)]
    message_format: Vec<String>,
    #[arg(
		short,
		long,
		action = ArgAction::Count,
		help("[cargo] Use verbose output (-vv very verbose/build.rs output)")
	)]
    verbose: u8,
    #[arg(
		long,
		value_name("WHEN"),
		ignore_case(false),
		value_parser(["auto", "always", "never"]),
		help("[cargo] Coloring")
	)]
    color: Option<String>,
    #[arg(long, help("[cargo] Require Cargo.lock and cache are up to date"), value_parser = clap::value_parser!(bool))]
    frozen: bool,
    #[arg(long, help("[cargo] Require Cargo.lock is up to date"), value_parser = clap::value_parser!(bool))]
    locked: bool,
    #[arg(long, help("[cargo] Run without accessing the network"), value_parser = clap::value_parser!(bool))]
    offline: bool,
    #[arg(
        long,
        value_name("OUTPUT"),
        default_value("human"),
        value_enum,
        help("Output format")
    )]
    output: OutputKind,
    #[arg(
        long,
        value_name("BACKEND"),
        default_value("depinfo"),
        value_enum,
        help("Backend to use for determining unused deps")
    )]
    backend: Backend,
    #[arg(
		long,
		id = "keep-going",
		help("Needed because the keep-going flag is asked about by cargo code"),
		value_parser = clap::value_parser!(bool),
	)]
    keep_going: bool,
}

impl OptUdeps {
    fn run<W: Write>(
        &self,
        config: &mut GlobalContext,
        stdout: W,
        clap_matches: &ArgMatches,
    ) -> CargoResult<i32> {
        if self.verbose > 0 {
            let mut shell = config.shell();
            shell.warn(
                "currently verbose command information (\"Running `..`\") are not correct.",
            )?;
            shell.warn("for example, `cargo-udeps` does these modifications:")?;
            shell.warn("- changes `$CARGO` to the value given from `cargo`")?;
        }

        config.configure(
            self.verbose.max(0).min(2) as u32,
            self.quiet,
            self.color.as_deref(),
            self.frozen,
            self.locked,
            self.offline,
            &self.target_dir,
            &["binary-dep-depinfo".to_string()],
            &[],
        )?;
        assert!(config.nightly_features_allowed);
        let ws = clap_matches.workspace(config)?;
        let test = match self.profile.as_deref() {
            None => false,
            Some("test") => true,
            Some(profile) => {
                return Err(anyhow::anyhow!(
                    "unknown profile: `{}`, only `test` is currently supported",
                    profile,
                ))
            }
        };
        let mode = CompileMode::Check { test };
        let pc = ProfileChecking::LegacyTestOnly;
        let compile_opts = clap_matches.compile_options(config, mode, Some(&ws), pc)?;
        let requested_kinds = &compile_opts.build_config.requested_kinds;
        let mut target_data = RustcTargetData::new(&ws, requested_kinds)?;

        let cli_features = CliFeatures::from_command_line(
            &self.features,
            self.all_features,
            !self.no_default_features,
        )?;
        let dry_run = false;
        let ws_resolve = cargo::ops::resolve_ws_with_opts(
            &ws,
            &mut target_data,
            requested_kinds,
            &cli_features,
            &Packages::All(Vec::new()).to_package_id_specs(&ws)?,
            HasDevUnits::Yes,
            ForceAllTargets::No,
            dry_run,
        )?;

        let packages = ws_resolve
            .pkg_set
            .get_many(ws_resolve.pkg_set.package_ids())?
            .into_iter()
            .map(|p| (p.package_id(), p))
            .collect::<HashMap<_, _>>();

        let dependency_names = ws
            .members()
            .map(|from| {
                let val = DependencyNames::new(
                    from,
                    &packages,
                    &ws_resolve.targeted_resolve,
                    &mut config.shell(),
                )?;
                let key = from.package_id();
                Ok((key, val))
            })
            .collect::<CargoResult<HashMap<_, _>>>()?;

        let data = Arc::new(Mutex::new(ExecData::new(&ws)?));
        let exec: Arc<dyn Executor + 'static> = Arc::new(Exec { data: data.clone() });
        cargo::ops::compile_with_exec(&ws, &compile_opts, &exec)?;
        let data = data.lock().unwrap();

        let mut used_normal_dev_dependencies = HashSet::new();
        let mut used_build_dependencies = HashSet::new();
        let mut normal_dependencies = dependency_names
            .iter()
            .flat_map(|(&m, d)| {
                d[dependency::DepKind::Normal]
                    .non_lib
                    .iter()
                    .map(move |&s| (m, s))
            })
            .collect::<HashSet<_>>();
        let mut dev_dependencies = dependency_names
            .iter()
            .flat_map(|(&m, d)| {
                d[dependency::DepKind::Development]
                    .non_lib
                    .iter()
                    .map(move |&s| (m, s))
            })
            .collect::<HashSet<_>>();
        let mut build_dependencies = dependency_names
            .iter()
            .flat_map(|(&m, d)| {
                d[dependency::DepKind::Build]
                    .non_lib
                    .iter()
                    .map(move |&s| (m, s))
            })
            .collect::<HashSet<_>>();

        let mut lib_stem_to_pkg_id = HashMap::new();
        for cmd_info in data.all_cmd_infos.iter() {
            let lib_stem = cmd_info.get_artifact_base_name();
            //println!("lib stem {} -> {}", lib_stem, cmd_info.pkg);
            lib_stem_to_pkg_id.insert(lib_stem, cmd_info.pkg);
        }
        enum BackendData {
            Depinfo(DepInfo),
        }
        for cmd_info in data.relevant_cmd_infos.iter() {
            let backend_data = match self.backend {
                Backend::Depinfo => {
                    let depinfo = cmd_info.get_depinfo(&mut config.shell())?;
                    BackendData::Depinfo(depinfo)
                }
            };
            // may not be workspace member
            if let Some(dependency_names) = dependency_names.get(&cmd_info.pkg) {
                let collect_names =
                    |dnv: &DependencyNamesValue,
                     used_dependencies: &mut HashSet<(PackageId, InternedString)>,
                     dependencies: &mut HashSet<(PackageId, InternedString)>| {
                        match &backend_data {
                            BackendData::Depinfo(depinfo) => {
                                for dep in depinfo.deps_of_depfile() {
                                    let fs = if let Some(fs) = dep.file_stem() {
                                        fs
                                    } else {
                                        continue;
                                    };
                                    let fs: String = match fs.to_str() {
                                        Some(v) => v.to_string(),
                                        _ => continue,
                                    };
                                    // The file names are like cratename-hash.rmeta or .rlib,
                                    // where "hash" is a hash string that cargo calls "metadata"
                                    // internally and computes in its "compute_metadata" function,
                                    // and cratename is the snakecased crate name.

                                    // First, we continue if there is no - in the filename.
                                    // it's likely a source file or some other artifact we aren't
                                    // interested in. This is obviously only a stupid heuristic.
                                    let lib_name = match fs.split_once('-') {
                                        None => continue,
                                        Some((lib_name, _)) => lib_name,
                                    };

                                    // The metadata hash is not available through cargo's api
                                    // outside of the Executor trait impl. We do our best to obtain
                                    // the hashes from that impl, but the executor is not called
                                    // for anything but crates that have to be recompiled.
                                    // Thus, any crates that weren't recompiled we don't know the
                                    // metadata hash of. So we perform a check: if we know the metadata
                                    // hash, we use it, otherwise we don't.
                                    // This gives a bit surprising behaviour when re-running
                                    // cargo-udeps but at least sometimes the results are more accurate.

                                    if let Some(pkg_id) = lib_stem_to_pkg_id.get(&fs) {
                                        if let Some(dependency_name) = dnv.by_package_id.get(pkg_id)
                                        {
                                            used_dependencies
                                                .insert((cmd_info.pkg, *dependency_name));
                                        }
                                    } else {
                                        // TODO this is a hack as we unconditionally strip the prefix.
                                        // It won't work for proc macro crates that start with "lib".
                                        // See maybe_lib in the code above.
                                        let lib_name =
                                            lib_name.strip_prefix("lib").unwrap_or(lib_name);
                                        if let Some(dependency_names) =
                                            dnv.by_lib_true_snakecased_name.get(lib_name)
                                        {
                                            for dependency_name in dependency_names {
                                                used_dependencies
                                                    .insert((cmd_info.pkg, *dependency_name));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        for extern_crate_name in &cmd_info.extern_crate_names {
                            // We ignore:
                            // 1. the `lib` that `bin`s, `example`s, and `test`s in the same `Package` depend on
                            // 2. crates bundled with `rustc` such as `proc-macro`
                            if let Some(dependency_name) =
                                dnv.by_extern_crate_name.get(&**extern_crate_name)
                            {
                                dependencies.insert((cmd_info.pkg, *dependency_name));
                            }
                        }
                    };

                collect_names(
                    &dependency_names.normal,
                    &mut used_normal_dev_dependencies,
                    &mut normal_dependencies,
                );
                collect_names(
                    &dependency_names.development,
                    &mut used_normal_dev_dependencies,
                    &mut dev_dependencies,
                );
                collect_names(
                    &dependency_names.build,
                    &mut used_build_dependencies,
                    &mut build_dependencies,
                );
            }
        }

        use anyhow::Context;
        let workspace_ignore = ws
            .custom_metadata()
            .map::<CargoResult<_>, _>(|workspace_metadata| {
                let PackageMetadata {
                    cargo_udeps: PackageMetadataCargoUdeps { ignore },
                } = workspace_metadata
                    .clone()
                    .try_into()
                    .context("could not parse `workspace.metadata.cargo-udeps`")?;
                Ok(ignore)
            })
            .transpose()?;

        let mut outcome = Outcome::default();

        let included_packages = compile_opts
            .spec
            .get_packages(&ws)?
            .iter()
            .map(|x| x.package_id())
            .collect::<HashSet<_>>();
        for (dependencies, used_dependencies, kind) in &[
            (
                &normal_dependencies,
                &used_normal_dev_dependencies,
                dependency::DepKind::Normal,
            ),
            (
                &dev_dependencies,
                &used_normal_dev_dependencies,
                dependency::DepKind::Development,
            ),
            (
                &build_dependencies,
                &used_build_dependencies,
                dependency::DepKind::Build,
            ),
        ] {
            for &(id, dependency) in *dependencies {
                // This package may have been explicitly excluded via flags.
                if !included_packages.contains(&id) {
                    continue;
                }

                let ignore = ws_resolve
                    .pkg_set
                    .get_one(id)?
                    .manifest()
                    .custom_metadata()
                    .map::<CargoResult<_>, _>(|package_metadata| {
                        let PackageMetadata {
                            cargo_udeps: PackageMetadataCargoUdeps { ignore },
                        } = package_metadata
                            .clone()
                            .try_into()
                            .context("could not parse `package.metadata.cargo-udeps`")?;
                        Ok(ignore)
                    })
                    .transpose()?;

                if !used_dependencies.contains(&(id, dependency)) {
                    if ignore.map_or(false, |ignore| ignore.contains(*kind, dependency))
                        || workspace_ignore
                            .as_ref()
                            .map_or(false, |ignore| ignore.contains(*kind, dependency))
                    {
                        config
                            .shell()
                            .info(format_args!("Ignoring `{}` ({:?})", dependency, kind))?;
                    } else {
                        outcome
                            .unused_deps
                            .entry(id)
                            .or_insert(OutcomeUnusedDeps::new(packages[&id].manifest_path())?)
                            .unused_deps_mut(*kind)
                            .insert(dependency);
                    }
                }
            }
        }

        outcome.success = outcome.unused_deps.values().all(
            |OutcomeUnusedDeps {
                 normal,
                 development,
                 build,
                 ..
             }| { normal.is_empty() && development.is_empty() && build.is_empty() },
        );

        if !outcome.success {
            let mut note = "".to_owned();

            if !self.all_targets {
                note += "Note: These dependencies might be used by other targets.\n";

                if !self.lib
                    && !self.bins
                    && !self.examples
                    && !self.tests
                    && !self.benches
                    && self.bin.is_empty()
                    && self.example.is_empty()
                    && self.test.is_empty()
                    && self.bench.is_empty()
                {
                    note += "      To find dependencies that are not used by any target, enable `--all-targets`.\n";
                }
            }

            if dependency_names.values().any(DependencyNames::has_non_lib) {
                note += "Note: Some dependencies are non-library packages.\n";
                note += "      `cargo-udeps` regards them as unused.\n";
            }

            note += "Note: They might be false-positive.\n";
            note += "      For example, `cargo-udeps` cannot detect usage of crates that are only used in doc-tests.\n";
            note += "      To ignore some dependencies, write `package.metadata.cargo-udeps.ignore` in Cargo.toml.\n";

            outcome.note = Some(note);
        }

        outcome.print(self.output, stdout)?;
        Ok(if outcome.success { 0 } else { 1 })
    }
}

struct ExecData {
    cargo_exe: OsString,
    supports_color: bool,
    workspace_members: Vec<PackageId>,
    relevant_cmd_infos: Vec<CmdInfo>,
    all_cmd_infos: Vec<CmdInfo>,
}

impl ExecData {
    fn new(ws: &Workspace<'_>) -> CargoResult<Self> {
        // `$CARGO` should be present when `cargo-udeps` is executed as `cargo udeps ..` or `cargo run -- udeps ..`.
        let cargo_exe = env::var_os(cargo::CARGO_ENV)
            .map(Ok::<_, anyhow::Error>)
            .unwrap_or_else(|| {
                // Unless otherwise specified, `$CARGO` is set to `config.cargo_exe()` for compilation commands which points at `cargo-udeps`.
                let cargo_exe = ws.gctx().cargo_exe()?;
                ws.gctx().shell().warn(format!(
                    "Couldn't find $CARGO environment variable. Setting it to {}",
                    cargo_exe.display(),
                ))?;
                ws.gctx().shell().warn(
                    "`cargo-udeps` currently does not support basic Cargo commands such as `build`",
                )?;
                Ok(cargo_exe.into())
            })?;
        Ok(Self {
            cargo_exe,
            supports_color: ws.gctx().shell().err_supports_color(),
            workspace_members: ws.members().map(Package::package_id).collect(),
            relevant_cmd_infos: Vec::new(),
            all_cmd_infos: Vec::new(),
        })
    }
}

struct Exec {
    data: Arc<Mutex<ExecData>>,
}

impl Executor for Exec {
    fn exec(
        &self,
        cmd: &ProcessBuilder,
        id: PackageId,
        target: &Target,
        mode: CompileMode,
        on_stdout_line: &mut dyn FnMut(&str) -> CargoResult<()>,
        on_stderr_line: &mut dyn FnMut(&str) -> CargoResult<()>,
    ) -> CargoResult<()> {
        let cmd_info = cmd_info(id, target.is_custom_build(), cmd).unwrap_or_else(|e| {
            panic!("Couldn't obtain crate info {:?}: {:?}", id, e);
        });

        let mut cmd = cmd.clone();

        let is_path = id.source_id().is_path();
        let is_workspace_member;

        {
            // TODO unwrap used
            let mut bt = self.data.lock().unwrap();

            is_workspace_member = bt.workspace_members.contains(&id);

            bt.all_cmd_infos.push(cmd_info.clone());

            // If the crate is not a in the workspace,
            // we are not interested in its information.
            if is_workspace_member {
                bt.relevant_cmd_infos.push(cmd_info.clone());
            }
            assert!(
                !(!is_path && is_workspace_member),
                "`{}` is a workspace member but is not from a filesystem path",
                id,
            );
            if (!cmd_info.cap_lints_allow) != is_path {
                on_stderr_line(&format!(
                    "{} (!cap_lints_allow)={} differs from is_path={} for id={}",
                    if bt.supports_color {
                        Color::Yellow.bold().paint("warning:").to_string()
                    } else {
                        "warning:".to_owned()
                    },
                    !cmd_info.cap_lints_allow,
                    is_path,
                    id,
                ))?;
            }
            cmd.env(cargo::CARGO_ENV, &bt.cargo_exe);
        }
        if is_workspace_member {
            // This reduces the save analysis files that are being created a little
            std::env::set_var(
                "RUST_SAVE_ANALYSIS_CONFIG",
                r#"{ "reachable_only": false, "full_docs": false, "pub_only": false, "distro_crate": false, "signatures": false, "borrow_data": false }"#,
            );
        }
        DefaultExecutor.exec(&cmd, id, target, mode, on_stdout_line, on_stderr_line)?;
        Ok(())
    }
    fn force_rebuild(&self, unit: &Unit) -> bool {
        let bt = self.data.lock().unwrap();
        bt.workspace_members.contains(&unit.pkg.package_id())
    }
}

#[derive(Clone, Debug)]
struct CmdInfo {
    pkg: PackageId,
    #[allow(dead_code)]
    custom_build: bool,
    crate_name: String,
    crate_type: String,
    extra_filename: String,
    cap_lints_allow: bool,
    out_dir: String,
    extern_crate_names: HashSet<String>,
}

impl CmdInfo {
    fn get_artifact_base_name(&self) -> String {
        let maybe_lib = if self.crate_type.ends_with("lib") || self.crate_type == "proc-macro" {
            "lib"
        } else {
            ""
        };
        maybe_lib.to_owned() + &self.crate_name + &self.extra_filename
    }
    fn get_depinfo_filename(&self) -> String {
        self.crate_name.clone() + &self.extra_filename + ".d"
    }
    fn get_depinfo_path(&self) -> PathBuf {
        Path::new(&self.out_dir).join(self.get_depinfo_filename())
    }
    fn get_depinfo(&self, shell: &mut Shell) -> CargoResult<DepInfo> {
        let p = self.get_depinfo_path();
        shell.info(format_args!("Loading depinfo from {:?}", p))?;
        let di = parse_rustc_dep_info(&p)?;
        let di = di
            .iter()
            .map(|(v, w)| {
                let w = w.iter().map(PathBuf::from).collect::<Vec<_>>();
                (PathBuf::from(v), w)
            })
            .collect::<Vec<_>>();
        Ok(DepInfo {
            di,
            f_name: self.get_depinfo_filename(),
        })
    }
}

struct DepInfo {
    di: Vec<(PathBuf, Vec<PathBuf>)>,
    f_name: String,
}

impl DepInfo {
    fn deps_of_depfile(&self) -> Vec<PathBuf> {
        self.di
            .iter()
            .find(|(v, _w)| v.file_name() == Some(&std::ffi::OsString::from(&self.f_name)))
            .map(|v| v.1.clone())
            .unwrap_or_default()
    }
}

// Bases on function with same name from cargo source src/cargo/core/compiler/fingerprint.rs
/// Parse the `.d` dep-info file generated by rustc.
///
/// Result is a Vec of `(target, prerequisites)` tuples where `target` is the
/// rule name, and `prerequisites` is a list of files that it depends on.
fn parse_rustc_dep_info(rustc_dep_info: &Path) -> CargoResult<Vec<(String, Vec<String>)>> {
    let contents = std::fs::read_to_string(rustc_dep_info)?;
    contents
        .lines()
        .filter_map(|l| l.find(": ").map(|i| (l, i)))
        .map(|(line, pos)| {
            let target = &line[..pos];
            let mut deps = line[pos + 2..].split_whitespace();

            let mut ret = Vec::new();
            while let Some(s) = deps.next() {
                let mut file = s.to_string();
                while file.ends_with('\\') {
                    file.pop();
                    file.push(' ');
                    file.push_str(deps.next().ok_or_else(|| {
                        anyhow::anyhow!("malformed dep-info format, trailing \\".to_string())
                    })?);
                }
                ret.push(file);
            }
            Ok((target.to_string(), ret))
        })
        .collect()
}

fn cmd_info(id: PackageId, custom_build: bool, cmd: &ProcessBuilder) -> CargoResult<CmdInfo> {
    let mut args_iter = cmd.get_args();
    let mut crate_name = None;
    let mut crate_type = None;
    let mut extra_filename = None;
    let mut cap_lints_allow = false;
    let mut out_dir = None;
    let mut extern_crate_names = HashSet::new();
    while let Some(v) = args_iter.next() {
        if v == "--extern" {
            if let Some(arg) = args_iter.next() {
                let splitter = arg
                    .to_str()
                    .expect("non-utf8 paths not supported atm")
                    .split('=')
                    .collect::<Vec<_>>();
                match *splitter {
                    [name] | [name, _] => extern_crate_names.insert(name.to_owned()),
                    _ => panic!("invalid format for extern arg: {:?}", arg),
                };
            }
        } else if v == "--crate-name" {
            if let Some(name) = args_iter.next() {
                crate_name = Some(
                    name.to_str()
                        .expect("non-utf8 crate names not supported")
                        .to_owned(),
                );
            }
        } else if v == "--crate-type" {
            if let Some(ty) = args_iter.next() {
                crate_type = Some(
                    ty.to_str()
                        .expect("non-utf8 crate names not supported")
                        .to_owned(),
                );
            }
        } else if v == "--cap-lints" {
            if let Some(c) = args_iter.next() {
                if c == "allow" {
                    cap_lints_allow = true;
                }
            }
        } else if v == "--out-dir" {
            if let Some(d) = args_iter.next() {
                out_dir = Some(
                    d.to_str()
                        .expect("non-utf8 crate names not supported")
                        .to_owned(),
                );
            }
        } else if v == "-C" {
            if let Some(arg) = args_iter.next() {
                let arg = arg.to_str().expect("non-utf8 args not supported atm");
                let mut splitter = arg.split('=');
                if let (Some(n), Some(p)) = (splitter.next(), splitter.next()) {
                    if n == "extra-filename" {
                        extra_filename = Some(p.to_owned());
                    }
                }
            }
        }
    }
    let pkg = id;
    let crate_name = crate_name.ok_or_else(|| anyhow::anyhow!("crate name needed"))?;
    let crate_type = crate_type.unwrap_or_else(|| "bin".to_owned());
    let extra_filename = extra_filename.ok_or_else(|| anyhow::anyhow!("extra-filename needed"))?;
    let out_dir = out_dir.ok_or_else(|| anyhow::anyhow!("outdir needed"))?;

    Ok(CmdInfo {
        pkg,
        custom_build,
        crate_name,
        crate_type,
        extra_filename,
        cap_lints_allow,
        out_dir,
        extern_crate_names,
    })
}

#[derive(Debug, Default)]
struct DependencyNames {
    normal: DependencyNamesValue,
    development: DependencyNamesValue,
    build: DependencyNamesValue,
}

impl DependencyNames {
    fn new(
        from: &Package,
        packages: &HashMap<PackageId, &Package>,
        resolve: &Resolve,
        shell: &mut Shell,
    ) -> CargoResult<Self> {
        let mut this = Self::default();

        let from = from.package_id();

        for (to_pkg, deps) in resolve.deps(from) {
            let to_pkg = packages
                .get(&to_pkg)
                .unwrap_or_else(|| panic!("could not find `{}`", to_pkg));

            // Not all dependencies contain `lib` targets as it is OK to append non-library packages to `Cargo.toml`.
            // Their `bin` targets can be built with `cargo build --bins -p <SPEC>` and are available in build scripts.
            if let Some(to_lib) = to_pkg.targets().iter().find(|t| t.is_lib()) {
                let extern_crate_name = resolve
                    .extern_crate_name_and_dep_name(from, to_pkg.package_id(), to_lib)?
                    .0
                    .as_str();
                let lib_true_snakecased_name = to_lib.crate_name();

                for dep in deps {
                    assert_eq!(dep.package_name(), to_pkg.name());
                    let names = &mut this[dep.kind()];
                    names
                        .by_extern_crate_name
                        .insert(extern_crate_name, dep.name_in_toml());
                    if let Some(pkg) = names
                        .by_package_id
                        .insert(to_pkg.package_id(), dep.name_in_toml())
                    {
                        shell.warn(format!(
                            "duplicate package mentioned in toml {}. {pkg}",
                            to_pkg.package_id()
                        ))?;
                    }

                    // Two `Dependenc`ies with the same name point at the same `Package`.
                    names
                        .by_lib_true_snakecased_name
                        .entry(lib_true_snakecased_name.clone())
                        .or_insert_with(HashSet::new)
                        .insert(dep.name_in_toml());
                }
            } else {
                for dep in deps {
                    this[dep.kind()].non_lib.insert(dep.name_in_toml());
                }
            }
        }

        let ambiguous_names = |kinds: &[dependency::DepKind]| -> BTreeMap<_, _> {
            kinds
                .iter()
                .flat_map(|&k| &this[k].by_lib_true_snakecased_name)
                .filter(|(_, v)| v.len() > 1)
                .flat_map(|(k, v)| v.iter().map(move |&v| (v, k.deref())))
                .collect()
        };

        let ambiguous_normal_dev = ambiguous_names(&[
            dependency::DepKind::Normal,
            dependency::DepKind::Development,
        ]);
        let ambiguous_build = ambiguous_names(&[dependency::DepKind::Build]);

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

    fn has_non_lib(&self) -> bool {
        [
            dependency::DepKind::Normal,
            dependency::DepKind::Development,
            dependency::DepKind::Build,
        ]
        .iter()
        .any(|&k| !self[k].non_lib.is_empty())
    }
}

impl Index<dependency::DepKind> for DependencyNames {
    type Output = DependencyNamesValue;

    fn index(&self, index: dependency::DepKind) -> &DependencyNamesValue {
        match index {
            dependency::DepKind::Normal => &self.normal,
            dependency::DepKind::Development => &self.development,
            dependency::DepKind::Build => &self.build,
        }
    }
}

impl IndexMut<dependency::DepKind> for DependencyNames {
    fn index_mut(&mut self, index: dependency::DepKind) -> &mut DependencyNamesValue {
        match index {
            dependency::DepKind::Normal => &mut self.normal,
            dependency::DepKind::Development => &mut self.development,
            dependency::DepKind::Build => &mut self.build,
        }
    }
}

#[derive(Debug, Default)]
struct DependencyNamesValue {
    by_extern_crate_name: HashMap<&'static str, InternedString>,
    by_lib_true_snakecased_name: HashMap<String, HashSet<InternedString>>,
    by_package_id: HashMap<PackageId, InternedString>,
    non_lib: HashSet<InternedString>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct PackageMetadata {
    #[serde(default)]
    cargo_udeps: PackageMetadataCargoUdeps,
}

#[derive(Debug, Default, Deserialize)]
struct PackageMetadataCargoUdeps {
    #[serde(default)]
    ignore: PackageMetadataCargoUdepsIgnore,
}

#[derive(Debug, Default, Deserialize)]
struct PackageMetadataCargoUdepsIgnore {
    #[serde(default)]
    normal: HashSet<String>,
    #[serde(default)]
    development: HashSet<String>,
    #[serde(default)]
    build: HashSet<String>,
}

impl PackageMetadataCargoUdepsIgnore {
    fn contains(&self, kind: dependency::DepKind, name_in_toml: InternedString) -> bool {
        match kind {
            dependency::DepKind::Normal => &self.normal,
            dependency::DepKind::Development => &self.development,
            dependency::DepKind::Build => &self.build,
        }
        .contains(&*name_in_toml)
    }
}

#[derive(Default, Debug, Serialize)]
struct Outcome {
    success: bool,
    unused_deps: BTreeMap<PackageId, OutcomeUnusedDeps>,
    note: Option<String>,
}

impl Outcome {
    fn print(&self, output: OutputKind, stdout: impl Write) -> io::Result<()> {
        match output {
            OutputKind::Human => self.print_human(stdout),
            OutputKind::Json => self.print_json(stdout),
        }
    }

    fn print_human(&self, mut stdout: impl Write) -> io::Result<()> {
        if self.success {
            writeln!(stdout, "All deps seem to have been used.")?;
        } else {
            writeln!(stdout, "unused dependencies:")?;

            for (
                member,
                OutcomeUnusedDeps {
                    normal,
                    development,
                    build,
                    ..
                },
            ) in &self.unused_deps
            {
                fn edge_and_joint(p: bool) -> (char, char) {
                    if p {
                        (' ', '└')
                    } else {
                        ('│', '├')
                    }
                }

                writeln!(stdout, "`{}`", member)?;

                for (deps, (edge, joint), prefix) in &[
                    (
                        normal,
                        edge_and_joint(development.is_empty() && build.is_empty()),
                        "",
                    ),
                    (development, edge_and_joint(build.is_empty()), "dev-"),
                    (build, (' ', '└'), "build-"),
                ] {
                    if !deps.is_empty() {
                        writeln!(stdout, "{}─── {}dependencies", joint, prefix)?;
                        let mut deps = deps.iter().peekable();
                        while let Some(dep) = deps.next() {
                            let joint = if deps.peek().is_some() { '├' } else { '└' };
                            writeln!(stdout, "{}    {}─── {:?}", edge, joint, dep)?;
                        }
                    }
                }
            }

            if let Some(note) = &self.note {
                write!(stdout, "{}", note)?;
            }
        }
        stdout.flush()
    }

    fn print_json(&self, mut stdout: impl Write) -> io::Result<()> {
        let json = serde_json::to_string(self).expect("should not fail");
        writeln!(stdout, "{}", json)?;
        stdout.flush()
    }
}

#[derive(Debug, Serialize)]
struct OutcomeUnusedDeps {
    manifest_path: String,
    normal: BTreeSet<InternedString>,
    development: BTreeSet<InternedString>,
    build: BTreeSet<InternedString>,
}

impl OutcomeUnusedDeps {
    fn new(manifest_path: &Path) -> CargoResult<Self> {
        let manifest_path = manifest_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("{:?} is not valid utf-8", manifest_path))?
            .to_owned();

        Ok(Self {
            manifest_path,
            normal: BTreeSet::new(),
            development: BTreeSet::new(),
            build: BTreeSet::new(),
        })
    }

    fn unused_deps_mut(&mut self, kind: dependency::DepKind) -> &mut BTreeSet<InternedString> {
        match kind {
            dependency::DepKind::Normal => &mut self.normal,
            dependency::DepKind::Development => &mut self.development,
            dependency::DepKind::Build => &mut self.build,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum OutputKind {
    Human,
    Json,
}

impl FromStr for OutputKind {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, &'static str> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            _ => Err(r#"expected "human" or "json" (you should not see this message)"#),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum Backend {
    Depinfo,
}

impl FromStr for Backend {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, &'static str> {
        match s {
            "depinfo" => Ok(Self::Depinfo),
            _ => Err(r#"expected "depinfo" (you should not see this message)"#),
        }
    }
}

trait ShellExt {
    fn info<T: fmt::Display>(&mut self, message: T) -> CargoResult<()>;
}

impl ShellExt for Shell {
    fn info<T: fmt::Display>(&mut self, message: T) -> CargoResult<()> {
        match self.verbosity() {
            Verbosity::Quiet => Ok(()),
            _ => self.print_ansi_stderr(
                format!(
                    "{} {}\n",
                    if self.err_supports_color() {
                        Color::Cyan.bold().paint("info:").to_string()
                    } else {
                        "info:".to_owned()
                    },
                    message,
                )
                .as_ref(),
            ),
        }
    }
}
