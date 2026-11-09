//! MPS writer (01 §1.5): the text `Model.write("*.mps")` produces, byte for byte. P0 compares the SHA-256 of the
//! presolved model written this way (docs/10 §6), so section order, padding and number format follow `a4590c`.
//! shortcut: linear part only (no Q, SOS, general constraints, multi-objective, names over 254 bytes); extend with
//! those sections when a compared model has them.
use crate::Model;
use std::fmt::Write;

/// `a8030d`: `%d` for integers below 2e9, else `%.15g`, else `%.16e` when `%.15g` does not round-trip.
pub fn fmt_num(x: f64) -> String {
    if x.abs() < 2e9 && x == x.floor() {
        return format!("{}", x as i32);
    }
    let g = c_g(x, 15);
    if g.parse::<f64>().map_or(true, |y| x - y != 0.0) {
        return c_exp(x, 16);
    }
    g
}

/// C `%.{prec}e`: Rust prints the same correctly rounded digits but a bare exponent ("e-1" for "e-01").
pub fn c_exp(x: f64, prec: usize) -> String {
    let s = format!("{:.*e}", prec, x);
    let (mant, exp) = s.split_once('e').unwrap();
    let e: i32 = exp.parse().unwrap();
    format!("{mant}e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs())
}

/// C `%.{prec}g`: style from the exponent of the rounded `%.{prec-1}e` form, trailing zeros removed.
pub fn c_g(x: f64, prec: usize) -> String {
    if !x.is_finite() {
        return if x.is_nan() { "nan".into() } else if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let s = format!("{:.*e}", prec - 1, x);
    let e: i32 = s.split_once('e').unwrap().1.parse().unwrap();
    let strip = |t: &str| -> String {
        if t.contains('.') { t.trim_end_matches('0').trim_end_matches('.').to_string() } else { t.to_string() }
    };
    if !(-4..prec as i32).contains(&e) {
        let (mant, _) = s.split_once('e').unwrap();
        return format!("{}e{}{:02}", strip(mant), if e < 0 { '-' } else { '+' }, e.abs());
    }
    strip(&format!("{:.*}", (prec as i32 - 1 - e) as usize, x))
}

/// `9c9635`: one empty name or one with a space or colon (or, for variables, a duplicate) drops the whole name
/// array, and `9c8829`/`9c89c5` then print `c<i>` for rows and `<vtype><j>(<column length>)` for columns.
fn names(m: &Model) -> (Vec<String>, Vec<String>) {
    let bad = |v: &[String]| v.iter().any(|s| s.is_empty() || s.contains([' ', ':']));
    let rows = if bad(&m.constr_names) {
        (0..m.num_constrs()).map(|i| format!("c{i}")).collect()
    } else {
        m.constr_names.clone()
    };
    let mut seen = std::collections::HashSet::new();
    let cols = if bad(&m.var_names) || !m.var_names.iter().all(|s| seen.insert(s)) {
        (0..m.num_vars()).map(|j| format!("{}{j}({})", m.vtype[j] as char, m.col_len[j])).collect()
    } else {
        m.var_names.clone()
    };
    (rows, cols)
}

pub fn to_string(m: &Model) -> String {
    let mut o = String::new();
    let (row_names, col_names) = names(m);
    let _ = writeln!(o, "NAME {}", m.name);
    // `obj` and `objcon` are as read (the MPS reader does not flip them for MAX)
    if m.sense < 0 {
        o.push_str("OBJSENSE MAX\n");
    }
    o.push_str("ROWS\n N  OBJ\n");
    for (i, &s) in m.row_sense.iter().enumerate() {
        let c = match s { b'=' => 'E', b'<' => 'L', _ => 'G' };
        let _ = writeln!(o, " {c}  {:<8}", row_names[i]);
    }
    o.push_str("COLUMNS\n");
    // `a7f063`: any non-continuous column makes it a MIP (integer markers, BV/LI bounds)
    let mip = m.vtype.iter().any(|&t| t != b'C');
    let mut marker = false;
    for (j, name) in col_names.iter().enumerate() {
        // integer columns ('B', 'I', 'N'; not 'S') sit between INTORG and INTEND markers
        let int = mip && !matches!(m.vtype[j], b'C' | b'S');
        if int != marker {
            let _ = writeln!(o, "    MARKER    'MARKER'                 '{}'", if int { "INTORG" } else { "INTEND" });
            marker = int;
        }
        let c = m.obj[j];
        if c != 0.0 || m.col_len[j] == 0 {
            let _ = writeln!(o, "    {name:<8}  OBJ       {}", fmt_num(c));
        }
        let b = m.col_beg[j] as usize;
        for k in b..b + m.col_len[j] as usize {
            let _ = writeln!(o, "    {name:<8}  {:<8}  {}", row_names[m.col_ind[k] as usize], fmt_num(m.col_val[k]));
        }
    }
    if marker {
        o.push_str("    MARKER    'MARKER'                 'INTEND'\n");
    }
    o.push_str("RHS\n");
    if m.objcon != 0.0 {
        let _ = writeln!(o, "    RHS1      OBJ       {}", fmt_num(-m.objcon));
    }
    for (i, &r) in m.rhs.iter().enumerate() {
        if r != 0.0 {
            let _ = writeln!(o, "    RHS1      {:<8}  {}", row_names[i], fmt_num(r));
        }
    }
    o.push_str("BOUNDS\n");
    for (j, name) in col_names.iter().enumerate() {
        let (mut l, mut u, t) = (m.lb[j], m.ub[j], m.vtype[j]);
        let bin = mip && t == b'B';
        if bin {
            // binaries are written clipped to [0, 1]
            u = u.min(1.0);
            l = l.max(0.0);
        }
        if l <= -1e30 {
            if u >= 1e30 {
                let _ = writeln!(o, " FR BND1      {name:<8}");
            } else {
                let _ = writeln!(o, " MI BND1      {name:<8}");
                let _ = writeln!(o, " UP BND1      {name:<8}  {}", fmt_num(u));
            }
            continue;
        }
        if l == u && t != b'S' && t != b'N' {
            let _ = writeln!(o, " FX BND1      {name:<8}  {}", fmt_num(l));
            if bin {
                let _ = writeln!(o, " BV BND1      {name:<8}");
            }
            continue;
        }
        if bin {
            let _ = writeln!(o, " BV BND1      {name:<8}");
            if l == 0.0 && u == 1.0 {
                continue;
            }
        }
        if mip && t == b'I' && u >= 1e30 {
            let _ = writeln!(o, " LI BND1      {name:<8}  {}", fmt_num(l));
        } else if l != 0.0 {
            let _ = writeln!(o, " LO BND1      {name:<8}  {}", fmt_num(l));
        }
        if t == b'S' || t == b'N' {
            let _ = writeln!(o, " SC BND1      {name:<8}  {}", fmt_num(u));
        } else if u < 1e30 {
            if u < 0.0 && l == 0.0 {
                let _ = writeln!(o, " LO BND1      {name:<8}  {}", fmt_num(l));
            }
            let _ = writeln!(o, " UP BND1      {name:<8}  {}", fmt_num(u));
        }
    }
    o.push_str("ENDATA\n");
    o
}

#[cfg(test)]
mod tests {
    use super::fmt_num;
    use crate::Model;

    #[test]
    fn integer_markers_and_bounds() {
        let m = Model {
            name: "t".into(),
            sense: 1,
            obj: vec![0.0; 5],
            lb: vec![0.0, 0.0, 0.0, -1e30, 0.5],
            ub: vec![1.0, 3.0, 1e30, 2.0, 0.5],
            vtype: b"BBICB".to_vec(),
            col_beg: vec![0; 5],
            col_len: vec![0; 5],
            var_names: ["a", "b", "c", "d", "e"].map(String::from).to_vec(),
            ..Default::default()
        };
        let s = super::to_string(&m);
        let cols = s.split_once("COLUMNS\n").unwrap().1.split_once("RHS\n").unwrap().0;
        assert_eq!(cols.matches("'INTORG'").count(), 2);
        assert_eq!(cols.matches("'INTEND'").count(), 2);
        // a [0,1] binary is BV only, a wider one is clipped to [0,1]; an integer with no ub gets LI; a fixed binary FX + BV
        let bounds = s.split_once("BOUNDS\n").unwrap().1;
        assert_eq!(bounds, " BV BND1      a       \n BV BND1      b       \n LI BND1      c         0\n MI BND1      d       \n UP BND1      d         2\n FX BND1      e         0.5\n BV BND1      e       \nENDATA\n");
    }

    #[test]
    fn number_format_matches_a8030d() {
        for (x, s) in [(80.0, "80"), (-0.4, "-0.4"), (0.301, "0.301"), (0.12615255658005028, "1.2615255658005028e-01"),
                       (0.04359999999999997, "4.3599999999999972e-02"), (-1.3199999999999998, "-1.3199999999999998e+00"),
                       (9567275.2, "9567275.2"), (3e9, "3000000000"), (1e30, "1e+30"), (1.5e-5, "1.5e-05")] {
            assert_eq!(fmt_num(x), s, "{x:e}");
        }
    }
}
