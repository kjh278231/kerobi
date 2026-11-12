//! Input states and external backend replies only; original answers stay in the Python comparator.
use rsolver_lp::crossover_factor::*;
use rsolver_lp::crossover_supplier::PushColumn;
use std::str::SplitWhitespace;

fn next<T: std::str::FromStr>(t: &mut SplitWhitespace<'_>) -> T {
    t.next().unwrap().parse().ok().unwrap()
}
const KEYS: [&str; 5] = ["status", "numeric", "failure", "request", "k"];
struct Reply { ret: i32, changes: [Option<i32>; 5] }
impl Reply {
    fn read(t: &mut SplitWhitespace<'_>) -> Self {
        let ret = next(t);
        let changes = std::array::from_fn(|_| {
            let token = t.next().unwrap();
            if token == "_" { None } else { Some(token.parse().unwrap()) }
        });
        Self { ret, changes }
    }
    fn apply(&self, s: &mut FactorState) {
        for (target, value) in [&mut s.status, &mut s.numeric, &mut s.failure, &mut s.request, &mut s.updates].into_iter().zip(self.changes) {
            if let Some(v) = value { *target = v; }
        }
    }
    fn json(&self, quality: bool) -> String {
        let mut fields = vec![format!("\"{}\":{}", if quality { "increment" } else { "ret" }, self.ret)];
        for (key, value) in KEYS.into_iter().zip(self.changes) {
            if let Some(v) = value { fields.push(format!("\"{key}\":{v}")); }
        }
        format!("{{{}}}", fields.join(","))
    }
}
struct Backend<'a> {
    factors: Vec<Reply>, qualities: Vec<Reply>, fi: usize, qi: usize,
    original: &'a PushColumn, spike: &'a PushColumn,
    calls: Vec<String>, quality_restored: bool,
}
impl PushFactorBackend for Backend<'_> {
    fn factor(&mut self, state: &mut FactorState, v: &FactorVectors<'_>) -> i32 {
        let fallback = Reply { ret: 99, changes: [None; 5] };
        let r = self.factors.get(self.fi).unwrap_or(&fallback);
        self.fi += 1;
        self.calls.push(format!("{{\"event\":\"factor\",\"request\":{},\"alpha_current\":{},\"spike_original\":{},\"reply\":{}}}",
            state.request, std::ptr::eq(v.workspace, v.current), std::ptr::eq(v.spike, self.spike), r.json(false)));
        r.apply(state);
        r.ret
    }
    fn quality(&mut self, state: &mut FactorState, enter: i32, period: i32, v: &FactorVectors<'_>) -> i32 {
        let fallback = Reply { ret: 0, changes: [Some(12), None, None, Some(0), None] };
        let r = self.qualities.get(self.qi).unwrap_or(&fallback);
        self.qi += 1;
        self.quality_restored &= std::ptr::eq(v.workspace, self.original) && std::ptr::eq(v.spike, self.spike);
        self.calls.push(format!("{{\"event\":\"quality\",\"enter\":{enter},\"period\":{period},\"request\":{},\"reply\":{}}}",
            state.request, r.json(true)));
        r.apply(state);
        r.ret
    }
}
fn sentinel(value: f64) -> PushColumn { PushColumn { count: 1, indices: vec![0], values: vec![value] } }

#[test]
fn factor_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let source = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_factor_rust/input.txt")).unwrap();
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let name = t.next().unwrap();
        let kind = t.next().unwrap();
        if kind == "select" {
            let selection = FactorSelection {
                m: next(&mut t), request: next(&mut t), numeric: next(&mut t), updates: next(&mut t),
                limit: next(&mut t), interval: next(&mut t), column_work: next(&mut t), eta_count: next(&mut t),
                l_nnz: next(&mut t), u_nnz: next(&mut t), support: next(&mut t),
            };
            let d = factor_select(&selection);
            output.push(format!("{name};{{\"request\":{},\"call\":{},\"preserved\":true}}", d.request, d.call));
        } else {
            assert_eq!(kind, "wrapper");
            let mut state = FactorState {
                request: next(&mut t), numeric: next(&mut t), failure: next(&mut t), status: next(&mut t), updates: next(&mut t),
            };
            let mut frame = FactorFrame { period: next(&mut t), attempt: next(&mut t), numeric_limit: next(&mut t), enter: next(&mut t) };
            let factors = (0..next::<usize>(&mut t)).map(|_| Reply::read(&mut t)).collect();
            let qualities = (0..next::<usize>(&mut t)).map(|_| Reply::read(&mut t)).collect();
            let original = sentinel(-0.0);
            let current = sentinel(0.125);
            let spike = sentinel(-7.5);
            let before = [&original, &current, &spike].map(|v| v.values[0].to_bits());
            let mut v = FactorVectors { workspace: &original, current: &current, spike: &spike };
            let mut backend = Backend { factors, qualities, fi: 0, qi: 0, original: &original, spike: &spike, calls: vec![], quality_restored: true };
            let exit = factor_run(&mut state, &mut frame, &mut v, &mut backend);
            let code = match exit { FactorExit::Continue => 200, FactorExit::Recompute => 201, FactorExit::Retry => 202, FactorExit::Error(c) => c };
            let retry = i32::from(exit == FactorExit::Retry);
            let preserved = before == [&original, &current, &spike].map(|v| v.values[0].to_bits());
            output.push(format!("{name};{{\"exit\":{code},\"request\":{},\"numeric\":{},\"failure\":{},\"status\":{},\"k\":{},\"period\":{},\"retry\":{retry},\"alpha_restored\":{},\"spike_restored\":{},\"contents_preserved\":{preserved},\"calls\":[{}],\"quality_restored\":{},\"factor_consumed\":{},\"quality_consumed\":{}}}",
                state.request, state.numeric, state.failure, state.status, state.updates, frame.period,
                std::ptr::eq(v.workspace, &original), std::ptr::eq(v.spike, &spike), backend.calls.join(","), backend.quality_restored, backend.fi, backend.qi));
        }
        assert!(t.next().is_none(), "extra input: {name}");
    }
    let directory = format!("{root}/target/m3_push_factor_rust");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(format!("{directory}/trace.txt"), output.join("\n")+"\n").unwrap();
}
