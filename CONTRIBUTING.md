# Contributing to cargo-udeps

Thanks for taking the time to contribute! :tada::+1:

## Development

### Running code locally

Suppose you downloaded cargo-udeps to `/home/user/cargo-udeps` and you want to run
this local version of cargo-udeps on some target directory `/home/user/check-me`. Consider these
steps:
```bash
cd /home/user/check-me
cargo +nightly run --manifest-path /home/user/cargo-udeps/Cargo.toml udeps
```
This way you can make changes to your local version of cargo-udeps and see how modified version
of cargo-udeps acts on some target repository.

### Running tests

Nothing special, just `cargo test`.
