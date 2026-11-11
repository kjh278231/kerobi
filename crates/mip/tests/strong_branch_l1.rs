use rsolver_mip::strong_branch::{evaluate, Backend, Candidate, LpResult, LpValue, Side};
use std::{collections::VecDeque, fmt::Write};

fn number(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) }

struct Replay {
    outcomes: VecDeque<LpResult>,
    trace: String,
}
impl Backend for Replay {
    type Error = ();
    fn bound(&mut self, j: usize, side: Side, x: f64) -> Result<(), ()> {
        writeln!(self.trace, "bound {j} {} {:016x}", if side == Side::Lower {62} else {60}, x.to_bits()).unwrap();
        Ok(())
    }
    fn flush(&mut self) -> Result<(), ()> { self.trace.push_str("flush\n"); Ok(()) }
    fn solve(&mut self) -> Result<LpResult, ()> {
        self.trace.push_str("solve\n");
        Ok(self.outcomes.pop_front().expect("unexpected solve"))
    }
    fn reset(&mut self) -> Result<(), ()> { self.trace.push_str("reset\n"); Ok(()) }
    fn restore(&mut self) -> Result<(), ()> { self.trace.push_str("restore\n"); Ok(()) }
}

#[test]
#[ignore = "run by tools/mip_sb_control_rust.py with original inputs"]
fn replay() {
    let input = std::fs::read_to_string(std::env::var("SB_CONTROL_INPUT").unwrap()).unwrap();
    let mut trace = String::new();
    for line in input.lines() {
        let a: Vec<_> = line.split_whitespace().collect();
        assert_eq!(a.len(), 14);
        let c = Candidate { column: a[1].parse().unwrap(), x: number(a[2]), lower: number(a[3]),
            upper: number(a[4]), int_tol: number(a[5]), parent_obj: number(a[6]) };
        let mut outcomes = VecDeque::new();
        for k in [7, 10] {
            let v = number(a[k+1]);
            outcomes.push_back(LpResult { value: match a[k] {
                "2" => LpValue::Optimal(v), "7" => LpValue::Limited(v), _ => panic!("uncovered status"),
            }, iterations: number(a[k+2]) });
        }
        let mut lp = Replay { outcomes, trace: String::new() };
        let mut cost = number(a[13]);
        let r = evaluate(&c, &mut lp, &mut cost).unwrap();
        assert!(lp.outcomes.is_empty());
        writeln!(trace, "case {}", a[0]).unwrap();
        trace.push_str(&lp.trace);
        writeln!(trace, "result {:016x} {:016x} {:016x}", r.down.to_bits(), r.up.to_bits(), cost.to_bits()).unwrap();
    }
    std::fs::write(std::env::var("SB_CONTROL_OUTPUT").unwrap(), trace).unwrap();
}

// Synthetic C boundary checks; these do not extend the original-run coverage.
#[test]
fn tolerance_clamp_and_signed_zero() {
    let c = Candidate { column: 2, x: 0.999999, lower: -0.0, upper: 3.0,
        int_tol: 0.00001, parent_obj: 1.0 };
    let mut lp = Replay { outcomes: [
        LpResult { value: LpValue::Optimal(0.0), iterations: 4.0 },
        LpResult { value: LpValue::Limited(0.5), iterations: 7.0 },
    ].into(), trace: String::new() };
    let mut cost = 5.0;
    let result = evaluate(&c, &mut lp, &mut cost).unwrap();
    assert_eq!((result.down, result.up, cost), (1.0, 1.0, 16.0));
    assert!(lp.trace.starts_with("bound 2 62 4000000000000000\nflush\nsolve\nreset\nrestore\n"));
    assert!(lp.trace.contains("bound 2 62 8000000000000000\nbound 2 60 3ff0000000000000"));
    let c = Candidate { parent_obj: -0.0, ..c };
    lp.outcomes = [
        LpResult { value: LpValue::Optimal(0.0), iterations: 0.0 },
        LpResult { value: LpValue::Limited(0.0), iterations: 0.0 },
    ].into();
    let result = evaluate(&c, &mut lp, &mut cost).unwrap();
    assert_eq!(result.up.to_bits(), (-0.0f64).to_bits());
    assert_eq!(result.down.to_bits(), 0.0f64.to_bits());
}
