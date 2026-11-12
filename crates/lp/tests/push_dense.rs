//! M3-04c-21i `036ade` dense switch: (A) the own dense LU from the original's `dense:lu` state
//! (`bench/oracle/m3_push_dense_rust/unit.txt`) and (B) whole push refactors with the dense switch on
//! (`refactor.txt`). Comparisons: `tools/push_dense_rust.py --compare`. Contract: docs/m3/primal_push_dense_lu.md.
use rsolver_lp::lu::dense_lu;

fn hexf(s: &str) -> f64 {
    let b = u64::from_str_radix(s, 16).unwrap().to_be_bytes();
    f64::from_bits(u64::from_le_bytes(b))
}

fn fhex(x: f64) -> String {
    x.to_bits().to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn root() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
}

/// (A): `case idx n zero mat16 scr16 first cols… rows… a…` → `case idx ret|restart cols… rows… a…`.
#[test]
fn dense_unit_trace() {
    let src = std::fs::read_to_string(format!("{}/bench/oracle/m3_push_dense_rust/unit.txt", root()))
        .expect("tools/push_dense_rust.py --write");
    let mut lines = src.lines();
    let mut out = vec![lines.next().unwrap().to_string()];
    for ln in lines {
        let t: Vec<&str> = ln.split(' ').collect();
        let (case, idx, n) = (t[0], t[1], t[2].parse::<usize>().unwrap());
        let zero = hexf(t[3]);
        let (mat16, scr16, first): (usize, usize, bool) =
            (t[4].parse().unwrap(), t[5].parse().unwrap(), t[6] == "1");
        let mut p = 7;
        let mut take = |k: usize| {
            let v = t[p..p + k].to_vec();
            p += k;
            v
        };
        let mut cols: Vec<usize> = take(n).iter().map(|x| x.parse().unwrap()).collect();
        let mut rows: Vec<usize> = take(n).iter().map(|x| x.parse().unwrap()).collect();
        let mut a: Vec<f64> = take(n * n).iter().map(|x| hexf(x)).collect();
        assert_eq!(p, t.len(), "{case} {idx}: extra tokens");
        let ret = match dense_lu(&mut a, n, &mut cols, &mut rows, zero, mat16, scr16, first) {
            Some(d) => d.to_string(),
            None => "restart".to_string(),
        };
        let mut o = vec![case.to_string(), idx.to_string(), ret];
        o.extend(cols.iter().map(|x| x.to_string()));
        o.extend(rows.iter().map(|x| x.to_string()));
        o.extend(a.iter().map(|&x| fhex(x)));
        out.push(o.join(" "));
    }
    let dir = format!("{}/target/m3_push_dense_rust", root());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/unit-trace.txt"), out.join("\n") + "\n").unwrap();
}
