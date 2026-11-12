//! M3-04c-10 replay: chains `push_pick`/`push_apply` over the push steps of the natural captures, carrying the Rust
//! state from step to step, and writes a trace. A `seg` line restarts the state (push start or after a refactor);
//! a `step` line gives the supplier's candidate and sparse column. The comparison is
//! `tools/push_step_chain_rust.py --compare`; this test only produces the trace.
use rsolver_lp::crossover::{push_apply, push_pick, PushLp, PushParams, PushState, PushStep};

fn f(token: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(token, 16).unwrap().swap_bytes()) // captures store little-endian bytes
}

fn hex(v: f64) -> String {
    format!("{:016x}", v.to_bits().swap_bytes())
}

fn hexs(v: &[f64]) -> String {
    v.iter().map(|&x| hex(x)).collect::<Vec<_>>().join(" ")
}

fn ints(v: &[i32]) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")
}

fn marks(v: &[u8]) -> String {
    v.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ")
}

struct Model {
    n: usize,
    m: usize,
    starts: Vec<i64>,
    lengths: Vec<i32>,
    rows: Vec<i32>,
    values: Vec<f64>,
    cost: Vec<f64>,
    lb: Vec<f64>,
    ub: Vec<f64>,
}

fn state_fields(st: &PushState, full: bool) -> String {
    let mut s = format!(";{p}vstat {};{p}header {};{p}basic {};{p}width {};{p}marks {};{p}iter {}", ints(&st.vstat),
                        ints(&st.header), hexs(&st.basic), hexs(&st.width), marks(&st.marks), hex(st.iterations),
                        p = if full { "z." } else { "a." });
    if full {
        s += &format!(";z.xs {};z.rhs {}", hexs(&st.xs), hexs(&st.rhs));
    }
    s
}

#[test]
fn chain_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_step_chain/input.txt")).unwrap();
    let mut lines = text.lines();
    let mut out = vec![lines.next().unwrap().to_string()];
    let (mut model, mut par, mut st) = (None::<Model>, None::<PushParams>, None::<PushState>);
    for line in lines {
        let mut t = line.split_whitespace();
        let mut next = || t.next().unwrap();
        match next() {
            "seg" => {
                let _name = next();
                let [n, m, nnz]: [usize; 3] = [(); 3].map(|_| next().parse().unwrap());
                let [h, piv_tol, inf, fixed, iterations] = [(); 5].map(|_| f(next()));
                let [leave, request]: [i32; 2] = [(); 2].map(|_| next().parse().unwrap());
                let starts = (0..n).map(|_| next().parse().unwrap()).collect();
                let lengths = (0..n).map(|_| next().parse().unwrap()).collect();
                let rows = (0..nnz).map(|_| next().parse().unwrap()).collect();
                let values = (0..nnz).map(|_| f(next())).collect();
                let [cost, lb, ub] = [(); 3].map(|_| (0..n + m).map(|_| f(next())).collect::<Vec<_>>());
                let vstat = (0..n + m).map(|_| next().parse().unwrap()).collect();
                let header = (0..m).map(|_| next().parse().unwrap()).collect();
                let xs = (0..n + m).map(|_| f(next())).collect();
                let [basic, width] = [(); 2].map(|_| (0..m).map(|_| f(next())).collect::<Vec<_>>());
                let mk: Vec<u8> = (0..m).map(|_| u8::from_str_radix(next(), 16).unwrap()).collect();
                let rhs = (0..m).map(|_| f(next())).collect();
                let violations = mk.iter().filter(|&&c| c == b'B' || c == b'A').count() as i32;
                model = Some(Model { n, m, starts, lengths, rows, values, cost, lb, ub });
                par = Some(PushParams { h, piv_tol, inf, fixed });
                // theta/pivot start as NaN: a value leaking from before the segment would show in the trace.
                st = Some(PushState { vstat, header, xs, basic, width, marks: mk, rhs, violations, iterations,
                                      enter: -1, leave, direction: 0, leave_col: -1, request, theta: f64::NAN,
                                      pivot: f64::NAN });
            }
            "step" => {
                let name = next().to_string();
                let j: usize = next().parse().unwrap();
                let updates: i32 = next().parse().unwrap();
                let count: usize = next().parse().unwrap();
                let mut idx: Vec<i32> = (0..count).map(|_| next().parse().unwrap()).collect();
                let mut val: Vec<f64> = (0..count).map(|_| f(next())).collect();
                let (md, par, st) = (model.as_ref().unwrap(), par.as_ref().unwrap(), st.as_mut().unwrap());
                let lp = PushLp { n: md.n, m: md.m, starts: &md.starts, lengths: &md.lengths, rows: &md.rows,
                                  values: &md.values, cost: &md.cost, lb: &md.lb, ub: &md.ub };
                let mut factor_updates = || updates;
                let pick = push_pick(&lp, st, par, j, &idx, &mut val, &mut factor_updates);
                let first_alpha: Vec<f64> = if pick.second.is_some() { val.iter().map(|v| -v).collect() } else {
                    val.clone()
                };
                let seen = |k: usize| format!("{} {} {}", pick.seen[k].0, pick.seen[k].1, pick.seen[k].2);
                let ratio = |r: (i32, f64, f64)| format!("{} {} {}", r.0, hex(r.1), hex(r.2));
                let mut row = format!("{name};path {};seen0 {};frame {} {} {};alpha1 {};ratio1 {}",
                                      if pick.violated { "V" } else { "F" }, seen(0), hex(pick.entry.r),
                                      hex(pick.entry.dl), hex(pick.entry.du), hexs(&first_alpha), ratio(pick.first));
                if let Some(r2) = pick.second {
                    row += &format!(";seen1 {};alpha2 {};ratio2 {}", seen(1), hexs(&val), ratio(r2));
                }
                row += &format!(";step {:?} {}", pick.step, hex(st.theta));
                let mut applied = String::new();
                push_apply(&lp, st, par, &pick, &mut idx, &mut val,
                           &mut |s: &PushState| applied = state_fields(s, false));
                if pick.step == PushStep::Defer {
                    applied = state_fields(st, false); // step:defer sees the state the step left untouched
                }
                row += &format!(";sel {} {} {}", st.enter, st.leave, st.direction);
                row += &applied;
                row += &state_fields(st, true);
                out.push(row);
            }
            other => panic!("unknown record {other}"),
        }
    }
    let dir = format!("{root}/target/m3_push_step_chain");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), out.join("\n") + "\n").unwrap();
}
