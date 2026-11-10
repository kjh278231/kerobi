//! Node LP control of `5ce67b` for a node that holds a stored basis: the basis and method handed to the node LP and to
//! the numeric re-solve, and what the re-solve result does to the node. The LP is supplied by the caller, so the same
//! control serves the B&B and a replay of original LP outcomes.
//! shortcut: only paths with original evidence are ported; the rest return `Unported` and the caller keeps its
//! previous handling. Not called from the B&B yet: it needs node bases (M2-10b) and a primal warm start and LP cutoff
//! status (M1); upgrade then.

/// Gurobi LP status codes used here.
pub const OPTIMAL: i32 = 2;
pub const CUTOFF: i32 = 6;
pub const NUMERIC: i32 = 12;
/// Node status written by `600d25` for an LP that ended in CUTOFF.
pub const NODE_CUTOFF: i32 = 5;

/// Stored node basis (`I+0x98/0xa0`) in Gurobi codes: structurals 0, −1, −2, −3; rows 0, −1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeBasis {
    pub vbasis: Vec<i32>,
    pub cbasis: Vec<i32>,
}

/// What the control reads besides the basis.
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    /// The node is the root of its MIP (`node == M+0x790`).
    pub root: bool,
    /// Root LP status cache `M+0xa8`.
    pub cache: i32,
    /// Warm-start LP method `param_3+0x18`.
    pub warm_method: i32,
}

/// One LP solve request: method and the basis it starts from.
#[derive(Clone, Copy, Debug)]
pub struct LpCall<'a> {
    pub method: i32,
    pub basis: &'a NodeBasis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The node LP ended in 12 and is not re-solved (not the root, root cache not 2; C:769-770).
    Solved { lp_status: i32 },
    /// Re-solved; node status and whether the node basis is stored again (`5d4427`).
    Retried { node_status: i32, lp_status: i32, basis_saved: bool },
    /// A path that is not ported; the caller keeps its previous handling. Covers any first status other than 12
    /// (the normalization before the re-solve test, C:678-760, is not ported: e.g. 5 with cache 2 becomes 12 and is
    /// re-solved), the root (C:921 also resets the MIP status 5 to 1 there) and re-solve results other than 6.
    Unported { lp_status: i32 },
}

/// `5ce67b` with a stored node basis, no dual norms, no adaptive method choice (`param_7 == 0`) and `a7f923 == 0`.
pub fn process_node_lp(ctx: &Ctx, basis: &NodeBasis, mut solve: impl FnMut(LpCall) -> i32) -> Outcome {
    // C:351-364: a node basis selects the warm method param_3[3]; above 2 it is 1 away from the root
    let method = if ctx.warm_method > 2 && !ctx.root { 1 } else { ctx.warm_method };
    let status = solve(LpCall { method, basis });
    if status != NUMERIC || ctx.root {
        return Outcome::Unported { lp_status: status };
    }
    // C:769-770: away from the root, numerical trouble is re-solved only when the root cache is 2
    if ctx.cache != OPTIMAL {
        return Outcome::Solved { lp_status: status };
    }
    // C:924-933: same node basis, the other simplex from the unclamped param_3[3] (0 picks dual)
    let status = solve(LpCall { method: i32::from(ctx.warm_method == 0), basis });
    // C:944-1035 with cache 2 leaves CUTOFF as is (no bound improvement: local_b8 is 1e101 on this path), then
    // 600d25 prunes the node; only LP status 2 stores the basis (C:1358)
    if status == CUTOFF {
        return Outcome::Retried { node_status: NODE_CUTOFF, lp_status: status, basis_saved: false };
    }
    Outcome::Unported { lp_status: status }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(ctx: Ctx, statuses: &[i32]) -> (Outcome, Vec<i32>) {
        let basis = NodeBasis { vbasis: vec![0, -1], cbasis: vec![0] };
        let mut methods = Vec::new();
        let out = process_node_lp(&ctx, &basis, |call| {
            assert_eq!(call.basis, &basis);
            methods.push(call.method);
            statuses[methods.len() - 1]
        });
        (out, methods)
    }

    // Synthetic branch checks of the C rules; the original-run evidence is the L1 fixture (tests/retry_l1.rs).
    #[test]
    fn retry_rules() {
        let child = Ctx { root: false, cache: 2, warm_method: 1 };
        assert_eq!(run(child, &[2]), (Outcome::Unported { lp_status: 2 }, vec![1]));
        assert_eq!(run(child, &[5]), (Outcome::Unported { lp_status: 5 }, vec![1]));
        assert_eq!(run(Ctx { warm_method: 3, ..child }, &[12, 6]).1, vec![1, 0]);
        assert_eq!(run(child, &[12, 6]), (Outcome::Retried { node_status: 5, lp_status: 6, basis_saved: false }, vec![1, 0]));
        assert_eq!(run(Ctx { warm_method: 0, ..child }, &[12, 6]).1, vec![0, 1]);
        assert_eq!(run(child, &[12, 2]).0, Outcome::Unported { lp_status: 2 });
        assert_eq!(run(child, &[12, 12]).0, Outcome::Unported { lp_status: 12 });
        assert_eq!(run(Ctx { cache: -1, ..child }, &[12]), (Outcome::Solved { lp_status: 12 }, vec![1]));
        assert_eq!(run(Ctx { root: true, ..child }, &[12]), (Outcome::Unported { lp_status: 12 }, vec![1]));
        assert_eq!(run(Ctx { root: true, warm_method: 3, ..child }, &[2]).1, vec![3]);
    }
}
