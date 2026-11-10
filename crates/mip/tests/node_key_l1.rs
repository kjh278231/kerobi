//! L1 replay of M2-06b-2: the captured inputs of every 627f3e heap key write go through `child_key`/`deep_key` and
//! the keys are written. Inputs are generated and compared by `tools/mip_node_key_check.py`
//! (docs/m2/mip_node_key.md); run through it, not alone.
use rsolver_mip::node_heap::{child_key, deep_key};
use std::fmt::Write;

fn bits(t: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(t, 16).unwrap())
}

fn replay(input: &str) -> String {
    let mut lines = input.lines();
    let head = lines.next().expect("empty input");
    assert!(head.starts_with("case "), "{head}");
    let mut out = format!("{head}\n");
    for line in lines {
        // "end" closes the input so a truncated file fails instead of replaying fewer writes
        if line == "end" {
            return out;
        }
        // path bound nfrac root_nfrac obj_step [parent_depth ncols]; path B has no depth term
        let t: Vec<&str> = line.split_whitespace().collect();
        let k = child_key(bits(t[1]), t[2].parse().unwrap(), t[3].parse().unwrap(), bits(t[4]));
        let k = match t[0] {
            "A" => deep_key(k, t[5].parse().unwrap(), t[6].parse().unwrap()),
            "B" => k,
            _ => panic!("bad line {line}"),
        };
        writeln!(out, "{:016x}", k.to_bits()).unwrap();
    }
    panic!("truncated input");
}

#[test]
#[ignore = "driven by tools/mip_node_key_check.py, which generates the input"]
fn node_key_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("NODE_KEY_IN").unwrap_or(format!("{root}/target/mip_node_key/in"));
    let dout = std::env::var("NODE_KEY_OUT").unwrap_or(format!("{root}/target/mip_node_key/out"));
    std::fs::create_dir_all(&dout).unwrap();
    let mut n = 0;
    for e in std::fs::read_dir(&din).unwrap() {
        let p = e.unwrap().path();
        let case = p.file_name().unwrap().to_str().unwrap().strip_suffix(".input.txt").unwrap().to_string();
        std::fs::write(format!("{dout}/{case}.keys.txt"), replay(&std::fs::read_to_string(&p).unwrap())).unwrap();
        n += 1;
    }
    assert!(n > 0, "no inputs in {din}");
}
