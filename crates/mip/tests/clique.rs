//! `451bed` (`State::clique_table`) against Gurobi: each fixture in bench/gurobi_presolve_mip/clique holds the
//! presolve state at a `451bed` call (`tools/mip_state_dump.js`), the call's row list and `bin_only`, and the clique
//! table, row outputs and work Gurobi produced (Frida probe of the output arrays). Compared bit for bit.
use rsolver_mip::state::{Node, State, NIL};
use rsolver_mip::Params;
use rsolver_model::Model;
use std::collections::HashMap;

struct Tok<'a>(std::iter::Peekable<std::str::SplitAsciiWhitespace<'a>>);
impl Tok<'_> {
    fn s(&mut self) -> &str {
        self.0.next().expect("fixture ended early")
    }
    fn i(&mut self) -> i64 {
        self.s().parse().unwrap()
    }
    fn f(&mut self) -> f64 {
        self.s().parse().unwrap()
    }
    fn vi(&mut self, k: usize) -> Vec<i32> {
        (0..k).map(|_| self.i() as i32).collect()
    }
    fn vf(&mut self, k: usize) -> Vec<f64> {
        (0..k).map(|_| self.f()).collect()
    }
}

/// The dumped state on an empty model of the same size; live nodes are shared by their row and column lists, removed
/// ones (column -1) get a node per list, which is all `451bed` can tell apart.
fn load(text: &str) -> (State, Option<Vec<i32>>, bool, Tok<'_>) {
    let body = text.split_once('\n').unwrap().1; // first line: a comment
    let mut t = Tok(body.split_ascii_whitespace().peekable());
    let (n, m) = (t.i() as usize, t.i() as usize);
    let (mu, work, nbin, bin_only, nlist) = (t.f(), t.f(), t.i() as i32, t.i() != 0, t.i());
    let list = (nlist >= 0).then(|| t.vi(nlist as usize));
    let md = Model {
        sense: 1,
        obj: vec![0.0; n],
        lb: vec![0.0; n],
        ub: vec![1.0; n],
        vtype: vec![b'C'; n],
        col_beg: vec![0; n],
        col_len: vec![0; n],
        row_sense: vec![b'<'; m],
        rhs: vec![0.0; m],
        var_names: vec![String::new(); n],
        constr_names: vec![String::new(); m],
        ..Model::default()
    };
    let mut st = State::new(&md, &Params::default());
    st.lb = t.vf(n);
    st.ub = t.vf(n);
    st.obj = t.vf(n);
    st.vtype = t.s().as_bytes().to_vec();
    st.rhs = t.vf(m);
    st.sense = t.s().as_bytes().to_vec();
    st.row_cnt = t.vi(m);
    st.col_cnt = t.vi(n);
    st.inf_max = t.vi(m);
    st.inf_min = t.vi(m);
    st.act_max = t.vf(m);
    st.act_min = t.vf(m);
    st.nd.clear();
    let mut live = HashMap::new();
    let link = |nd: &mut Vec<Node>, head: &mut u32, tail: &mut u32, k: u32, row: bool| {
        if *tail == NIL {
            *head = k;
        } else if row {
            nd[*tail as usize].nr = k;
        } else {
            nd[*tail as usize].nc = k;
        }
        *tail = k;
    };
    for r in 0..m {
        let mut tail = NIL;
        for _ in 0..t.i() {
            let (col, val) = (t.i() as i32, t.f());
            let k = st.nd.len() as u32;
            st.nd.push(Node { val, row: r as i32, col, nr: NIL, nc: NIL });
            if col >= 0 {
                live.insert((r as i32, col), k);
            }
            link(&mut st.nd, &mut st.row_head[r], &mut tail, k, true);
        }
    }
    for j in 0..n {
        let mut tail = NIL;
        for _ in 0..t.i() {
            let (row, col, val) = (t.i() as i32, t.i() as i32, t.f());
            let k = if col >= 0 {
                live[&(row, col)]
            } else {
                st.nd.push(Node { val, row, col, nr: NIL, nc: NIL });
                st.nd.len() as u32 - 1
            };
            link(&mut st.nd, &mut st.col_head[j], &mut tail, k, false);
        }
    }
    st.work_mult = mu;
    st.work = work;
    st.nbin = nbin;
    (st, list, bin_only, t)
}

fn check(name: &str) {
    let path = format!("{}/../../bench/gurobi_presolve_mip/clique/{name}.txt", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap();
    let (mut st, list, bin_only, mut t) = load(&text);
    let got = st.clique_table(&Params::default(), list.as_deref(), bin_only).unwrap();
    let ncl = t.i() as usize;
    let work = t.f();
    assert_eq!(got.len(), ncl, "{name}: cliques");
    assert_eq!(st.work.to_bits(), work.to_bits(), "{name}: work {} vs {work}", st.work);
    if ncl == 0 {
        assert_eq!(got, Default::default(), "{name}");
        return;
    }
    let nlit = {
        let start = t.vi(ncl + 1);
        assert_eq!(got.start, start, "{name}: start");
        start[ncl] as usize
    };
    assert_eq!(got.lits, t.vi(nlit), "{name}: lits");
    assert_eq!(got.sense, t.s().as_bytes(), "{name}: sense");
    let flag = t.vi(ncl);
    if flag[0] >= 0 {
        assert_eq!(got.flag, flag, "{name}: flag");
    }
    let (nrec, maxlen) = (t.i() as usize, t.i() as i32);
    assert_eq!((got.rec_rows.len(), got.maxlen), (nrec, maxlen), "{name}: rows, maxlen");
    assert_eq!(got.rec_rows, t.vi(nrec), "{name}: rows");
    let rf = t.vi(nrec);
    if rf.first().is_some_and(|&x| x >= 0) {
        assert_eq!(got.rec_flag.iter().map(|&x| x as i32).collect::<Vec<_>>(), rf, "{name}: row flags");
    }
    assert_eq!(got.colstart, t.vi(st.n + 1), "{name}: colstart");
    assert_eq!(got.cidx, t.vi(nlit), "{name}: cidx");
}

/// 45885d's call: row cliques only (all `=`), the binary-pair pass runs without finding pairs
#[test]
fn glass4_rows() {
    check("glass4_451_1");
}

/// 40 implied pairs from the binary pass (`<` cliques of continuous literals)
#[test]
fn neos859080_pairs() {
    check("neos859080_451_1");
}

/// partly covered rows (recorded rows with and without a clique)
#[test]
fn sp150x300d_partial() {
    check("sp150x300d_451_1");
}

/// no clique: outputs stay empty, the binary pass is skipped (150 binaries > n - 2)
#[test]
fn mas74_none() {
    check("mas74_451_1");
}

/// probing's call: all rows, binaries only
#[test]
fn graphdraw_probing() {
    check("graphdraw-domain_451_3");
}
