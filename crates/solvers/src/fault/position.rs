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
        (self << 1) | 1
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
    use super::{compute_gindex, Position};

    const MAX_DEPTH: u64 = 4;

    #[test]
    fn position_correctness_static() {
        let mut p = compute_gindex(0, 0);
        assert_eq!(p, 1);
        assert_eq!(p.depth(), 0);
        assert_eq!(p.index_at_depth(), 0);
        let mut r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 31);
        assert_eq!(r.index_at_depth(), 15);

        p = compute_gindex(1, 0);
        assert_eq!(p, 2);
        assert_eq!(p.depth(), 1);
        assert_eq!(p.index_at_depth(), 0);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 23);
        assert_eq!(r.index_at_depth(), 7);

        p = compute_gindex(1, 1);
        assert_eq!(p, 3);
        assert_eq!(p.depth(), 1);
        assert_eq!(p.index_at_depth(), 1);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 31);
        assert_eq!(r.index_at_depth(), 15);

        p = compute_gindex(2, 0);
        assert_eq!(p, 4);
        assert_eq!(p.depth(), 2);
        assert_eq!(p.index_at_depth(), 0);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 19);
        assert_eq!(r.index_at_depth(), 3);

        p = compute_gindex(2, 1);
        assert_eq!(p, 5);
        assert_eq!(p.depth(), 2);
        assert_eq!(p.index_at_depth(), 1);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 23);
        assert_eq!(r.index_at_depth(), 7);

        p = compute_gindex(2, 2);
        assert_eq!(p, 6);
        assert_eq!(p.depth(), 2);
        assert_eq!(p.index_at_depth(), 2);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 27);
        assert_eq!(r.index_at_depth(), 11);

        p = compute_gindex(2, 3);
        assert_eq!(p, 7);
        assert_eq!(p.depth(), 2);
        assert_eq!(p.index_at_depth(), 3);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 31);
        assert_eq!(r.index_at_depth(), 15);

        p = compute_gindex(3, 0);
        assert_eq!(p, 8);
        assert_eq!(p.depth(), 3);
        assert_eq!(p.index_at_depth(), 0);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 17);
        assert_eq!(r.index_at_depth(), 1);

        p = compute_gindex(3, 1);
        assert_eq!(p, 9);
        assert_eq!(p.depth(), 3);
        assert_eq!(p.index_at_depth(), 1);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 19);
        assert_eq!(r.index_at_depth(), 3);

        p = compute_gindex(3, 2);
        assert_eq!(p, 10);
        assert_eq!(p.depth(), 3);
        assert_eq!(p.index_at_depth(), 2);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 21);
        assert_eq!(r.index_at_depth(), 5);

        p = compute_gindex(3, 3);
        assert_eq!(p, 11);
        assert_eq!(p.depth(), 3);
        assert_eq!(p.index_at_depth(), 3);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 23);
        assert_eq!(r.index_at_depth(), 7);

        p = compute_gindex(3, 4);
        assert_eq!(p, 12);
        assert_eq!(p.depth(), 3);
        assert_eq!(p.index_at_depth(), 4);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 25);
        assert_eq!(r.index_at_depth(), 9);

        p = compute_gindex(3, 5);
        assert_eq!(p, 13);
        assert_eq!(p.depth(), 3);
        assert_eq!(p.index_at_depth(), 5);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 27);
        assert_eq!(r.index_at_depth(), 11);

        p = compute_gindex(3, 6);
        assert_eq!(p, 14);
        assert_eq!(p.depth(), 3);
        assert_eq!(p.index_at_depth(), 6);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 29);
        assert_eq!(r.index_at_depth(), 13);

        p = compute_gindex(3, 7);
        assert_eq!(p, 15);
        assert_eq!(p.depth(), 3);
        assert_eq!(p.index_at_depth(), 7);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 31);
        assert_eq!(r.index_at_depth(), 15);

        p = compute_gindex(4, 0);
        assert_eq!(p, 16);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 0);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 16);
        assert_eq!(r.index_at_depth(), 0);

        p = compute_gindex(4, 1);
        assert_eq!(p, 17);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 1);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 17);
        assert_eq!(r.index_at_depth(), 1);

        p = compute_gindex(4, 2);
        assert_eq!(p, 18);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 2);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 18);
        assert_eq!(r.index_at_depth(), 2);

        p = compute_gindex(4, 3);
        assert_eq!(p, 19);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 3);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 19);
        assert_eq!(r.index_at_depth(), 3);

        p = compute_gindex(4, 4);
        assert_eq!(p, 20);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 4);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 20);
        assert_eq!(r.index_at_depth(), 4);

        p = compute_gindex(4, 5);
        assert_eq!(p, 21);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 5);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 21);
        assert_eq!(r.index_at_depth(), 5);

        p = compute_gindex(4, 6);
        assert_eq!(p, 22);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 6);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 22);
        assert_eq!(r.index_at_depth(), 6);

        p = compute_gindex(4, 7);
        assert_eq!(p, 23);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 7);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 23);
        assert_eq!(r.index_at_depth(), 7);

        p = compute_gindex(4, 8);
        assert_eq!(p, 24);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 8);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 24);
        assert_eq!(r.index_at_depth(), 8);

        p = compute_gindex(4, 9);
        assert_eq!(p, 25);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 9);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 25);
        assert_eq!(r.index_at_depth(), 9);

        p = compute_gindex(4, 10);
        assert_eq!(p, 26);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 10);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 26);
        assert_eq!(r.index_at_depth(), 10);

        p = compute_gindex(4, 11);
        assert_eq!(p, 27);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 11);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 27);
        assert_eq!(r.index_at_depth(), 11);

        p = compute_gindex(4, 12);
        assert_eq!(p, 28);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 12);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 28);
        assert_eq!(r.index_at_depth(), 12);

        p = compute_gindex(4, 13);
        assert_eq!(p, 29);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 13);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 29);
        assert_eq!(r.index_at_depth(), 13);

        p = compute_gindex(4, 14);
        assert_eq!(p, 30);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 14);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 30);
        assert_eq!(r.index_at_depth(), 14);

        p = compute_gindex(4, 15);
        assert_eq!(p, 31);
        assert_eq!(p.depth(), 4);
        assert_eq!(p.index_at_depth(), 15);
        r = p.right_index(MAX_DEPTH);
        assert_eq!(r, 31);
        assert_eq!(r.index_at_depth(), 15);
    }
}
