# Changelog

## Release 0.1.58 - September 15, 2025

- Update `cargo` dependency to 0.90

## Release 0.1.57 - July 6, 2025

- Update `cargo` dependency to 0.89

## Release 0.1.56 - June 4, 2025

- Update `cargo` dependency to 0.88

## Release 0.1.55 - February 20, 2025

- Update `cargo` dependency to 0.86

## Release 0.1.54 - January 9, 2025

- Update `cargo` dependency to 0.85

## Release 0.1.53 - November 30, 2024

- Update `cargo` dependency to 0.84

## Release 0.1.52 - October 27, 2024

- Update `cargo` dependency to 0.83

## Release 0.1.51 - September 06, 2024

- Update `cargo` dependency to 0.82

## Release 0.1.50 - July 29, 2024

- Update `cargo` dependency to 0.81

## Release 0.1.49 - July 1, 2024

- Update `cargo` dependency to 0.80

## Release 0.1.48 - May 5, 2024

- Update `cargo` dependency to 0.78

## Release 0.1.47 - March 10, 2024

- Updated lockfile release

## Release 0.1.46 - February 10, 2024

- Update `cargo` dependency to 0.77

## Release 0.1.45 - December 29, 2023

- Update `cargo` dependency to 0.76
- Support ignoring dependencies across entire workspace. Contributed by [@cmtm](https://github.com/cmtm).

## Release 0.1.44 - November 23, 2023

- Update `cargo` dependency to 0.75.1

## Release 0.1.43 - October 17, 2023

- Update `cargo` dependency to 0.74.0

## Release 0.1.42 - August 27, 2023

- Update `cargo` dependency to 0.73.1

## Release 0.1.41 - July 20, 2023

- Update `cargo` dependency to 0.72.0
- Update `nu-ansy-term` dependency to 0.48.0

## Release 0.1.40 - June 4, 2023

- Update cargo dependency to 0.71.0

## Release 0.1.39 - May 6, 2023

- Build release artifacts for more platforms. Contributed by [@NobodyXu](https://github.com/NobodyXu)
- Build release artifacts in `ubuntu-20.04` for better compatibility. Contributed by [@NobodyXu](https://github.com/NobodyXu)
- Replace `ansi-term` with `nu-ansi-term`. Contributed by [@tottoto](https://github.com/tottoto)

## Release 0.1.38 - May 2, 2023

- Fix of the fix of the continuous deployment asset inclusion.

## Release 0.1.37 - May 2, 2023

- Fix `--manifest-path` CLI argument not working. Contributed by [@ithinuel](https://github.com/ithinuel)
- Fix CI to include assets upon release. Contributed by [@AtkinsChang](https://github.com/AtkinsChang)

## Release 0.1.36 - April 29, 2023

- Update cargo dependency to 0.70.0
- Update clap to 0.4.0
- Respect shell verbosity
- Make `--exclude` also exclude packages from results
- CI modernizations

## Release 0.1.35 - November 3, 2022

- Update cargo dependency to 0.66.0

## Release 0.1.34 - September 22, 2022

- Update cargo dependency to 0.65.0

## Release 0.1.33 - September 15, 2022

- Change default backend from `save-analysis` to `depinfo` in the wake of [save-analysis removal](https://github.com/rust-lang/rust/pull/101841).

## Release 0.1.32 - August 22, 2022

- Fix [type casting issue](https://github.com/est31/cargo-udeps/issues/135) introduced by the cargo update that broke many command line flags.

## Release 0.1.31 - August 22, 2022

- Update cargo dependency to 0.64.0
- A new backend flag that supports transitive dependencies but ignores macro usages. Contributed by [@artslob](https://github.com/artslob)

## Release 0.1.30 - July 06, 2022

- Update cargo dependency to 0.63.0

## Release 0.1.29 - May 20, 2022

- Update cargo dependency to 0.62.0

## Release 0.1.28 - April 13, 2022

- Update cargo dependency to 0.61.0

## Release 0.1.27 - February 25, 2022

- Update cargo dependency to 0.60.0

## Release 0.1.26 - January 20, 2022

- Update cargo dependency to 0.59.0

## Release 0.1.25 - December 10, 2021

- Update cargo dependency to 0.58.0.
- Avoid using the `-Z save-analysis` flag unless really needed. Contributed by [@pvdrz](https://github.com/pvdrz)

## Release 0.1.24 - October 22, 2021

- Update cargo dependency to 0.57.0.

## Release 0.1.23 - September 15, 2021

- Update cargo dependency to 0.56.0.

## Release 0.1.22 - June 17, 2021

- Update cargo dependency to 0.54.0.

## Release 0.1.21 - May 7, 2021

- Update cargo dependency to 0.53.0.

## Release 0.1.20 - Mar 26, 2021

- Update cargo dependency to 0.52.0.

## Release 0.1.19 - Mar 9, 2021

- Update lockfile version of rand_core to 0.6.2 in response to a [security issue in 0.6.1](https://rustsec.org/advisories/RUSTSEC-2021-0023). Contributed by [@rajivshah3](https://github.com/rajivshah3)

## Release 0.1.18 - Feb 12, 2021

- Update cargo dependency to 0.51.0.

## Release 0.1.17 - Jan 24, 2021

- Update cargo dependency to 0.50.0.

## Release 0.1.16 - November 20, 2020

- Only force rebuilds on workspace members.
- Update cargo dependency to 0.49.0.

## Release 0.1.15 - October 10, 2020

- Update cargo dependency to 0.48.0.

## Release 0.1.14 - August 28, 2020

- Update cargo dependency to 0.47.0.

## Release 0.1.13 - July 19, 2020

- Update cargo dependency to 0.46.0.
- Remove `tempdir` dev dependency in favour of `tempfile`. Contributed by [@paolobarbolini](https://github.com/paolobarbolini)

## Release 0.1.12 - June 6, 2020

- Update cargo dependency to 0.45.0.

## Release 0.1.11 - April 23, 2020

- Update cargo dependency to 0.44.0. Contributed by [@lu-zero](https://github.com/lu-zero)
- Add new backend based on [binary depinfo](https://github.com/rust-lang/rust/issues/63012),
  can be turned on using `--backend depinfo`.

## Release 0.1.10 - March 14, 2020

- Fix [regression](https://github.com/est31/cargo-udeps/issues/62) of 0.1.9 that broke compilation with proc macros

## Release 0.1.9 - March 12, 2020

- Update cargo dependency to 0.43.0. Contributed by [@lu-zero](https://github.com/lu-zero)
- Add ability to turn off warnings for specific dependencies

## Release 0.1.8 - February 29, 2020

- [Fix inability](https://github.com/est31/cargo-udeps/issues/53) to `cargo install cargo-udeps`
- Add `--output json` option

## Release 0.1.7 - January 31, 2020

- Update cargo dependency to 0.42.0
- Don't assume that specified dependencies have lib targets

## Release 0.1.6 - December 2, 2019

- Update cargo dependency to 0.40.0
- Upgrade to Cargo.lock v2 format
- Improved testsuite
- Rust 2018 migration
- Switch from travis CI to Github actions

## Release 0.1.5 - September 27, 2019

- Downgrade the "same `lib` name" error to warning
- Distinguish build dependencies from normal/dev ones
- Start of a testsuite

## Release 0.1.4 - September 11, 2019

- Support for running multiple times

## Release 0.1.3 - September 10, 2019

- Bugfixes around the dependency renaming feature
- Nicer error reports

## Release 0.1.2 - September 09, 2019

- Support dependency renaming. Contributed by [@qryxip](https://github.com/qryxip).
- Ability to print version using the --version flag

## Release 0.1.1 - August 31, 2019

- Support for rlibs, staticlibs, dylibs, etc.
- Support for checking dependencies of proc-macro crates
- Non zero exit code when there are unused dependencies
- Less verbose output

## Release 0.1.0 - August 30, 2019

Initial release. Reporting unused dependencies.
