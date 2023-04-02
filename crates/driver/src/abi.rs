use ethers::prelude::abigen;

// Generates the bindings for the `DisputeGame_Factory` contract.
abigen!(
    DisputeGame_Factory,
    r"[
        event DisputeGameCreated(address indexed, uint8 indexed, bytes32 indexed)
        function create(uint8 gameType, bytes32 rootClaim, bytes calldata extraData) external returns (address _proxy)
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
        function ROOT_CLAIM() external view returns (bytes32)
        function L2_BLOCK_NUMBER() external view returns (uint256)
        function challenges(address) external view returns (bool)
        function challenge(bytes calldata) external
    ]"
);
