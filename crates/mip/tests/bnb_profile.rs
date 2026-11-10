//! M2-10a harness: run `bnb::solve_original` with NodeLimit on the models in `RSOLVER_PROFILE_MODELS` (MPS paths,
//! space separated) and print one `RESULT` line each. With `bench/oracle/m2_10a/profile.patch` applied, bnb.rs also
//! prints a `PROFILE` line per solve. Run through `tools/mip_bnb_profile.sh` (docs/m2/mip_bnb_node_cost.md).
use rsolver_mip::{bnb, Params};

#[test]
#[ignore]
fn bnb_profile() {
    let models = std::env::var("RSOLVER_PROFILE_MODELS").expect("RSOLVER_PROFILE_MODELS");
    let nodes: f64 = std::env::var("RSOLVER_PROFILE_NODES").map_or(500.0, |v| v.parse().unwrap());
    for path in models.split_whitespace() {
        let model = rsolver_model::mps::read_file(path).unwrap();
        let p = Params { node_limit: nodes, ..Params::default() };
        let t = std::time::Instant::now();
        let r = bnb::solve_original(&model, &p).unwrap();
        let wall = t.elapsed().as_secs_f64();
        let name = path.rsplit('/').next().unwrap().trim_end_matches(".mps.gz");
        println!("RESULT {name} status={:?} nodes={} iters={} bound={:016x} obj={} path={} wall={wall:.6}",
            r.status, r.nodes, r.iterations, r.bound.to_bits(),
            r.incumbent_objective.map_or("none".into(), |v| format!("{:016x}", v.to_bits())), r.path);
    }
}
