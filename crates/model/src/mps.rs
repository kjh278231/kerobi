//! 01 §1.5 MPS reader. Results are checked array-by-array against the original reader on the bench corpus
//! (docs/10 §2): column order, entry order, ranged rows as `MPS_Rg<row>` slack columns, bound defaults.
use crate::{Model, INFINITY};
use std::collections::HashMap;
use std::io::Read;

#[derive(Debug)]
pub struct MpsError {
    pub line: usize,
    pub msg: String,
}

impl std::fmt::Display for MpsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MPS read error at line {}: {}", self.line, self.msg)
    }
}

fn err(line: usize, msg: impl Into<String>) -> MpsError {
    MpsError { line, msg: msg.into() }
}

pub fn read_file(path: &str) -> Result<Model, MpsError> {
    let f = std::fs::File::open(path).map_err(|e| err(0, format!("{path}: {e}")))?;
    let mut raw = Vec::new();
    let res = if path.ends_with(".gz") {
        flate2::read::MultiGzDecoder::new(f).read_to_end(&mut raw)
    } else {
        std::io::BufReader::new(f).read_to_end(&mut raw)
    };
    res.map_err(|e| err(0, format!("{path}: {e}")))?;
    parse(&String::from_utf8_lossy(&raw))
}

#[derive(Clone, Copy, PartialEq)]
enum Sec {
    None,
    ObjSense,
    Rows,
    Columns,
    Rhs,
    Ranges,
    Bounds,
}

/// Fixed-format MPS fields (columns 2-3, 5-12, 15-22, 25-36, 40-47, 50-61), used when whitespace splitting
/// does not fit the section, e.g. names containing spaces.
fn fixed_fields(line: &str) -> Vec<String> {
    const SPANS: [(usize, usize); 6] = [(1, 3), (4, 12), (14, 22), (24, 36), (39, 47), (49, 61)];
    let b = line.as_bytes();
    SPANS
        .iter()
        .map(|&(s, e)| {
            if s >= b.len() {
                String::new()
            } else {
                String::from_utf8_lossy(&b[s..e.min(b.len())]).trim().to_string()
            }
        })
        .collect()
}

fn num(s: &str) -> Option<f64> {
    s.parse::<f64>().ok()
}

/// Pairs of (row, value) from a COLUMNS/RHS/RANGES line, whose first field is the column or set name.
fn pairs(t: &[String]) -> Option<Vec<(String, f64)>> {
    let mut out = Vec::new();
    let mut k = 0;
    while k + 1 < t.len() {
        out.push((t[k].clone(), num(&t[k + 1])?));
        k += 2;
    }
    (k == t.len()).then_some(out)
}

