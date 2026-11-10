//! Global open-node heap of the B&B tree (`T+0x120`/`+0x130`): min heap on the node key `I+0x50` with no secondary
//! key, so equal keys are ordered only by the insert/pop/prune rules below (docs/m2/mip_node_heap_rust.md).
//! Comparisons are the original float comparisons: -0 and +0 are the same key.

/// Entries are (key, bound, node).
pub struct NodeHeap<T> {
    h: Vec<(f64, f64, T)>,
}

impl<T> Default for NodeHeap<T> {
    fn default() -> Self {
        Self { h: Vec::new() }
    }
}

impl<T> NodeHeap<T> {
    pub fn len(&self) -> usize {
        self.h.len()
    }

    pub fn is_empty(&self) -> bool {
        self.h.is_empty()
    }

    /// `T+0x170`: bound of the root entry, 1e100 when empty.
    pub fn min_bound(&self) -> f64 {
        self.h.first().map_or(1e100, |e| e.1)
    }

    /// Entries in array order.
    pub fn iter(&self) -> impl Iterator<Item = (f64, f64, &T)> {
        self.h.iter().map(|(k, b, t)| (*k, *b, t))
    }

    /// `6e5802`: append and move up while the parent key is greater; a new equal key stays below.
    pub fn push(&mut self, key: f64, bound: f64, node: T) {
        let mut i = self.h.len();
        self.h.push((key, bound, node));
        while i != 0 {
            let p = (i - 1) >> 1;
            if self.h[p].0 <= key {
                break;
            }
            self.h.swap(i, p);
            i = p;
        }
    }

    /// `6e8eb3` pop: take the root, move the last entry to the root and sift it down.
    pub fn pop(&mut self) -> Option<(f64, f64, T)> {
        if self.h.is_empty() {
            return None;
        }
        let top = self.h.swap_remove(0);
        sift_down(&mut self.h, 0);
        Some(top)
    }

    /// `6e460c`: in array order drop the entries with `cutoff <= bound` (handed to `dropped` in that order) and keep the
    /// others in order, then sift down from the last index to 0.
    pub fn prune(&mut self, cutoff: f64, mut dropped: impl FnMut(T)) {
        let all = std::mem::take(&mut self.h);
        for e in all {
            if cutoff <= e.1 {
                dropped(e.2);
            } else {
                self.h.push(e);
            }
        }
        for i in (0..self.h.len()).rev() {
            sift_down(&mut self.h, i);
        }
    }
}

/// Sift-down shared by `6e8eb3` and `6e460c`: with two children take the right one on equal keys and stop when the key
/// is <= the smaller one; with one child move only when the child is strictly smaller.
fn sift_down<T>(h: &mut [(f64, f64, T)], mut i: usize) {
    let n = h.len();
    let Some(x) = h.get(i).map(|e| e.0) else { return };
    loop {
        let (c1, c2) = (2 * i + 1, 2 * i + 2);
        if n <= c1 {
            return;
        }
        let k1 = h[c1].0;
        if n <= c2 {
            if k1 <= x && x != k1 {
                h.swap(i, c1);
            }
            return;
        }
        let k2 = h[c2].0;
        let m = if k2 <= k1 { k2 } else { k1 };
        if x <= m {
            return;
        }
        let j = if k2 <= k1 { c2 } else { c1 };
        h.swap(i, j);
        i = j;
    }
}

/// Heap key of a branching child before the depth term (docs/m2/mip_node_key.md section 2.1): `bound` is the child's
/// `I+0x18`, `nfrac` the parent LP's fractional count, `root_nfrac` `M+0xa4` (root LP fractional count) and
/// `obj_step` `M+0x40` (-1e100 when the objective has no step, then the key is the fractional count alone).
pub fn child_key(bound: f64, nfrac: i32, root_nfrac: i32, obj_step: f64) -> f64 {
    let n = f64::from(nfrac);
    if obj_step == -1e100 {
        return n;
    }
    let t = (n / f64::from(root_nfrac.max(1))).min(1.0) * 2f64.powi(-48) * bound.abs().max(1.0);
    t + bound
}

/// Depth term of the key: a parent deeper than 100 times the column count adds `max(|key|, 1) * 2^-45`.
pub fn deep_key(key: f64, parent_depth: i32, ncols: i32) -> f64 {
    if f64::from(parent_depth) > f64::from(ncols) * 100.0 {
        key + key.abs().max(1.0) * 2f64.powi(-45)
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::{child_key, deep_key, NodeHeap};

    fn ids(h: &NodeHeap<u32>) -> Vec<u32> {
        h.iter().map(|e| *e.2).collect()
    }

    #[test]
    fn keys() {
        // ratio capped at 1, |b| < 1 counts as 1, -1e100 step keys on the count; values from the M2-06b-2 captures
        assert_eq!(child_key(0.0, 9, 6, 0.0).to_bits(), 0x3cf0000000000000);
        assert_eq!(child_key(0.0, 2, 6, 0.0), 2f64.powi(-48) * (2.0 / 6.0));
        assert_eq!(child_key(0.0, 2, 4, -1e100), 2.0);
        assert_eq!(deep_key(2.0, 101, 1), 2.0 + 2.0 * 2f64.powi(-45));
        assert_eq!(deep_key(2.0, 100, 1), 2.0);
    }

    #[test]
    fn equal_keys_follow_the_original_rules() {
        let mut h = NodeHeap::default();
        for (i, k) in [1.0, 1.0, 1.0, 0.0].into_iter().enumerate() {
            h.push(k, k, i as u32);
        }
        // an equal parent stops the insert; the smaller key 0 moves to the root
        assert_eq!(ids(&h), [3, 0, 2, 1]);
        assert_eq!(h.min_bound(), 0.0);
        // last entry 1 goes to the root and stops: its key equals the smaller child
        assert_eq!(h.pop().map(|e| e.2), Some(3));
        assert_eq!(ids(&h), [1, 0, 2]);

        let mut h = NodeHeap::default();
        for (i, k) in [0.0, 1.0, 1.0, 2.0].into_iter().enumerate() {
            h.push(k, k, i as u32);
        }
        // last entry 3 (key 2) goes to the root; equal children 1 and 2: the right one moves up
        assert_eq!(h.pop().map(|e| e.2), Some(0));
        assert_eq!(ids(&h), [2, 1, 3]);
        // -0 key is smaller than 1 and moves to the root; prune drops bound >= cutoff in array order, keeps the rest in
        // order and re-heapifies from the last index
        h.push(-0.0, 5.0, 9);
        assert_eq!(ids(&h), [9, 2, 3, 1]);
        let mut gone = Vec::new();
        h.prune(5.0, |n| gone.push(n));
        assert_eq!(gone, [9]);
        assert_eq!(ids(&h), [2, 3, 1]);
        h.prune(f64::NEG_INFINITY, |n| gone.push(n));
        assert!(h.is_empty() && h.min_bound() == 1e100);
        assert_eq!(gone, [9, 2, 3, 1]);
    }
}
