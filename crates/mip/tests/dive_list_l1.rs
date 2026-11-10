//! L1 replay of M2-03d-10: the deactivated dive row list and the D swap run on the operations of the M2-03d-7/03d-8
//! captures and write every list update and swap. Inputs are generated and the trace compared by
//! `tools/mip_dive_list_rust.py` (docs/m2/mip_dive_list_rust.md); run through it, not alone.
use rsolver_mip::dive_rows::RowList;
use std::collections::HashMap;
use std::fmt::Write;

fn hex(t: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(t, 16).unwrap())
}

fn nums<T: std::str::FromStr>(t: &[&str]) -> Vec<T>
where
    T::Err: std::fmt::Debug,
{
    t.iter().map(|v| v.parse().unwrap()).collect()
}

fn join<T: std::fmt::Display>(v: impl IntoIterator<Item = T>) -> String {
    v.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")
}

fn bits(v: &[f64]) -> String {
    join(v.iter().map(|x| format!("{:016x}", x.to_bits())))
}

/// API LP of the swap target in D layout (column starts, lengths, entries, rhs).
struct D {
    beg: Vec<i64>,
    len: Vec<i32>,
    ind: Vec<i32>,
    val: Vec<f64>,
    rhs: Vec<f64>,
}

fn replay(input: &str) -> String {
    let mut lines = input.lines();
    let mut next = || lines.next().expect("truncated input");
    let head = next().to_string();
    assert!(head.starts_with("case "), "{head}");
    let mut out = format!("{head}\n");
    let mut lps: HashMap<i64, RowList> = HashMap::new();
    let mut d: Option<D> = None;
    let mut n = 0;
    loop {
        // "end" closes the input so a truncated file fails instead of replaying fewer events
        let line = next();
        if line == "end" {
            break;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        let id: i64 = t.get(1).map_or(0, |v| v.parse().unwrap());
        match t[0] {
            "lp" => assert!(lps.insert(id, RowList::default()).is_none(), "lp {id} twice"),
            "rows" => lps.get_mut(&id).unwrap().changed(t[2].parse().unwrap()),
            "deact" => lps.get_mut(&id).unwrap().deactivate(t[2].parse().unwrap(), &nums::<i32>(&t[3..])).unwrap(),
            "react_all" => lps.get_mut(&id).unwrap().reactivate_all(t[2].parse().unwrap()).unwrap(),
            "update" => {
                let l = lps.get_mut(&id).unwrap();
                l.update();
                writeln!(out, "list {id} {} | {}", t[2], join(l.list())).unwrap();
                n += 1;
            }
            "dlp" => {
                let (cbeg, ind, val, rhs): (Vec<i64>, Vec<i32>, Vec<f64>, Vec<f64>) = (
                    nums(&next().split_whitespace().collect::<Vec<_>>()),
                    nums(&next().split_whitespace().collect::<Vec<_>>()),
                    next().split_whitespace().map(hex).collect(),
                    next().split_whitespace().map(hex).collect(),
                );
                let len = cbeg.windows(2).map(|w| (w[1] - w[0]) as i32).collect();
                assert!(ind.len() == val.len() && *cbeg.last().unwrap() as usize == ind.len());
                d = Some(D { beg: cbeg[..cbeg.len() - 1].to_vec(), len, ind, val, rhs });
            }
            "swap_in" => {
                let (l, d) = (lps.get_mut(&id).unwrap(), d.as_mut().expect("swap without dlp"));
                let built = l.state().0;
                let list = join(l.list());
                l.swap_in(&d.beg, &mut d.len, &mut d.ind, &mut d.val, &mut d.rhs);
                let (mut ind, mut val) = (vec![], vec![]);
                for (j, &s) in d.beg.iter().enumerate() {
                    let r = s as usize..s as usize + d.len[j] as usize;
                    ind.extend_from_slice(&d.ind[r.clone()]);
                    val.extend_from_slice(&d.val[r]);
                }
                writeln!(out, "swap_in {id} {} {} {} | {list} | {} | {} | {} | {}", t[2], built as i32, l.state().1, join(&d.len),
                         join(ind), bits(&val), bits(&d.rhs)).unwrap();
            }
            "swap_out" => {
                let (l, d) = (lps.get_mut(&id).unwrap(), d.as_mut().expect("swap without dlp"));
                l.swap_out(&mut d.len, &mut d.rhs);
                writeln!(out, "swap_out {id} {} {} | {} | {}", t[2], l.state().1, join(&d.len), bits(&d.rhs)).unwrap();
            }
            other => panic!("unknown record {other}"),
        }
    }
    assert!(n > 0, "no list updates");
    out
}

#[test]
#[ignore = "driven by tools/mip_dive_list_rust.py, which generates the input"]
fn dive_list_replay() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let din = std::env::var("DIVE_LIST_IN").unwrap_or(format!("{root}/target/mip_dive_list_rust/in"));
    let dout = std::env::var("DIVE_LIST_OUT").unwrap_or(format!("{root}/target/mip_dive_list_rust/out"));
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
