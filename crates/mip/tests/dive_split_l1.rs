//! L1 replay of M2-03d-14: every 10f448 rebuild of the M2-03d-13 captures runs through `RowList::swap_in`/`swap_out`
//! and writes the changed columns, reduced lengths and rhs rows. Inputs are generated and the trace compared by
//! `tools/mip_dive_split_rust.py` (docs/m2/mip_dive_split_rust.md); run through it, not alone.
use rsolver_mip::dive_rows::RowList;
use std::collections::HashMap;
use std::fmt::Write;

fn nums<T: std::str::FromStr>(line: &str) -> Vec<T>
where
    T::Err: std::fmt::Debug,
{
    line.split_whitespace().map(|v| v.parse().unwrap()).collect()
}

fn join<T: std::fmt::Display>(v: impl IntoIterator<Item = T>) -> String {
    v.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")
}

fn bits(v: &[f64]) -> String {
    join(v.iter().map(|x| format!("{:016x}", x.to_bits())))
}

#[derive(Clone)]
struct D {
    beg: Vec<i64>,
    len: Vec<i32>,
    ind: Vec<i32>,
    val: Vec<f64>,
}

fn replay(input: &str) -> String {
    let mut lines = input.lines();
    let mut next = || lines.next().expect("truncated input");
    let head = next().to_string();
    assert!(head.starts_with("case "), "{head}");
    let mut out = format!("{head}\n");
    let mut kept: HashMap<u32, D> = HashMap::new();
    let mut n = 0;
    loop {
        // "end" closes the input so a truncated file fails instead of replaying fewer rebuilds
        let line = next();
        if line == "end" {
            break;
        }
        // build <serial> <rows> <keep 0|1> full|ref <serial>
        let t: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(t[0], "build", "{line}");
        let (serial, rows, keep): (u32, usize, bool) = (t[1].parse().unwrap(), t[2].parse().unwrap(), t[3] == "1");
        let mut d = match t[4] {
            "full" => {
                let (beg, len, ind) = (nums(next()), nums(next()), nums(next()));
                let val = next().split_whitespace().map(|v| f64::from_bits(u64::from_str_radix(v, 16).unwrap())).collect();
                D { beg, len, ind, val }
            }
            "ref" => kept[&t[5].parse::<u32>().unwrap()].clone(),
            other => panic!("unknown entry {other}"),
        };
        let list: Vec<i32> = nums(next());
        let nonzero: Vec<usize> = nums(next());
        // entry rhs is not captured: +0 on listed rows whose entry rhs was +0, 1.0 elsewhere (contract section 1)
        let mut rhs = vec![1.0; rows];
        for &r in &list {
            rhs[r as usize] = 0.0;
        }
        for &r in &nonzero {
            rhs[r] = 1.0;
        }
        let (entry, rhs0) = (d.clone(), rhs.clone());

        let mut l = RowList::default();
        l.deactivate(rows, &list).unwrap();
        l.update();
        l.swap_in(&d.beg, &mut d.len, &mut d.ind, &mut d.val, &mut rhs);
        let (built, refs) = l.state();
        let xlen: Vec<String> =
            d.len.iter().zip(&entry.len).enumerate().filter(|(_, (a, b))| a != b).map(|(j, (a, _))| format!("{j} {a}")).collect();
        let rhs_diff: Vec<String> = rhs
            .iter()
            .zip(&rhs0)
            .enumerate()
            .filter(|(_, (a, b))| a.to_bits() != b.to_bits())
            .map(|(i, (a, _))| format!("{i} {:016x}", a.to_bits()))
            .collect();
        let mut cols = String::new();
        for (j, &s) in d.beg.iter().enumerate() {
            let r = s as usize..s as usize + entry.len[j] as usize;
            let same_val = d.val[r.clone()].iter().zip(&entry.val[r.clone()]).all(|(a, b)| a.to_bits() == b.to_bits());
            if d.ind[r.clone()] != entry.ind[r.clone()] || !same_val {
                writeln!(cols, "col {j} | {} | {}", join(&d.ind[r.clone()]), bits(&d.val[r])).unwrap();
            }
        }
        l.swap_out(&mut d.len, &mut rhs);
        let restored = d.len == entry.len && rhs.iter().zip(&rhs0).all(|(a, b)| a.to_bits() == b.to_bits());
        writeln!(out, "build {serial} {} {refs} {} {} | {} | {}", built as i32, l.state().1, restored as i32, join(xlen), join(rhs_diff)).unwrap();
        out.push_str(&cols);
        if keep {
            kept.insert(serial, d);
        }
        n += 1;
    }
    assert!(n > 0, "no rebuilds");
    out
}

#[test]
#[ignore = "driven by tools/mip_dive_split_rust.py, which generates the input"]
fn dive_split_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("DIVE_SPLIT_IN").unwrap_or(format!("{root}/target/mip_dive_split_rust/in"));
    let dout = std::env::var("DIVE_SPLIT_OUT").unwrap_or(format!("{root}/target/mip_dive_split_rust/out"));
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
