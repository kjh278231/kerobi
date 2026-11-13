//! M3-04f-4a replay: `251b4f` (with the original's BTRAN on the captured factor), the S+0x2d8 recompute after the
//! pushes and the automatic dual check, from the `dv:enter` state of each call. The original's outputs are read only
//! by tools/xo_dual_rust.py. Contract: docs/m3/crossover_dual_values.md.
use rsolver_lp::crossover_btran::*;
use rsolver_lp::crossover_cleanup::dual_check;
use rsolver_lp::crossover_dual::*;
use rsolver_lp::crossover_ftran::{Buffer, VecSnap};
use std::str::SplitWhitespace;

fn next<T: std::str::FromStr>(t: &mut SplitWhitespace<'_>) -> T {
    t.next().unwrap().parse().ok().unwrap()
}
fn float(t: &mut SplitWhitespace<'_>) -> f64 {
    let h = t.next().unwrap();
    assert_eq!(h.len(), 16);
    f64::from_le_bytes(std::array::from_fn(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap()))
}
fn opt_float(t: &mut SplitWhitespace<'_>) -> Option<f64> {
    if next::<u8>(t) == 0 {
        None
    } else {
        Some(float(t))
    }
}
fn bits(v: f64) -> String {
    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn ints(t: &mut SplitWhitespace<'_>) -> Vec<i64> {
    let n = next::<usize>(t);
    (0..n).map(|_| next(t)).collect()
}
fn usizes(t: &mut SplitWhitespace<'_>) -> Vec<usize> {
    ints(t).into_iter().map(|x| x as usize).collect()
}
fn i32s(t: &mut SplitWhitespace<'_>) -> Vec<i32> {
    ints(t).into_iter().map(|x| x as i32).collect()
}
fn floats(t: &mut SplitWhitespace<'_>) -> Vec<f64> {
    let n = next::<usize>(t);
    (0..n).map(|_| float(t)).collect()
}
fn rows(t: &mut SplitWhitespace<'_>, m: usize) -> Vec<SparseRow> {
    (0..m).map(|_| SparseRow { cols: usizes(t), vals: floats(t) }).collect()
}
fn opt_i(x: &Option<i32>) -> String {
    x.map_or("null".into(), |v| v.to_string())
}
fn opt_f(x: &Option<f64>) -> String {
    x.map_or("null".into(), |v| format!("\"{}\"", bits(v)))
}
fn list<T>(xs: &[T], f: impl Fn(&T) -> String) -> String {
    format!("[{}]", xs.iter().map(f).collect::<Vec<_>>().join(","))
}
fn hexes(xs: &[f64]) -> String {
    list(xs, |v| format!("\"{}\"", bits(*v)))
}
fn vector(v: &VecSnap) -> String {
    let idx = v.idx.as_ref().map_or("null".into(), |i| list(i, opt_i));
    format!("{{\"count\":{},\"idx\":{idx},\"val\":{}}}", opt_i(&v.count), list(&v.val, opt_f))
}
fn name(b: Buffer) -> &'static str {
    match b {
        Buffer::In => "in",
        Buffer::Out => "out",
    }
}
fn alias(a: &[(Buffer, Buffer); 2]) -> String {
    format!(
        "{{\"in\":{{\"idx\":\"{}\",\"val\":\"{}\"}},\"out\":{{\"idx\":\"{}\",\"val\":\"{}\"}}}}",
        name(a[0].0),
        name(a[0].1),
        name(a[1].0),
        name(a[1].1)
    )
}
fn scratch(s: &Scratch) -> String {
    format!(
        "{{\"x1b0\":{},\"x1b8\":{}}}",
        list(&s.marks, |(i, v)| format!("[{i},{v}]")),
        list(&s.vals, |(i, v)| format!("[{i},\"{}\"]", bits(*v)))
    )
}

