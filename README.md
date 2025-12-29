## cargo-udeps


[![crates.io](https://img.shields.io/crates/v/cargo-udeps.svg)](https://crates.io/crates/cargo-udeps)
[![dependency status](https://deps.rs/repo/github/est31/cargo-udeps/status.svg)](https://deps.rs/repo/github/est31/cargo-udeps)

Find unused dependencies in Cargo.toml.

While compilation of this tool also works on Rust stable,
it needs Rust nightly to actually run.
As it includes `cargo` as a dependency, it will likely compile with the latest rustc release, as well as the one before it.

### Installation

#### GitHub Releases

<https://github.com/est31/cargo-udeps/releases>

#### `cargo install` ([crates.io](https://crates.io/crates/cargo-udeps))

```
cargo install cargo-udeps --locked
```

#### `cargo install` ([`master`](https://github.com/est31/cargo-udeps/tree/master))

```
cargo install --git https://github.com/est31/cargo-udeps --locked
```

#### Dedicated packages

Some package managers have packaged `cargo-udeps`:

* Nix/Nix OS: `cargo-udeps`
* Arch Linux: `pacman -S cargo-udeps`
* Homebrew: `brew install cargo-udeps`

### Usage

```
cargo +nightly udeps
```

It either prints out a "unused crates" line listing the crates,
or it prints out a line saying that no crates were unused.

### pre-commit

You can use it as [pre-commit](https://pre-commit.com/) hook:

```yaml
- repo: https://github.com/est31/cargo-udeps
  rev: v0.1.47
  hooks:
  - id: udeps
```

## Ignoring some of the dependencies

To ignore some of the dependencies, add `package.metadata.cargo-udeps.ignore` to `Cargo.toml`.

```toml
[package.metadata.cargo-udeps.ignore]
normal = ["if_chain"]
#development = []
#build = []

[dependencies]
if_chain = "1.0.0" # Used only in doc-tests, which `cargo-udeps` cannot check.
```

Alternatively, add dependencies to `workspace.metadata.cargo-udeps.ignore` in the
workpace `Cargo.toml` to ignore them in all packages in the workspace.

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
* https://github.com/casey/just/pull/587
* https://github.com/Garvys/rustfst/pull/76
* https://github.com/yewstack/yew_router/pull/252
* https://github.com/rust-bitcoin/rust-bitcoincore-rpc/pull/169
* https://github.com/hendrikmaus/helm-templexer/pull/85
* https://github.com/itchysats/itchysats/commit/99076ecb907b3bfc5f31ffcdad9716df1869c8f7
* https://github.com/isographlabs/isograph/commit/9da885db555c945d0cc3667e2a2aa94573cd8fc7

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### License
[license]: #license

This tool is distributed under the terms of both the MIT license
and the Apache License (Version 2.0), at your option.

See [LICENSE](LICENSE) for details.

#### License of your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
