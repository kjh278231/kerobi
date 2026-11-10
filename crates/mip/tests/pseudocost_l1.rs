//! L1 replay of M2-06f: the M2-06e pseudocost events run on `Pseudocost` and the gate decisions, weights and entries
//! are written. Inputs are generated and compared by `tools/mip_pseudocost_rust.py` (docs/m2/mip_pseudocost_rust.md);
//! run through it, not alone.
use rsolver_mip::pseudocost::{decay_fires, node_weight, DecayLp, NodeLp, Pseudocost};
use std::collections::HashMap;
use std::fmt::Write;

fn bits(t: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(t, 16).unwrap())
}

fn int(t: &str) -> i32 {
    t.parse().unwrap()
}

fn entry(out: &mut String, t: &Pseudocost, j: usize) {
    write!(out, " {j}:{}:{}:{:016x}:{:016x}", t.cnt[0][j], t.cnt[1][j], t.sum[0][j].to_bits(), t.sum[1][j].to_bits())
        .unwrap();
}

fn snap(out: &mut String, t: &Pseudocost) {
    out.push_str("snap");
    for j in 0..t.cnt[0].len() {
        if t.cnt[0][j] != 0 || t.cnt[1][j] != 0 || t.sum[0][j].to_bits() != 0 || t.sum[1][j].to_bits() != 0 {
            entry(out, t, j);
        }
    }
    out.push('\n');
}

/// gate/dgate fields: parent status depth j down continuous x lp p
fn node(t: &[&str]) -> NodeLp {
    NodeLp {
        has_parent: t[0] == "1",
        status: int(t[1]),
        depth: int(t[2]),
        j: int(t[3]),
        down: t[4] == "1",
        continuous: t[5] == "1",
        x: bits(t[6]),
        lp: bits(t[7]),
        p: bits(t[8]),
    }
}

fn get<'a>(tabs: &'a mut HashMap<&str, Pseudocost>, k: &str, line: &str) -> &'a mut Pseudocost {
    tabs.get_mut(k).unwrap_or_else(|| panic!("unknown table in {line}"))
}

fn replay(input: &str) -> String {
    let mut lines = input.lines();
    let head = lines.next().expect("empty input");
    assert!(head.starts_with("case "), "{head}");
    let mut out = format!("{head}\n");
    let mut tabs: HashMap<&str, Pseudocost> = HashMap::new();
    for line in lines {
        // "end" closes the input so a truncated file fails instead of replaying fewer events
        if line == "end" {
            assert!(tabs.is_empty(), "{head}: tables never freed");
            return out;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        match t[0] {
            "new" => {
                let p = Pseudocost::new(t[2].parse().unwrap());
                snap(&mut out, &p);
                assert!(tabs.insert(t[1], p).is_none(), "{head}: table {} twice", t[1]);
            }
            "seed" => {
                let map: Vec<i32> = t[4..].iter().map(|s| int(s)).collect();
                let src = tabs.get(t[3]).unwrap_or_else(|| panic!("{head}: unknown seed source in {line}"));
                let p = Pseudocost::seeded(t[2].parse().unwrap(), src, &map);
                snap(&mut out, &p);
                assert!(tabs.insert(t[1], p).is_none(), "{head}: table {} twice", t[1]);
            }
            "free" => {
                snap(&mut out, &tabs.remove(t[1]).unwrap_or_else(|| panic!("{head}: free of unknown table {}", t[1])));
            }
            "snap" => snap(&mut out, get(&mut tabs, t[1], line)),
            "show" => {
                let p = get(&mut tabs, t[1], line);
                out.push_str("show");
                entry(&mut out, p, t[2].parse().unwrap());
                out.push('\n');
            }
            "set" => {
                let p = get(&mut tabs, t[1], line);
                let j: usize = t[2].parse().unwrap();
                p.cnt[0][j] = int(t[3]);
                p.cnt[1][j] = int(t[4]);
                p.sum[0][j] = bits(t[5]);
                p.sum[1][j] = bits(t[6]);
            }
            "gate" => {
                let p = get(&mut tabs, t[1], line);
                let n = node(&t[2..11]);
                let cut = bits(t[11]);
                if p.node_update(&n, cut) {
                    write!(out, "gate 1 {:016x}", node_weight(n.depth, cut).to_bits()).unwrap();
                    entry(&mut out, p, n.j as usize);
                    out.push('\n');
                } else {
                    out.push_str("gate 0\n");
                }
            }
            "dgate" => writeln!(out, "dgate {}", u8::from(decay_fires(&node(&t[1..10])))).unwrap(),
            "dec" => {
                // dec t none | dec t status k s n alt vec anc... (anc j:down:value-or-x, value into the read vector)
                let p = get(&mut tabs, t[1], line);
                if t[2] == "none" {
                    p.decay(None, std::iter::empty());
                } else {
                    let n: usize = t[5].parse().unwrap();
                    let (alt_on, vec_on) = (t[6] == "1", t[7] == "1");
                    let mut val = vec![0.0; n];
                    let mut anc = Vec::new();
                    for a in &t[8..] {
                        let q: Vec<&str> = a.split(':').collect();
                        let j = int(q[0]);
                        if q[2] != "x" {
                            val[j as usize] = bits(q[2]);
                        }
                        anc.push((j, q[1] == "1"));
                    }
                    // the vector the original does not read is not captured: NaN, so reading it shows
                    let other = vec![f64::NAN; n];
                    let (st, k) = (int(t[2]), int(t[3]));
                    let reads_vec = st == 6 || k >= 0;
                    let pick = |on: bool, read: bool| on.then_some(if read { &val[..] } else { &other[..] });
                    let l = DecayLp { status: st, k, s: int(t[4]), alt: pick(alt_on, !reads_vec), vec: pick(vec_on, reads_vec) };
                    p.decay(Some(&l), anc);
                }
            }
            "sym" => {
                let cls: Vec<i32> = t[2..].iter().map(|s| int(s)).collect();
                get(&mut tabs, t[1], line).symmetry(&cls);
            }
            _ => panic!("bad line {line}"),
        }
    }
    panic!("truncated input");
}

#[test]
#[ignore = "driven by tools/mip_pseudocost_rust.py, which generates the input"]
fn pseudocost_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("PSEUDOCOST_IN").unwrap_or(format!("{root}/target/mip_pseudocost_rust/in"));
    let dout = std::env::var("PSEUDOCOST_OUT").unwrap_or(format!("{root}/target/mip_pseudocost_rust/out"));
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
