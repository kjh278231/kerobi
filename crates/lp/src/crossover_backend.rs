//! Serial numeric backend of the primal push loop on the original factor layout: `crossover_lu` (FTRAN `2207de`,
//! support BTRAN `221173`, Forrest–Tomlin update and renumbering of `05b257`) and `crossover_refactor` (`036ade` with
//! the `1da0b2` block phase and the setup after it), with the factor statistics the selector reads. The quality helper
//! `089f61`, the update's failure/reallocation paths and the pricing objects are not implemented: reaching one panics.
//! Contract and C references: docs/m3/primal_push_loop_lu.md.
use crate::crossover::PushState;
use crate::crossover_ftran::VecSnap;
use crate::crossover_loop::{FactorCtl, FactorSite, FactorStats, PushLoopBackend, Renumber};
use crate::crossover_lu::LuState;
use crate::crossover_refactor::{refactor, Alloc, Matrix};
use crate::crossover_supplier::PushColumn;
use crate::lu::BlockState;

pub struct SerialBackend {
    pub lu: LuState,
    pub matrix: Matrix,
    /// Markowitz threshold of `036ade`.
    pub tol: f64,
    /// `1da0b2` block state on S (S+0x2c0/0x4e0/0x4e4) the next `036ade` enters with.
    pub block: BlockState,
    /// S+0x46c (never written inside the loop).
    pub interval: i32,
    /// S+0x3a8, the threshold of the dense work-column count.
    pub drop: f64,
    /// F+0x190: work-column counts accumulated by the updates since the last refactor.
    pub column_work: i64,
    /// F+0x1e8.
    pub u_nnz: i64,
}

/// What a refactor hands back besides the new factor: the renumbered push state of `LuState::renumber`.
pub struct Factored {
    pub backend: SerialBackend,
    pub header: Vec<i32>,
    pub leave: i32,
    pub vstat: Vec<i32>,
}

/// F+0x1e8 after a refactor (`05b257` C:1978–1986): F+0xb0[m] + m, with F+0xb0[m] the end of the packed U rows.
fn u_stat(lu: &LuState) -> i64 {
    let m = lu.m;
    if m == 0 {
        0
    } else {
        lu.re[m - 1] + m as i64
    }
}

/// L statistic F+0x1e0 = m + F+0x58[m] (C:1985), as the refactor already sets it.
fn l_stat(lu: &LuState) -> i64 {
    if lu.m == 0 {
        0
    } else {
        lu.sc.x1e0
    }
}

impl SerialBackend {
    /// The push's first factor call (request 10 before the loop): refactor `header` from no previous factor with the
    /// allocation `alloc`, then renumber the push state.
    #[allow(clippy::too_many_arguments)]
    pub fn initial(matrix: Matrix, zero: f64, tol: f64, alloc: Alloc, block: BlockState, interval: i32, drop: f64,
                   header: &[i32], leave: i32, vstat: &[i32]) -> Factored {
        let mut block = block;
        let rf = refactor(&matrix, header, zero, tol, alloc, None, &mut block).expect("initial refactor");
        // 05b257 C:1399 raises S+0x2c0 before each refactor; the next one enters with it raised
        block.x2c0 += 1;
        let mut lu = rf.state;
        let r = lu.renumber(header, leave, vstat, &rf.o, &rf.n);
        let u_nnz = u_stat(&lu);
        Factored {
            backend: SerialBackend { lu, matrix, tol, block, interval, drop, column_work: 0, u_nnz },
            header: r.header,
            leave: r.leave,
            vstat: r.vstat,
        }
    }

    /// `05b257` C:144–251: the update adds the work column's count + 1 (dense: 1 + its entries above S+0x3a8).
    fn add_column_work(&mut self, w: &PushColumn) {
        let c = if w.count < 0 {
            1 + w.values.iter().filter(|v| self.drop < v.abs()).count() as i64
        } else {
            i64::from(w.count) + 1
        };
        self.column_work += c;
    }
}

