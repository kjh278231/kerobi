//! Input-only replay of the M3-04c-21c support-round BTRAN calls; the original kernel/leave states are read only by
//! tools/push_btran_rust.py.
use rsolver_lp::crossover_btran::*;
use rsolver_lp::crossover_ftran::{Buffer, Cells, VecSnap};
use std::str::SplitWhitespace;

fn next<T: std::str::FromStr>(t: &mut SplitWhitespace<'_>) -> T {
    t.next().unwrap().parse().ok().unwrap()
}
fn float(t: &mut SplitWhitespace<'_>) -> f64 {
    let h = t.next().unwrap();
    assert_eq!(h.len(), 16);
    f64::from_le_bytes(std::array::from_fn(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap()))
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
fn cache(c: &Option<CacheSnap>) -> String {
    c.as_ref().map_or("null".into(), |c| {
        format!("{{\"count\":{},\"idx\":{},\"val\":{}}}", c.count, list(&c.idx, opt_i), list(&c.val, opt_f))
    })
}

fn run(line: &str) -> (String, String) {
    let mut t = line.split_whitespace();
    let case: String = next(&mut t);
    let m: usize = next(&mut t);
    let zero = float(&mut t);
    let count: i32 = next(&mut t);
    let in_idx: Vec<i32> = ints(&mut t).into_iter().map(|x| x as i32).collect();
    let in_val = floats(&mut t);
    let out_count: i32 = next(&mut t);
    let cache0 = if next::<u8>(&mut t) == 0 {
        None
    } else {
        let count: i32 = next(&mut t);
        let idx = ints(&mut t);
        let val = floats(&mut t);
        let mut c = Cache { count, idx: Cells::unknown(m), val: Cells::unknown(m) };
        for (k, (&i, &v)) in idx.iter().zip(&val).enumerate() {
            c.idx.set(k, i as i32);
            c.val.set(k, v);
        }
        Some(c)
    };
    let x1d8: i32 = next(&mut t);
    let work = float(&mut t);
    let relabel = usizes(&mut t);
    let gather = usizes(&mut t);
    let upos = usizes(&mut t);
    let uord = usizes(&mut t);
    let diag = floats(&mut t);
    let urow = rows(&mut t, m);
    let flag: Vec<i32> = ints(&mut t).into_iter().map(|x| x as i32).collect();
    let tt: usize = next(&mut t);
    let mult: i32 = next(&mut t);
    let f108 = float(&mut t);
    let f110 = float(&mut t);
    let d8: usize = next(&mut t);
    let e4: usize = next(&mut t);
    let ltail: i64 = next(&mut t);
    let lrow = rows(&mut t, m);
    let levels: i32 = next(&mut t);
    let (lev, lst, bucket) = if levels >= 0 {
        let lev = usizes(&mut t);
        let lst = usizes(&mut t);
        let bucket: Vec<i32> = ints(&mut t).into_iter().map(|x| x as i32).collect();
        (lev, lst, Some(bucket))
    } else {
        (Vec::new(), Vec::new(), None)
    };
    let f1e0: i64 = next(&mut t);
    let f184: i32 = next(&mut t);
    let ne: usize = next(&mut t);
    let etas = (0..ne).map(|_| BtranEta { row: next(&mut t), idx: usizes(&mut t), val: floats(&mut t) }).collect();
    assert!(t.next().is_none(), "{case}: trailing input");
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
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut s = BtranState::new(m, count, &in_idx, &in_val, out_count, cache0, x1d8, work, bucket);
        btran(&f, &mut s);
        s
    }));
    let body = match result {
        Err(e) => {
            let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()));
            format!("{{\"error\":{:?}}}", msg.unwrap_or_default())
        }
        Ok(s) => {
            let kernels: Vec<String> = s
                .events
                .iter()
                .map(|e| {
                    let out = if e.leave {
                        format!("\"output\":{}", vector(&e.output))
                    } else {
                        format!("\"output_count\":{}", opt_i(&e.output.count))
                    };
                    format!(
                        "{{\"kernel\":\"21a941\",\"phase\":\"{}\",\"input\":{},{out},\"alias\":{},\"scratch\":{},\"cache\":{},\"x1d8\":{},\"x200\":\"{}\"}}",
                        if e.leave { "leave" } else { "enter" },
                        vector(&e.input),
                        alias(&e.alias),
                        scratch(&e.scratch),
                        cache(&e.cache),
                        e.x1d8,
                        bits(e.work)
                    )
                })
                .collect();
            let (out, inp) = s.snapshot(m);
            let bucket = s.bucket.as_ref().map_or("null".into(), |b| list(b, |v| v.to_string()));
            let br = &s.branches;
            format!(
                "{{\"kernels\":[{}],\"leave\":{{\"output\":{},\"input_after\":{},\"alias\":{},\"cache\":{},\"x1d8\":{},\"scratch\":{},\"bucket\":{bucket},\"x200\":\"{}\"}},\"branches\":{{\"entry\":\"{}\",\"u\":\"{}\",\"kernel\":\"{}\",\"eta\":\"{}\",\"lt\":\"{}\"}},\"error\":null}}",
                kernels.join(","),
                vector(&out),
                vector(&inp),
                alias(&s.alias()),
                cache(&s.cache_snap()),
                s.x1d8,
                scratch(&s.scratch()),
                bits(s.work),
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
fn btran_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let source = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_btran_rust/input.txt")).unwrap();
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    std::panic::set_hook(Box::new(|_| {}));
    for line in lines {
        let (case, body) = run(line);
        output.push(format!("{case};{body}"));
    }
    let _ = std::panic::take_hook();
    let dir = format!("{root}/target/m3_push_btran_rust");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), output.join("\n") + "\n").unwrap();
}
