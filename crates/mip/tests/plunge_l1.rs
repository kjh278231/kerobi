//! L1 replay of M2-06d: the M2-06c capture inputs of every top-level `6e8eb3` call run through `plunge` (seed, rule,
//! pick, tail) and the results are written. Inputs are generated and compared by `tools/mip_plunge_rust.py`
//! (docs/m2/mip_plunge_rust.md); run through it, not alone.
use rsolver_mip::plunge::{pick, rule, seed, tail, Item, RuleIn, TailIn, Thr};
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn hx(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s, 16).unwrap())
}

fn bits(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

fn opt(a: Option<i64>) -> String {
    a.map_or("-".to_string(), |x| x.to_string())
}

/// One call block: the lines between `call` and `.`.
fn call(block: &[Vec<&str>], out: &mut String) {
    let line = |tag: &str| block.iter().find(|w| w[0] == tag).unwrap_or_else(|| panic!("no {tag} line"));
    let num = |w: &[&str], i: usize| -> i64 { w[i].parse().unwrap() };
    let pin = line("pin");
    let rk = line("rk");
    let (k_entry, w28) = (num(pin, 1) as i32, hx(pin[2]));
    let k = num(rk, 1) as i32;
    let hint = num(pin, 3) == 0 && num(pin, 4) != 0 && k_entry < 10;
    let rin = RuleIn {
        k,
        gap: hx(pin[7]),
        all_int: num(pin, 8) == num(pin, 9),
        hint,
        m848: num(pin, 5) != 0,
        branch_dir: num(pin, 10) as i32,
        mx808: num(pin, 6) != 0,
        s: num(rk, 5) as i32,
        focus: num(pin, 11) as i32,
    };
    let mut rng = seed(k_entry, w28);
    let s0 = rng.0;
    let asked = Cell::new(false);
    let r = rule(
        &rin,
        || {
            asked.set(true);
            assert!(rk[6] != "-", "5e6efd asked but not called by the original");
            num(rk, 6) != 0
        },
        &mut rng,
    );
    writeln!(out, "seed {s0:016x}").unwrap();
    writeln!(out, "rule {r} {} {} {:016x} {}", hint as u8, (rng.0 != s0) as u8, rng.0, asked.get() as u8).unwrap();

    let mut items: Vec<Item<i64>> = block
        .iter()
        .filter(|w| w[0] == "it")
        .map(|w| Item { node: num(w, 1), est: hx(w[2]), depth: num(w, 3) as i32, excluded: num(w, 4) != 0, bound: hx(w[5]) })
        .collect();
    // parent table from the tail chains: Some(p) known (p may be null), absent = unknown
    let mut par: HashMap<i64, Option<i64>> = HashMap::new();
    for w in block.iter().filter(|w| w[0] == "par") {
        par.insert(num(w, 1), if w[2] == "-" { None } else { Some(num(w, 2)) });
    }
    let unknown = Cell::new(false);
    let mut parent = |x: i64| match par.get(&x) {
        Some(&p) => p,
        None => {
            unknown.set(true);
            None
        }
    };
    let mut work = hx(line("work")[1]);
    // the captured pick, present only where the original rule's score is not ported or the 06c-2 replay had no chain
    let given = || line("given")[1].parse::<usize>().unwrap();
    let idx = match pick(r, &items, &mut rng, &mut parent, &mut work) {
        // a rule 1/6 pick that asked a parent above the tail chains is not replayable: continue from the capture
        Some(_) if unknown.get() => {
            writeln!(out, "unrep {}", given()).unwrap();
            given()
        }
        Some(i) => {
            writeln!(out, "pick {i}").unwrap();
            i
        }
        None => {
            writeln!(out, "given {}", given()).unwrap();
            given()
        }
    };
    writeln!(out, "work {}", bits(work)).unwrap();

    unknown.set(false);
    let tin = TailIn { k, m2c: num(rk, 2) as i32, focus: rin.focus, t138: rk[3].parse().unwrap(), t120: rk[4].parse().unwrap(), hint };
    let thr = || {
        let w = line("thr");
        Thr { f9a41: num(w, 1) as i32, pool: hx(w[2]), v8239: hx(w[3]), cut: hx(w[4]) }
    };
    let mut rm = Vec::new();
    let t = tail(&mut items, idx, &tin, thr, &mut parent, |x, heap| rm.push((x, heap)), &mut work);
    assert!(!unknown.get(), "tail asked a parent above the chains");
    writeln!(out, "sel {} {} {}", t.sel.node, t.d, opt(t.anc)).unwrap();
    if let Some(x) = t.thr {
        writeln!(out, "thr {}", bits(x)).unwrap();
    }
    for (x, heap) in rm {
        writeln!(out, "rm {x} {}", if heap { "heap" } else { "drop" }).unwrap();
    }
    write!(out, "tend {}", bits(work)).unwrap();
    for it in &items {
        write!(out, " {}", it.node).unwrap();
    }
    out.push('\n');
}

fn replay(input: &str, out: &mut String) {
    let mut lines = input.lines();
    let head = lines.next().expect("empty input");
    assert!(head.starts_with("case "), "{head}");
    writeln!(out, "{head}").unwrap();
    let mut block: Vec<Vec<&str>> = Vec::new();
    for l in lines {
        // "end" closes the input so a truncated file fails instead of replaying fewer calls
        if l == "end" {
            assert!(block.is_empty(), "end inside a call");
            return;
        }
        let w: Vec<&str> = l.split_whitespace().collect();
        match w[0] {
            "call" => {
                assert!(block.is_empty(), "call inside a call");
                writeln!(out, "call {}", w[1]).unwrap();
                block.push(w);
            }
            "." => {
                call(&block, out);
                block.clear();
            }
            _ => block.push(w),
        }
    }
    panic!("truncated input");
}

#[test]
#[ignore = "driven by tools/mip_plunge_rust.py, which generates the input"]
fn plunge_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("PLUNGE_IN").unwrap_or(format!("{root}/target/mip_plunge_rust/in"));
    let dout = std::env::var("PLUNGE_OUT").unwrap_or(format!("{root}/target/mip_plunge_rust/out"));
    std::fs::create_dir_all(&dout).unwrap();
    let mut n = 0;
    for e in std::fs::read_dir(&din).unwrap() {
        let p = e.unwrap().path();
        let case = p.file_name().unwrap().to_str().unwrap().strip_suffix(".input.txt").unwrap().to_string();
        let input = std::fs::read_to_string(&p).unwrap();
        let mut out = String::new();
        // a panic (missing input asked by a mutant, truncated input) fails only its own case
        if catch_unwind(AssertUnwindSafe(|| replay(&input, &mut out))).is_err() {
            out.push_str("panic\n");
        }
        std::fs::write(format!("{dout}/{case}.trace.txt"), out).unwrap();
        n += 1;
    }
    assert!(n > 0, "no inputs in {din}");
}
