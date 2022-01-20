# Changelog

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
