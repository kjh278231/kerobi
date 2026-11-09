//! `3bbf87` / `3bde19`: new columns / rows at the end of the state (docs/m2_red_outer_a.md §3.5), with the tracker
//! extensions `535d61` / `536369`. Used by `3ff744`'s column aggregation; `594c0c` (PRESS2MIP) calls them too.
use crate::state::{State, INF, NIL};

impl State {
    /// `3bbf87(env, st, k)`: columns n..n+k, empty, continuous, bounds [0, INF], objective 0, flags 0, substitution
    /// multipliers 1; queued at tracker level 0 and marked (`535d61`). No work. Returns the first new index.
    pub fn add_cols(&mut self, k: usize) -> usize {
        let n0 = self.n;
        self.n += k;
        let n = self.n;
        self.col_head.resize(n, NIL);
        self.col_cnt.resize(n, 0);
        self.col_flag.resize(n, 0);
        self.lb.resize(n, 0.0);
        self.ub.resize(n, INF);
        self.obj.resize(n, 0.0);
        self.mult_lo.resize(n, 1.0);
        self.mult_hi.resize(n, 1.0);
        self.vtype.resize(n, b'C');
        if self.mark.len() < n {
            self.mark.resize(n, false);
        }
        let active = self.tracker.active;
        self.tracker.cols.grow(n0, k, &active);
        n0
    }

    /// `3bde19(env, st, k)`: rows m..m+k, empty `<` rows with rhs 0 and zero activities; queued at tracker level 0
    /// and marked (`536369`). No work. Returns the first new index.
    pub fn add_rows(&mut self, k: usize) -> usize {
        let m0 = self.m;
        self.m += k;
        let m = self.m;
        self.row_head.resize(m, NIL);
        self.row_cnt.resize(m, 0);
        self.inf_max.resize(m, 0);
        self.inf_min.resize(m, 0);
        self.act_max.resize(m, 0.0);
        self.act_min.resize(m, 0.0);
        self.rhs.resize(m, 0.0);
        self.sense.resize(m, b'<');
        self.row_int.resize(m, 0.0);
        if self.mark.len() < m {
            self.mark.resize(m, false);
        }
        let active = self.tracker.active;
        self.tracker.rows.grow(m0, k, &active);
        m0
    }
}

#[cfg(test)]
mod tests {
    use crate::tracker::Queue;

    #[test]
    fn grow_queues_and_marks() {
        let mut q = Queue::new(2);
        q.grow(2, 2, &[false, true, false]);
        assert_eq!((q.a[0], q.b[0]), (4, 4));
        assert_eq!(&q.list[0][..4], &[0, 1, 2, 3]);
        // level 1 active: the new indices go after `a` (next round), flag bit 1 set; level 2 inactive: current part
        assert_eq!((q.a[1], q.b[1]), (2, 4));
        assert_eq!((q.a[2], q.b[2]), (4, 4));
        assert_eq!(&q.flag[2..], &[2 | 4, 2 | 4]);
    }
}
