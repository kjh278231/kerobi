use rsolver_mip::strong_branch::{run, Backend, Batch, Bounds, Change, Error, LpResult, LpValue, Propagation, Side};
use std::{collections::VecDeque, fmt::Write};

fn number(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) }
fn dir(side: Side) -> u8 { if side == Side::Lower { 62 } else { 60 } }

/// Replays recorded backend responses in call order; a call of the wrong kind stops the batch.
/// Response lines: `prop out count (col flags lower upper)* work`, `solve status value iters work`,
/// `values v*`, `timer r work`, `restore work` (`-` for an absent value).
struct Replay {
    responses: VecDeque<Vec<String>>,
    trace: String,
}
impl Replay {
    fn next(&mut self, kind: &str) -> Result<Vec<String>, ()> {
        match self.responses.front() {
            Some(r) if r[0] == kind => Ok(self.responses.pop_front().unwrap()),
            _ => { writeln!(self.trace, "unexpected {kind}").unwrap(); Err(()) }
        }
    }
    fn work(&mut self, kind: &str, work: &mut f64) -> Result<Vec<String>, ()> {
        writeln!(self.trace, "{kind} {:016x}", work.to_bits()).unwrap();
        let r = self.next(kind)?;
        *work = number(r.last().unwrap());
        Ok(r)
    }
}
impl Backend for Replay {
    type Error = ();
    fn poll(&mut self) -> Result<(), ()> { self.trace.push_str("poll\n"); Ok(()) }
    fn expired(&mut self, work: &mut f64) -> Result<bool, ()> { Ok(self.work("timer", work)?[1] != "0") }
    fn propagate(&mut self, j: usize, side: Side, x: f64, work: &mut f64) -> Result<Propagation, ()> {
        write!(self.trace, "prop {j} {} {:016x} ", dir(side), x.to_bits()).unwrap();
        let r = self.work("prop", work)?;
        let changes = r[3..r.len()-1].chunks(4).map(|c| Change { column: c[0].parse().unwrap(),
            flags: c[1].parse().unwrap(), lower: number(&c[2]), upper: number(&c[3]) }).collect();
        Ok(Propagation { contradiction: r[1] != "0", changes })
    }
    fn flush(&mut self) -> Result<(), ()> { self.trace.push_str("flush\n"); Ok(()) }
    fn solve(&mut self, work: &mut f64) -> Result<LpResult, ()> {
        let r = self.work("solve", work)?;
        let v = |s: &str| if s == "-" { 0.0 } else { number(s) };
        let value = match r[1].as_str() {
            "2" => LpValue::Optimal(v(&r[2])), "3" | "6" => LpValue::Infeasible,
            "7" | "11" | "-1000" => LpValue::Limited(v(&r[2])),
            _ => panic!("uncovered status"),
        };
        Ok(LpResult { value, iterations: v(&r[3]) })
    }
    fn values(&mut self, js: &[usize]) -> Result<Vec<f64>, ()> {
        writeln!(self.trace, "values {}", js.iter().map(|j| j.to_string()).collect::<Vec<_>>().join(" ")).unwrap();
        Ok(self.next("values")?[1..].iter().map(|s| number(s)).collect())
    }
    fn reset(&mut self) -> Result<(), ()> { self.trace.push_str("reset\n"); Ok(()) }
    fn restore(&mut self, work: &mut f64) -> Result<(), ()> { self.work("restore", work).map(|_| ()) }
    fn bound(&mut self, j: usize, side: Side, x: f64) -> Result<(), ()> {
        writeln!(self.trace, "bound {j} {} {:016x}", dir(side), x.to_bits()).unwrap();
        Ok(())
    }
}

fn bits(v: &[f64]) -> String { v.iter().map(|x| format!("{:016x}", x.to_bits())).collect::<Vec<_>>().join(" ") }

