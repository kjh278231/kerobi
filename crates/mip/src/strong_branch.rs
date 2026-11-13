//! One strong-branching batch: the candidate loop with an external LP, propagation and
//! saved-state backend. The backend owns the LP numbers, the propagation itself, the parent
//! snapshot and the timer; this module owns the candidate control and the values it writes.
//! shortcut: integer, unflipped candidates only. Propagation contradiction returns Unported
//! (no original case yet); upgrade when one is captured.

const INF: f64 = 1e100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side { Lower, Upper }

/// Bounds a direction restores, indexed by column.
#[derive(Clone, Copy)]
pub enum Bounds<'a> {
    /// Node with propagation data: original unscaled bounds from the getters; only bounds the
    /// propagation tightened are requested back.
    Propagate { lower: &'a [f64], upper: &'a [f64] },
    /// No node (VarBranch=3): the LP's internal bounds, column scale and infinity. The candidate's
    /// own bound is requested directly and requested back unconditionally.
    Direct { lower: &'a [f64], upper: &'a [f64], scale: Option<&'a [f64]>, infinity: f64 },
}

/// Inputs read before the loop. Arrays indexed by column hold the parent LP solution;
/// `columns` is the candidate order (all integer columns).
pub struct Batch<'a> {
    pub columns: &'a [usize],
    pub x: &'a [f64],
    pub bounds: Bounds<'a>,
    pub int_tol: f64,
    pub parent_obj: f64,
    pub stall_limit: i32,
}

#[derive(Clone, Copy, Debug)]
pub enum LpValue {
    /// Status 2: objective value.
    Optimal(f64),
    /// Status 7, 11 or -1000: dual bound of the LP workspace.
    Limited(f64),
    /// Status 3 or 6.
    Infeasible,
}

#[derive(Clone, Copy, Debug)]
pub struct LpResult {
    pub value: LpValue,
    pub iterations: f64,
}

/// Flag bit 1: lower bound changed, bit 2: upper bound changed. `lower`/`upper` are the
/// bounds after propagation; restoration requests the original value where it was tightened.
#[derive(Clone, Copy, Debug)]
pub struct Change {
    pub column: usize,
    pub flags: u8,
    pub lower: f64,
    pub upper: f64,
}

#[derive(Clone, Debug)]
pub struct Propagation {
    pub contradiction: bool,
    pub changes: Vec<Change>,
}

/// Calls taking `work` may add their own work. Bounds are unscaled requests that the next
/// flush applies.
pub trait Backend {
    type Error;
    fn poll(&mut self) -> Result<(), Self::Error>;
    fn expired(&mut self, work: &mut f64) -> Result<bool, Self::Error>;
    fn propagate(&mut self, column: usize, side: Side, value: f64, work: &mut f64)
        -> Result<Propagation, Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
    fn solve(&mut self, work: &mut f64) -> Result<LpResult, Self::Error>;
    /// Current LP solution at `columns`.
    fn values(&mut self, columns: &[usize]) -> Result<Vec<f64>, Self::Error>;
    fn reset(&mut self) -> Result<(), Self::Error>;
    fn restore(&mut self, work: &mut f64) -> Result<(), Self::Error>;
    fn bound(&mut self, column: usize, side: Side, value: f64) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug)]
pub struct Evaluation {
    pub down: Vec<f64>,
    pub up: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    Backend(E),
    Unported,
}

/// maxsd(gain, 1e-10) on both gains, then the geometric mean shifted by 1e-5.
fn score(a: f64, b: f64) -> f64 {
    let a = if a > 1e-10 { a } else { 1e-10 };
    let b = if b > 1e-10 { b } else { 1e-10 };
    ((a + 1e-5) * (b + 1e-5)).sqrt() - 1e-5
}

struct Loop<'a, B: Backend> {
    b: &'a Batch<'a>,
    lp: &'a mut B,
    iterations: &'a mut f64,
    work: &'a mut f64,
    down: Vec<f64>,
    up: Vec<f64>,
    down_est: Vec<f64>,
    up_est: Vec<f64>,
    eps: f64,
}

