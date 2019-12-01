# Changelog

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
