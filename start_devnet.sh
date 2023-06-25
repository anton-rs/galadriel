#!/bin/bash

# Set up environment variables
source .env.devnet

# Change accordingly
MONOREPO_DIR="$HOME/dev/op/monorepo"

# Pre-funded devnet account
DEVNET_SPONSOR="ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

export ETH_RPC_URL=http://localhost:8545

# Boot up the devnet
# Before booting, we make sure that we have a fresh devnet
(cd $MONOREPO_DIR && make devnet-down && make devnet-clean && L2OO_ADDRESS="0x6900000000000000000000000000000000000000" make devnet-up-deploy)

find_image_id() {
  IMAGE_ID=$(docker images --format="{{.Repository}} {{.ID}}" | rg "$1" | cut -d' ' -f2)
  docker container ls --all --filter=ancestor=$IMAGE_ID --format "{{.ID}}"
}

# Fetching balance of the sponsor
echo "----------------------------------------------------------------"
echo " - Fetching balance of the sponsor"
echo " - Balance: $(cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)"
echo "----------------------------------------------------------------"

echo ""

echo "----------------------------------------------------------------"
echo " - Sending 500 eth to the devnet challenger public key"
cast send $OP_CHALLENGER_PUB_KEY --value $(cast --to-wei 500) --private-key $DEVNET_SPONSOR
echo " - Done!"
echo "----------------------------------------------------------------"

echo ""

echo "----------------------------------------------------------------"
echo " All done! Loading into the control center in 3 seconds..."
echo "----------------------------------------------------------------"

sleep 3

mprocs \
  "L1= docker attach $(find_image_id 'l1')" \
  "L2= docker attach $(find_image_id 'l2')" \
  "OP_NODE= docker attach $(find_image_id 'op-node')" \
  "PROPOSER= docker attach $(find_image_id 'op-proposer')" \
  "BATCHER= docker attach $(find_image_id 'op-batcher')" \
  "CHALLENGER= cargo run --bin op-challenger -- -vvvvv"
