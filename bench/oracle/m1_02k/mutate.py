"""M1-02k Rust mutations: each one is applied to crates/lp/src/lu.rs, built (tools/build_local.sh) and run through
tools/m1_btran_check.py --rust; the file is restored afterwards. Expected: every mutation rejected except those listed
as undetectable in docs/m1/m1_02k_btran_ut.md. usage (worktree root): python3 bench/oracle/m1_02k/mutate.py"""
import os, subprocess
LU = 'crates/lp/src/lu.rs'
MUT = {
    'old 3*nnz test': ('(self.dsum as i32).wrapping_mul(nnz as i32) > self.head.len() as i32', '3 * nnz > self.head.len()'),
    'dense U^T does not force dense etas': ('(ut_dense || m < self.sparse_mult * list.len())', '(m < self.sparse_mult * list.len())'),
    'dense U^T without |x| <= ZERO': ('if t.abs() <= ZERO || w[k].abs() <= ZERO {', 'if t.abs() <= ZERO {'),
    'no LU+0x108 > 5 path': ('if self.d108 > 5.0 {', 'if self.d108 > 5e300 {'),
    'prefix inputs as tail': ('if self.upos[k] < self.prefix {', 'if self.upos[k] < 0 {'),
    'never switch to the linear scan': ('if (m - top) as f64 <= heap.len() as f64 * 6.0 {', 'if (m - top) as f64 <= heap.len() as f64 * 0.0 {'),
    'dense U^T does not force dense L^T': ('if !dense_eta && !ut_dense && self.sparse_mult * xs.len() <= m {', 'if !dense_eta && self.sparse_mult * xs.len() <= m {'),
    'dense U^T hands L^T the listed values only': ('        if dense_eta || ut_dense {', '        if dense_eta {'),
    '9e376 force ignored': ('self.urow_len(r) == 0 && self.ut_dense(1)', 'self.ut_dense(1)'),
}
src = open(LU).read()
res = {}
try:
    for name, (a, b) in MUT.items():
        assert src.count(a) == 1, name
        open(LU, 'w').write(src.replace(a, b))
        subprocess.run(['tools/build_local.sh'], check=True, capture_output=True)
        p = subprocess.run([os.path.abspath('../.venv/bin/python'), 'tools/m1_btran_check.py', '--rust'], capture_output=True, text=True,
                           env=dict(os.environ, PYTHONPATH=os.path.abspath('python')))
        last = (p.stdout.strip().splitlines() or ['?'])[-1]
        # only a checker FAIL line counts as rejection; a crash is an error
        res[name] = 'REJECTED' if p.returncode == 1 and last.startswith('FAIL') else 'SURVIVED' if p.returncode == 0 else 'ERROR'
        if res[name] == 'ERROR':
            print(p.stderr[-600:])
        print(res[name], name, '|', last[:200], flush=True)
finally:
    open(LU, 'w').write(src)
    subprocess.run(['tools/build_local.sh'], check=True, capture_output=True)
print('MUTATIONS', sum(v == 'REJECTED' for v in res.values()), '/', len(res), 'rejected')
