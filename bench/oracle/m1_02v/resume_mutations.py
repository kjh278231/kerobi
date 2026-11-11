"""Resume the 17 cases after the 21 completed mutants; see the M1-02v contract."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[3] / 'tools'))
import m1_factor_work_check as check
assert len(check.MUTANTS) == 38
assert check.MUTANTS[21][0] == 'c2761 k + s'
check.MUTANTS = check.MUTANTS[21:]
sys.argv.append('--mutate')
raise SystemExit(check.main())
