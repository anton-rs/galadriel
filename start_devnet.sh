#!/bin/bash

# Set up environment variables
source .env.devnet

# Change accordingly
MONOREPO_DIR="$HOME/dev/optimism/monorepo"

# Boot up the devnet
(cd $MONOREPO_DIR && make devnet-down && make devnet-up)

# Deploy the mock dispute game contract
(cd ./testdata/mock-dgf && forge script script/DeployMocks.s.sol --rpc-url http://localhost:8545 --private-key $OP_CHALLENGER_KEY --broadcast)

echo "----------------------------------------------------------------"
echo "                     Devnet is up and running                   " 
echo " - L1 RPC: http://localost:8545"
echo " - L2 WS: ws://localost:8546"
echo " - L2 RPC: http://localost:9545"
echo " - OP Node RPC: http://localost:7545"
echo " - Dispute game factory: $OP_CHALLENGER_DGF"
echo "----------------------------------------------------------------"
