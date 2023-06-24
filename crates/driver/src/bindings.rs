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

// Generates the bindings for the `FaultDisputeGame` contract.
abigen!(
    FaultDisputeGame,
    r"[
        function attack(uint256 _parentIndex, bytes32 _pivot) external payable
        function defend(uint256 _parentIndex, bytes32 _pivot) external payable
        function claimData(uint256 _index) external view returns ((uint32,bool,bytes32,uint128,uint128))
        function step(uint256 _stateIndex, uint256 _claimIndex, bool _isAttack, bytes calldata _stateData, bytes calldata _proof) external
        function resolve() external returns (uint8)
        function rootClaim() external pure returns (bytes32)
        function createdAt() external view returns (uint64)
        function l2BlockNumber() external view returns (uint256)
    ]"
);
