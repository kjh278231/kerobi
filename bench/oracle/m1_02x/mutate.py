"""M1-02x Rust mutations: each one is applied to crates/lp/src/lu.rs, built (tools/build_local.sh) and run through
tools/m1_lt_check.py --rust; the file is restored afterwards. Expected: every mutation rejected except those listed
as undetectable in docs/m1/m1_02x_lt_depth.md. usage (worktree root): python3 bench/oracle/m1_02x/mutate.py"""
import os, subprocess
LU = 'crates/lp/src/lu.rs'
MUT = {
    'no LU+0x140 test': ('(nnz == 0 || (self.ltmax as f64) < m as f64 * 0.1)', 'true'),
    '< becomes <=': ('(self.ltmax as f64) < m as f64 * 0.1', '(self.ltmax as f64) <= m as f64 * 0.1'),
    '0.1 becomes 0.2': ('(self.ltmax as f64) < m as f64 * 0.1', '(self.ltmax as f64) < m as f64 * 0.2'),
    'no nnz 0 case': ('(nnz == 0 || (self.ltmax as f64) < m as f64 * 0.1)', '((self.ltmax as f64) < m as f64 * 0.1)'),
    'diagonal-U site only': ('if !dense_eta && !ut_dense && self.lt_sparse_ok(xs.len()) {', 'if !dense_eta && !ut_dense && self.sparse_mult * xs.len() <= m {'),
    'updated-U site only': ('if !sparse_u || !self.lt_sparse_ok(xs.len()) {', 'if !sparse_u || self.sparse_mult * xs.len() > m {'),
}
src = open(LU).read()
res = {}
try:
    for name, (a, b) in MUT.items():
        assert src.count(a) == 1, name
        open(LU, 'w').write(src.replace(a, b))
        subprocess.run(['tools/build_local.sh'], check=True, capture_output=True)
        p = subprocess.run([os.path.abspath('../.venv/bin/python'), 'tools/m1_lt_check.py', '--rust'], capture_output=True, text=True,
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
