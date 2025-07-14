mod runner;

use anyhow::anyhow;
use cargo::CargoResult;
use pretty_assertions::assert_eq;

use crate::runner::Runner;

use std::{error, fmt};

#[derive(Debug)]
enum VersionError {
    Stdout,
}

impl error::Error for VersionError {}

// Implement `Display` for `version_error` required for `std::error::Error`.
impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error: expect the command to output version information from `stderr`"
        )
    }
}

#[test]
fn check_version_exists() -> CargoResult<()> {
    match Runner::new("cargo_udeps_test_check_version_information")?
        .arg("--version")
        .run()
    {
        Ok(_) => Err(anyhow!(VersionError::Stdout)),
        Err(error) => {
            assert_eq!(
                format!("cargo-udeps {}\n", env!("CARGO_PKG_VERSION")),
                error.to_string(),
            );
            Ok(())
        }
    }
}
