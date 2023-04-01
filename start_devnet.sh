#!/bin/bash

# Set up environment variables
source .env.devnet

# Change accordingly
MONOREPO_DIR="$HOME/dev/optimism/monorepo"

# Boot up the devnet
(cd $MONOREPO_DIR && make devnet-up)

# Deploy the mock dispute game contract
(cd ./testdata/mock-dgf && forge script script/DeployMocks.s.sol --rpc-url http://localhost:8545 --private-key $OP_CHALLENGER_KEY --broadcast)

echo "----------------------------------------------------------------"
echo " - Paste the environment variable logged by the forge script"
echo " - into the \`.env.devnet\` file and then source it again before"
echo " - running the \`op-challenger\`."
echo "----------------------------------------------------------------"
