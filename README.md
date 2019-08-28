## cargo-udeps

Find unused dependencies in Cargo.toml.

Needs Rust stable to compile and Rust nightly to run.

```
cargo install --git https://github.com/est31/cargo-udeps
```

Then run it using:

```
cargo +nightly udeps
```

## Known bugs

There are no false positives, but some unused crates might not be detected.
This includes crates used by std and its dependencies as well as crates that
are already being used by dependencies of the studied crate.

### License
[license]: #license

This tool is distributed under the terms of both the MIT license
and the Apache License (Version 2.0), at your option.

See [LICENSE](LICENSE) for details.

#### License of your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
