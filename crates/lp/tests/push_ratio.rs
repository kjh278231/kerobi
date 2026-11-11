//! M3-04c replay: runs the feasible ratio on inputs projected from original captures and writes a trace. The
//! comparison against the captures is `tools/primal_push_ratio_rust.py --compare`; this test only produces the trace.
//! M3-04c-2 adds the violated-basis ratio on the synthetic `054eef` calls (`tools/push_violated_rust.py --compare`),
//! M3-04c-7 the `056973` retry on synthetic calls and natural retry inputs (`tools/push_retry_rust.py --compare`).
//! M3-04c-5 adds the feasible ratio's small-pivot second pass on synthetic `055f68` calls with the `12dd49` result
//! (`tools/primal_push_reentry_rust.py --compare`).
use rsolver_lp::crossover::{feasible_ratio, infeasible_ratio, retry_ratio, Reentry};

type Ratio = fn(f64, f64, f64, f64, Option<&[i32]>, &[f64], &[f64], &[f64], &[u8], f64, &mut f64, &mut f64, &mut f64)
                -> Result<i32, Reentry>;

fn f(token: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(token, 16).unwrap().swap_bytes()) // captures store little-endian bytes
}

fn hex(v: f64) -> String {
    format!("{:016x}", v.to_bits().swap_bytes())
}

/// The 04c inputs never reach the second pass's `12dd49` call; asking for it is a replay failure.
#[allow(clippy::too_many_arguments)]
fn feasible(h: f64, piv: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64], width: &[f64],
            marks: &[u8], coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64) -> Result<i32, Reentry> {
    let mut refactor = false;
    Ok(feasible_ratio(h, piv, inf, cap, idx, val, x, width, marks, coef, work, theta, pivot,
                      &mut || panic!("12dd49 asked: not captured"), &mut refactor))
}

#[test]
fn ratio_trace() {
    trace("bench/oracle/m3_push_ratio_rust/input.txt", "target/m3_push_ratio_rust", feasible);
}

/// Lines are those of [`trace`] with the `12dd49` result after the scalars; the output adds the refactor request
/// (0/1) and how many times the update count was asked.
#[test]
fn reentry_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_reentry/rust_input.txt")).unwrap();
    let mut lines = text.lines();
    let mut out = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let mut next = || t.next().unwrap();
        let name = next().to_string();
        let dense = next() == "1";
        let m: usize = next().parse().unwrap();
        let [harris, piv, inf, cap, coef, mut work, mut theta, mut pivot] = [(); 8].map(|_| f(next()));
        let updates: i32 = next().parse().unwrap();
        let count: usize = if dense { m } else { next().parse().unwrap() };
        let idx: Vec<i32> = if dense { vec![] } else { (0..count).map(|_| next().parse().unwrap()).collect() };
        let val: Vec<f64> = (0..count).map(|_| f(next())).collect();
        let x: Vec<f64> = (0..m).map(|_| f(next())).collect();
        let width: Vec<f64> = (0..m).map(|_| f(next())).collect();
        let marks = (0..m).map(|_| u8::from_str_radix(next(), 16).unwrap()).collect::<Vec<_>>();
        let (mut refactor, mut asked) = (false, 0);
        let pos = feasible_ratio(harris, piv, inf, cap, (!dense).then_some(&idx[..]), &val, &x, &width, &marks, coef,
                                 &mut work, &mut theta, &mut pivot, &mut || { asked += 1; updates }, &mut refactor);
        out.push(format!("{name} {pos} {} {} {} {} {asked}", hex(theta), hex(pivot), hex(work), refactor as u8));
    }
    let dir = format!("{root}/target/m3_reentry");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), out.join("\n") + "\n").unwrap();
}

#[test]
fn infeasible_ratio_trace() {
    trace("bench/oracle/m3_push_violated/ratio_input.txt", "target/m3_push_violated", infeasible_ratio);
}

#[test]
fn retry_ratio_trace() {
    trace("bench/oracle/m3_push_retry/ratio_input.txt", "target/m3_push_retry", retry_ratio);
}

fn trace(input: &str, out_dir: &str, ratio: Ratio) {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/{input}")).unwrap();
    let mut lines = text.lines();
    let mut out = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let mut next = || t.next().unwrap();
        let name = next().to_string();
        let dense = next() == "1";
        let m: usize = next().parse().unwrap();
        let [harris, piv, inf, cap, coef, mut work, mut theta] = [(); 7].map(|_| f(next()));
        let count: usize = if dense { m } else { next().parse().unwrap() };
        let idx: Vec<i32> = if dense { vec![] } else { (0..count).map(|_| next().parse().unwrap()).collect() };
        let val: Vec<f64> = (0..count).map(|_| f(next())).collect();
        let x: Vec<f64> = (0..m).map(|_| f(next())).collect();
        let width: Vec<f64> = (0..m).map(|_| f(next())).collect();
        let marks = (0..m).map(|_| u8::from_str_radix(next(), 16).unwrap()).collect::<Vec<_>>();
        let mut pivot = f64::NAN;
        let idx = (!dense).then_some(&idx[..]);
        out.push(match ratio(harris, piv, inf, cap, idx, &val, &x, &width, &marks, coef, &mut work, &mut theta,
                                      &mut pivot) {
            Ok(pos) => format!("{name} {pos} {} {} {}", hex(theta), hex(pivot), hex(work)),
            Err(_) => format!("{name} reentry"),
        });
    }
    let dir = format!("{root}/{out_dir}");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), out.join("\n") + "\n").unwrap();
}
