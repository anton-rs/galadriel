// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./MockDisputeGameFactory.sol";

/// @title MockDisputeGame_OutputAttestation
/// @dev This contract is used for testing the `op-challenger`'s `OutputAttestationDriver`
///      on a local devnet.
contract MockDisputeGame_OutputAttestation {
    Claim public immutable ROOT_CLAIM;
    mapping(address => bool) public challenges;

    constructor(Claim _rootClaim, address _creator) {
        ROOT_CLAIM = _rootClaim;
        challenges[_creator] = true;
    }

    function challenge(bytes calldata _signature) external {
        challenges[msg.sender] = true;
        _signature;
    }
}
