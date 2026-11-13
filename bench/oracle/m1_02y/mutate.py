"""M1-02y Rust mutations of the FT new-diagonal peel: each one is applied to crates/lp/src/simplex.rs or lu.rs, built
(tools/build_local.sh) and run through tools/m1_diag_peel_check.py --rust; the file is restored afterwards. Expected:
every mutation rejected. ("Peel without the below-9 sequential case" is not listed: with one peeled term and at most 8
terms there is no full lane block, so the lane sum equals the sequential one.)
usage (worktree root): python3 bench/oracle/m1_02y/mutate.py"""
import os, subprocess
SIM, LU = 'crates/lp/src/simplex.rs', 'crates/lp/src/lu.rs'
PEEL = 'lu_peel: (interval + n) % 2 == 0,'
DIAG = 'diag = dot8_peel(eta.iter().map(|&(i, e)| spike[i] * e), diag_peel);'
MUT = {
    'never peel': (SIM, PEEL, 'lu_peel: false,'),
    'always peel': (SIM, PEEL, 'lu_peel: true,'),
    'inverted parity': (SIM, PEEL, 'lu_peel: (interval + n) % 2 == 1,'),
    'interval after the Full-pricing cap': (SIM, PEEL, 'lu_peel: (((m as f64).powf(0.65) as usize).max(20) + n) % 2 == 0,'),
    'diagonal ignores the flag (flag reported right)': (LU, DIAG, DIAG.replace('diag_peel);', 'false);')),
}
srcs = {f: open(f).read() for f in (SIM, LU)}
for f, a, _ in MUT.values():
    assert srcs[f].count(a) == 1
res = {}
try:
    for name, (f, old, new) in MUT.items():
        open(f, 'w').write(srcs[f].replace(old, new))
        b = subprocess.run(['tools/build_local.sh'], capture_output=True, text=True)
        assert 'error' not in b.stdout, b.stdout[-800:]
        p = subprocess.run([os.path.abspath('../.venv/bin/python'), 'tools/m1_diag_peel_check.py', '--rust'], capture_output=True,
                           text=True, env=dict(os.environ, PYTHONPATH=os.path.abspath('python')))
        last = (p.stdout.strip().splitlines() or ['?'])[-1]
        # only a checker FAIL line counts as rejection; a crash is an error
        res[name] = 'REJECTED' if p.returncode == 1 and last.startswith('FAIL') else 'SURVIVED' if p.returncode == 0 else 'ERROR'
        if res[name] == 'ERROR':
            print(p.stderr[-600:])
        print(res[name], name, '|', last[:220], flush=True)
        open(f, 'w').write(srcs[f])
finally:
    for f, v in srcs.items():
        open(f, 'w').write(v)
    subprocess.run(['tools/build_local.sh'], check=True, capture_output=True)
print('MUTATIONS', sum(v == 'REJECTED' for v in res.values()), '/', len(res), 'rejected')
