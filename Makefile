
all: devnet-up start

devnet-up:
	./start_devnet.sh

start:
	cargo run -- --l1-ws-endpoint $OP_CHALLENGER_L1_WS --trusted-op-node-endpoint $OP_CHALLENGER_TRUSTED_OP_NODE_RPC --l2-output-oracle $OP_CHALLENGER_L2OO --dispute-game-factory $OP_CHALLENGER_DGF