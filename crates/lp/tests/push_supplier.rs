//! Replay input states and external FTRAN outputs only; the Python comparator owns the expected answers.
use rsolver_lp::crossover_supplier::{supplier_next, PushColumn, SupplierBuffers, SupplierFtran, SupplierModel, SupplierState};
use std::str::SplitWhitespace;

fn next<T: std::str::FromStr>(t: &mut SplitWhitespace<'_>) -> T {
    t.next().unwrap().parse().ok().unwrap()
}
fn f(t: &mut SplitWhitespace<'_>) -> f64 {
    f64::from_bits(u64::from_str_radix(t.next().unwrap(), 16).unwrap().swap_bytes())
}
fn hex(v: f64) -> String { format!("{:016x}", v.to_bits().swap_bytes()) }
fn ints(v: &[i32]) -> String { v.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ") }
fn column(t: &mut SplitWhitespace<'_>, m: usize) -> PushColumn {
    let count: i32 = next(t);
    let indices = (0..count.max(0)).map(|_| next(t)).collect();
    let values = (0..if count < 0 { m } else { count as usize }).map(|_| f(t)).collect();
    PushColumn { count, indices, values }
}
fn show(v: &PushColumn) -> String {
    let mut fields = vec![v.count.to_string()];
    if v.count >= 0 { fields.extend(v.indices.iter().map(ToString::to_string)); }
    fields.extend(v.values.iter().map(|&x| hex(x)));
    fields.join(" ")
}
fn canary() -> PushColumn { PushColumn { count: 1, indices: vec![0], values: vec![-0.0] } }

struct Backend {
    raw: PushColumn,
    spike_after: PushColumn,
    calls: usize,
    input_seen: Option<PushColumn>,
    spike_address: usize,
    spike_same: bool,
    spike_present: bool,
    spike_before: Option<PushColumn>,
    output_address: usize,
    output_same: bool,
}
impl SupplierFtran for Backend {
    fn ftran(&mut self, input: &PushColumn, output: &mut PushColumn, spike: Option<&mut PushColumn>, work: &mut f64) {
        self.calls += 1;
        self.input_seen = Some(input.clone());
        self.spike_same &= spike.as_deref().is_none_or(|s| s as *const _ as usize == self.spike_address);
        self.output_same &= output as *const _ as usize == self.output_address;
        self.spike_present = spike.is_some();
        self.spike_before = spike.as_deref().cloned();
        if let Some(s) = spike { *s = self.spike_after.clone(); }
        *output = self.raw.clone();
        // Distinct canary: checks that the caller passes through this exact mutable work accumulator.
        *work += 0.5;
    }
}

#[test]
fn supplier_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let source = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_supplier_rust/input.txt")).unwrap();
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let name = t.next().unwrap();
        let (m, n, nnz): (usize, usize, usize) = (next(&mut t), next(&mut t), next(&mut t));
        let drop = f(&mut t);
        let (total, position, deferred, support): (usize, usize, usize, i32) =
            (next(&mut t), next(&mut t), next(&mut t), next(&mut t));
        let count_sum = f(&mut t);
        let successes = next(&mut t);
        let starts: Vec<i64> = (0..n).map(|_| next(&mut t)).collect();
        let lengths: Vec<i32> = (0..n).map(|_| next(&mut t)).collect();
        let rows: Vec<i32> = (0..nnz).map(|_| next(&mut t)).collect();
        let values: Vec<f64> = (0..nnz).map(|_| f(&mut t)).collect();
        let candidates = (0..total).map(|_| next(&mut t)).collect();
        let failures = (0..n+m).map(|_| next(&mut t)).collect();
        let mark_count: i32 = next(&mut t);
        let support_marks = if mark_count < 0 { None } else { Some((0..mark_count).map(|_| next(&mut t)).collect()) };
        let initial_spike = column(&mut t, m);
        let has_backend: i32 = next(&mut t);
        let raw = if has_backend != 0 { column(&mut t, m) } else { canary() };
        let spike_after = if has_backend != 0 { column(&mut t, m) } else { initial_spike.clone() };
        assert!(t.next().is_none(), "extra input in {name}");
        let model = SupplierModel { m, n, starts: &starts, lengths: &lengths, rows: &rows, values: &values, drop };
        let mut state = SupplierState { candidates, position, deferred, failures, support, support_marks, count_sum, successes };
        let mut buffers = SupplierBuffers { input: canary(), alpha: canary(), spike: initial_spike };
        let mut backend = Backend { raw, spike_after, calls: 0, input_seen: None,
                                    spike_address: &buffers.spike as *const _ as usize, spike_same: true,
                                    spike_present: false, spike_before: None,
                                    output_address: &buffers.alpha as *const _ as usize, output_same: true };
        let mut work = 42.25;
        let ret = supplier_next(&model, &mut state, &mut buffers, &mut work, &mut backend);
        let marks = state.support_marks.as_ref().map_or_else(|| "null".into(), |v| ints(v));
        output.push(format!("{name};ret {ret};position {};deferred {};support {};successes {};count_sum {};candidates {};failures {};support_marks {};ftran_count {};alpha {};input {};spike {};work {};output_same {};spike_present {};spike_before {};backend_input {};spike_same {}",
            state.position, state.deferred, state.support, state.successes, hex(state.count_sum), ints(&state.candidates),
            ints(&state.failures), marks, backend.calls, show(&buffers.alpha), show(&buffers.input), show(&buffers.spike),
            hex(work), backend.output_same, backend.spike_present,
            backend.spike_before.as_ref().map_or_else(|| "null".into(), show),
            backend.input_seen.as_ref().map_or_else(|| "null".into(), show), backend.spike_same));
    }
    let directory = format!("{root}/target/m3_push_supplier_rust");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(format!("{directory}/trace.txt"), output.join("\n")+"\n").unwrap();
}
