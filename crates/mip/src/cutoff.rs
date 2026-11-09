//! Linear objective lattice (`5e2931`) and improvement cutoff (`6f45ed`).
use rsolver_model::Model;
use crate::Params;

/// The objective step in normalized domain types; zero means no usable lattice.
pub(crate) fn objective_step(model: &Model, p: &Params) -> f64 {
    if p.objgcd == 0 { return 0.0; }
    let mut coefficients = Vec::new();
    for (&c, &t) in model.obj.iter().zip(&model.vtype) {
        if c.abs() > 1e-10 {
            if t == b'C' { return 0.0; }
            coefficients.push(c.abs());
        }
    }
    match coefficients.len() {
        0 => -1e100,
        1 => coefficients[0],
        _ => {
            let max = coefficients.iter().copied().fold(0.0, f64::max);
            let step = crate::iint::gcd_f64(&coefficients);
            if p.intfeastol * max < step { step } else { 0.0 }
        }
    }
}

/// `5e2931`: retain a stronger parent lattice only when the objective offsets are congruent.
pub(crate) fn inherit_step(step: f64, offset: f64, parent_step: f64, parent_offset: f64) -> f64 {
    if step >= 0.0 && step < parent_step {
        let ratio = (offset - parent_offset) / parent_step;
        if ratio - (ratio + 1e-10).floor() <= 1e-10 { return parent_step; }
    }
    step
}

/// All inputs use minimization coordinates; the parameter cutoff is already normalized.
pub(crate) fn improvement_cutoff(incumbent: f64, user: f64, step: f64, offset: f64, p: &Params) -> f64 {
    if user < incumbent { return user; }
    if incumbent >= 1e100 { return 1e100; }
    let strict = incumbent - (incumbent.abs() + 1.0) * 1e-10;
    if step <= 0.0 { return strict; }
    let tolerance = ((incumbent.abs() + 1.0) * 1e-6).max(step * p.intfeastol).max(p.opttol);
    let multiple = (((incumbent - offset) - (step * 1e-10).max(1e-10)) / step).floor() * step;
    let base = offset + tolerance;
    let mut lattice = base + multiple;
    let exact = if multiple.abs() < base.abs() { lattice - base == multiple } else { lattice - multiple == base };
    if !exact {
        lattice *= if lattice <= 0.0 { 0.9999999999999998 } else { 1.0000000000000002 };
    }
    lattice.min(strict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_cutoff_fixtures_match_bits() {
        for line in include_str!("../../../bench/oracle/mip_cutoff/cutoff.tsv").lines().skip(1) {
            let v: Vec<f64> = line.split('\t').map(|s| s.parse().unwrap()).collect();
            let p = Params { intfeastol: v[4], opttol: v[5], ..Params::default() };
            assert_eq!(improvement_cutoff(v[0], v[1], v[2], v[3], &p).to_bits(), v[6].to_bits(), "{line}");
        }
    }

    #[test]
    fn native_objective_steps_match_bits() {
        for line in include_str!("../../../bench/oracle/mip_cutoff/step.tsv").lines().skip(1) {
            let fields: Vec<_> = line.split('\t').collect();
            let obj = fields[0].split(',').map(|s| s.parse().unwrap()).collect();
            let model = Model { obj, vtype: fields[1].as_bytes().to_vec(), ..Model::default() };
            let p = Params { intfeastol: fields[2].parse().unwrap(), objgcd: fields[3].parse().unwrap(), ..Params::default() };
            let expected: f64 = fields[4].parse().unwrap();
            assert_eq!(objective_step(&model, &p).to_bits(), expected.to_bits(), "{line}");
        }
    }

    #[test]
    fn parent_step_requires_congruent_offsets_and_enabled_inference() {
        assert_eq!(inherit_step(0.0, 10.0, 3.0, 7.0), 3.0);
        assert_eq!(inherit_step(0.0, 10.5, 3.0, 7.0), 0.0);
        assert_eq!(inherit_step(6.0, 10.0, 3.0, 7.0), 6.0);
        assert_eq!(inherit_step(-1e100, 10.0, 3.0, 7.0), -1e100);
    }
}
