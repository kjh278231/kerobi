//! Input-only replay. Original poststates are read solely by the Python comparator.
use rsolver_lp::crossover_recompute::*;
use std::str::SplitWhitespace;

fn next<T: std::str::FromStr>(tokens: &mut SplitWhitespace<'_>) -> T {
    tokens.next().unwrap().parse().ok().unwrap()
}
fn float(tokens: &mut SplitWhitespace<'_>) -> f64 {
    let hex = tokens.next().unwrap();
    assert_eq!(hex.len(), 16);
    f64::from_le_bytes(std::array::from_fn(|i| u8::from_str_radix(&hex[i*2..i*2+2], 16).unwrap()))
}
fn floats(tokens: &mut SplitWhitespace<'_>, count: usize) -> Vec<f64> {
    // Give empty, distinct buffers different identities too.
    let mut result = Vec::with_capacity(count.max(1));
    result.extend((0..count).map(|_| float(tokens)));
    result
}
fn bits(v: f64) -> String {
    v.to_le_bytes().iter().map(|x| format!("{x:02x}")).collect()
}
fn array(values: &[f64]) -> String {
    format!("[{}]", values.iter().map(|v| format!("\"{}\"", bits(*v))).collect::<Vec<_>>().join(","))
}

#[test]
fn recompute_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let source = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_recompute_rust/input.txt")).unwrap();
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let name = t.next().unwrap();
        let m = next::<usize>(&mut t);
        let n = next::<usize>(&mut t);
        let infinity = float(&mut t);
        let tolerance = float(&mut t);
        let model_status = next(&mut t);
        let attempt = next(&mut t);
        let request = next(&mut t);
        let in_place = next::<u8>(&mut t) != 0;
        let header: Vec<i32> = (0..m).map(|_| next(&mut t)).collect();
        let lower = floats(&mut t, n);
        let upper = floats(&mut t, n);
        let raw = floats(&mut t, m);
        assert!(t.next().is_none(), "extra input: {name}");
        let input = RecomputeInput { header: &header, lower: &lower, upper: &upper,
            infinity, tolerance, model_status, attempt };
        let mut basic = Vec::with_capacity(m.max(1));
        basic.extend(if in_place { raw.clone() } else { vec![123.5; m] });
        let raw_pointer = if in_place { basic.as_ptr() } else { raw.as_ptr() };
        let mut width = vec![-88.0; m];
        let mut marks = vec![b'?'; m];
        let mut state = RecomputeState { pinf: -99.0, violations: 29, retry: 0, request };
        let values = if in_place { RecomputeValues::InPlace(&mut basic) }
                     else { RecomputeValues::Separate { raw: &raw, basic: &mut basic } };
        let exit = push_recompute(&input, values, &mut width, &mut marks, &mut state);
        let exit = match exit { RecomputeExit::Normal => "normal", RecomputeExit::Retry => "retry" };
        let raw_basic = raw_pointer == basic.as_ptr();
        let mark_json = marks.iter().map(|v| format!("\"{v:02x}\"")).collect::<Vec<_>>().join(",");
        output.push(format!("{name};{{\"exit\":\"{exit}\",\"basic\":{},\"width\":{},\"marks\":[{mark_json}],\"pinf\":\"{}\",\"violations\":{},\"retry\":{},\"request\":{},\"raw_basic\":{raw_basic},\"preserved\":{{\"header\":{header:?},\"lb\":{},\"ub\":{}}}}}",
            array(&basic), array(&width), bits(state.pinf), state.violations, state.retry, state.request,
            array(&lower), array(&upper)));
    }
    let directory = format!("{root}/target/m3_push_recompute_rust");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(format!("{directory}/trace.txt"), output.join("\n")+"\n").unwrap();
}
