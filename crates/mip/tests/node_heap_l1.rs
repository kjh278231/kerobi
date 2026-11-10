//! L1 replay of M2-06b: the M2-06a top-level heap operations run on `NodeHeap` and the pops, prune removals and heap
//! snapshots are written. Inputs are generated and compared by `tools/mip_node_heap_rust.py`
//! (docs/m2/mip_node_heap_rust.md); run through it, not alone.
use rsolver_mip::node_heap::NodeHeap;
use std::collections::HashMap;
use std::fmt::Write;

fn bits(t: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(t, 16).unwrap())
}

fn replay(input: &str) -> String {
    let mut lines = input.lines();
    let head = lines.next().expect("empty input");
    assert!(head.starts_with("case "), "{head}");
    let mut out = format!("{head}\n");
    let mut trees: HashMap<&str, NodeHeap<u64>> = HashMap::new();
    for line in lines {
        // "end" closes the input so a truncated file fails instead of replaying fewer operations
        if line == "end" {
            return out;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        if t[0] == "tree" {
            assert!(trees.insert(t[1], NodeHeap::default()).is_none(), "{head}: tree {} twice", t[1]);
            continue;
        }
        if t[0] == "free" {
            assert!(trees.remove(t[1]).is_some(), "{head}: free of unknown tree {}", t[1]);
            continue;
        }
        let h = trees.get_mut(t[1]).unwrap_or_else(|| panic!("{head}: unknown tree in {line}"));
        match t[0] {
            "ins" => h.push(bits(t[3]), bits(t[4]), t[2].parse().unwrap()),
            // an empty heap is written, not a panic, so a code mutant fails only its own case
            "pop" => match h.pop() {
                Some(e) => writeln!(out, "pop {}", e.2).unwrap(),
                None => out.push_str("pop none\n"),
            },
            "prune" => {
                out.push_str("prune");
                h.prune(bits(t[2]), |n| write!(out, " {n}").unwrap());
                out.push('\n');
            }
            "snap" => {
                write!(out, "snap {} {:016x}", h.len(), h.min_bound().to_bits()).unwrap();
                for (k, b, n) in h.iter() {
                    write!(out, " {n}:{:016x}:{:016x}", k.to_bits(), b.to_bits()).unwrap();
                }
                out.push('\n');
            }
            _ => panic!("bad line {line}"),
        }
    }
    panic!("truncated input");
}

#[test]
#[ignore = "driven by tools/mip_node_heap_rust.py, which generates the input"]
fn node_heap_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("NODE_HEAP_IN").unwrap_or(format!("{root}/target/mip_node_heap_rust/in"));
    let dout = std::env::var("NODE_HEAP_OUT").unwrap_or(format!("{root}/target/mip_node_heap_rust/out"));
    std::fs::create_dir_all(&dout).unwrap();
    let mut n = 0;
    for e in std::fs::read_dir(&din).unwrap() {
        let p = e.unwrap().path();
        let case = p.file_name().unwrap().to_str().unwrap().strip_suffix(".input.txt").unwrap().to_string();
        std::fs::write(format!("{dout}/{case}.trace.txt"), replay(&std::fs::read_to_string(&p).unwrap())).unwrap();
        n += 1;
    }
    assert!(n > 0, "no inputs in {din}");
}