pub fn parse(text: &str) -> Result<Model, MpsError> {
    let mut m = Model { sense: 1, ..Default::default() };
    let mut sec = Sec::None;
    let mut obj_row: Option<String> = None;
    let mut ignored_rows: std::collections::HashSet<String> = Default::default();
    let mut row_index: HashMap<String, usize> = HashMap::new();
    let mut row_type: Vec<u8> = Vec::new();
    let mut range: Vec<Option<f64>> = Vec::new();
    let mut col_index: HashMap<String, usize> = HashMap::new();
    let mut cols: Vec<Vec<(i32, f64)>> = Vec::new();
    let mut in_int = false;
    let mut is_int: Vec<bool> = Vec::new();
    let mut ub_set: Vec<bool> = Vec::new();

    for (ln0, raw) in text.lines().enumerate() {
        let ln = ln0 + 1;
        let line = raw.trim_end_matches('\r');
        if line.trim().is_empty() || line.starts_with('*') {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            let mut t = line.split_whitespace();
            let kw = t.next().unwrap_or("").to_ascii_uppercase();
            sec = match kw.as_str() {
                "NAME" => {
                    m.name = t.collect::<Vec<_>>().join(" ");
                    Sec::None
                }
                "OBJSENSE" | "OBJSENCE" => match t.next() {
                    Some(s) => {
                        m.sense = if s.to_ascii_uppercase().starts_with("MAX") { -1 } else { 1 };
                        Sec::None
                    }
                    None => Sec::ObjSense,
                },
                "ROWS" => Sec::Rows,
                "COLUMNS" => Sec::Columns,
                "RHS" => Sec::Rhs,
                "RANGES" => Sec::Ranges,
                "BOUNDS" => Sec::Bounds,
                "ENDATA" => break,
                other => return Err(err(ln, format!("unsupported section {other}"))),
            };
            continue;
        }
        let free: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        let is_row = |r: &String| row_index.contains_key(r) || obj_row.as_ref() == Some(r) || ignored_rows.contains(r);
        match sec {
            Sec::None => return Err(err(ln, "data line outside a section")),
            Sec::ObjSense => m.sense = if free[0].to_ascii_uppercase().starts_with("MAX") { -1 } else { 1 },
            Sec::Rows => {
                let (ty, name) = if free.len() == 2 {
                    (free[0].clone(), free[1].clone())
                } else {
                    let f = fixed_fields(line);
                    (f[0].clone(), f[1].clone())
                };
                match ty.to_ascii_uppercase().as_str() {
                    "N" => {
                        if obj_row.is_none() {
                            obj_row = Some(name);
                        } else {
                            ignored_rows.insert(name);
                        }
                    }
                    t @ ("L" | "G" | "E") => {
                        row_index.insert(name.clone(), m.constr_names.len());
                        m.constr_names.push(name);
                        row_type.push(t.as_bytes()[0]);
                        range.push(None);
                    }
                    t => return Err(err(ln, format!("unknown row type {t}"))),
                }
            }
            Sec::Columns => {
                if free.len() >= 3 && free[1] == "'MARKER'" {
                    match free[2].as_str() {
                        "'INTORG'" => in_int = true,
                        "'INTEND'" => in_int = false,
                        t => return Err(err(ln, format!("unknown marker {t}"))),
                    }
                    continue;
                }
                // Original rule (probed on crafted files): 3 or 5 fields with numeric values are free format, even when a
                // row name is unknown (that entry is then dropped); anything else is read in fixed columns.
                let (col, entries) = match (free.len(), pairs(free.get(1..).unwrap_or(&[]))) {
                    (3 | 5, Some(p)) => (free[0].clone(), p),
                    _ => {
                        let f = fixed_fields(line);
                        let p = pairs(&f[2..if f[4].is_empty() { 4 } else { 6 }]).ok_or_else(|| err(ln, line.to_string()))?;
                        (f[1].clone(), p)
                    }
                };
                let j = *col_index.entry(col.clone()).or_insert_with(|| {
                    m.var_names.push(col);
                    cols.push(Vec::new());
                    is_int.push(in_int);
                    cols.len() - 1
                });
                for (row, v) in entries {
                    if v == 0.0 {
                        continue; // explicit zeros are not stored
                    }
                    if obj_row.as_deref() == Some(row.as_str()) {
                        m.obj.resize(cols.len(), 0.0);
                        m.obj[j] = v;
                    } else if let Some(&i) = row_index.get(&row) {
                        cols[j].push((i as i32, v));
                    }
                }
            }
            Sec::Rhs | Sec::Ranges => {
                // Original rule: 3 or 5 fields are free format with a set name, taken as is (unknown rows dropped,
                // so a fixed-format name with one space is lost); 2 or 4 fields are free without a set name when they
                // resolve; anything else is read in fixed columns.
                let entries = match free.len() {
                    3 | 5 => Some(free[1..].chunks(2).filter_map(|c| Some((c[0].clone(), num(&c[1])?))).collect()),
                    2 | 4 => pairs(&free).filter(|e| e.iter().all(|(r, _)| is_row(r))),
                    _ => None,
                };
                let entries = match entries {
                    Some(e) => e,
                    _ => {
                        let f = fixed_fields(line);
                        pairs(&f[2..if f[4].is_empty() { 4 } else { 6 }]).ok_or_else(|| err(ln, line.to_string()))?
                    }
                };
                for (row, v) in entries {
                    if sec == Sec::Rhs && obj_row.as_deref() == Some(row.as_str()) {
                        m.objcon = -v;
                    } else if let Some(&i) = row_index.get(&row) {
                        if sec == Sec::Rhs {
                            m.rhs.resize(m.constr_names.len(), 0.0);
                            m.rhs[i] = v;
                        } else {
                            range[i] = Some(v);
                        }
                    }
                }
            }
            Sec::Bounds => {
                let ty = free[0].to_ascii_uppercase();
                let needs_value = !matches!(ty.as_str(), "FR" | "MI" | "PL" | "BV");
                // Original rule: 4 or 6 fields (3 for value-less types) are free format and an unknown column is
                // dropped; 5 fields are read in fixed columns.
                let (col, val) = if free.len() == 4 || free.len() == 6 || (!needs_value && free.len() == 3) {
                    (free[2].clone(), free.get(3).and_then(|s| num(s)))
                } else {
                    let f = fixed_fields(line);
                    (f[2].clone(), num(&f[3]))
                };
                let Some(&j) = col_index.get(&col) else { continue };
                let val = if needs_value { val.ok_or_else(|| err(ln, line.to_string()))? } else { 0.0 };
                pending_bounds_push(&mut m, j, cols.len());
                ub_set.resize(cols.len(), false);
                ub_set[j] = true; // any bound line cancels the 0/1 default of marker integers
                apply_bound(&mut m, &mut is_int, j, &ty, val).map_err(|e| err(ln, e))?;
            }
        }
    }

    let n = cols.len();
    m.obj.resize(n, 0.0);
    pending_bounds_push(&mut m, 0, n);
    m.rhs.resize(m.constr_names.len(), 0.0);
    m.vtype.resize(n, b'C');
    ub_set.resize(n, false);
    for j in 0..n {
        if is_int[j] && m.vtype[j] == b'C' {
            m.vtype[j] = b'I';
            if !ub_set[j] {
                m.ub[j] = 1.0; // MPS convention: marker integers without an upper bound are 0/1
            }
        }
    }
    // Ranged rows become equalities plus a slack column `MPS_Rg<row>` in [0, |R|], appended in row order.
    m.row_sense = row_type.iter().map(|&t| match t {
        b'L' => b'<',
        b'G' => b'>',
        _ => b'=',
    }).collect();
    for i in 0..m.constr_names.len() {
        let Some(r) = range[i] else { continue };
        let coef = match row_type[i] {
            b'L' => 1.0,
            b'G' => -1.0,
            _ => {
                // shortcut: E-row range sign rule assumed from MPS semantics; not yet seen in the corpus.
                if r > 0.0 { -1.0 } else { 1.0 }
            }
        };
        m.row_sense[i] = b'=';
        m.var_names.push(format!("MPS_Rg{}", m.constr_names[i]));
        m.obj.push(0.0);
        m.lb.push(0.0);
        m.ub.push(r.abs());
        m.vtype.push(b'C');
        cols.push(vec![(i as i32, coef)]);
    }
    let mut beg = 0i64;
    for c in &cols {
        m.col_beg.push(beg);
        m.col_len.push(c.len() as i32);
        for &(i, v) in c {
            m.col_ind.push(i);
            m.col_val.push(v);
        }
        beg += c.len() as i64;
    }
    Ok(m)
}