/// Input: `batch id stall tol parent cost0 work0`, then `cols`, `x`, `lower`, `upper`
/// (by column), `path`, then responses, then `end`.
#[test]
#[ignore = "run by tools/mip_sb_loop_rust.py with original inputs"]
fn replay() {
    let input = std::fs::read_to_string(std::env::var("SB_LOOP_INPUT").unwrap()).unwrap();
    let mut lines = input.lines().map(|l| l.split_whitespace().map(str::to_string).collect::<Vec<_>>());
    let mut trace = String::new();
    while let Some(h) = lines.next() {
        assert_eq!(h[0], "batch");
        let mut arr = |tag: &str| { let l = lines.next().unwrap(); assert_eq!(l[0], tag); l[1..].to_vec() };
        let cols: Vec<usize> = arr("cols").iter().map(|s| s.parse().unwrap()).collect();
        let [x, lower, upper] = ["x", "lower", "upper"].map(|t| arr(t).iter().map(|s| number(s)).collect::<Vec<_>>());
        // `path prop` or `path direct infinity scale*` (scale by column).
        let path = arr("path");
        let scale: Vec<f64> = path.iter().skip(2).map(|s| number(s)).collect();
        let bounds = match path[0].as_str() {
            "prop" => Bounds::Propagate { lower: &lower, upper: &upper },
            "direct" => Bounds::Direct { lower: &lower, upper: &upper, scale: Some(&scale), infinity: number(&path[1]) },
            p => panic!("path {p}"),
        };
        let mut responses = VecDeque::new();
        for l in lines.by_ref() {
            if l[0] == "end" { break; }
            responses.push_back(l);
        }
        let b = Batch { columns: &cols, x: &x, bounds, int_tol: number(&h[3]),
            parent_obj: number(&h[4]), stall_limit: h[2].parse().unwrap() };
        let mut lp = Replay { responses, trace: String::new() };
        let (mut cost, mut work) = (number(&h[5]), number(&h[6]));
        let r = run(&b, &mut lp, &mut cost, &mut work);
        writeln!(trace, "case {}", h[1]).unwrap();
        trace.push_str(&lp.trace);
        match r {
            Ok(r) => writeln!(trace, "result {} | {} | {:016x}", bits(&r.down), bits(&r.up), cost.to_bits()).unwrap(),
            Err(e) => writeln!(trace, "error {e:?}").unwrap(),
        }
        if !lp.responses.is_empty() {
            writeln!(trace, "unused {}", lp.responses.len()).unwrap();
        }
    }
    std::fs::write(std::env::var("SB_LOOP_OUTPUT").unwrap(), trace).unwrap();
}

// Synthetic C boundary checks; these do not extend the original-run coverage.
fn script(lines: &[&str]) -> Replay {
    Replay { responses: lines.iter().map(|l| l.split_whitespace().map(str::to_string).collect()).collect(),
        trace: String::new() }
}

#[test]
fn clamp_and_signed_zero() {
    // Status 2 below the parent and status 7 with the dual below it both take the parent.
    let x = [0.5];
    let b = Batch { columns: &[0], x: &x, bounds: Bounds::Propagate { lower: &[0.0], upper: &[1.0] }, int_tol: 1e-5, parent_obj: 1.0, stall_limit: 2 };
    let mut lp = script(&["prop 0 0 0", "solve 2 0000000000000000 - 0", "restore 0",
        "prop 0 0 0", "solve 7 3fe0000000000000 - 0", "restore 0", "solve 2 - - 0"]);
    let (mut cost, mut work) = (0.0, 0.0);
    let r = run(&b, &mut lp, &mut cost, &mut work).unwrap();
    assert_eq!((r.down[0], r.up[0]), (1.0, 1.0));
    let b = Batch { parent_obj: -0.0, ..b };
    let mut lp = script(&["prop 0 0 0", "solve 2 0000000000000000 - 0", "restore 0",
        "prop 0 0 0", "solve 7 0000000000000000 - 0", "restore 0", "solve 2 - - 0"]);
    let r = run(&b, &mut lp, &mut cost, &mut work).unwrap();
    assert_eq!(r.up[0].to_bits(), (-0.0f64).to_bits());
    assert_eq!(r.down[0].to_bits(), 0.0f64.to_bits());
}

#[test]
fn contradiction_unported_timer_stops() {
    let x = [0.5, 0.5];
    let b = Batch { columns: &[0, 1], x: &x, bounds: Bounds::Propagate { lower: &[0.0; 2], upper: &[1.0; 2] }, int_tol: 1e-5, parent_obj: 0.0, stall_limit: 2 };
    let mut lp = script(&["prop 1 0 0"]);
    assert!(matches!(run(&b, &mut lp, &mut 0.0, &mut 0.0), Err(Error::Unported)));
    // Down infeasible resets the stall counter and moves on; an expired timer ends the batch
    // with the closing flush and re-solve, leaving later candidates unevaluated.
    let mut lp = script(&["prop 0 0 0", "solve 7 3ff0000000000000 - 0", "restore 0",
        "prop 0 0 0", "solve 6 - - 0", "restore 0", "timer 1 3ff0000000000000", "solve 2 - - 4000000000000000"]);
    let mut work = 0.0;
    let r = run(&b, &mut lp, &mut 0.0, &mut work).unwrap();
    assert!(lp.trace.ends_with("poll\ntimer 0000000000000000\nflush\nsolve 3ff0000000000000\n"));
    assert_eq!((r.up[0], r.down[0]), (1.0, 1e100));
    assert_eq!((r.up[1], r.down[1]), (-1e100, -1e100));
    assert_eq!(work, 2.0);
}

