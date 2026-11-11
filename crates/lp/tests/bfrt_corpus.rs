//! L1 replay of M1-02f: the captured inputs of every `1c35c5` call in the M1-02e BFRT corpus go through
//! `bfrt::bfrt`. Inputs are generated and compared by `tools/m1_bfrt_rust.py` (docs/m1/m1_02f_bfrt_rust.md);
//! run through it, not alone.
use rsolver_lp::bfrt::{bfrt, Cand};
use std::fmt::Write;

fn bits(t: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(t, 16).unwrap())
}

fn replay(input: &str) -> String {
    let mut lines = input.lines();
    let mut out = String::new();
    while let Some(head) = lines.next() {
        // "end" closes the input so a truncated file fails instead of replaying fewer calls
        if head == "end" {
            return out;
        }
        // call nondeg delta feas ncand, then ncand lines: j a d range back cnt fac
        let h: Vec<&str> = head.split_whitespace().collect();
        assert_eq!(h[0], "call", "{head}");
        let n: usize = h[4].parse().unwrap();
        let cands: Vec<Cand> = (0..n)
            .map(|_| {
                let t: Vec<&str> = lines.next().expect("truncated call").split_whitespace().collect();
                Cand { j: t[0].parse().unwrap(), a: bits(t[1]), d: bits(t[2]), range: bits(t[3]), back: t[4] == "1", cnt: bits(t[5]), fac: bits(t[6]) }
            })
            .collect();
        let (q, flips) = bfrt(&cands, bits(h[2]), bits(h[3]), h[1] == "1", &mut rsolver_lp::work::Work::new());
        let q = q.map_or(-1, |q| q as i64);
        writeln!(out, "{q}{}", flips.iter().map(|j| format!(" {j}")).collect::<String>()).unwrap();
    }
    panic!("truncated input");
}

#[test]
#[ignore = "driven by tools/m1_bfrt_rust.py, which generates the input"]
fn bfrt_corpus_replay() {
    let din = std::env::var("BFRT_IN").expect("BFRT_IN");
    let dout = std::env::var("BFRT_OUT").expect("BFRT_OUT");
    std::fs::write(dout, replay(&std::fs::read_to_string(din).unwrap())).unwrap();
}
