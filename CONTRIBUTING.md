# Contributing

Thanks for wanting to contribute! :yellow_heart:

## Dependencies

To work in this repo, you'll need to install:
1. [Rust Toolchain](https://rustup.rs/)

## Getting Started

1. Clone the repo
```sh
git clone git@github.com:clabby/op-challenger.git
```
2. Start the devnet.
```sh
nvim ./start_devnet.sh # edit MONOREPO_DIR accordingly
./start_devnet.sh
```
3. Start the `op-challenger` with information, warning, and error traces enabled.
```sh
cargo run --bin op-challenger -- -vv
```

## Linting

To lint your code, run:
```sh
cargo +nightly fmt -- && cargo +nightly clippy --all --all-features -- -D warnings
```
