//! Input-only replay of the M3-04c-21e push factor chains on `crossover_lu`: the factor state is injected only at
//! refactors (`036ade` output), everything between is the backend's own. Expected states are read only by
//! tools/push_lu_rust.py.
use rsolver_lp::crossover_btran::Cache;
use rsolver_lp::crossover_ftran::{Buffer, Cells, Spike, VecSnap};
use rsolver_lp::crossover_lu::*;
use rsolver_lp::crossover_refactor::{refactor, Alloc, Matrix, BUCKET_UNKNOWN};
use rsolver_lp::lu::BlockState;
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
fn longs(t: &mut SplitWhitespace<'_>) -> Vec<i64> {
    let n = next::<usize>(t);
    (0..n).map(|_| next(t)).collect()
}
fn ints(t: &mut SplitWhitespace<'_>) -> Vec<i32> {
    longs(t).into_iter().map(|x| x as i32).collect()
}
fn floats(t: &mut SplitWhitespace<'_>) -> Vec<f64> {
    let n = next::<usize>(t);
    (0..n).map(|_| float(t)).collect()
}
fn opt_ints(t: &mut SplitWhitespace<'_>) -> Option<Vec<i32>> {
    (next::<u8>(t) != 0).then(|| ints(t))
}
fn list<T>(xs: &[T], f: impl Fn(&T) -> String) -> String {
    format!("[{}]", xs.iter().map(f).collect::<Vec<_>>().join(","))
}
fn il(xs: &[i32]) -> String {
    list(xs, |v| v.to_string())
}
fn ll(xs: &[i64]) -> String {
    list(xs, |v| v.to_string())
}
fn fl(xs: &[f64]) -> String {
    list(xs, |v| format!("\"{}\"", bits(*v)))
}
fn oil(x: &Option<Vec<i32>>) -> String {
    x.as_ref().map_or("null".into(), |v| il(v))
}
fn opt_i(x: &Option<i32>) -> String {
    x.map_or("null".into(), |v| v.to_string())
}
fn opt_f(x: &Option<f64>) -> String {
    x.map_or("null".into(), |v| format!("\"{}\"", bits(v)))
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
fn cache_json(c: &Cache) -> String {
    let n = c.count.max(0) as usize;
    format!(
        "{{\"count\":{},\"idx\":{},\"val\":{}}}",
        c.count,
        list(&(0..n).map(|k| c.idx.peek(k)).collect::<Vec<_>>(), opt_i),
        list(&(0..n).map(|k| c.val.peek(k)).collect::<Vec<_>>(), opt_f)
    )
}

/// The factor in the capture's `btFactor` layout (tools/push_lu_agent.js).
fn snap(s: &LuState) -> String {
    let m = s.m;
    let sc = &s.sc;
    let cols = s.u_cols();
    let rows = s.u_rows();
    let etas = list(&s.etas, |e| {
        let (idx, val, _) = s.eta_entries(e);
        let (mi, mv) = s.eta_mod16(e);
        format!(
            "{{\"r\":{},\"len\":{},\"idx_off\":{},\"val_off\":{},\"idx_mod16\":{mi},\"val_mod16\":{mv},\"idx\":{},\"val\":{}}}",
            e.row,
            e.len,
            e.off,
            e.off,
            il(&idx),
            fl(&val)
        )
    });
    let n = s.spike.count.max(0) as usize;
    let sidx: Vec<Option<i32>> = (0..n).map(|k| s.spike.idx.peek(k)).collect();
    let sval: Vec<Option<f64>> = sidx.iter().map(|i| i.and_then(|i| s.spike.val.peek(i as usize))).collect();
    let marks: Vec<String> = (0..m).filter(|&i| s.mark[i] != 0).map(|i| format!("[{i},{}]", s.mark[i])).collect();
    let vals: Vec<String> =
        (0..m).filter(|&i| s.vals[i].to_bits() != 0).map(|i| format!("[{i},\"{}\"]", bits(s.vals[i]))).collect();
    format!(
        concat!(
            "{{\"m\":{m},\"zero\":\"{zero}\",\"header\":{header},",
            "\"perm\":{{\"x20\":{x20},\"x28\":{x28},\"x30\":{x30},\"x38\":{x38},\"x40\":{x40},\"x48\":{x48}}},",
            "\"diag\":{diag},\"L\":{{\"beg\":{lbeg},\"rows\":{lrows},\"vals\":{lvals},\"levels\":{x128},\"lev\":{llev},\"lst\":{llst}}},",
            "\"U\":{{\"col_start\":{cs},\"col_end\":{ce},\"col_bound\":{cb},\"col\":{col},\"row_start\":{rs},\"row_end\":{re},\"row\":{row}}},",
            "\"scalars\":{{\"xd8\":{xd8},\"xdc\":{xdc},\"xe4\":{xe4},\"xe8\":{xe8},\"xec\":{xec},\"x100\":{x100},\"x104\":{x104},",
            "\"x108\":\"{x108}\",\"x110\":\"{x110}\",\"x128\":{x128},\"x180\":{x180},\"x184\":{x184},\"x188\":{x188},\"x18c\":{x18c},",
            "\"x158\":{x158},\"x160\":{x160},\"x168\":{x168},\"x1f8\":\"{x1f8}\",\"pool_mod16\":[{pm0},{pm1}],\"x140\":{x140},\"x1e0\":{x1e0}}},",
            "\"prefix\":{{\"list\":{f0},\"flag\":{f8}}},\"eta_oldest\":{oldest},\"etas\":{etas},",
            "\"spike\":{{\"count\":{scount},\"indices\":{sidx},\"values\":{sval}}},",
            "\"scratch\":{{\"x1b0\":[{marks}],\"x1b8\":[{vals}]}},",
            "\"Lrow\":{{\"beg\":{rbeg},\"cols\":{rcols},\"vals\":{rvals},\"levels\":{x140},\"lev\":{rlev},\"lst\":{rlst},\"bucket\":{bucket}}},",
            "\"cache\":{cache},\"x1d8\":{x1d8}}}"
        ),
        m = m,
        zero = bits(s.zero),
        header = il(&s.header),
        x20 = il(&s.x20),
        x28 = il(s.x28_x38().0),
        x30 = il(&s.x30),
        x38 = il(s.x28_x38().1),
        x40 = il(&s.x40),
        x48 = il(&s.x48),
        diag = fl(&s.diag),
        lbeg = ll(&s.lbeg),
        lrows = il(&s.lrows),
        lvals = fl(&s.lvals),
        x128 = sc.x128,
        llev = oil(&s.llev),
        llst = oil(&s.llst),
        cs = ll(&s.cs),
        ce = ll(&s.ce),
        cb = ll(&s.cb),
        col = list(&cols, |(r, v)| format!("{{\"rows\":{},\"vals\":{}}}", il(r), fl(v))),
        rs = ll(&s.rs),
        re = ll(&s.re),
        row = list(&rows, |r| format!(
            "{{\"cols\":{},\"vals\":{}}}",
            list(&r.cols, |c| c.to_string()),
            fl(&r.vals)
        )),
        xd8 = sc.xd8,
        xdc = sc.xdc,
        xe4 = sc.xe4,
        xe8 = sc.xe8,
        xec = s.f0.len(),
        x100 = sc.x100,
        x104 = sc.x104,
        x108 = bits(sc.x108),
        x110 = bits(sc.x110),
        x180 = s.etas.len(),
        x184 = sc.x184,
        x188 = sc.x188,
        x18c = sc.x18c,
        x158 = sc.x158,
        x160 = sc.x160,
        x168 = sc.x168,
        x1f8 = bits(sc.x1f8),
        pm0 = sc.pool_mod16[0],
        pm1 = sc.pool_mod16[1],
        x140 = sc.x140,
        x1e0 = sc.x1e0,
        f0 = il(&s.f0),
        f8 = il(&s.f8),
        oldest = il(&s.oldest),
        etas = etas,
        scount = s.spike.count,
        sidx = list(&sidx, opt_i),
        sval = list(&sval, opt_f),
        marks = marks.join(","),
        vals = vals.join(","),
        rbeg = ll(&s.rbeg),
        rcols = il(&s.rcols),
        rvals = fl(&s.rvals),
        rlev = oil(&s.rlev),
        rlst = oil(&s.rlst),
        // the captured bucket is F+0x1a8[m..m+F140]; FTRAN A2 may have written further counts (F128 > F140)
        bucket = s.bucket.as_ref().map_or("null".into(), |b| list(&b[..(sc.x140 + 1) as usize], |&v| if v == BUCKET_UNKNOWN {
            "null".into()
        } else {
            v.to_string()
        })),
        cache = cache_json(&s.cache),
        x1d8 = s.x1d8,
    )
}

/// The refactor's factor as injected (lu:leave, without header, spike, cache and F1d8).
fn inject(t: &mut SplitWhitespace<'_>, m: usize, prev: Option<&LuState>) -> LuState {
    let zero = float(t);
    let (x20, x28, x30, x38, x40, x48) = (ints(t), ints(t), ints(t), ints(t), ints(t), ints(t));
    let (x40_is_x28, x48_is_x38) = (next::<u8>(t) != 0, next::<u8>(t) != 0);
    let diag = floats(t);
    let (lbeg, lrows, lvals) = (longs(t), ints(t), floats(t));
    let x128: i32 = next(t);
    let (llev, llst) = (opt_ints(t), opt_ints(t));
    let (rbeg, rcols, rvals) = (longs(t), ints(t), floats(t));
    let x140: i32 = next(t);
    let (rlev, rlst, bucket) = (opt_ints(t), opt_ints(t), opt_ints(t));
    let (cs, ce, cb, rs, re) = (longs(t), longs(t), longs(t), longs(t), longs(t));
    let ucol: Vec<(Vec<i32>, Vec<f64>)> = (0..m).map(|_| (ints(t), floats(t))).collect();
    let urow: Vec<(Vec<i32>, Vec<f64>)> = (0..m).map(|_| (ints(t), floats(t))).collect();
    let sc = Scalars {
        xd8: next(t),
        xdc: next(t),
        xe4: next(t),
        xe8: next(t),
        x100: next(t),
        x104: next(t),
        x108: float(t),
        x110: float(t),
        x128,
        x184: next(t),
        x188: next(t),
        x18c: next(t),
        x158: next(t),
        x160: next(t),
        x168: next(t),
        x1f8: float(t),
        pool_mod16: [next(t), next(t)],
        x140,
        x1e0: next(t),
    };
    let (f0, f8, oldest) = (ints(t), ints(t), ints(t));
    let ne: usize = next(t);
    let mut etas = Vec::new();
    let mut eta_entries = Vec::new();
    for _ in 0..ne {
        let (row, off) = (next::<i32>(t), next::<i64>(t));
        let (idx, val) = (ints(t), floats(t));
        etas.push(EtaRec { row, len: idx.len(), off });
        eta_entries.push((off, idx, val));
    }
    // 036ade C:362, C:5378: rows at the pool start, columns at pool + local_1c0 = F+0x160/2 after the factor
    assert_eq!(sc.x160 % 2, 0, "odd F+0x160 after a refactor");
    let colbase = sc.x160 / 2;
    let size = sc.x168.max(sc.x158) as usize;
    let mut pidx = Cells::unknown(size);
    let mut pval = Cells::unknown(size);
    for j in 0..m {
        for (k, (&i, &v)) in ucol[j].0.iter().zip(&ucol[j].1).enumerate() {
            let p = (colbase + cs[j]) as usize + k;
            pidx.set(p, i);
            pval.set(p, v);
        }
    }
    for i in 0..m {
        for (k, (&c, &v)) in urow[i].0.iter().zip(&urow[i].1).enumerate() {
            let p = rs[i] as usize + k;
            pidx.set(p, c);
            pval.set(p, v);
        }
    }
    for (off, idx, val) in &eta_entries {
        for (k, (&i, &v)) in idx.iter().zip(val).enumerate() {
            pidx.set(*off as usize + k, i);
            pval.set(*off as usize + k, v);
        }
    }
    let (spike, cache, mark, vals) = match prev {
        Some(p) => (p.spike.clone(), p.cache.clone(), p.mark.clone(), p.vals.clone()),
        None => (
            Spike { count: 0, idx: Cells::unknown(m), val: Cells::unknown(m) },
            Cache { count: 0, idx: Cells::unknown(m), val: Cells::unknown(m) },
            vec![0; m],
            vec![0.0; m],
        ),
    };
    let mut s = LuState {
        m,
        zero,
        header: vec![0; m],
        x20,
        x28,
        x30,
        x38,
        x40,
        x48,
        x40_is_x28,
        x48_is_x38,
        diag,
        lbeg,
        lrows,
        lvals,
        llev,
        llst,
        rbeg,
        rcols,
        rvals,
        rlev,
        rlst,
        bucket,
        rc: re.clone(), // C:2580–2593: F+0xc0 = F+0xb8 after a refactor
        cs,
        ce,
        cb,
        rs,
        re,
        colbase,
        pidx,
        pval,
        sc,
        f0,
        f8,
        oldest,
        etas,
        spike,
        cache,
        x1d8: -1,
        mark,
        vals,
    };
    // the spike value buffer is zero off its list (C invariant, docs/m3/primal_push_lu_rust.md §2)
    for i in 0..m {
        s.spike.val.set(i, 0.0);
    }
    s
}

fn run_case(lines: &[&str], out: &mut Vec<String>) {
    let mut st: Option<LuState> = None;
    let mut matrix: Option<Matrix> = None;
    let mut failed: Option<String> = None;
    // 21h: the 1da0b2 block state (S+0x2c0/0x4e0/0x4e4) the next refactor enters with; None = phase off (21g)
    let mut block: Option<BlockState> = None;
    for line in lines {
        let mut t = line.split_whitespace();
        let op: String = next(&mut t);
        let case: String = next(&mut t);
        let ev: usize = next(&mut t);
        let m: usize = next(&mut t);
        if let Some(msg) = &failed {
            out.push(format!("{case} {ev} {{\"op\":\"{op}\",\"error\":{msg:?},\"skipped\":true}}"));
            continue;
        }
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<String, String> {
            match op.as_str() {
                "M" => {
                    let (mm, n) = (next::<usize>(&mut t), next::<usize>(&mut t));
                    assert_eq!(mm, m);
                    let (beg, len, ind, val) = (longs(&mut t), ints(&mut t), ints(&mut t), floats(&mut t));
                    matrix = Some(Matrix { m, n, beg, len, ind, val });
                    Ok("{\"op\":\"M\"}".into())
                }
                "K" => {
                    // 21h: the block state of S at the push's first refactor (the capture's 036ade entry)
                    block = Some(BlockState { x2c0: next(&mut t), x4e0: next(&mut t), x4e4: next(&mut t) });
                    Ok("{\"op\":\"K\"}".into())
                }
                "G" => {
                    // 21g: the refactor's factor comes from the Rust factorization, nothing of the original's output
                    let a = matrix.as_ref().ok_or("refactor before the matrix")?;
                    let z = t.next().unwrap();
                    let tol = float(&mut t);
                    let alloc = if next::<u8>(&mut t) != 0 {
                        let x158 = next(&mut t);
                        let pool_mod16 = [next(&mut t), next(&mut t)];
                        Alloc { x158, pool_mod16, x188: next(&mut t), x1f8: float(&mut t) }
                    } else {
                        Alloc::of(st.as_ref().ok_or("refactor without an allocation")?)
                    };
                    let zero = match (z, st.as_ref()) {
                        ("-", Some(p)) => p.zero,
                        ("-", None) => return Err("ZERO unknown".into()),
                        (h, _) => float(&mut h.split_whitespace()),
                    };
                    let (header, leave, vstat) = (ints(&mut t), next::<i32>(&mut t), ints(&mut t));
                    let enter = block.unwrap_or(BlockState::OFF);
                    let mut b = enter;
                    let rf = refactor(a, &header, zero, tol, alloc, st.as_ref(), &mut b).map_err(|e| format!("{e:?}"))?;
                    // 05b257 C:1399 raises S+0x2c0 before every refactor; the push's refactors follow each other on S
                    let bj = block.map_or(String::new(), |_| {
                        format!(
                            ",\"block\":{{\"enter\":{{\"x2c0\":{},\"x4e0\":{},\"x4e4\":{}}},\"leave\":{{\"x4e0\":{},\"x4e4\":{}}}}}",
                            enter.x2c0, enter.x4e0, enter.x4e4, b.x4e0, b.x4e4
                        )
                    });
                    if block.is_some() {
                        block = Some(BlockState { x2c0: b.x2c0 + 1, ..b });
                    }
                    // M3-04c-21i: the dense switch (step, n, alignment, dense order and array) when the kernel took it
                    let dj = rf.dense.as_ref().map_or(String::new(), |d| {
                        let hx: Vec<String> = d.a_in.iter().map(|x| format!("\"{}\"", bits(*x))).collect();
                        format!(
                            ",\"dense\":{{\"steps\":{},\"n\":{},\"mat_mod16\":{},\"rows\":{},\"cols\":{},\"a\":[{}]}}",
                            d.step,
                            d.n,
                            d.mat16,
                            il(&d.rows.iter().map(|&x| x as i32).collect::<Vec<_>>()),
                            il(&d.cols_in.iter().map(|&x| x as i32).collect::<Vec<_>>()),
                            hx.join(",")
                        )
                    });
                    let mut s = rf.state;
                    let r = s.renumber(&header, leave, &vstat, &rf.o, &rf.n);
                    let json = format!(
                        "{{\"op\":\"G\",\"perm\":{{\"x20\":{},\"x28\":{},\"x30\":{},\"x38\":{}}},\"header\":{},\"leave\":{},\"vstat\":{},\"spike_count\":{},\"buffers\":{{\"spike_idx_is_old_header\":true,\"header_is_old_spike_idx\":true}},\"factor\":{}{}}}",
                        il(&s.x20),
                        il(&rf.o),
                        il(&s.x30),
                        il(&rf.n),
                        il(&r.header),
                        r.leave,
                        il(&r.vstat),
                        s.spike.count,
                        snap(&s),
                        bj
                    ) + "";
                    let json = if dj.is_empty() { json } else { format!("{}{}}}", &json[..json.len() - 1], dj) };
                    st = Some(s);
                    Ok(json)
                }
                "R" => {
                    let mut s = inject(&mut t, m, st.as_ref());
                    let (header, leave, vstat) = (ints(&mut t), next::<i32>(&mut t), ints(&mut t));
                    let (o, n) = (ints(&mut t), ints(&mut t));
                    if let Some(p) = st.as_ref() {
                        // the spike list entering the call is the backend's own
                        s.spike = p.spike.clone();
                    }
                    let r = s.renumber(&header, leave, &vstat, &o, &n);
                    let json = format!(
                        "{{\"op\":\"R\",\"header\":{},\"leave\":{},\"vstat\":{},\"spike_count\":{},\"buffers\":{{\"spike_idx_is_old_header\":true,\"header_is_old_spike_idx\":true}},\"factor\":{}}}",
                        il(&r.header),
                        r.leave,
                        il(&r.vstat),
                        s.spike.count,
                        snap(&s)
                    );
                    st = Some(s);
                    Ok(json)
                }
                "F" => {
                    let s = st.as_mut().ok_or("FTRAN before the first refactor")?;
                    let (count, idx, val) = (next::<i32>(&mut t), ints(&mut t), floats(&mut t));
                    let (save, spike) = (next::<i32>(&mut t), next::<u8>(&mut t) != 0);
                    if spike {
                        s.wrapper_clear();
                    }
                    let enter = snap(s);
                    let f = s.ftran(count, &idx, &val, save, spike).map_err(|e| format!("{e:?}"))?;
                    let (o, i) = f.snapshot(m);
                    let br = &f.branches;
                    Ok(format!(
                        "{{\"op\":\"F\",\"enter\":{enter},\"leave\":{},\"output\":{},\"input_after\":{},\"alias\":{},\"branches\":{{\"input\":\"{}\",\"l\":\"{}\",\"r\":\"{}\",\"spike\":\"{}\",\"u\":\"{}\"}}}}",
                        snap(s),
                        vector(&o),
                        vector(&i),
                        alias(&[(f.input.idx, f.input.val), (f.output.idx, f.output.val)]),
                        br.input,
                        br.l,
                        br.r,
                        br.spike,
                        br.u
                    ))
                }
                "B" => {
                    let s = st.as_mut().ok_or("BTRAN before the first refactor")?;
                    let (count, idx, val) = (next::<i32>(&mut t), ints(&mut t), floats(&mut t));
                    let (oc, cache, work) = (next::<i32>(&mut t), next::<u8>(&mut t) != 0, float(&mut t));
                    let enter = snap(s);
                    let b = s.btran(count, &idx, &val, oc, cache, work);
                    let (o, i) = b.snapshot(m);
                    let br = &b.branches;
                    Ok(format!(
                        "{{\"op\":\"B\",\"enter\":{enter},\"leave\":{},\"output\":{},\"input_after\":{},\"alias\":{},\"branches\":{{\"entry\":\"{}\",\"u\":\"{}\",\"kernel\":\"{}\",\"eta\":\"{}\",\"lt\":\"{}\"}}}}",
                        snap(s),
                        vector(&o),
                        vector(&i),
                        alias(&b.alias()),
                        br.entry,
                        br.u,
                        br.kernel,
                        br.eta,
                        br.lt
                    ))
                }
                "U" => {
                    let s = st.as_mut().ok_or("FT update before the first refactor")?;
                    let (r, header) = (next::<usize>(&mut t), ints(&mut t));
                    s.header = header;
                    let enter = snap(s);
                    let x1d8 = s.x1d8;
                    let rep = s.ft_update(r).map_err(|e| format!("{e:?}"))?;
                    // lu:roweta leave state: the cache right after 09e47c (it is emptied at the exit) is in the report
                    Ok(format!(
                        "{{\"op\":\"U\",\"enter\":{enter},\"roweta\":{},\"roweta_cache\":{},\"x1d8_enter\":{x1d8},\"leave\":{},\"report\":{{\"cache_reuse\":{},\"interval_trivial\":{},\"eta\":{},\"local80\":{},\"promoted\":{},\"cycle\":{},\"moved_column\":{},\"moved_rows\":{},\"dot_lanes\":{}}}}}",
                        rep.roweta.map_or("null".into(), |r| r.to_string()),
                        rep.roweta_cache.as_ref().map_or("null".into(), |c| format!(
                            "{{\"count\":{},\"idx\":{},\"val\":{}}}",
                            c.count,
                            list(&c.idx, opt_i),
                            list(&c.val, opt_f)
                        )),
                        snap(s),
                        rep.cache_reuse,
                        rep.interval_trivial,
                        rep.eta,
                        rep.local80,
                        rep.promoted,
                        rep.cycle,
                        rep.moved_column,
                        rep.moved_rows,
                        rep.dot_lanes
                    ))
                }
                _ => Err(format!("unknown op {op}")),
            }
        }));
        let r = match res {
            Ok(r) => r,
            Err(e) => Err(e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default()),
        };
        match r {
            Ok(json) => {
                assert!(t.next().is_none(), "{case} {ev}: trailing input");
                out.push(format!("{case} {ev} {json}"));
            }
            Err(msg) => {
                out.push(format!("{case} {ev} {{\"op\":\"{op}\",\"error\":{msg:?}}}"));
                failed = Some(msg);
            }
        }
    }
}

