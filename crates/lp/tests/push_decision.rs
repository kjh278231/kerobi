//! M3-04c-6 replay: runs the push step decisions on raw fields projected from original captures and writes a trace.
//! The comparison is `tools/primal_push_decision_rust.py --compare`; this test only produces the trace.
use rsolver_lp::crossover::{feasible_ratio, push_direction, push_leave_state, push_step, PushStep,
                            PushTol, violated_ratio};

fn f(token: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(token, 16).unwrap().swap_bytes()) // captures store little-endian bytes
}

fn hex(v: f64) -> String {
    format!("{:016x}", v.to_bits().swap_bytes())
}

#[test]
fn decision_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_decision_rust/input.txt")).unwrap();
    let mut lines = text.lines();
    let mut out = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let mut next = || t.next().unwrap();
        let name = next().to_string();
        let violated = next() == "V"; // the original takes the violated path while some basic row is marked B or A
        let [n, m, j]: [usize; 3] = [(); 3].map(|_| next().parse().unwrap());
        let [h, piv_tol, inf, fixed] = [(); 4].map(|_| f(next()));
        let pos: i32 = next().parse().unwrap();
        let (mut theta, pivot) = (f(next()), f(next()));
        let mut floats = |k: usize| (0..k).map(|_| f(next())).collect::<Vec<_>>();
        let (cost, lb, ub, x, slack) = (floats(n + m), floats(n + m), floats(n + m), floats(n), floats(m));
        let header: Vec<i32> = (0..m).map(|_| next().parse().unwrap()).collect();
        let count: usize = next().parse().unwrap();
        let idx: Vec<i32> = (0..count).map(|_| next().parse().unwrap()).collect();
        let mut val: Vec<f64> = (0..count).map(|_| f(next())).collect();
        let (basic, width) = ((0..m).map(|_| f(next())).collect::<Vec<_>>(), (0..m).map(|_| f(next())).collect::<Vec<_>>());
        let marks: Vec<u8> = (0..m).map(|_| u8::from_str_radix(next(), 16).unwrap()).collect();

        let tol = PushTol::new(h);
        let entry = push_direction(j, &x, &slack, &lb, &ub, &cost, &header, &idx, &mut val, &tol, inf,
                                   violated.then_some(&marks[..]));
        // Link check: the ratio test on the Rust-flipped column, compared against the captured ratio output.
        let (mut work, mut ltheta, mut lpivot) = (0.0, f64::NAN, f64::NAN);
        let ratio = if violated { violated_ratio } else { feasible_ratio }; // violated: 054eef, then the 056973 retry
        let link = match ratio(h, piv_tol, inf, inf, Some(&idx), &val, &basic, &width, &marks, 0.0, &mut work, &mut ltheta,
                               &mut lpivot) {
            Ok(p) => format!("{p} {} {}", hex(ltheta), hex(lpivot)),
            Err(_) => "reentry x x".to_string(),
        };
        let head = format!("{name} {} {} {} {} {} {} {} {}", entry.decrease as u8, hex(entry.r), hex(entry.dl),
                           hex(entry.du), hex(tol.e), hex(tol.reject), hex(tol.reject_neg), hex(tol.leave));
        let alpha: Vec<String> = val.iter().map(|&v| hex(v)).collect();
        let decision = match push_step(&entry, &tol, inf, pos, &mut theta, pivot) {
            Err(_) => "recall x x".to_string(),
            Ok(step) => {
                let state = if step == PushStep::Pivot {
                    let p = pos as usize;
                    let row = idx[p] as usize;
                    let lv = header[row] as usize;
                    push_leave_state(ub[lv] - lb[lv], fixed, val[p], theta, marks[row], basic[row], width[row], &tol)
                        .to_string()
                } else {
                    "x".to_string()
                };
                format!("{step:?} {} {state}", hex(theta))
            }
        };
        out.push(format!("{head} {decision} {link} {}", alpha.join(" ")));
    }
    let dir = format!("{root}/target/m3_push_decision_rust");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), out.join("\n") + "\n").unwrap();
}
