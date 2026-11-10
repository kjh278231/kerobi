//! L1 replay of M2-03d-9: the dive row selection runs on the inputs of every M2-03d-8 `d:sel` capture event and
//! writes the selected rows and `66661c` calls. Inputs are generated and the trace compared by
//! `tools/mip_dive_rows_rust.py` (docs/m2/mip_dive_rows_rust.md); run through it, not alone.
use rsolver_mip::dive_rows::{select, Sel};
use std::collections::HashMap;
use std::fmt::Write;

fn hex(t: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(t, 16).unwrap())
}

fn nums<T: std::str::FromStr>(line: &str) -> Vec<T>
where
    T::Err: std::fmt::Debug,
{
    line.split_whitespace().map(|v| v.parse().unwrap()).collect()
}

fn hexes(line: &str) -> Vec<f64> {
    line.split_whitespace().map(hex).collect()
}

fn pairs(line: &str) -> Vec<(usize, &str)> {
    let t: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(t.len() % 2, 0);
    t.chunks(2).map(|p| (p[0].parse().unwrap(), p[1])).collect()
}

struct Model {
    beg: Vec<usize>,
    end: Vec<usize>,
    ind: Vec<i32>,
    val: Vec<f64>,
    rhs: Vec<f64>,
    sense: Vec<u8>,
    vtype: Vec<u8>,
    lb: Vec<f64>,
    ub: Vec<f64>,
}

fn replay(input: &str) -> String {
    let mut lines = input.lines();
    let mut next = || lines.next().expect("truncated input");
    let head = next().to_string();
    assert!(head.starts_with("case "), "{head}");
    let mut out = format!("{head}\n");
    let mut models: HashMap<i64, Model> = HashMap::new();
    let mut n = 0;
    loop {
        // "end" closes the input so a truncated file fails instead of replaying fewer events
        let line = next();
        if line == "end" {
            break;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        match t[0] {
            "model" => {
                let (id, nrows, ncols): (i64, usize, usize) = (t[1].parse().unwrap(), t[2].parse().unwrap(), t[3].parse().unwrap());
                let (sense, vtype) = (next().as_bytes().to_vec(), next().as_bytes().to_vec());
                let len: Vec<usize> = nums(next());
                let (ind, val, rhs) = (nums(next()), hexes(next()), hexes(next()));
                let mut beg = vec![0];
                for l in &len {
                    beg.push(beg.last().unwrap() + l);
                }
                let end = beg[1..].to_vec();
                beg.pop();
                assert!(sense.len() == nrows && vtype.len() == ncols && len.len() == nrows && rhs.len() == nrows);
                assert!(ind.len() == *end.last().unwrap_or(&0) && val.len() == ind.len());
                let m = Model { beg, end, ind, val, rhs, sense, vtype, lb: vec![f64::NAN; ncols], ub: vec![f64::NAN; ncols] };
                assert!(models.insert(id, m).is_none(), "model {id} twice");
            }
            "sel" => {
                let call = t[1];
                let id: i64 = t[2].parse().unwrap();
                let (nrows, ncuts, dual): (usize, usize, i32) = (t[3].parse().unwrap(), t[4].parse().unwrap(), t[5].parse().unwrap());
                let (feastol, cutoff, obj_off) = (hex(t[6]), if t[7] == "-" { 1e100 } else { hex(t[7]) }, hex(t[8]));
                let m = models.get_mut(&id).expect("unknown model");
                assert_eq!(m.rhs.len(), nrows);
                for (j, h) in pairs(next()) {
                    m.lb[j] = hex(h);
                }
                for (j, h) in pairs(next()) {
                    m.ub[j] = hex(h);
                }
                assert!(m.lb.iter().chain(&m.ub).all(|v| !v.is_nan()), "sel {call}: bounds not fully known");
                let mut flags = vec![0i32; nrows + ncuts];
                for (i, v) in pairs(next()) {
                    flags[i] = v.parse().unwrap();
                }
                let b7line = next();
                let b7 = if b7line == "-" {
                    vec![]
                } else {
                    let (cnt, rest) = b7line.split_once(' ').unwrap_or((b7line, ""));
                    let mut b = vec![-1i32; cnt.parse::<usize>().unwrap().min(nrows)];
                    for (i, v) in pairs(rest) {
                        b[i] = v.parse().unwrap();
                    }
                    b
                };
                let kinds: Vec<i32> = nums(next());
                let attr40: Vec<usize> = nums(next());
                let mut slack = vec![0.0; nrows];
                slack.extend(hexes(next()));
                let rc = hexes(next());
                assert!(kinds.len() == ncuts && slack.len() == nrows + ncuts && rc.len() == m.lb.len());
                let cuts: Vec<(i32, bool)> = kinds.iter().enumerate().map(|(j, &k)| (k, attr40.contains(&j))).collect();
                let s = Sel {
                    beg: &m.beg, end: &m.end, ind: &m.ind, val: &m.val, rhs: &m.rhs, sense: &m.sense, vtype: &m.vtype,
                    lb: &m.lb, ub: &m.ub, flags: &flags, slack: &slack, rc: &rc, b7: &b7, cuts: &cuts, feastol,
                    dual_reductions: dual, cutoff, obj_off,
                };
                let (mut calls, mut w) = (Vec::new(), 0.0);
                let rows = select(&s, &mut calls, &mut w);
                write!(out, "sel {call} |").unwrap();
                rows.iter().for_each(|r| write!(out, " {r}").unwrap());
                out.push_str(" |");
                calls.iter().for_each(|(r, v)| write!(out, " {r} {}", *v as i32).unwrap());
                out.push('\n');
                n += 1;
            }
            other => panic!("unknown record {other}"),
        }
    }
    assert!(n > 0, "no selection events");
    out
}

#[test]
#[ignore = "driven by tools/mip_dive_rows_rust.py, which generates the input"]
fn dive_rows_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("DIVE_ROWS_IN").unwrap_or(format!("{root}/target/mip_dive_rows_rust/in"));
    let dout = std::env::var("DIVE_ROWS_OUT").unwrap_or(format!("{root}/target/mip_dive_rows_rust/out"));
    std::fs::create_dir_all(&dout).unwrap();
    let mut cases = 0;
    for e in std::fs::read_dir(&din).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_str().unwrap().to_string();
        let Some(case) = name.strip_suffix(".input.txt") else { continue };
        let trace = replay(&std::fs::read_to_string(&p).unwrap());
        std::fs::write(format!("{dout}/{case}.trace.txt"), trace).unwrap();
        cases += 1;
    }
    assert!(cases > 0, "no inputs in {din}");
}
