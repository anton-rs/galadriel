# Contributing

Thanks for wanting to contribute! :yellow_heart:

## Dependencies

To work in this repo, you'll need to install:
1. [Rust Toolchain](https://rustup.rs/)
1. [Docker](https://docs.docker.com/get-docker/)
1. [ripgrep](https://github.com/BurntSushi/ripgrep)
1. [mprocs](https://github.com/pvolok/mprocs)

And clone the [Optimism Monorepo](https://github.com/ethereum-optimism/optimism)

## Getting Started

1. Clone the repo
```sh
git clone git@github.com:clabby/op-challenger.git
```
2. Configure your dev environment
```sh
# Set the MONOREPO_DIR variable
nvim start_devnet.sh
# On the L1 service, port forward the websocket endpoint port (8546)
nvim $MONOREPO_DIR/ops-bedrock/docker-compose.yml
# Start the devnet and deploy the mock dispute game factory
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
