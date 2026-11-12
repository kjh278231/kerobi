//! Input-only replay of the M3-04c-21a FTRAN calls; the original kernel/leave states are read only by
//! tools/push_ftran_rust.py.
use rsolver_lp::crossover_ftran::*;
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
fn spike(s: &Option<SpikeSnap>) -> String {
    s.as_ref().map_or("null".into(), |s| {
        format!("{{\"count\":{},\"indices\":{},\"values\":{}}}", s.count, list(&s.indices, opt_i), list(&s.values, opt_f))
    })
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

#[test]
fn ftran_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let source = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_ftran_rust/input.txt")).unwrap();
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let case: String = next(&mut t);
        let m: usize = next(&mut t);
        let zero = float(&mut t);
        let save: i32 = next(&mut t);
        let sp = if next::<u8>(&mut t) == 0 {
            None
        } else {
            let count = next(&mut t);
            let idx = ints(&mut t);
            let vals = floats(&mut t);
            let mut s = Spike { count, idx: Cells::unknown(m), val: Cells::unknown(m) };
            for (k, (&i, &v)) in idx.iter().zip(&vals).enumerate() {
                s.idx.set(k, i as i32);
                s.val.set(i as usize, v);
            }
            Some(s)
        };
        let count: i32 = next(&mut t);
        let idx: Vec<i32> = ints(&mut t).into_iter().map(|x| x as i32).collect();
        let val = floats(&mut t);
        let (gather, relabel, upos, uord) = (usizes(&mut t), usizes(&mut t), usizes(&mut t), usizes(&mut t));
        let diag = floats(&mut t);
        let (lbeg, lrow, lval) = (usizes(&mut t), usizes(&mut t), floats(&mut t));
        let ucol = (0..m)
            .map(|_| {
                let tail = next(&mut t);
                UColumn { tail, rows: usizes(&mut t), vals: floats(&mut t) }
            })
            .collect();
        let (d8, e4, tt, mult) = (next(&mut t), next(&mut t), next(&mut t), next(&mut t));
        let dsum = float(&mut t) + float(&mut t);
        let levels = next(&mut t);
        let (f0, f8, oldest) = (usizes(&mut t), ints(&mut t), ints(&mut t));
        let ne: usize = next(&mut t);
        let etas = (0..ne)
            .map(|_| {
                let row = next(&mut t);
                let peel = next::<u8>(&mut t) != 0;
                EtaRecord { row, peel, idx: usizes(&mut t), val: floats(&mut t) }
            })
            .collect();
        assert!(t.next().is_none(), "extra input {case}");
        let f = FtranFactor {
            m,
            zero,
            gather,
            relabel,
            upos,
            uord,
            diag,
            lbeg,
            lrow,
            lval,
            ucol,
            d8,
            e4,
            t: tt,
            mult,
            dsum,
            levels,
            llev: Vec::new(),
            llst: Vec::new(),
            f0,
            f8: f8.into_iter().map(|x| x as i32).collect(),
            oldest,
            etas,
        };
        let mut s = FtranState::new(m, count, &idx, &val, sp);
        let r = ftran(&f, &mut s, save);
        let kernels = list(&s.events, |e| {
            let out = e.output.as_ref().map_or(String::new(), |o| format!(",\"output\":{}", vector(o)));
            format!(
                "{{\"kernel\":\"{}\",\"phase\":\"{}\",\"input\":{}{out},\"alias\":{},\"spike\":{}}}",
                e.kernel,
                if e.leave { "leave" } else { "enter" },
                vector(&e.input),
                alias(&e.alias),
                spike(&e.spike)
            )
        });
        let (out, inp) = s.snapshot(m);
        let marks: Vec<String> = (0..m).filter(|&i| s.mark[i] != 0).map(|i| format!("[{i},{}]", s.mark[i])).collect();
        let vals: Vec<String> =
            (0..m).filter(|&i| s.vals[i].to_bits() != 0).map(|i| format!("[{i},\"{}\"]", bits(s.vals[i]))).collect();
        let leave = format!(
            "{{\"output\":{},\"input_after\":{},\"alias\":{},\"spike\":{},\"scratch\":{{\"x1b0\":[{}],\"x1b8\":[{}]}}}}",
            vector(&out),
            vector(&inp),
            alias(&[(s.input.idx, s.input.val), (s.output.idx, s.output.val)]),
            spike(&s.spike_snap()),
            marks.join(","),
            vals.join(",")
        );
        let br = &s.branches;
        let error = match r {
            Ok(()) => "null".to_string(),
            Err(e) => format!("\"{e:?}\""),
        };
        output.push(format!(
            "{case};{{\"error\":{error},\"kernels\":{kernels},\"leave\":{leave},\"branches\":{{\"input\":\"{}\",\"l\":\"{}\",\"r\":\"{}\",\"spike\":\"{}\",\"u\":\"{}\"}}}}",
            br.input, br.l, br.r, br.spike, br.u
        ));
    }
    let dir = format!("{root}/target/m3_push_ftran_rust");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), output.join("\n") + "\n").unwrap();
}