#[test]
fn restore_requests_only_tightened_bounds() {
    // Column 1 keeps its original bounds although both flags are set; column 2 was tightened.
    let x = [0.5, 0.0, 0.0];
    let b = Batch { columns: &[0], x: &x, bounds: Bounds::Propagate { lower: &[0.0, 0.0, 0.0], upper: &[1.0, 4.0, 4.0] }, int_tol: 1e-5,
        parent_obj: 0.0, stall_limit: 2 };
    let mut lp = script(&["prop 0 3 0 1 3ff0000000000000 3ff0000000000000 1 3 0000000000000000 4010000000000000 2 3 3ff0000000000000 4000000000000000 0",
        "solve 7 0000000000000000 - 0", "restore 0",
        "prop 0 1 0 2 0000000000000000 0000000000000000 0", "solve 7 0000000000000000 - 0", "restore 0", "solve 2 - - 0"]);
    run(&b, &mut lp, &mut 0.0, &mut 0.0).unwrap();
    let requests: Vec<_> = lp.trace.lines().filter(|l| l.starts_with("bound")).collect();
    assert_eq!(requests, ["bound 0 62 0000000000000000", "bound 2 62 0000000000000000", "bound 2 60 4010000000000000",
        "bound 0 60 3ff0000000000000"]);
}

#[test]
fn repeated_change_and_up_prefill() {
    // A repeated list entry requests nothing more but still counts 6 work.
    let x = [0.5, 0.5];
    let b = Batch { columns: &[0, 1], x: &x, bounds: Bounds::Propagate { lower: &[0.0; 2], upper: &[1.0; 2] }, int_tol: 1e-5,
        parent_obj: 0.0, stall_limit: 9 };
    // Up LP of candidate 0 is optimal at the parent value with candidate 1 at 1.0 (its ceil): up[1] = parent.
    let mut lp = script(&["prop 0 2 0 1 3ff0000000000000 3ff0000000000000 0 1 3ff0000000000000 3ff0000000000000 0",
        "solve 2 0000000000000000 - 0", "values 3ff0000000000000", "restore 0",
        "prop 0 0 0", "solve 7 0000000000000000 - 0", "restore 0", "timer 0 0",
        "prop 0 0 0", "solve 7 0000000000000000 - 0", "restore 0", "solve 2 - - 0"]);
    let mut work = 0.0;
    let r = run(&b, &mut lp, &mut 0.0, &mut work).unwrap();
    assert_eq!(lp.trace.matches("bound 0 62").count(), 1);
    assert!(lp.trace.contains("prop 0 60 0000000000000000 prop 4028000000000000"));
    // Candidate 1 skips its up solve: only the down propagation follows the timer.
    assert!(lp.trace.contains("timer 0000000000000000\nprop 1 60 "));
    assert_eq!((r.up[1], r.down[1]), (0.0, 0.0));
}

#[test]
fn direct_requests_and_restores_own_bound() {
    // No propagation: the candidate's bound is requested, then requested back unconditionally
    // from the internal bounds (scaled; beyond infinity -> +-1e100) with no work added.
    let x = [0.5, 2.5];
    let (lower, upper, scale) = ([-1e30, -0.0], [4.0, 1e30], [0.5, 2.0]);
    let b = Batch { columns: &[0, 1], x: &x, bounds: Bounds::Direct { lower: &lower, upper: &upper,
        scale: Some(&scale), infinity: 1e30 }, int_tol: 1e-5, parent_obj: 0.0, stall_limit: 9 };
    let mut lp = script(&["solve 7 0000000000000000 - 0", "restore 0", "solve 7 0000000000000000 - 0", "restore 0",
        "timer 0 0", "solve 6 - - 0", "restore 0", "solve 2 - - 0"]);
    let mut work = 0.0;
    run(&b, &mut lp, &mut 0.0, &mut work).unwrap();
    let requests: Vec<_> = lp.trace.lines().filter(|l| l.starts_with("bound")).collect();
    assert_eq!(requests, ["bound 0 62 3ff0000000000000", "bound 0 62 d4b249ad2594c37d", "bound 0 60 0000000000000000",
        "bound 0 60 4000000000000000", "bound 1 62 4008000000000000", "bound 1 62 8000000000000000"]);
    assert!(!lp.trace.contains("prop"));
    assert_eq!(work, 0.0);
}
