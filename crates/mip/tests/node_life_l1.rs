//! L1 replay of M2-06f-3: the M2-06f-2 node creations, release requests and decay entries run on `NodeLife`, and the
//! stages, `5fa560` calls and decay ancestor chains are written. Inputs are generated and compared by
//! `tools/mip_node_life_rust.py` (docs/m2/mip_node_life_rust.md); run through it, not alone.
use rsolver_mip::node_life::NodeLife;
use std::collections::HashMap;
use std::fmt::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn state(t: &NodeLife, k: usize) -> &'static str {
    match (t.alive(k), t.has_info(k), t.has_lp(k)) {
        (false, ..) => "gone",
        (true, false, _) => "I0",
        (true, true, false) => "I1L0",
        (true, true, true) => "I1L1",
    }
}

fn replay(input: &str, out: &mut String) {
    let mut lines = input.lines();
    let head = lines.next().expect("empty input");
    assert!(head.starts_with("case "), "{head}");
    writeln!(out, "{head}").unwrap();
    let mut t = NodeLife::default();
    for line in lines {
        // "end" closes the input so a truncated file fails instead of replaying fewer events
        if line == "end" {
            return;
        }
        let w: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize| -> usize { w[i].parse().unwrap() };
        match w[0] {
            "nn" => {
                let parent = if w[1] == "-" { None } else { Some(num(1)) };
                writeln!(out, "nn {}", t.create(parent, w[2] == "1")).unwrap();
            }
            "rel" => {
                let (k, s): (usize, i32) = (num(1), w[2].parse().unwrap());
                // the cascade only reaches k and its ancestors, so their states before are the 5fa560 inputs
                let mut pre = HashMap::new();
                let mut a = Some(k);
                while let Some(x) = a.filter(|&x| t.alive(x)) {
                    pre.insert(x, state(&t, x));
                    a = t.parent(x);
                }
                let lp = t.has_lp(k);
                let mut calls = Vec::new();
                let old = t.release(k, s, |x, s| calls.push((x, s)));
                write!(out, "rel {}", old.map_or("-".to_string(), |o| o.to_string())).unwrap();
                if s == 6 {
                    write!(out, " L{}", lp as u8).unwrap();
                }
                out.push('\n');
                for (x, s) in calls {
                    writeln!(out, "fr {x} {s} {} {}", pre.get(&x).copied().unwrap_or("?"), state(&t, x)).unwrap();
                }
            }
            "dec" => {
                let k = num(1);
                if !t.has_info(k) {
                    out.push_str("dec noI\n");
                } else if !t.has_lp(k) {
                    out.push_str("dec noL\n");
                } else {
                    out.push_str("dec");
                    let mut last = k;
                    for a in t.ancestors(k) {
                        write!(out, " {a}").unwrap();
                        last = a;
                    }
                    // the chain ends at an ancestor without I (read, then stop) or past the root
                    match t.parent(last) {
                        Some(a) => writeln!(out, " {a} noI").unwrap(),
                        None => out.push_str(" nop\n"),
                    }
                }
            }
            _ => panic!("bad line {line}"),
        }
    }
    panic!("truncated input");
}

#[test]
#[ignore = "driven by tools/mip_node_life_rust.py, which generates the input"]
fn node_life_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("NODE_LIFE_IN").unwrap_or(format!("{root}/target/mip_node_life_rust/in"));
    let dout = std::env::var("NODE_LIFE_OUT").unwrap_or(format!("{root}/target/mip_node_life_rust/out"));
    std::fs::create_dir_all(&dout).unwrap();
    let mut n = 0;
    for e in std::fs::read_dir(&din).unwrap() {
        let p = e.unwrap().path();
        let case = p.file_name().unwrap().to_str().unwrap().strip_suffix(".input.txt").unwrap().to_string();
        let input = std::fs::read_to_string(&p).unwrap();
        let mut out = String::new();
        // a panic (a code mutant reaching a freed node) fails only its own case
        if catch_unwind(AssertUnwindSafe(|| replay(&input, &mut out))).is_err() {
            out.push_str("panic\n");
        }
        std::fs::write(format!("{dout}/{case}.trace.txt"), out).unwrap();
        n += 1;
    }
    assert!(n > 0, "no inputs in {din}");
}