/// The BTRAN factor in the field order of tools/push_btran_rust.py `line` from `relabel` on.
fn factor(t: &mut SplitWhitespace<'_>, m: usize, zero: f64) -> (BtranFactor, Option<Vec<i32>>) {
    let relabel = usizes(t);
    let gather = usizes(t);
    let upos = usizes(t);
    let uord = usizes(t);
    let diag = floats(t);
    let urow = rows(t, m);
    let flag = i32s(t);
    let tt: usize = next(t);
    let mult: i32 = next(t);
    let f108 = float(t);
    let f110 = float(t);
    let d8: usize = next(t);
    let e4: usize = next(t);
    let ltail: i64 = next(t);
    let lrow = rows(t, m);
    let levels: i32 = next(t);
    let (lev, lst, bucket) = if levels >= 0 {
        let lev = usizes(t);
        let lst = usizes(t);
        (lev, lst, Some(i32s(t)))
    } else {
        (Vec::new(), Vec::new(), None)
    };
    let f1e0: i64 = next(t);
    let f184: i32 = next(t);
    let ne: usize = next(t);
    let etas = (0..ne).map(|_| BtranEta { row: next(t), idx: usizes(t), val: floats(t) }).collect();
    let f = BtranFactor {
        m,
        zero,
        relabel,
        gather,
        upos,
        uord,
        diag,
        urow,
        flag,
        t: tt,
        mult,
        f108,
        f110,
        d8,
        e4,
        ltail,
        lrow,
        levels,
        lev,
        lst,
        f1e0,
        f184,
        etas,
    };
    (f, bucket)
}

fn run(line: &str) -> (String, String) {
    let mut t = line.split_whitespace();
    let case: String = next(&mut t);
    let n: usize = next(&mut t);
    let m: usize = next(&mut t);
    let cost = floats(&mut t);
    let vstat = i32s(&mut t);
    let header = i32s(&mut t);
    let nc: usize = next(&mut t);
    let corrections: Vec<(usize, f64)> = (0..nc).map(|_| (next(&mut t), float(&mut t))).collect();
    let drop = float(&mut t);
    let start = usizes(&mut t);
    let len = usizes(&mut t);
    let cols = usizes(&mut t);
    let vals = floats(&mut t);
    let work = opt_float(&mut t);
    let out_count: i32 = next(&mut t);
    let x1d8: i32 = next(&mut t);
    let zero = float(&mut t);
    let (f, bucket) = factor(&mut t, m, zero);
    let tol380 = opt_float(&mut t);
    let opttol = opt_float(&mut t);
    assert!(t.next().is_none(), "{case}: trailing input");
    let e = DualInput {
        n,
        m,
        cost: &cost,
        vstat: &vstat,
        header: &header,
        corrections: &corrections,
        drop,
        a: RowMatrix { start: &start, len: &len, cols: &cols, vals: &vals },
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dual_values(&e, &f, out_count, x1d8, bucket, work)));
    let body = match result {
        Err(err) => {
            let msg = err.downcast_ref::<String>().cloned().or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()));
            format!("{{\"error\":{:?}}}", msg.unwrap_or_default())
        }
        Ok(c) => {
            let (out, inp) = c.btran.snapshot(m);
            let s2d8 = tol380.map_or("null".into(), |tol| format!("\"{}\"", bits(dual_infeasibility(&c.d, &vstat, tol))));
            let r14 = opttol.map_or("null".into(), |tol| (dual_check(&c.d, &vstat, tol) as i32).to_string());
            let br = &c.btran.branches;
            format!(
                "{{\"rhs\":{},\"btran\":{{\"output\":{},\"input_after\":{},\"alias\":{},\"x200\":\"{}\",\"scratch\":{}}},\"y\":{},\"d\":{},\"work\":{},\"s2d8\":{s2d8},\"r14\":{r14},\"branches\":{{\"entry\":\"{}\",\"u\":\"{}\",\"kernel\":\"{}\",\"eta\":\"{}\",\"lt\":\"{}\"}},\"error\":null}}",
                hexes(&c.rhs),
                vector(&out),
                vector(&inp),
                alias(&c.btran.alias()),
                bits(c.btran.work),
                scratch(&c.btran.scratch()),
                hexes(&c.y),
                hexes(&c.d),
                opt_f(&c.work),
                br.entry,
                br.u,
                br.kernel,
                br.eta,
                br.lt
            )
        }
    };
    (case, body)
}

#[test]
fn dual_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let gz = std::fs::File::open(format!("{root}/bench/oracle/m3_xo_dual_rust/input.txt.gz")).unwrap();
    let mut source = String::new();
    std::io::Read::read_to_string(&mut flate2::read::GzDecoder::new(gz), &mut source).unwrap();
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    std::panic::set_hook(Box::new(|_| {}));
    for line in lines {
        let (case, body) = run(line);
        output.push(format!("{case};{body}"));
    }
    let _ = std::panic::take_hook();
    let dir = format!("{root}/target/m3_xo_dual_rust");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), output.join("\n") + "\n").unwrap();
}