impl<B: Backend> Loop<'_, B> {
    /// One direction: propagate (or request the bound), solve, consume, pre-fill later candidates, restore.
    fn direction(&mut self, k: usize, side: Side, value: f64) -> Result<(), Error<B::Error>> {
        let parent = self.b.parent_obj;
        let j = self.b.columns[k];
        let p = match self.b.bounds {
            Bounds::Propagate { .. } => {
                let p = self.lp.propagate(j, side, value, self.work).map_err(Error::Backend)?;
                if p.contradiction {
                    return Err(Error::Unported);
                }
                Some(p)
            }
            Bounds::Direct { .. } => {
                self.lp.bound(j, side, value).map_err(Error::Backend)?;
                None
            }
        };
        self.lp.flush().map_err(Error::Backend)?;
        let r = self.lp.solve(self.work).map_err(Error::Backend)?;
        *self.iterations += r.iterations;
        let v = match r.value {
            // maxsd operand order, as the original.
            LpValue::Optimal(obj) => if obj > parent { obj } else { parent },
            LpValue::Infeasible => INF,
            LpValue::Limited(dual) => if parent > dual { parent } else { dual },
        };
        if side == Side::Lower { self.up[k] = v } else { self.down[k] = v }
        if let LpValue::Optimal(_) = r.value {
            self.prefill(k, v)?;
        }
        self.lp.reset().map_err(Error::Backend)?;
        self.lp.restore(self.work).map_err(Error::Backend)?;
        let (p, lower, upper) = match (p, self.b.bounds) {
            (Some(p), Bounds::Propagate { lower, upper }) => (p, lower, upper),
            (_, Bounds::Direct { lower, upper, scale, infinity }) => {
                // Read at candidate start (C:410-427): beyond +-infinity becomes +-1e100, else scaled.
                let s = |v: f64| match scale { Some(s) => v * s[j], None => v };
                let v = if side == Side::Lower {
                    if -infinity < lower[j] { s(lower[j]) } else { -INF }
                } else if upper[j] < infinity { s(upper[j]) } else { INF };
                return self.lp.bound(j, side, v).map_err(Error::Backend);
            }
            _ => unreachable!(),
        };
        // The original clears a column's flags after its first entry, so a repeated entry is a no-op.
        let mut done: Vec<usize> = Vec::new();
        for c in &p.changes {
            if done.contains(&c.column) {
                continue;
            }
            done.push(c.column);
            let (lo, up) = (lower[c.column], upper[c.column]);
            if c.flags & 1 != 0 && lo < c.lower {
                self.lp.bound(c.column, Side::Lower, lo).map_err(Error::Backend)?;
            }
            if c.flags & 2 != 0 && c.upper < up {
                self.lp.bound(c.column, Side::Upper, up).map_err(Error::Backend)?;
            }
        }
        *self.work += p.changes.len() as f64 * 6.0;
        Ok(())
    }

    /// An optimal direction LP is feasible for a later candidate's direction when that candidate
    /// sits at its parent floor/ceil there: cap that estimate, and take the parent value when the gain is nil.
    fn prefill(&mut self, k: usize, obj: f64) -> Result<(), Error<B::Error>> {
        let later = &self.b.columns[k + 1..];
        if later.is_empty() {
            return Ok(());
        }
        let v = self.lp.values(later).map_err(Error::Backend)?;
        let parent = self.b.parent_obj;
        let nil = self.eps > obj - parent;
        for (i, (&j, &v)) in later.iter().zip(&v).enumerate() {
            let kk = k + 1 + i;
            let x = self.b.x[j];
            if x.floor() + 1e-10 >= v {
                if nil { self.down[kk] = parent; }
                self.down_est[kk] = if self.down_est[kk] < obj { self.down_est[kk] } else { obj };
            }
            if v >= x.ceil() - 1e-10 {
                if nil { self.up[kk] = parent; }
                self.up_est[kk] = if self.up_est[kk] < obj { self.up_est[kk] } else { obj };
            }
        }
        Ok(())
    }
}

/// Starts after the parent snapshot, ObjVal, solution and original bounds are read; ends after
/// the closing flush and parent re-solve. Backend errors return immediately without restoration.
pub fn run<B: Backend>(b: &Batch, lp: &mut B, iterations: &mut f64, work: &mut f64)
    -> Result<Evaluation, Error<B::Error>>
{
    let n = b.columns.len();
    let parent = b.parent_obj;
    let mut s = Loop { b, lp, iterations, work, down: vec![-INF; n], up: vec![-INF; n],
        down_est: vec![INF; n], up_est: vec![INF; n], eps: (parent.abs() + 1.0) * 1e-15 };
    let (mut best, mut stall) = (0.0, 0);
    if n > 0 {
        s.lp.poll().map_err(Error::Backend)?;
    }
    let mut k = 0;
    while k < n {
        let split = (b.x[b.columns[k]] + b.int_tol).floor();
        // Both estimates come from earlier optimal LPs; skip the candidate if they cannot beat best.
        let gated = s.up[k] <= -INF && s.down_est[k] < INF && s.up_est[k] < INF
            && best >= score(s.down_est[k] - parent, s.up_est[k] - parent);
        let mut judge = true;
        if !gated {
            if s.up[k] <= -INF {
                s.up[k] = parent;
                s.direction(k, Side::Lower, split + 1.0)?;
            }
            if s.up[k] >= INF {
                judge = false;
            } else if s.down[k] <= -INF && !(s.down_est[k] < INF
                && best >= score(s.down_est[k] - parent, s.up[k] - parent)) {
                s.down[k] = parent;
                s.direction(k, Side::Upper, split)?;
            }
        }
        if judge {
            let (dn, up) = (s.down[k], s.up[k]);
            if up >= INF && dn >= INF {
                break;
            }
            if dn >= INF || up >= INF {
                stall = 0;
            } else {
                let v = score(dn - parent, up - parent);
                if v > best {
                    best = v;
                    stall = 0;
                } else {
                    stall += 1;
                    if stall >= b.stall_limit {
                        break;
                    }
                }
            }
        }
        k += 1;
        if k >= n {
            break;
        }
        s.lp.poll().map_err(Error::Backend)?;
        // Expiry leaves like the stall exit: closing flush and parent re-solve, values so far.
        if s.lp.expired(s.work).map_err(Error::Backend)? {
            break;
        }
    }
    s.lp.flush().map_err(Error::Backend)?;
    s.lp.solve(s.work).map_err(Error::Backend)?;
    Ok(Evaluation { down: s.down, up: s.up })
}
