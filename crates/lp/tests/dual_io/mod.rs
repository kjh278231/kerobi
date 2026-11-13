//! Shared JSON text for the `251b4f` replays (M3-04f-4a `xo_dual`, M3-04f-4b `push_init_ext` link): the field
//! names and encodings that tools/xo_dual_rust.py and tools/xo_link_rust.py read.
#![allow(dead_code)]
use rsolver_lp::crossover_btran::{BtranFactor, Scratch};
use rsolver_lp::crossover_dual::DualCall;
use rsolver_lp::crossover_ftran::{Buffer, VecSnap};

pub fn bits(v: f64) -> String {
    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
pub fn opt_i(x: &Option<i32>) -> String {
    x.map_or("null".into(), |v| v.to_string())
}
pub fn opt_f(x: &Option<f64>) -> String {
    x.map_or("null".into(), |v| format!("\"{}\"", bits(v)))
}
pub fn list<T>(xs: &[T], f: impl Fn(&T) -> String) -> String {
    format!("[{}]", xs.iter().map(f).collect::<Vec<_>>().join(","))
}
pub fn hexes(xs: &[f64]) -> String {
    list(xs, |v| format!("\"{}\"", bits(*v)))
}
pub fn vector(v: &VecSnap) -> String {
    let idx = v.idx.as_ref().map_or("null".into(), |i| list(i, opt_i));
    format!("{{\"count\":{},\"idx\":{idx},\"val\":{}}}", opt_i(&v.count), list(&v.val, opt_f))
}
fn name(b: Buffer) -> &'static str {
    match b {
        Buffer::In => "in",
        Buffer::Out => "out",
    }
}
pub fn alias(a: &[(Buffer, Buffer); 2]) -> String {
    format!(
        "{{\"in\":{{\"idx\":\"{}\",\"val\":\"{}\"}},\"out\":{{\"idx\":\"{}\",\"val\":\"{}\"}}}}",
        name(a[0].0),
        name(a[0].1),
        name(a[1].0),
        name(a[1].1)
    )
}
pub fn scratch(s: &Scratch) -> String {
    format!(
        "{{\"x1b0\":{},\"x1b8\":{}}}",
        list(&s.marks, |(i, v)| format!("[{i},{v}]")),
        list(&s.vals, |(i, v)| format!("[{i},\"{}\"]", bits(*v)))
    )
}

/// The `251b4f` result fields (without the caller's S+0x2d8 / r14) and the BTRAN branch names, as JSON members.
pub fn call_members(c: &DualCall, m: usize) -> String {
    let (out, inp) = c.btran.snapshot(m);
    let br = &c.btran.branches;
    format!(
        "\"rhs\":{},\"btran\":{{\"output\":{},\"input_after\":{},\"alias\":{},\"x200\":\"{}\",\"scratch\":{}}},\"y\":{},\"d\":{},\"work\":{},\"branches\":{{\"entry\":\"{}\",\"u\":\"{}\",\"kernel\":\"{}\",\"eta\":\"{}\",\"lt\":\"{}\"}}",
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

/// The factor as the space-separated field list of tools/xo_dual_rust.py `factor_fields` (lists are `len v…`).
pub fn factor_text(f: &BtranFactor, bucket: Option<&[i32]>) -> String {
    fn us(o: &mut Vec<String>, v: &[usize]) {
        o.push(v.len().to_string());
        o.extend(v.iter().map(|x| x.to_string()));
    }
    fn is(o: &mut Vec<String>, v: &[i32]) {
        o.push(v.len().to_string());
        o.extend(v.iter().map(|x| x.to_string()));
    }
    fn fs(o: &mut Vec<String>, v: &[f64]) {
        o.push(v.len().to_string());
        o.extend(v.iter().map(|&x| bits(x)));
    }
    let mut o = Vec::new();
    for p in [&f.relabel, &f.gather, &f.upos, &f.uord] {
        us(&mut o, p);
    }
    fs(&mut o, &f.diag);
    for r in &f.urow {
        us(&mut o, &r.cols);
        fs(&mut o, &r.vals);
    }
    is(&mut o, &f.flag);
    o.extend([f.t.to_string(), f.mult.to_string(), bits(f.f108), bits(f.f110), f.d8.to_string(), f.e4.to_string()]);
    o.push(f.ltail.to_string());
    for r in &f.lrow {
        us(&mut o, &r.cols);
        fs(&mut o, &r.vals);
    }
    o.push(f.levels.to_string());
    if f.levels >= 0 {
        us(&mut o, &f.lev);
        us(&mut o, &f.lst);
        is(&mut o, bucket.unwrap_or(&[]));
    }
    o.extend([f.f1e0.to_string(), f.f184.to_string(), f.etas.len().to_string()]);
    for e in &f.etas {
        o.push(e.row.to_string());
        us(&mut o, &e.idx);
        fs(&mut o, &e.val);
    }
    o.join(" ")
}
