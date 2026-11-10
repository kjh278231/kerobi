//! L1 replay of M2-03d-2: the retry control runs on the original node basis with the original LP outcomes injected
//! and writes an `rsolver.mip_retry.v1` trace. The comparison is `tools/mip_retry_rust_expect.py --compare`; this
//! test only produces the trace. Synthetic LP outcomes: it says nothing about the Rust LP.
use rsolver_mip::retry::{process_node_lp, Ctx, NodeBasis, Outcome};
use std::fmt::Write;

fn ints(line: &str) -> Vec<i32> {
    line.split_whitespace().map(|v| v.parse().unwrap()).collect()
}

fn list(v: &[i32]) -> String {
    format!("[{}]", v.iter().map(i32::to_string).collect::<Vec<_>>().join(", "))
}

#[test]
fn nidda_stress_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/mip_retry_rust/nidda_stress.input.txt")).unwrap();
    let mut lines = text.lines();
    let head = ints(lines.next().unwrap());
    let basis = NodeBasis { vbasis: ints(lines.next().unwrap()), cbasis: ints(lines.next().unwrap()) };
    assert_eq!((basis.vbasis.len(), basis.cbasis.len()), (head[0] as usize, head[1] as usize));
    let ctx = Ctx { root: head[2] != 0, cache: head[3], warm_method: head[4] };
    let injected = [head[5], head[6]];

    let mut events = Vec::new();
    let outcome = process_node_lp(&ctx, &basis, |call| {
        let k = events.len() / 2;
        let status = injected[k];
        let (name, done) = if k == 0 { ("node_lp", "node_lp_done") } else { ("retry", "retry_done") };
        events.push(format!(r#"{{"ev": "{name}", "method": {}, "vbasis": {}, "cbasis": {}}}"#,
            call.method, list(&call.basis.vbasis), list(&call.basis.cbasis)));
        events.push(format!(r#"{{"ev": "{done}", "status": {status}}}"#));
        status
    });
    match outcome {
        Outcome::Retried { node_status, lp_status, basis_saved } => events.push(format!(
            r#"{{"ev": "node_done", "node_status": {node_status}, "lp_status": {lp_status}, "basis_saved": {basis_saved}}}"#)),
        other => events.push(format!(r#"{{"ev": "outcome", "value": "{other:?}"}}"#)),
    }

    let mut out = String::new();
    write!(out, r#"{{"schema": "rsolver.mip_retry.v1", "case": "nidda_stress", "branch": "node_basis_direct", "input": {{"cols": {}, "rows": {}, "root": {}, "cache": {}, "warm_method": {}, "node_basis": {{"vbasis": {}, "cbasis": {}}}}}, "injected": {{"first_lp_status": {}, "retry_lp_status": {}}}, "events": [{}]}}"#,
        head[0], head[1], ctx.root, ctx.cache, ctx.warm_method, list(&basis.vbasis), list(&basis.cbasis),
        injected[0], injected[1], events.join(", ")).unwrap();
    let dir = format!("{root}/target/mip_retry_rust");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/nidda_stress.trace.json"), out).unwrap();
}
