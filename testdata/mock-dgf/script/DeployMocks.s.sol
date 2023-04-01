// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import { MockDisputeGameFactory } from "src/MockDisputeGameFactory.sol";
import { MockDisputeGame_OutputAttestation } from "src/MockDisputeGame_OutputAttestation.sol";

/// @notice This script deploys the mock dispute game factory.
contract DeployMocks is Script {
    function run() public {
        // Deploy the mock dispute game factory
        bytes memory bytecode = type(MockDisputeGameFactory).creationCode;
        bytes32 salt = bytes32(uint256(keccak256(abi.encodePacked("MockDisputeGameFactory.op-challenger"))) + 1);
        address addr;

        vm.broadcast();
        assembly {
            addr := create2(0, add(bytecode, 0x20), mload(bytecode), salt)
        }

        // Write the address to the devnet ENV
        console.log(string.concat("ENV Variable: export OP_CHALLENGER_DGF=\"", vm.toString(addr), "\""));
    }
}
