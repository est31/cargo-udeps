## cargo-udeps


[![crates.io](https://img.shields.io/crates/v/cargo-udeps.svg)](https://crates.io/crates/cargo-udeps)
[![dependency status](https://deps.rs/repo/github/est31/cargo-udeps/status.svg)](https://deps.rs/repo/github/est31/cargo-udeps)

Find unused dependencies in Cargo.toml.

While compilation of this tool also works on Rust stable,
it needs Rust nightly to actually run.

### Installation

#### GitHub Releases

<https://github.com/est31/cargo-udeps/releases>

#### `cargo install` ([crates.io](https://crates.io/crates/cargo-udeps))

```
cargo install cargo-udeps
```

#### `cargo install` ([`master`](https://github.com/est31/cargo-udeps/tree/master))

```
cargo install --git https://github.com/est31/cargo-udeps
```

### Usage

```
cargo +nightly udeps
```

It either prints out a "unused crates" line listing the crates,
or it prints out a line saying that no crates were unused.

## Known bugs

* Some unused crates might not be detected.
  This includes crates used by std and its dependencies as well as crates that
  are already being used by dependencies of the studied crate.

* Crates are currently only handled on a per name basis.
  Two crates with the same name but different versions would be a problem.

## Trophy case

This is a list of cases where unused dependencies were found using cargo-udeps.
You are welcome to expand it:

* https://github.com/nushell/nushell/pull/519
* https://github.com/servo/pathfinder/pull/236
* https://github.com/oconnor663/shared_child.rs/commit/5929637f5cf1bebc5d608b4d98fd5c8a10626712
* https://github.com/oconnor663/bao/commit/d216ee7c04e3587925dee68cce0b2a1ba44bc1d2
* https://github.com/dabreegster/abstreet/commit/03b685673bebbc95e2bcbd7c85358547bcffe8c3
* https://github.com/rust-lang/crater/pull/446
* https://github.com/kodegenix/kg-tree/commit/0270ec495887cf0ff7580155db4ff12664614ee8
* https://github.com/opereon/opereon/commit/4d29cf174c0b178c1484f698ceb0e654f95a78d0
* https://github.com/djg/audioipc-2/commit/de0fc58cf1e87079027fce06b50eeffa6ae23d54

### License
[license]: #license

This tool is distributed under the terms of both the MIT license
and the Apache License (Version 2.0), at your option.

See [LICENSE](LICENSE) for details.

#### License of your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
