//! M3-04c replay: runs the feasible ratio on inputs projected from original captures and writes a trace. The
//! comparison against the captures is `tools/primal_push_ratio_rust.py --compare`; this test only produces the trace.
use rsolver_lp::crossover::feasible_ratio;

fn f(token: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(token, 16).unwrap().swap_bytes()) // captures store little-endian bytes
}

fn hex(v: f64) -> String {
    format!("{:016x}", v.to_bits().swap_bytes())
}

#[test]
fn ratio_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_ratio_rust/input.txt")).unwrap();
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
        out.push(match feasible_ratio(harris, piv, inf, cap, idx, &val, &x, &width, &marks, coef, &mut work, &mut theta,
                                      &mut pivot) {
            Ok(pos) => format!("{name} {pos} {} {} {}", hex(theta), hex(pivot), hex(work)),
            Err(_) => format!("{name} reentry"),
        });
    }
    let dir = format!("{root}/target/m3_push_ratio_rust");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), out.join("\n") + "\n").unwrap();
}
