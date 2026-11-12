//! Input-only replay; original poststates are read only by the Python comparator.
use rsolver_lp::crossover_round::*;
use rsolver_lp::crossover_supplier::PushColumn;
use std::str::SplitWhitespace;

fn next<T: std::str::FromStr>(t: &mut SplitWhitespace<'_>) -> T {
    t.next().unwrap().parse().ok().unwrap()
}
fn float(t: &mut SplitWhitespace<'_>) -> f64 {
    let h = t.next().unwrap();
    assert_eq!(h.len(), 16);
    f64::from_le_bytes(std::array::from_fn(|i| u8::from_str_radix(&h[i*2..i*2+2],16).unwrap()))
}
fn bits(v: f64) -> String { v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect() }
fn ints(t: &mut SplitWhitespace<'_>) -> Vec<i32> {
    let n = next::<usize>(t);
    (0..n).map(|_| next(t)).collect()
}
fn optional(t: &mut SplitWhitespace<'_>) -> Option<i32> {
    match t.next().unwrap() { "_" => None, s => Some(s.parse().unwrap()) }
}
fn mark_json(x: &Option<Vec<i32>>) -> String { x.as_ref().map_or("null".into(), |v| format!("{v:?}")) }
fn column(t: &mut SplitWhitespace<'_>) -> PushColumn {
    let count = next(t);
    let indices = ints(t);
    let n = next::<usize>(t);
    let values = (0..n).map(|_| float(t)).collect();
    PushColumn { count, indices, values }
}
fn vector(v: &PushColumn) -> String {
    let values = v.values.iter().map(|x| format!("\"{}\"", bits(*x))).collect::<Vec<_>>().join(",");
    let indices = if v.count < 0 { "null".into() } else { format!("{:?}",v.indices) };
    format!("{{\"count\":{},\"indices\":{indices},\"values\":[{values}]}}",v.count)
}
struct Reply { ret: i32, fields: [Option<i32>;4] }
struct Backend { raw: Option<PushColumn>, replies: Vec<Reply>, cursor: usize, calls: Vec<String> }
impl RoundBtran for Backend {
    fn btran(&mut self, unit: &PushColumn, _work: &mut f64) -> PushColumn {
        self.calls.push(format!("{{\"event\":\"btran\",\"input\":{}}}",vector(unit)));
        self.raw.take().expect("unexpected BTRAN call")
    }
}
impl RoundFactor for Backend {
    fn factor(&mut self, state: &mut RoundState) -> i32 {
        let request = state.request;
        let r = &self.replies[self.cursor]; self.cursor += 1;
        let mut reply = format!("\"ret\":{}",r.ret);
        for (key, field, value) in ["request","status","numeric","failure"].into_iter()
            .zip([&mut state.request,&mut state.status,&mut state.numeric,&mut state.failure])
            .zip(r.fields).map(|((k,f),v)|(k,f,v)) {
            if let Some(v) = value { reply.push_str(&format!(",\"{key}\":{v}")); *field = v; }
        }
        // Requests are observed before the backend changes them.
        self.calls.push(format!("{{\"event\":\"factor\",\"request\":{request},\"reply\":{{{reply}}}}}"));
        r.ret
    }
}

#[test]
fn round_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"),"/../..");
    let source = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_round_rust/input.txt")).unwrap();
    let mut lines = source.lines();
    let mut output = vec![lines.next().unwrap().to_string()];
    for line in lines {
        let mut t = line.split_whitespace();
        let name = t.next().unwrap(); let kind = t.next().unwrap();
        // Capture availability only; no expected value is given to Rust.
        let observe_approved = next::<u8>(&mut t) != 0;
        let m = next(&mut t); let candidates = ints(&mut t);
        let position = next(&mut t); let deferred = next(&mut t); let failures = ints(&mut t);
        let marks = if next::<u8>(&mut t) == 0 { None } else { Some(ints(&mut t)) };
        let support = next(&mut t); let approved = optional(&mut t); let count_sum = float(&mut t);
        let successes = next(&mut t); let request = next(&mut t); let status = next(&mut t);
        let numeric = next(&mut t); let failure = next(&mut t); let retry = next(&mut t);
        let frame = RoundFrame { attempt: next(&mut t), numeric_limit: next(&mut t) };
        let enter = next(&mut t); let leave = next(&mut t);
        let raw = if next::<u8>(&mut t) == 0 { None } else { Some(column(&mut t)) };
        let n = next::<usize>(&mut t);
        let replies = (0..n).map(|_| Reply { ret: next(&mut t), fields: std::array::from_fn(|_|optional(&mut t)) }).collect();
        assert!(t.next().is_none(),"extra input {name}");
        let mut state = RoundState { m,candidates,position,deferred,failures,marks,support,approved,count_sum,
                                    successes,request,status,numeric,failure,retry };
        let mut backend = Backend { raw,replies,cursor:0,calls:vec![] };
        let mut work = 0.0;
        let exit = match kind {
            "defer" => { round_defer(&mut state,enter); RoundExit::Continue },
            "pass" => { round_pass(&mut state); RoundExit::Continue },
            "accept" | "begin_accept" => {
                if kind == "begin_accept" { round_begin(&mut state); }
                round_accept(&mut state); RoundExit::Continue
            },
            "exhausted" => {
                if round_exhausted(&mut state) == RoundExit::End { RoundExit::End }
                else { round_restart(&mut state) }
            },
            "gate" => round_exhausted(&mut state),
            "support" => { round_support(&mut state); RoundExit::Continue },
            "restart" => round_restart(&mut state),
            "btran" => round_btran(&mut state,leave,&mut work,&mut backend),
            "wrapper" => round_factor(&mut state,&frame,&mut backend),
            "clear" => { round_clear(&mut state); RoundExit::Continue },
            _ => panic!("unknown kind {kind}"),
        };
        // The comparator observes missing calls directly, including behavioral mutations
        // which return before consuming all original backend replies.
        let (exit,code) = match exit {
            RoundExit::Continue => ("continue",0), RoundExit::End => ("end",0),
            RoundExit::Factor => ("factor",0), RoundExit::Recompute => ("recompute",0),
            RoundExit::Retry => ("retry",0), RoundExit::Error(code) => ("error",code),
        };
        let s = &state;
        let approved = if observe_approved {
            format!(",\"approved\":{}",s.approved.map_or("null".into(), |v| v.to_string()))
        } else { String::new() };
        output.push(format!("{name};{{\"exit\":\"{exit}\",\"return_code\":{code},\"m\":{},\"total\":{},\"candidates\":{:?},\"position\":{},\"deferred\":{},\"failures\":{:?},\"marks\":{},\"support\":{}{approved},\"count_sum\":\"{}\",\"successes\":{},\"request\":{},\"status\":{},\"numeric\":{},\"failure\":{},\"retry\":{},\"calls\":[{}]}}",
            s.m,s.candidates.len(),s.candidates,s.position,s.deferred,s.failures,mark_json(&s.marks),s.support,
            bits(s.count_sum),s.successes,s.request,s.status,s.numeric,s.failure,s.retry,backend.calls.join(",")));
    }
    let dir = format!("{root}/target/m3_push_round_rust");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/trace.txt"),output.join("\n")+"\n").unwrap();
}
