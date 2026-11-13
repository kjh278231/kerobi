//! M1-02w-3 injection replay: every original `036ade` of a Method=1 run factored by `Lu::factor_dense` from its
//! captured basis head, entry block state, Markowitz tol (S+0x390 bits) and pool size F+0x158. Output goes to a file
//! for tools/m1_dense_check.py.
//! Input lines: `model path`, then per factor `x2c0 x4e0 x4e4 tol_bits cap head...`; output per factor
//! `next_cap x4e0 x4e4 d8 dc e0 e4 t dense_n mode70 diag_bits...` (next_cap −1 without a shortage, dense_n −1 without
//! the dense switch, mode70 = the `local_70` mode was entered (0xd85b4, before its pool test), diagonal by factor step = the position after the refactor's renumbering, hex bits).
use rsolver_lp::lu::{BlockState, Lu};
use rsolver_lp::spx::Spx;
use std::fmt::Write;

fn replay(input: &str) -> String {
    let mut out = String::new();
    let mut spx = None;
    for line in input.lines() {
        if let Some(p) = line.strip_prefix("model ") {
            spx = Some(Spx::new(&rsolver_model::mps::read_file(p).unwrap()));
            writeln!(out, "{line}").unwrap();
            continue;
        }
        let w: Vec<&str> = line.split_whitespace().collect();
        let v = |i: usize| w[i].parse::<i64>().unwrap();
        let mut st = BlockState { x2c0: v(0) as i32, x4e0: v(1) as i32, x4e4: v(2) as i32 };
        let tol = f64::from_bits(u64::from_str_radix(w[3], 16).unwrap());
        let cap = v(4);
        let head: Vec<usize> = w[5..].iter().map(|x| x.parse().unwrap()).collect();
        // a branch the port lacks panics ("not ported"): report it as the line `panic <message>` and go on
        let sp = spx.as_ref().unwrap();
        let lu = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Lu::factor_dense(sp, &head, tol, &mut st, cap))) {
            Ok(lu) => lu,
            Err(e) => {
                let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()));
                writeln!(out, "panic {}", msg.unwrap_or_default().replace('\n', " ")).unwrap();
                continue;
            }
        };
        let f = lu.block_fields();
        let next = lu.short.map_or(-1, |s| s.next_cap(cap));
        let dense = lu.dense.as_ref().map_or(-1, |d| d.n as i64);
        write!(out, "{next} {} {} {} {} {} {} {} {dense} {}", st.x4e0, st.x4e4, f[0], f[1], f[2], f[3], f[4], lu.pool_trace.iter().any(|e| e.kind == "mode70") as i32)
            .unwrap();
        if lu.short.is_none() && lu.complete() {
            for s in lu.parts().steps {
                write!(out, " {:016x}", s.2.to_bits()).unwrap();
            }
        }
        out.push('\n');
    }
    out
}

#[test]
#[ignore = "driven by tools/m1_dense_check.py --replay, which generates the input"]
fn m1_dense_replay() {
    let din = std::env::var("DENSE_IN").expect("DENSE_IN");
    let dout = std::env::var("DENSE_OUT").expect("DENSE_OUT");
    std::fs::write(dout, replay(&std::fs::read_to_string(din).unwrap())).unwrap();
}
