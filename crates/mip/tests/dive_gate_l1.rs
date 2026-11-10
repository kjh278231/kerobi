//! L1 replay of M2-03d-12: the DynamicRows gates run on the raw inputs of every M2-03d-11 gate evaluation and write
//! the decision and the `a7f0ad`/`a7f923` calls. Inputs are generated and the trace compared by
//! `tools/mip_dive_gate_rust.py` (docs/m2/mip_dive_gate_rust.md); run through it, not alone.
use rsolver_mip::dive_rows::Gate;
use std::cell::Cell;
use std::fmt::Write;

/// Return of a call the original made; '-' when it did not call, so a call the original skipped panics.
fn eax(t: &str, name: &str, head: &str, k: usize) -> bool {
    assert!(t != "-", "{head} evaluation {k}: {name} called, the original did not call it");
    t.parse::<i32>().unwrap() != 0
}

fn replay(input: &str) -> String {
    let mut lines = input.lines();
    let head = lines.next().expect("empty input");
    assert!(head.starts_with("case "), "{head}");
    let mut out = format!("{head}\n");
    for (k, line) in lines.enumerate() {
        // "end" closes the input so a truncated file fails instead of replaying fewer evaluations
        if line == "end" {
            return out;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        let v: Vec<i32> = t[1..8].iter().map(|x| x.parse().unwrap()).collect();
        let g = Gate { dyn_rows: v[0], depth: v[1], dive_depth: v[2], lp_status: v[3], has_flags: v[4] == 0, has_sol: v[5] == 0, p3_18: v[6] };
        match t[0] {
            "526" => writeln!(out, "526 {}", g.react() as i32).unwrap(),
            "1581" => {
                let (qp, qcp) = (Cell::new(false), Cell::new(false));
                let s = g.select(
                    || {
                        qp.set(true);
                        eax(t[8], "a7f0ad", head, k)
                    },
                    || {
                        qcp.set(true);
                        eax(t[9], "a7f923", head, k)
                    },
                );
                writeln!(out, "1581 {} {} {}", qp.get() as i32, qcp.get() as i32, s as i32).unwrap();
            }
            _ => panic!("bad line {line}"),
        }
    }
    panic!("truncated input");
}

#[test]
#[ignore = "driven by tools/mip_dive_gate_rust.py, which generates the input"]
fn dive_gate_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("DIVE_GATE_IN").unwrap_or(format!("{root}/target/mip_dive_gate_rust/in"));
    let dout = std::env::var("DIVE_GATE_OUT").unwrap_or(format!("{root}/target/mip_dive_gate_rust/out"));
    std::fs::create_dir_all(&dout).unwrap();
    let mut n = 0;
    for e in std::fs::read_dir(&din).unwrap() {
        let p = e.unwrap().path();
        let case = p.file_name().unwrap().to_str().unwrap().strip_suffix(".input.txt").unwrap().to_string();
        std::fs::write(format!("{dout}/{case}.trace.txt"), replay(&std::fs::read_to_string(&p).unwrap())).unwrap();
        n += 1;
    }
    assert!(n > 0, "no input in {din}");
}
