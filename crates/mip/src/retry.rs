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
    /// Solved once without a re-solve; the caller handles `lp_status` as before.
    Solved { lp_status: i32 },
    /// Re-solved; node status and whether the node basis is stored again (`5d4427`).
    Retried { node_status: i32, lp_status: i32, basis_saved: bool },
    /// The re-solve ended on a path that is not ported; the caller keeps its previous handling.
    Unported { lp_status: i32 },
}

/// `5ce67b` with a stored node basis, no dual norms, no adaptive method choice (`param_7 == 0`) and `a7f923 == 0`.
pub fn process_node_lp(ctx: &Ctx, basis: &NodeBasis, mut solve: impl FnMut(LpCall) -> i32) -> Outcome {
    // C:351-361: a node basis selects the warm method param_3[3]
    let status = solve(LpCall { method: ctx.warm_method, basis });
    // C:769-770: numerical trouble is re-solved when the root cache is 2 or at the root
    if status != NUMERIC || !(ctx.cache == OPTIMAL || ctx.root) {
        return Outcome::Solved { lp_status: status };
    }
    // C:924-933: same node basis, the other simplex (param_3[3] == 0 picks dual)
    let status = solve(LpCall { method: i32::from(ctx.warm_method == 0), basis });
    // C:944-1035 with cache 2 leaves CUTOFF as is (no bound improvement: local_b8 is 1e101 on this path), then
    // 600d25 prunes the node; only LP status 2 stores the basis (C:1358)
    if status == CUTOFF && ctx.cache == OPTIMAL {
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
        assert_eq!(run(child, &[2]), (Outcome::Solved { lp_status: 2 }, vec![1]));
        assert_eq!(run(child, &[12, 6]), (Outcome::Retried { node_status: 5, lp_status: 6, basis_saved: false }, vec![1, 0]));
        assert_eq!(run(Ctx { warm_method: 0, ..child }, &[12, 6]).1, vec![0, 1]);
        assert_eq!(run(child, &[12, 2]).0, Outcome::Unported { lp_status: 2 });
        assert_eq!(run(child, &[12, 12]).0, Outcome::Unported { lp_status: 12 });
        assert_eq!(run(Ctx { cache: -1, ..child }, &[12]), (Outcome::Solved { lp_status: 12 }, vec![1]));
        assert_eq!(run(Ctx { root: true, cache: -1, ..child }, &[12, 6]), (Outcome::Unported { lp_status: 6 }, vec![1, 0]));
    }
}