#[test]
fn lu_chain_trace() {
    replay("m3_push_lu_rust");
}

/// M3-04c-21g: the same chains with every refactor's factor generated by Rust (no injection).
#[test]
fn refactor_chain_trace() {
    replay("m3_push_refactor_rust");
}

/// M3-04c-21h: the generated chains with the 1da0b2 2×2 block phase on, its state chained from the push's first refactor.
#[test]
fn block_chain_trace() {
    replay("m3_push_block_rust");
}

/// M3-04c-21i: the push refactors of the dense-switch models (refactor boundary only: no FTRAN/BTRAN/FT between).
#[test]
fn dense_chain_trace() {
    replay("m3_push_dense_rust");
}

fn replay(dir: &str) {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let Ok(source) = std::fs::read_to_string(format!("{root}/bench/oracle/{dir}/input.txt")) else {
        panic!("missing bench/oracle/{dir}/input.txt (tools/push_lu_rust.py --write)");
    };
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    let body: Vec<&str> = lines.collect();
    let mut i = 0;
    while i < body.len() {
        let case = body[i].split_whitespace().nth(1).unwrap();
        let mut j = i;
        while j < body.len() && body[j].split_whitespace().nth(1).unwrap() == case {
            j += 1;
        }
        run_case(&body[i..j], &mut output);
        i = j;
    }
    let dir = format!("{root}/target/{dir}");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"), output.join("\n") + "\n").unwrap();
}
