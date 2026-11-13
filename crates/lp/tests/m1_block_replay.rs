//! M1-01e injection replay: every original `036ade` of a Method=1 run factored from its captured basis head and
//! entry block state (S+0x2c0/0x4e0/0x4e4); the exit state and LU fields go to a file for tools/m1_block_check.py.
//! Input lines: `model path`, then per factor `x2c0 x4e0 x4e4 head...`; output per factor `x4e0 x4e4 d8 dc e0 e4 t`.
use rsolver_lp::lu::{BlockState, Lu, MARKOWITZ_TOL};
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
        let v: Vec<i64> = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
        let mut st = BlockState { x2c0: v[0] as i32, x4e0: v[1] as i32, x4e4: v[2] as i32 };
        let head: Vec<usize> = v[3..].iter().map(|&j| j as usize).collect();
        let lu = Lu::factor_block(spx.as_ref().unwrap(), &head, MARKOWITZ_TOL, &mut st);
        let f = lu.block_fields();
        writeln!(out, "{} {} {} {} {} {} {}", st.x4e0, st.x4e4, f[0], f[1], f[2], f[3], f[4]).unwrap();
    }
    out
}

#[test]
#[ignore = "driven by tools/m1_block_check.py --replay, which generates the input"]
fn m1_block_replay() {
    let din = std::env::var("BLOCK_IN").expect("BLOCK_IN");
    let dout = std::env::var("BLOCK_OUT").expect("BLOCK_OUT");
    std::fs::write(dout, replay(&std::fs::read_to_string(din).unwrap())).unwrap();
}
