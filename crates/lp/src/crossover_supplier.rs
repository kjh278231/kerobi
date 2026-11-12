//! Single-column primal push supplier. FTRAN is supplied by the caller; no solver loop calls this yet.

/// FTRAN vector. A negative count denotes `m` dense values (no indices); otherwise the first `count`
/// entries are sparse in their stored order. Backends must return finite values and valid row indices.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PushColumn {
    pub count: i32,
    pub indices: Vec<i32>,
    pub values: Vec<f64>,
}

/// Original constraint coordinates, before any factor/basis permutation.
pub struct SupplierModel<'a> {
    pub m: usize,
    pub n: usize,
    pub starts: &'a [i64],
    pub lengths: &'a [i32],
    pub rows: &'a [i32],
    pub values: &'a [f64],
    pub drop: f64,
}

/// Single-path context: batch capacity, fill and consumption must all be zero before using this API.
/// Candidates are in `[0, n+m)` and `deferred <= position <= candidates.len()`; failures has `n+m`
/// entries. With positive support, marks must contain `m` constraint-row entries. Otherwise it is unused.
pub struct SupplierState {
    pub candidates: Vec<i32>,
    pub position: usize,
    pub deferred: usize,
    pub failures: Vec<i32>,
    pub support: i32,
    pub support_marks: Option<Vec<i32>>,
    pub count_sum: f64,
    pub successes: i32,
}

/// Persistent scratch, alpha and factor spike buffers. On exhaustion all three remain untouched.
#[derive(Default)]
pub struct SupplierBuffers {
    pub input: PushColumn,
    pub alpha: PushColumn,
    pub spike: PushColumn,
}

/// External numeric boundary. Write the raw FTRAN result to `output`; update `spike` when supplied.
/// The supplier always passes its actual alpha buffer, and passes its factor spike only with support=0.
pub trait SupplierFtran {
    fn ftran(&mut self, input: &PushColumn, output: &mut PushColumn, spike: Option<&mut PushColumn>, work: &mut f64);
}

/// Consume candidates until one can be transformed, or return -1. Skips append to the existing deferred
/// prefix without changing failures or marks. Raw count statistics precede dense-to-sparse compression.
/// Finite nonnegative drop tolerance and valid CSC/vector dimensions are caller preconditions.
pub fn supplier_next(model: &SupplierModel<'_>, state: &mut SupplierState, buffers: &mut SupplierBuffers,
                     work: &mut f64, backend: &mut impl SupplierFtran) -> i32 {
    state.support = state.support.max(0);
    while state.position < state.candidates.len() {
        let j = state.candidates[state.position] as usize;
        state.position += 1;
        let failed = state.failures[j] >= 3;
        let conflict = !failed && state.support > 0 && {
            let marks = state.support_marks.as_ref().expect("positive support needs row marks");
            if j < model.n {
                let start = model.starts[j] as usize;
                let end = start + model.lengths[j] as usize;
                model.rows[start..end].iter().any(|&row| marks[row as usize] != 0)
            } else {
                marks[j - model.n] != 0
            }
        };
        if failed || conflict {
            state.candidates[state.deferred] = j as i32;
            state.deferred += 1;
            continue;
        }

        let input = &mut buffers.input;
        input.indices.clear();
        input.values.clear();
        if j < model.n {
            let start = model.starts[j] as usize;
            let end = start + model.lengths[j] as usize;
            input.count = model.lengths[j];
            input.indices.extend_from_slice(&model.rows[start..end]);
            input.values.extend_from_slice(&model.values[start..end]);
        } else {
            input.count = 1;
            input.indices.push((j - model.n) as i32);
            input.values.push(1.0);
        }
        let spike = if state.support == 0 { Some(&mut buffers.spike) } else { None };
        backend.ftran(input, &mut buffers.alpha, spike, work);
        let alpha = &mut buffers.alpha;
        let raw_count = alpha.count;
        state.successes += 1;
        state.count_sum = f64::from(raw_count) + state.count_sum;
        if raw_count < 0 {
            alpha.indices.clear();
            let mut kept = 0;
            for row in 0..model.m {
                let value = alpha.values[row];
                if model.drop < value.abs() {
                    alpha.values[kept] = value;
                    alpha.indices.push(row as i32);
                    kept += 1;
                }
            }
            alpha.count = kept as i32;
            alpha.values.truncate(kept);
        }
        return j as i32;
    }
    -1
}