/// An FTRAN/BTRAN output header as a push column; every listed cell must be known.
fn column(v: &VecSnap, what: &str) -> PushColumn {
    let count = v.count.unwrap_or_else(|| panic!("{what}: unknown output count"));
    let values: Vec<f64> = v.val.iter().map(|x| x.unwrap_or_else(|| panic!("{what}: unknown output value"))).collect();
    let indices = match &v.idx {
        Some(ix) => ix.iter().map(|x| x.unwrap_or_else(|| panic!("{what}: unknown output index"))).collect(),
        None => Vec::new(),
    };
    PushColumn { count, indices, values }
}

impl PushLoopBackend for SerialBackend {
    fn ftran(&mut self, input: &PushColumn, output: &mut PushColumn, spike: Option<&mut PushColumn>) {
        // 09d008 passes F+0x1c8 with save 1 when the supplier hands a spike (support 0), else null and save 0
        let with_spike = spike.is_some();
        if with_spike {
            self.lu.wrapper_clear();
        }
        let s = self
            .lu
            .ftran(input.count, &input.indices, &input.values, with_spike as i32, with_spike)
            .unwrap_or_else(|e| panic!("supplier FTRAN: {e:?}"));
        *output = column(&s.snapshot(self.lu.m).0, "supplier FTRAN");
    }

    fn btran(&mut self, input: &PushColumn) -> PushColumn {
        // the output header's entry count only reaches the input header after dense Lᵀ; the raw does not read it
        let s = self.lu.btran(input.count, &input.indices, &input.values, 0, true, 0.0);
        column(&s.snapshot(self.lu.m).0, "support BTRAN")
    }

    fn stats(&self) -> FactorStats {
        FactorStats {
            updates: self.lu.sc.x18c,
            limit: self.lu.sc.x188,
            interval: self.interval,
            column_work: self.column_work,
            eta_count: self.lu.sc.x184,
            l_nnz: l_stat(&self.lu),
            u_nnz: self.u_nnz,
        }
    }

    fn factor(&mut self, _site: FactorSite, ctl: &mut FactorCtl, st: &PushState, workspace: Option<&PushColumn>)
              -> (i32, Option<Renumber>) {
        if ctl.request == 0 {
            self.add_column_work(workspace.expect("update without a work column"));
            self.lu.header = st.header.clone();
            assert!(st.leave >= 0, "update without a leaving position");
            self.lu.ft_update(st.leave as usize).unwrap_or_else(|e| panic!("FT update: {e:?}"));
            return (0, None);
        }
        // C:1372–1379: F+0x1f0 = F+0x190 / k (not read here) and F+0x190 = 0 before the refactor
        self.column_work = 0;
        let alloc = Alloc::of(&self.lu);
        let rf = refactor(&self.matrix, &st.header, self.lu.zero, self.tol, alloc, Some(&self.lu), &mut self.block)
            .unwrap_or_else(|e| panic!("refactor: {e:?}"));
        self.block.x2c0 += 1;
        let mut lu = rf.state;
        lu.renumber(&st.header, st.leave, &st.vstat, &rf.o, &rf.n);
        self.u_nnz = u_stat(&lu);
        self.lu = lu;
        (0, Some(Renumber { old_of_new: rf.o, new_of_old: rf.n }))
    }

    fn quality(&mut self, _: &mut FactorCtl, _: i32, _: i32) -> i32 {
        panic!("quality helper 089f61 is not installed (M3-04c-15a)")
    }

    fn recompute_ftran(&mut self, rhs: &[f64], raw: &mut [f64]) {
        let s = self.lu.ftran(-1, &[], rhs, 0, false).unwrap_or_else(|e| panic!("RHS FTRAN: {e:?}"));
        let out = column(&s.snapshot(self.lu.m).0, "RHS FTRAN");
        assert_eq!(out.count, -1, "RHS FTRAN output not dense");
        raw.copy_from_slice(&out.values);
    }
}