/// The same kernels on original `09d112` calls (M1-02j/02m captures): input and factor state in, output out.
#[test]
fn m1_kernel_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let source = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_ftran_rust/m1_kernels.txt")).unwrap();
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let kernel: String = next(&mut t);
        let case: String = next(&mut t);
        let m: usize = next(&mut t);
        let idx: Vec<i32> = ints(&mut t).into_iter().map(|x| x as i32).collect();
        let val = floats(&mut t);
        let mut f = FtranFactor {
            m,
            zero: 1e-13,
            gather: vec![],
            relabel: vec![],
            upos: vec![],
            uord: vec![],
            diag: vec![],
            lbeg: vec![],
            lrow: vec![],
            lval: vec![],
            ucol: vec![],
            d8: 0,
            e4: 0,
            t: m,
            mult: 0,
            dsum: 0.0,
            levels: -1,
            llev: Vec::new(),
            llst: Vec::new(),
            f0: vec![],
            f8: vec![],
            oldest: vec![],
            etas: vec![],
        };
        let mut s = FtranState::new(m, idx.len() as i32, &idx, &val, None);
        let r = if kernel == "21686f" {
            f.t = next(&mut t);
            f.dsum = float(&mut t) + float(&mut t);
            f.uord = usizes(&mut t);
            f.f0 = usizes(&mut t);
            f.f8 = ints(&mut t).into_iter().map(|x| x as i32).collect();
            f.diag = floats(&mut t);
            f.ucol = (0..m)
                .map(|_| {
                    let tail = next(&mut t);
                    UColumn { tail, rows: usizes(&mut t), vals: floats(&mut t) }
                })
                .collect();
            u_sparse(&f, &mut s);
            Ok(())
        } else {
            f.e4 = next(&mut t);
            f.levels = next(&mut t);
            f.lbeg = usizes(&mut t);
            f.lrow = usizes(&mut t);
            f.lval = floats(&mut t);
            l_sparse(&f, &mut s)
        };
        assert!(t.next().is_none(), "extra input {case}");
        let error = match r {
            Ok(()) => "null".to_string(),
            Err(e) => format!("\"{e:?}\""),
        };
        let (out, _) = s.snapshot(m);
        let scratch = s.mark.iter().all(|&x| x == 0) && s.vals.iter().all(|x| x.to_bits() == 0);
        output.push(format!(
            "{case};{{\"error\":{error},\"path\":\"{}\",\"output\":{{\"count\":{},\"idx\":{},\"val\":{},\"scratch_clean\":{scratch}}}}}",
            if kernel == "21686f" { s.branches.u } else { s.branches.l },
            opt_i(&out.count),
            out.idx.as_ref().map_or("null".into(), |i| list(i, opt_i)),
            list(&out.val, opt_f)
        ));
    }
    let dir = format!("{root}/target/m3_push_ftran_rust");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/m1_trace.txt"), output.join("\n") + "\n").unwrap();
}
