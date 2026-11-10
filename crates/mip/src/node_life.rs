//! Node lifetime: `5ff358` creates a node N with its info I at stage 5; `5f9d4e` sets a stage and cascades `5fa560`
//! frees up the parent links. Tracks which nodes still have N, I and the LP result block L (`I+0x08`); the payloads
//! stay with the caller, which `release` tells about every `5fa560` call (docs/m2/mip_node_life_rust.md).

#[derive(Clone, Copy, Debug)]
struct Node {
    /// N+0x10
    parent: Option<usize>,
    /// N+0x22 (short): children not yet freed at stage 0
    children: i16,
    /// I+0x6c: children still at stage 5
    pending: i32,
    /// I+0x68
    stage: i32,
    /// N+0x18 != 0
    info: bool,
    /// I+0x08 != 0
    lp: bool,
    /// N == M+0x790
    root: bool,
}

/// The nodes by id (creation order); a node freed at stage 0 is gone.
#[derive(Clone, Debug, Default)]
pub struct NodeLife {
    nodes: Vec<Option<Node>>,
}

impl NodeLife {
    fn node(&self, k: usize) -> &Node {
        self.nodes[k].as_ref().unwrap_or_else(|| panic!("node {k} was freed"))
    }

    fn node_mut(&mut self, k: usize) -> &mut Node {
        self.nodes[k].as_mut().unwrap_or_else(|| panic!("node {k} was freed"))
    }

    /// `5ff358(parent, M, out)`: a node at stage 5 with I and no L; `root` when `out` is `M+0x790`. The parent (which
    /// must have I) counts one more child and one more child at stage 5. Ids count up from 0.
    pub fn create(&mut self, parent: Option<usize>, root: bool) -> usize {
        if let Some(p) = parent {
            let p = self.node_mut(p);
            assert!(p.info, "5ff358 reads the parent's I");
            p.children += 1;
            p.pending += 1;
        }
        self.nodes.push(Some(Node { parent, children: 0, pending: 0, stage: 5, info: true, lp: false, root }));
        self.nodes.len() - 1
    }

    /// `5f9d4e(env, N, T, s)`; returns the node's stage before (`I+0x68`), None when it has no I.
    ///
    /// Stage 6 creates L when missing (`5ce67b` before each node LP) and leaves the stage. Stage -1 is 0. Any other
    /// stage is written and the cascade calls `free(x, s)` after each `5fa560(x, s)`: a node freed at stage 0 drops
    /// its parent's child count, and the parent is freed at stage 0 when that reaches 0; a node leaving stage 5 drops
    /// the parent's count of stage-5 children, and at 0 the parent's I is freed (stage 1).
    /// shortcut: the tree accounting of `5f9d4e` (T+0x10 nodes, T+0x20 bytes) is not kept; add it with node memory.
    pub fn release(&mut self, k: usize, s: i32, mut free: impl FnMut(usize, i32)) -> Option<i32> {
        let first = self.node(k);
        let before = first.info.then_some(first.stage);
        if s == 6 {
            assert!(first.info, "5f9d4e stage 6 reads I");
            self.node_mut(k).lp = true;
            return before;
        }
        let (mut x, mut s) = (k, if s == -1 { 0 } else { s });
        loop {
            let n = self.node_mut(x);
            let old = if n.info { std::mem::replace(&mut n.stage, s) } else { 1 };
            let parent = n.parent;
            self.free(x, s);
            free(x, s);
            let Some(p) = parent else { return before };
            let pn = self.node_mut(p);
            if s == 0 {
                pn.children -= 1;
            }
            if pn.children == 0 {
                (x, s) = (p, 0);
                continue;
            }
            if !pn.info || s > 4 {
                return before;
            }
            if old < 5 {
                return before;
            }
            pn.pending -= 1;
            if pn.pending != 0 {
                return before;
            }
            if pn.stage < 3 {
                return before;
            }
            (x, s) = (p, 1);
        }
    }

    /// `5fa560(env, X, s)`: stage 0 frees N (and everything); otherwise the root keeps all at stage > 0, and other
    /// nodes (the root too at stage < 0) lose L at stage <= 5 and I at stage <= 2.
    fn free(&mut self, x: usize, s: i32) {
        if s == 0 {
            self.nodes[x] = None;
            return;
        }
        let n = self.node_mut(x);
        if n.root && n.info && s > 0 {
            return;
        }
        if s <= 5 {
            n.lp = false;
        }
        if s <= 2 {
            n.info = false;
        }
    }

    pub fn alive(&self, k: usize) -> bool {
        self.nodes.get(k).is_some_and(Option::is_some)
    }

    /// N+0x18 != 0 (false once N is freed)
    pub fn has_info(&self, k: usize) -> bool {
        self.nodes.get(k).and_then(Option::as_ref).is_some_and(|n| n.info)
    }

    /// I+0x08 != 0
    pub fn has_lp(&self, k: usize) -> bool {
        self.nodes.get(k).and_then(Option::as_ref).is_some_and(|n| n.info && n.lp)
    }

    pub fn parent(&self, k: usize) -> Option<usize> {
        self.node(k).parent
    }

    /// The ancestors `6ee1d3` reads, parent first, up to the first one without I (excluded) or the root.
    pub fn ancestors(&self, k: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(self.parent(k), |&a| self.parent(a)).take_while(|&a| self.has_info(a))
    }
}
