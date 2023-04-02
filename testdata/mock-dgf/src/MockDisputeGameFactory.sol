// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import { MockDisputeGame_OutputAttestation } from "./MockDisputeGame_OutputAttestation.sol";

/// @notice The type of proof system being used.
enum GameType {
    /// @dev The game will use a `IDisputeGame` implementation that utilizes fault proofs.
    FAULT,
    /// @dev The game will use a `IDisputeGame` implementation that utilizes validity proofs.
    VALIDITY,
    /// @dev The game will use a `IDisputeGame` implementation that utilizes attestation proofs.
    ATTESTATION
}

/// @notice A `Claim` type represents a 32 byte hash or other unique identifier for a claim about
///         a certain piece of information.
/// @dev For the `FAULT` `GameType`, this will be a root of the merklized state of the fault proof
///      program at the end of the state transition.
///      For the `ATTESTATION` `GameType`, this will be an output root.
type Claim is bytes32;

/// @title MockDisputeGameFactory
/// @dev This contract is used for testing the `op-challenger`'s `OutputAttestationDriver`
///      on a local devnet.
contract MockDisputeGameFactory {
    event DisputeGameCreated(address indexed disputeProxy, GameType indexed gameType, Claim indexed rootClaim);

    /// @notice Creates a new DisputeGame proxy contract.
    /// @param gameType The type of the DisputeGame - used to decide the proxy implementation
    /// @param rootClaim The root claim of the DisputeGame.
    /// @param extraData Any extra data that should be provided to the created dispute game.
    function create(GameType gameType, Claim rootClaim, bytes calldata extraData) external returns (MockDisputeGame_OutputAttestation mock) {
        mock = new MockDisputeGame_OutputAttestation(rootClaim, msg.sender);
        emit DisputeGameCreated(address(mock), gameType, rootClaim);
        extraData; // Unused
    }
}
