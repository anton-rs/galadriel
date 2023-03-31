use ethers::prelude::abigen;

// Generates the bindings for the `DisputeGame_Factory` contract.
abigen!(
    DisputeGame_Factory,
    r"[
        event DisputeGameCreated(address indexed, uint8 indexed, bytes32 indexed)
    ]"
);

// Generates the bindings for the `L2OutputOracle` contract.
abigen!(
    L2OutputOracle,
    r"[
        event OutputProposed(bytes32 indexed, uint256 indexed, uint256 indexed, uint256)
    ]"
);

// Generates the bindings for the `DisputeGame_OutputAttestation` contract.
abigen!(
    DisputeGame_OutputAttestation,
    r"[
        function challenge(bytes calldata signature) external
    ]"
);
