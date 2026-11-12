//! M3-04c-22 replay: runs `push_loop` from the first pass preparation of each M3-04c-21e push capture with a tape
//! backend (supplier FTRAN raw, support BTRAN raw, `05b257` results and renumbering, RHS FTRAN raw) and writes the
//! canonical event trace. The tape backend records the inputs the loop hands it. No original state is injected after
//! the start; the comparison is `tools/push_loop_rust.py --compare`.
mod loop_io;

use loop_io::*;
use rsolver_lp::crossover::PushState;
use rsolver_lp::crossover_loop::*;
use rsolver_lp::crossover_supplier::PushColumn;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

struct FactorTape {
    ret: i32,
    numeric: i32,
    failure: i32,
    status: i32,
    stats: FactorStats,
    renumber: Option<(Vec<i32>, Vec<i32>)>,
}

struct Tape {
    m: usize,
    ftran: VecDeque<PushColumn>,
    btran: VecDeque<PushColumn>,
    factor: VecDeque<FactorTape>,
    recompute: VecDeque<Vec<f64>>,
    stats: FactorStats,
    out: Out,
}

impl PushLoopBackend for Tape {
    fn ftran(&mut self, input: &PushColumn, output: &mut PushColumn, spike: Option<&mut PushColumn>) {
        let mut line = "ftran".to_string() + &col_fields("", input);
        line += &format!(";spike {}", spike.is_some() as i32);
        self.out.borrow_mut().push(line);
        *output = self.ftran.pop_front().expect("FTRAN tape exhausted");
    }
    fn btran(&mut self, _input: &PushColumn) -> PushColumn {
        self.btran.pop_front().expect("BTRAN tape exhausted")
    }
    fn stats(&self) -> FactorStats {
        self.stats
    }
    fn factor(&mut self, _site: FactorSite, ctl: &mut FactorCtl, _st: &PushState, _work: Option<&PushColumn>)
              -> (i32, Option<Renumber>) {
        let t = self.factor.pop_front().expect("factor tape exhausted");
        ctl.numeric = t.numeric;
        ctl.failure = t.failure;
        ctl.status = t.status;
        self.stats = t.stats;
        (t.ret, t.renumber.map(|(o, n)| Renumber { old_of_new: o, new_of_old: n }))
    }
    fn quality(&mut self, _: &mut FactorCtl, _: i32, _: i32) -> i32 {
        panic!("quality helper is not on the natural tape")
    }
    fn recompute_ftran(&mut self, rhs: &[f64], raw: &mut [f64]) {
        self.out.borrow_mut().push(format!("rin;rhs {}", hexs(rhs)));
        let r = self.recompute.pop_front().expect("recompute tape exhausted");
        assert_eq!(r.len(), self.m);
        raw.copy_from_slice(&r);
    }
}

#[test]
fn loop_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_loop_rust/input.txt")).unwrap();
    let mut lines = text.lines();
    let out: Out = Rc::new(RefCell::new(vec![lines.next().unwrap().to_string()]));
    // Written on drop, also when a tape runs out, so a divergence still leaves a trace to compare.
    let _dump = Dump(out.clone(), format!("{root}/target/m3_push_loop_rust"));
    let rest: Vec<&str> = lines.collect();
    for case in rest.chunks(10) {
        let name = rec(case, 0, "case").s().to_string();
        out.borrow_mut().push(format!("case;{name}"));
        let (c, mut s) = parse_case(case);
        let m = c.m;
        let stats = stat(&mut rec(case, 5, "stats"));
        let mut t = rec(case, 6, "ftran");
        let ftran = (0..t.i::<usize>()).map(|_| t.col(m)).collect();
        let mut t = rec(case, 7, "btran");
        let btran = (0..t.i::<usize>()).map(|_| t.col(m)).collect();
        let mut t = rec(case, 8, "factor");
        let factor = (0..t.i::<usize>())
            .map(|_| {
                let (ret, numeric, failure, status) = (t.i(), t.i(), t.i(), t.i());
                let stats = stat(&mut t);
                let renumber = if t.i::<u8>() != 0 { Some((t.is(m), t.is(m))) } else { None };
                FactorTape { ret, numeric, failure, status, stats, renumber }
            })
            .collect();
        let mut t = rec(case, 9, "recompute");
        let recompute = (0..t.i::<usize>()).map(|_| t.fs(m)).collect();
        let mut tape = Tape { m, ftran, btran, factor, recompute, stats, out: out.clone() };

        let lp = c.lp();
        let mut trace = tracer(out.clone(), false);
        let exit = push_loop(&lp, &c.par, &c.cfg, &mut s, &mut tape, &mut trace);
        assert_eq!(exit, LoopExit::End, "{name}");
        assert!(tape.ftran.is_empty() && tape.btran.is_empty() && tape.factor.is_empty() && tape.recompute.is_empty(),
                "{name}: tape left over");
    }
}
