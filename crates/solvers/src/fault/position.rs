//! The position module holds the [Position] trait and its implementations.

/// The [Position] trait defines the interface of a generalized index within a binary tree.
/// A "Generalized Index" is calculated as `2^{depth} + index_at_depth`.
pub trait Position {
    /// Returns the depth of the [Position] within the tree.
    fn depth(&self) -> u64;
    /// Returns the index at depth of the [Position] within the tree.
    fn index_at_depth(&self) -> u64;
    /// Returns the left child [Position] relative to the current [Position].
    fn left(&self) -> Self;
    /// Returns the right child [Position] relative to the current [Position].
    fn right(&self) -> Self;
    /// Returns the parent [Position] relative to the current [Position].
    fn parent(&self) -> Self;
    /// Returns the rightmost [Position] that commits to the same trace index as the current [Position].
    fn right_index(&self, max_depth: u64) -> Self;
    /// Returns the trace index that the current [Position] commits to.
    fn trace_index(&self, max_depth: u64) -> u64;
    /// Returns the relative [Position] for an attack or defense move against the current [Position].
    fn make_move(&self, is_attack: bool) -> Self;
}

/// Computes a generalized index from a depth and index at depth.
///
/// ### Takes
/// - `depth`: The depth of the generalized index.
/// - `index_at_depth`: The index at depth of the generalized index.
///
/// ### Returns
/// - `u128`: The generalized index: `2^{depth} + index_at_depth`.
pub fn compute_gindex(depth: u8, index_at_depth: u64) -> u128 {
    2u128.pow(depth as u32) + index_at_depth as u128
}

/// Implementation of the [Position] trait for the [std::u128] primitive type.
impl Position for u128 {
    fn depth(&self) -> u64 {
        127 - self.leading_zeros() as u64
    }

    fn index_at_depth(&self) -> u64 {
        (self - (1 << self.depth())) as u64
    }

    fn left(&self) -> Self {
        self << 1
    }

    fn right(&self) -> Self {
        self.left() | 1
    }

    fn parent(&self) -> Self {
        self >> 1
    }

    fn right_index(&self, max_depth: u64) -> Self {
        let remaining = max_depth - self.depth();
        (self << remaining) | ((1 << remaining) - 1)
    }

    fn trace_index(&self, max_depth: u64) -> u64 {
        self.right_index(max_depth).index_at_depth()
    }

    fn make_move(&self, is_attack: bool) -> Self {
        ((!is_attack as u128) | self) << 1
    }
}

#[cfg(test)]
mod test {
    use super::Position;

    /// A helper struct for testing the [Position] trait implementation for [std::u128].
    /// 0. `u64` - `depth`
    /// 1. `u64` - `index_at_depth`
    /// 2. `u128` - `right_index`
    /// 3. `u64` - `trace_index`
    struct PositionMetaData(u64, u64, u128, u64);

    const MAX_DEPTH: u64 = 4;
    const EXPECTED_VALUES: &[PositionMetaData] = &[
        PositionMetaData(0, 0, 31, 15),
        PositionMetaData(1, 0, 23, 7),
        PositionMetaData(1, 1, 31, 15),
        PositionMetaData(2, 0, 19, 3),
        PositionMetaData(2, 1, 23, 7),
        PositionMetaData(2, 2, 27, 11),
        PositionMetaData(2, 3, 31, 15),
        PositionMetaData(3, 0, 17, 1),
        PositionMetaData(3, 1, 19, 3),
        PositionMetaData(3, 2, 21, 5),
        PositionMetaData(3, 3, 23, 7),
        PositionMetaData(3, 4, 25, 9),
        PositionMetaData(3, 5, 27, 11),
        PositionMetaData(3, 6, 29, 13),
        PositionMetaData(3, 7, 31, 15),
        PositionMetaData(4, 0, 16, 0),
        PositionMetaData(4, 1, 17, 1),
        PositionMetaData(4, 2, 18, 2),
        PositionMetaData(4, 3, 19, 3),
        PositionMetaData(4, 4, 20, 4),
        PositionMetaData(4, 5, 21, 5),
        PositionMetaData(4, 6, 22, 6),
        PositionMetaData(4, 7, 23, 7),
        PositionMetaData(4, 8, 24, 8),
        PositionMetaData(4, 9, 25, 9),
        PositionMetaData(4, 10, 26, 10),
        PositionMetaData(4, 11, 27, 11),
        PositionMetaData(4, 12, 28, 12),
        PositionMetaData(4, 13, 29, 13),
        PositionMetaData(4, 14, 30, 14),
        PositionMetaData(4, 15, 31, 15),
    ];

    #[test]
    fn position_correctness_static() {
        for (p, v) in EXPECTED_VALUES.iter().enumerate() {
            let pos = (p + 1) as u128;
            assert_eq!(pos.depth(), v.0);
            assert_eq!(pos.index_at_depth(), v.1);
            let r = pos.right_index(MAX_DEPTH);
            assert_eq!(r, v.2);
            assert_eq!(r.index_at_depth(), v.3);
        }
    }
}
