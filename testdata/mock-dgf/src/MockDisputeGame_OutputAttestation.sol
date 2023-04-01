// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/// @title MockDisputeGame_OutputAttestation
/// @dev This contract is used for testing the `op-challenger`'s `OutputAttestationDriver`
///      on a local devnet.
contract MockDisputeGame_OutputAttestation {
    function challenge(bytes calldata _signature) external {
        // Do nothing
    }
}