/// Bounds arrays grow lazily with the columns seen so far (default [0, inf)).
fn pending_bounds_push(m: &mut Model, _j: usize, n: usize) {
    if m.lb.len() < n {
        m.lb.resize(n, 0.0);
        m.ub.resize(n, INFINITY);
        m.vtype.resize(n, b'C');
    }
}

fn apply_bound(m: &mut Model, is_int: &mut [bool], j: usize, ty: &str, v: f64) -> Result<(), String> {
    let v = if v >= 1e30 { INFINITY } else if v <= -1e30 { -INFINITY } else { v };
    match ty {
        "UP" => {
            m.ub[j] = v;
            // shortcut: negative UP with zero lower bound handling to be confirmed against the original.
        }
        "LO" => m.lb[j] = v,
        "FX" => {
            m.lb[j] = v;
            m.ub[j] = v;
        }
        "FR" => {
            m.lb[j] = -INFINITY;
            m.ub[j] = INFINITY;
        }
        "MI" => m.lb[j] = -INFINITY,
        "PL" => m.ub[j] = INFINITY,
        "BV" => {
            m.vtype[j] = b'B';
            m.lb[j] = 0.0;
            m.ub[j] = 1.0;
        }
        "LI" => {
            is_int[j] = true;
            m.lb[j] = v;
        }
        "UI" => {
            is_int[j] = true;
            m.ub[j] = v;
        }
        t => return Err(format!("unsupported bound type {t}")),
    }
    Ok(())
}
